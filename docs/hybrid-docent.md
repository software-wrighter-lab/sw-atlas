# Hybrid docent: deterministic matcher plus a typed decision model

Adopted 2026-09-22 by the repository owner, and carried into
[`plan.md`](plan.md) section 10 as Saga 3 (HT01). This file holds the
reasoning; where it and the plan disagree, the plan wins. Saga numbers
below are the ones after the insertion.

The question: how to reuse
[`demo-decision-model`](https://github.com/sw-ml-study/demo-decision-model) (TDM) to
build an in-browser, Docent-like chat that sends a visitor to the right
post, demo, video or repository, answers questions and meta-questions,
retrains on a schedule over every public artifact, and is *shown* by
measurement to improve on the current Docent's deterministic matcher.

## 1. Why the previous neural docents lost, and what that implies

Every trained docent so far was asked to do the matcher's job instead of
the matcher: predict the destination from a pooled bag of features. On the
54 held-out paraphrases, moe-microscope's scoreboard (its
`docs/reference/docent-results.md`) reads:

| Run | Paraphrase dest | Paraphrase intent | Unsupported recall | Val top-3 |
|---|---:|---:|---:|---:|
| MB01t text matcher | **0.685** | 0.481 | 0.40 | 0.983 |
| MB01 alias matcher | 0.630 | 0.463 | **1.00** | 0.996 |
| SAN01b no-FFN docent | 0.444 | 0.463 | 0.30 | 0.967 |
| CD01b dense docent | 0.407 | 0.407 | 0.30 | 0.959 |

(Transferred rows; origin moe-microscope, `mlpl-repl 0.22.0`.)

Read the columns, not the winner:

- The matchers are good at **recall of destinations** (top-3 near 1.0 on
  validation) and weak at **intent** (under 0.5 on paraphrases).
- A model trained on a few hundred template rows only knows words it saw,
  so as a destination classifier it cannot beat a matcher that reads every
  word in the catalog.
- SAN01 established that a pooled embedding with linear heads and no FFN
  loses nothing against its dense twin. That is exactly the TDM shape.

So stop running the matcher and the model as rivals. Let each do what it
is measurably good at.

## 2. The division of labour

```
 visitor sentence  +  conversation memory (program-owned)
        |
        +--------------------+---------------------------+
        |                    |                           |
        v                    v                           |
  A0 matcher (MB02)    TDM, one forward pass             |
  deterministic        pooled encoder, typed heads:      |
  - top-k candidate      Choice  intent (10 classes)     |
    resources with       Choice  resource kind wanted    |
    hit signals          Noul    is-meta, is-followup,   |
  - lexical score          off-topic, wants-link, ...    |
                         Choice  rerank over the k       |
                           candidates (dynamic set:      |
                           score = f(h_query, h_card))   |
        |                    |                           |
        +---------+----------+                           |
                  v                                      |
        policy (ordinary Rust, thresholds are data)  <---+
          - A0 confident and TDM agrees      -> act
          - A0 confident, TDM disagrees      -> A0 wins unless TDM
                                                margin > tau (fitted)
          - A0 weak, TDM confident in rerank -> act on TDM pick
          - both weak                        -> ranked alternatives
          - off-topic Noul high              -> "I only know about ..."
                  |
                  v
        answer = frame + catalog text, quoted, never composed
```

The model never emits a URL, a resource id or prose (invariant 1). It picks
among candidates the matcher and the catalog already produced. That is the
TDM rule, "every string a user sees is one of the candidates the program
offered", and the Atlas rule, "the catalog resolves", in one mechanism.

**Why this is best-of-both rather than a compromise:**

| Failure | Matcher alone | TDM alone | Hybrid |
|---|---|---|---|
| exact title/alias ("sw-cor24-apl") | right | often wrong, unseen token | matcher confident, acts |
| paraphrase ("the language with strange symbols") | sometimes | if seen in training | TDM reranks the matcher's top-k |
| intent ("is it finished?" vs "where is it?") | regex, 0.48 | learned head | TDM |
| off-topic ("what's the weather") | brittle | trained NONE class | Noul plus both-weak rule |
| a new post published last night | right, it is in the catalog | unknown label | matcher proposes it; the rerank head scores cards, not labels, so it can pick it without retraining |
| visitor misspells | fuzzy substring | exact vocab misses | either one can recover it |

The last-but-one row is why the rerank head must be a **dynamic-choice
scorer** (demo-decision-model's PR05, `f(h_state, h_question, h_choice)`)
and not a fixed softmax over resource ids. A head over ids needs retraining
for every new post and breaks invariant 2.

## 3. What to reuse from demo-decision-model, and how

Sibling repos are read-only, so every reuse is a pinned dependency or a
vendored copy. Nothing here edits them.

| Piece | Where | Reuse as | Why |
|---|---|---|---|
| decision contract: Choice, Noul, Scale; confidence, margin | `lib/decision.mlpl` | vendor, hash-pinned | the typed output record, already tested |
| hashed / exact-vocab featurizer, masked mean pool, Choice trainer | `lib/text.mlpl`, `lib/choice_model.mlpl` | vendor, hash-pinned | the trainer, and no Python (plan section 12.1) |
| Rust forward pass with 1e-9 parity against MLPL | `crates/tdm-model` (`features.rs`, `model.rs`) | git dependency at tag `tdm-v0.1.0`, whose contract is written up upstream | its abstraction gate keeps demo identifiers out of this crate |
| escalation policy: Act / NoneApplies / Escalate on min_known, confidence, margin | `crates/tdm-model/src/responder.rs` | copy the shape into the policy crate | this is the matcher-or-model arbitration |
| bundle format with an embedded parity set | `crates/tdm-model/src/bundle.rs` | extend into `snapshot/model/tdm.json` | trainer/browser drift fails a test |
| trace schema and validator; rejects output text that was not offered | `schemas/decision-trace-v1`, `crates/tdm-trace` | git dependency at tag `tdm-v0.1.0` | "why this answer?" (plan Saga 9 step 5) at no cost |
| Yew chat plus trace panel, training timeline | `crates/tdm-web` | pattern, not a dependency | the site's UI belongs to sw-campus / blog |
| "freeze labels before training data exists" | its probes discipline | adopt as a rule | the thing that makes the comparison honest |

What does **not** carry over: the ELIZA response table and the
conversation script. Atlas answers come from the catalog, not canned lines.
The frame-plus-slot mechanism does carry over (`Frame "{title} is in
{where}."`, with slots quoted from `catalog.json`).

Size, measured there: 33,065 parameters, about 264 KB in f64, 0.5 to 2.1 ms
per decision, about 25 s to train on an M1 Max (demo-decision-model
README). An Atlas TDM with a vocabulary about 10x larger is still inside
A1's 25 MiB budget, not A2's 64. This is an estimate until measured.

This is also the degenerate rung of plan section 6's depth ladder: a SAN
with zero attention layers *is* a pooled encoder with typed heads. Nothing
in the Needle plan is abandoned. The TDM becomes the rung Needle has to
beat.

## 4. Questions, answers and meta-questions

The model never writes prose, so "answering" means routing to text the
author already wrote. The corpus has plenty of it: 124 hand-written blog
abstracts, campus taglines and 21 stories, repo descriptions, video
descriptions, and later the shorts scripts.

| Intent | Example | Deterministic answer |
|---|---|---|
| navigate / find_resource | "where's the APL thing" | link plus location, top card |
| explain | "what is the 1130 emulator" | the card's abstract, quoted, plus link |
| status | "is the card reader working yet" | the Maturity field, rendered through a frame |
| recommend / next-in-series | "what should I read after part 2" | SeriesNext relation |
| related-kind | "is there a video of that?" | follow-up Noul plus memory of last resource, then a Demos / Video relation |
| compare | "APL vs Forth posts" | two cards side by side, no synthesis |
| **meta-corpus** | "how many posts about MoE", "what's newest" | counts and dates computed from the catalog |
| **meta-self** | "are you an LLM?", "how do you work?" | canned role text (role data, plan section 2) |
| **meta-why** | "why did you send me there?" | the last turn's trace: signals, candidates, probabilities |
| unsupported | "what's the weather" | the role's "I only know about..." line |

Meta-corpus answers are exact because code computes them from the
snapshot. That is a thing an LLM docent gets wrong and this one cannot.

## 5. Periodic retraining

The pipeline is the plan's Saga 11 shape, simplified by the TDM being cheap:

```
 cron (nightly or weekly)
   fetch-repos (non-fork, all orgs), blog, campus, shorts
   -> atlas-ingest -> corpus hash
   -> unchanged?  stop
   -> regenerate template questions from the new catalog
      (frozen eval sets are NOT regenerated)
   -> train the TDM in sw-MLPL (seconds to minutes, deterministic,
      byte-identical weights for identical inputs)
   -> export bundle + parity set -> Rust parity test
   -> eval harness: A0, TDM-only, hybrid on frozen sets
   -> gate: hybrid >= yesterday's hybrid and >= A0 on every frozen set,
      within the budget -> publish the changed files; otherwise keep
      yesterday's snapshot and write down why
```

The repo scope is already in the cache: `cache/github-repos.json` holds
283 repositories, **242 of them non-fork** across 15 accounts (103
softwarewrighter, 43 sw-embed, 24 sw-ml-study, 20 sw-vibe-coding, 19
sw-comp-history, ...). `fork == false` belongs in `atlas-ingest repos`
(step ingest-metadata) as the filter, not in the fetch script, so the cache
stays the unfiltered record. Forks the blog or campus links to are kept (owner
decision, 2026-09-22), which makes 251.

Because the rerank head scores cards rather than ids, a new post should
usually need **no** retraining at all, only an index rebuild. Saga 7 (SN01)
measures that; the scheduled retrain covers vocabulary drift.

## 6. Verification: how to show the hybrid improves on the matcher

The claim has to survive the scrutiny that sank CD01. Rules, then arms,
then metrics, then the gate.

**Rules.**
1. Freeze every evaluation set, labels included, *before* any training
   template exists, and commit it with its hash. demo-decision-model did
   this for its 96 probes; it is why its model-2 comparison is believable.
2. Leakage check by hash and by 8-gram overlap between train and frozen
   rows. Plan Saga 5 step 4 already requires this.
3. One number per arm per set, regenerable by one `just eval` recipe.
4. Enough rows to resolve the bar. With 54 paraphrases the 95% interval on
   a single accuracy is about ±13 points, too wide to tell +20 from +7.
   Target **≥ 300 paraphrase rows** (about ±5.5) across all resource kinds.
   Authored by hand or teacher-drafted and then hand-checked, never
   template-generated.

**Frozen sets.** Cross-corpus paraphrase (no alias verbatim), exact-name,
off-topic, ambiguous (expected set), meta (corpus / self / why),
multi-turn follow-ups (scripted two- and three-turn dialogs), and
new-resource (questions about resources removed from the training catalog
but present in the served catalog, the 1442 pattern).

**Arms.**

| Arm | What |
|---|---|
| A0 | today's Docent matcher (`sw-campus/pages/index.html` `predict` / `intentOf`), ported to Rust over the whole corpus = MB02 |
| A0-oracle@k | "correct answer is in the matcher's top-k": the **ceiling** for any reranker, and the first number to measure (section 8) |
| TDM | model alone, destination by rerank over all cards |
| H | hybrid, policy in section 2 |
| H-ablations | H without the rerank head; H without intent head; H with a hand-set tau instead of a fitted one |

**Metrics** (the plan's list, plus three for the hybrid claim): intent
accuracy, dest@1, top-3, MRR, unsupported recall, ambiguous top-2, ECE,
Brier, p50/p95 latency, bytes; plus

- **confidently-wrong rate**: wrong answers the policy acted on. What a
  visitor actually suffers from.
- **coverage/accuracy curve** over the abstention threshold.
- **regression rows**: questions A0 got right that H got wrong, listed by
  text in the report, not just counted.

**Statistics.** Paired, because every arm answers the same questions:
McNemar's test on per-question dest@1 (A0 vs H), and a paired bootstrap
95% CI on the margin. Report the CI next to the margin every time.

**Gate** (plan invariant 5 and the CLAUDE.md bar, applied to H as the
thing that would ship): H beats A0 by ≥ +20 points paraphrase dest@1 and
≥ +20 points intent, unsupported recall ≥ 0.8, the CI lower bound above
zero, regression rows ≤ a fixed small count, inside A1's budget. Anything
short of that is published as the result, and A0 keeps serving.

**Browser proof.** The bundle carries a parity set, and a wasm test asserts
Rust-in-browser probabilities match the trainer. The trace panel shows the
A0-only answer beside the hybrid's on every turn (demo-decision-model
already prints `matcher:` in its trace), so a visitor can see a
disagreement live. Opt-in local logging of disagreements feeds the next
frozen set (plan risk "teacher questions are unlike real ones").

## 7. Where this lands in the saga queue

Adopted by the repository owner on 2026-09-22; [`plan.md`](plan.md)
section 10 carries it:

1. Finish Saga 1 (`atlas-foundation`) unchanged; its hybrid-tdm-decision
   step records this decision.
2. Saga 2 `atlas-baseline` unchanged, with two additions: the A0-oracle@k
   row, and eval sets sized for section 6 rule 4.
3. **A new Saga 3 `hybrid-tdm` (HT01)** ahead of the Needle probe:
   vendor the TDM pieces; train intent, kind and Nouls plus the card
   rerank head in sw-MLPL; the Rust `atlas-tdm` crate with parity; the
   policy crate; eval all arms; publish the row either way.
4. The Needle probe (NP01) becomes Saga 4 and competes against H, not A0.
   If H clears the bar, Needle becomes an upgrade to the encoder inside H,
   not a replacement for the design.

Work order filed in
[`demo-decision-model-requests.md`](demo-decision-model-requests.md), which
asked for (a) PR05 dynamic choice sets, because the card rerank head needs
exactly that trainer, and (b) a tagged release of `tdm-model` /
`tdm-trace` to pin against. Both were answered on 2026-09-22: the tag is
`tdm-v0.1.0`, and PR05 is upstream's to build, so step 3 of the saga waits
on it rather than writing a second one. Its NV01 ("demo 02: campus
navigation, `lib/` unchanged") is effectively this project, so the finding
flows both ways.

## 8. The first measurement

Before any training: **A0-oracle@k on the paraphrase set.** If the correct
resource is in the matcher's top-5 for, say, 90% of paraphrases while
dest@1 is 0.685, a reranker has about 20 points to win, and the bar is
reachable in principle. If top-5 recall is itself near 0.7, the matcher
is not even *proposing* the right answers. Then the hybrid needs a second
candidate source (A1 embedding retrieval) before a reranker can help, and
that is the finding. It costs one afternoon on MB01t's existing fixtures.
