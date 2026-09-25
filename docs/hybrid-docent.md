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

Revised 2026-09-24 by a measurement from upstream, recorded in section 2a.
The short version: the model is never asked to choose among candidates it
has not trained on, and concepts are what carries a new resource.

```
 visitor sentence  +  conversation memory (program-owned)
        |
        +--------------------+---------------------------+
        |                    |                           |
        v                    v                           |
  A0 matcher (MB02)    TDM, one forward pass,            |
  deterministic        every head a FIXED label set:     |
  - reads every word     Choice  intent (10 classes)     |
    of every card        Choice  resource kind wanted    |
  - top-k candidates     Noul*   concepts (~703, multi-  |
    with their signals           label, one sigmoid each)|
  - strong on names      Noul    off-topic, meta,        |
    and brand-new text           follow-up, wants-link   |
        |                    |                           |
        |         concepts -> catalog lookup             |
        |         (deterministic; a post indexed         |
        |          after training is reachable the       |
        |          moment it carries a known concept)    |
        |                    |                           |
        +---------+----------+                           |
                  v                                      |
        policy (ordinary Rust, thresholds are data)  <---+
          - A0 confident and the model agrees  -> act
          - A0 confident, model disagrees      -> A0 wins unless the
                                                  model's margin > tau
          - A0 weak, concepts confident        -> act on the concept
                                                  lookup's ranking
          - rerank, but only among candidates the model trained on;
            a cold candidate keeps the matcher's order untouched
          - both weak, or off-topic Noul high  -> abstain, then suggest
                  |
                  v
        answer = frame + catalog text, quoted, never composed
```

The model still never emits a URL, a resource id or prose (invariant 1). It
emits an intent, a kind, a set of concepts and a few propositions;
deterministic code turns those into places and links.

**Why this is best-of-both rather than a compromise:**

| Failure | Matcher alone | TDM alone | Hybrid |
|---|---|---|---|
| exact title/alias ("sw-cor24-apl") | right | often wrong, unseen token | matcher confident, acts |
| paraphrase ("the language with strange symbols") | sometimes | if the words trained | concepts narrow the field; a warm card may be reranked |
| intent ("is it finished?" vs "where is it?") | regex, 0.48 | learned head | the model, and this is its clearest win |
| off-topic ("what's the weather") | brittle | trained NONE class | Noul plus the both-weak rule |
| a new post published last night | right, it is in the catalog | no label for it | it carries known concepts, so the concept lookup reaches it; reranking leaves it where the matcher put it |
| visitor misspells | fuzzy substring | exact vocab misses | either one can recover it |

### 2a. What changed, and the measurement that changed it

The earlier version of this section put a **dynamic-choice scorer** at the
centre: score every candidate card as text, so a card written after training
could still win. `demo-decision-model` built exactly that as PR05 and
measured it (their lesson DC01, on their DOCTOR corpus, `mlpl-repl 0.22.0`):

| Rows | full field (51 cards) | five cards, four trained | five cards, all cold |
|---|---:|---:|---:|
| cards that trained | 85.3% | 97.4% | 95.9% |
| cards held out of training entirely | **0.4%** | 36.5% | 18.6% (chance is 20%) |

Where cards trained, the generality costs about a point against a fixed head
(85.3% against model 4's 86.1% on the same rows) -- that part transfers.
Where they did not, the right card essentially never wins, and among
candidates that are all cold the scorer cannot tell which fits.

Their caveat is fair and matters here: a DOCTOR rule card is a decomposition
pattern whose overlap with its input is function words, while an Atlas query
and an Atlas card share *content* words ("sleep" against a card about
sleep). So 0.4% is a lower bound from an unfriendly regime, not a verdict on
card scorers, and the number this project must act on is its own. What DC01
does settle is that "a candidate written after training can be chosen" is
not a property of the architecture, and a design that assumed it was
assuming.

So the assumption is gone. Concepts do that job instead: the concept head is
a fixed label set over a curated vocabulary that changes at the pace of the
concept graph rather than the pace of publishing, a new resource arrives
carrying concepts that already exist, and the catalog resolves them without
the model scoring anything it has not seen. Reranking survives in the place
DC01 shows it works -- a small field of warm cards -- and cold candidates
keep the matcher's order, which turns "never worse than A0 on a new post"
from a hope into a property of the wiring.

### 2a-bis. Then they ran it on our data, and it does not work yet

On 2026-09-24 `demo-decision-model` took this project's own corpus and
question sets and measured the hybrid itself (their lesson AT01, their step
024): 642 resources, 386 questions, 72 resources held out entirely so the 91
questions pointing at them are exactly the "published last night" case. Four
findings, and none of them is comfortable.

**A defect this project would have vendored.** `lib/scorer.mlpl` normalized a
card to unit length with the zero-norm case guarded *after* `sqrt` -- correct
forward, `NaN` backward. One card whose every word is unknown to the
vocabulary turns every parameter into `NaN` on the first Adam step, silently:
training completes and prints an accuracy of 0.000 with an MRR of exactly
1.000, which is the signature of comparing against `NaN`. Demo 01's twelve
hand-written cards never contained such a card; a 642-resource catalog does.
Fixed upstream with an epsilon inside the root and pinned by a probe (their
finding Q6), so **the vendor must be taken at or after commit `b78a2e1`**.

**The rerank head does not work at this data scale.** Reranking their
matcher's top 20: 0.023 warm and 0.032 cold, against **0.050 for reranking at
random**, with training loss down at 0.011. It memorized its 174 training
questions and transferred nothing, and warm equals cold, so this is not the
held-out cards failing -- 308 questions cannot fit a 78,000-parameter space
over a 2,340-word vocabulary. The design is **unfunded, not refuted**: the
open number is how many labelled questions per resource it needs, and the
next experiment is synthetic positives -- each card's own title and summary
as pseudo-queries to teach the shared space, then fine-tuning on the real
questions. That multiplies the training pairs about tenfold without a single
new hand-written question, and it becomes a step of Saga 3 before the rerank
head rather than after it.

**The ceiling is the matcher's recall, not the model's cleverness.** Their
stand-in matcher reached recall@20 of 0.63, so reranking the top 20 caps
accuracy at 0.63 however good the reranker gets, and raising *k* makes the
reranking problem harder rather than easier. Their matcher is an IDF-weighted
token overlap and not this project's MB02, so the number that governs Saga 3
is MB02's own recall@k -- which makes it the most important thing Saga 2
produces, ahead of MB02's accuracy.

**Intent beat nothing.** A five-class intent Choice scored 0.697 against a
0.737 always-`FindResource` baseline, and the is-off-topic Noul landed exactly
on its always-false baseline. The cause is in our sets, not their model: 72%
of the 386 questions are `FindResource`, because they were drafted to test
whether a system finds the right resource. A set drafted to test destinations
does not teach intent. Two consequences: the harness reports the
majority-class baseline beside every accuracy and never a bare one, and the
sets need an intent-balanced supplement -- owner work, since he confirmed the
existing rows.

All of it carries their caveat, which is worth repeating: the sets were still
marked `Unconfirmed` when they ran (they are confirmed now), the matcher was
theirs rather than MB02, and each head was trained in one configuration with
no sweep. Read the direction, not the decimals.

### 2b. What this project is claiming, and what it is not

Upstream also measured the comparison this project will be asked about
(their LB01, same bounded-choice method, nothing generated): on 24 held-out
spam and phishing messages a local `llama3.2:3b` scored 0.792 **zero-shot**
against their trained model's 0.750, `gemma4:31b` scored 1.000, and the 3B
model was better calibrated untouched (ECE 0.006) than theirs after
temperature scaling (0.097). Their conclusion is the one to carry: a trained
typed decision model earns its place where no pretrained model has the
knowledge.

That is the test this design has to pass, and it is the reason to expect it
can. Nothing pretrained has seen 642 private artifacts, 703 curated concepts
or which post supersedes which; and a generative model in the browser is
ruled out below A4 anyway. So the claim is: **offline, a few hundred
kilobytes, no server, and unable to invent an exhibit** -- over a corpus no
foundation model knows. The claim is *not* that it beats a large model with
the catalog in its prompt at reading a sentence. When the comparison row is
measured here it will be published whichever way it falls.

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
| card scorer (PR05): `Scorer::rank`, candidates as text at call time | `lib/scorer.mlpl`, `crates/tdm-model/src/scorer.rs` | git dependency at tag `tdm-v0.1.0`; used only for warm reranking (section 2a) | delivered 2026-09-24 with the measurement that bounds where it may be used |
| in-browser training of a bundled program | `scripts/bundle-program`, their Live Editor | pattern, for a demo rather than the product | shows a model this size trains from scratch in seconds; the nightly path stays the product |
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

A new post needs **no** retraining to be findable: it is indexed with
concepts that already exist, and the concept head plus the catalog reach it
(section 2a). What the scheduled retrain buys is vocabulary drift and
*warming* -- a card the model has trained on can be reranked, a cold one
keeps the matcher's order. Saga 7 (SN01) measures the difference, and DC01
says to expect it to be large.

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
| H-ablations | H without the rerank head; H without the concept head; H without the intent head; H with a hand-set tau instead of a fitted one |
| H-warm vs H-cold | the same hybrid measured over candidates it trained on and candidates held out entirely, upstream's DC01 harness shape: full field, small warm field, small cold field. The cold field is the diagnostic that separates a weak reranker from a biased one, and it is the number that says whether the concept path is carrying new resources on its own |

**Metrics** (the plan's list, plus three for the hybrid claim): intent
accuracy, dest@1, top-3, MRR, unsupported recall, ambiguous top-2, ECE,
Brier, p50/p95 latency, bytes; plus

- **confidently-wrong rate**: wrong answers the policy acted on. What a
  visitor actually suffers from.
- **coverage/accuracy curve** over the abstention threshold.
- **regression rows**: questions A0 got right that H got wrong, listed by
  text in the report, not just counted.
- **the majority-class baseline beside every accuracy**, never a bare
  accuracy. AT01 measured an intent head at 0.697 where always answering
  `FindResource` scores 0.737; without the baseline in the same table that
  reads as a result.
- **recall@k for every candidate source**, because it is the ceiling on
  anything that reranks: AT01 capped at 0.63 with k=20 on these sets.

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
   vendor the TDM pieces at tag `tdm-v0.1.0`; train the intent, kind,
   concept and Noul heads in sw-MLPL; the Rust `atlas-tdm` crate with
   parity; the policy crate, including the suggestion and next-question
   generation an abstention needs; eval all arms, warm and cold; publish
   the row either way.
4. The Needle probe (NP01) becomes Saga 4 and competes against H, not A0.
   If H clears the bar, Needle becomes an upgrade to the encoder inside H,
   not a replacement for the design.

Work order filed in
[`demo-decision-model-requests.md`](demo-decision-model-requests.md), which
asked for (a) PR05 dynamic choice sets and (b) a tagged release of
`tdm-model` / `tdm-trace` to pin against. **Both were delivered by
2026-09-24**: the tag is `tdm-v0.1.0` at commit `b476842`, and PR05 is
`lib/scorer.mlpl` with `Scorer::rank` taking candidates as text at call
time. PR05 arrived with DC01, which is why section 2 was rewritten rather
than merely annotated -- the most useful thing that request produced was a
negative result that saved this project from shipping a design built on an
assumption. Upstream now carries
`docs/reference/downstream-requests.md`, so an ask no longer travels only
by someone relaying it.

Their NV01 ("demo 02: campus navigation, `lib/` unchanged") is effectively
this project, so the finding flows both ways: whatever forces a change to
the vendored `lib/` here is reported back there.

## 8. The first measurement

Before any training: **A0-oracle@k on the paraphrase set.** If the correct
resource is in the matcher's top-5 for, say, 90% of paraphrases while
dest@1 is 0.685, a reranker has about 20 points to win, and the bar is
reachable in principle. If top-5 recall is itself near 0.7, the matcher
is not even *proposing* the right answers. Then the hybrid needs a second
candidate source (A1 embedding retrieval) before a reranker can help, and
that is the finding. It costs one afternoon on MB01t's existing fixtures.
