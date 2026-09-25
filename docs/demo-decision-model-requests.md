# Requests to `demo-decision-model`

Work orders for agents operating in
[`sw-ml-study/demo-decision-model`](https://github.com/sw-ml-study/demo-decision-model).
This document does not authorize changes from this repository, and sw-atlas
cannot make them: every item here is a request to be revalidated against
that repository's own artifacts before its saga begins.

Raised 2026-09-22, when sw-atlas adopted the hybrid docent (Saga 3,
HT01). The reasoning is in [`hybrid-docent.md`](hybrid-docent.md), and the
saga is in [`plan.md`](plan.md) section 10. In short: sw-atlas's
deterministic matcher proposes candidate resources, and a typed decision
model built from that repository's primitives decides intent, the kind of
resource wanted and a few Nouls, and reranks the candidates. sw-atlas
vendors the MLPL library hash-pinned and pins the Rust crates to a tag. It
edits nothing there.

**Both delivered by 2026-09-24.** The tag landed on 2026-09-22 and PR05 on
2026-09-24, with a measurement this project had to act on. Neither item
below is an open ask; both are records, and the PR05 record is the one to
read before building on it.

## PR05 — dynamic choice sets (delivered, and it changed the design)

**Status, 2026-09-24: delivered.** `lib/scorer.mlpl` and
`crates/tdm-model/src/scorer.rs`, where `Scorer::rank` takes candidates as
text at call time, mlplunit-tested, demo-neutral, parity with the trainer to
`1e-9` on the rounded weights a consumer receives, and reachable at tag
`tdm-v0.1.0`. Their lesson is
[`DC01`](https://github.com/sw-ml-study/demo-decision-model/blob/main/docs/experiments/DC01-dynamic-choice-sets.md).

**The measurement, which matters more than the code.** Trained over 8,336
tuples spanning two questions, with twelve rules held out entirely (their
corpus, `mlpl-repl 0.22.0`):

| Rows | full field (51 cards) | five cards, four trained | five cards, all cold |
|---|---:|---:|---:|
| cards that trained | 85.3% | 97.4% | 95.9% |
| cards held out entirely | **0.4%** | 36.5% | 18.6% (chance 20%) |

Generality costs about a point where candidates trained (85.3% against a
fixed head's 86.1% on the same rows). Where they did not, the right card
essentially never wins.

**What sw-atlas did about it.** Rewrote
[`hybrid-docent.md`](hybrid-docent.md) section 2: concepts became the bridge
that carries a newly indexed resource, every head is a fixed label set, and
reranking is confined to candidates the model trained on, with cold ones
keeping the matcher's order. Their caveat -- their cards share only function
words with their inputs, ours share content words -- is why this project
still owes its own measurement rather than adopting 0.4% as its own number;
their hold-out harness shape (full field, small warm field, small cold
field) is adopted for it.

Filed as the step `hybrid-design-after-dc01`. A request that came back with
a negative result was worth more than one that came back with a feature.

**Ask.** Deliver the queued PR05 lesson: a question-conditioned scorer
`score_i = f(h_state, h_question, h_choice_i)` trained over
`(state, question, choices, answer)` tuples, so the same weights score
choices they never saw, with the generality-cost comparison against the
fixed-head Choice that the plan already describes.

**Why it matters here.** sw-atlas's reranker chooses among resource
*cards* (title, abstract, aliases) that the matcher proposes. A fixed head
over resource ids would need retraining for every new blog post, and would
let a stale model name a resource, which sw-atlas's first invariant
forbids. A card scorer is the only shape under which "a post published
last night can be chosen without retraining" is possible. sw-atlas Saga 7
(SN01) measures whether it is actually true.

**Acceptance.** An mlplunit-tested trainer and inference pair in `lib/`
with no demo identifiers, parity against the Rust forward pass, and a
measured accuracy on choices held out of training.

**The fallback was withdrawn and stayed withdrawn.** An earlier version of
this file said sw-atlas would write the scorer itself if it got there first.
It did not, and upstream's version arrived with an evaluation this project
would not have thought to run on itself -- twelve candidates held out
*entirely*, then scored in a field where every alternative is equally
unfamiliar. That harness is the reusable part.

## TAG — a revision to pin (delivered)

**Status, 2026-09-22: delivered.** Annotated tag `tdm-v0.1.0`, commit
`b476842`, with the contract written up in that repository's
`docs/reference/crate-contract.md`. What it promises, and what sw-atlas may
therefore rely on:

- `tdm-model` reads decision bundles of version 2 and 3, and the bundle
  `version` field does not change within a tag.
- Parity tolerance `1e-9`, measured against MLPL's own probabilities in the
  bundle's `parity` block -- and measured on the rounded weights a consumer
  actually receives, not on the trainer's unrounded ones.
- `tdm-trace` parses `decision-trace-v1` and enforces the two invariants
  sw-atlas cares about: exactly three decision kinds, and output text
  reconstructible from a table the trace carries.
- Both crates build standalone with `serde` and `serde_json` as their only
  dependencies, edition 2024, MSRV 1.85, verified by a consumer crate built
  outside that repository at that revision.

The pin, for whichever sw-atlas crate first consumes it in Saga 3:

```toml
tdm-model = { git = "https://github.com/sw-ml-study/demo-decision-model", tag = "tdm-v0.1.0" }
tdm-trace = { git = "https://github.com/sw-ml-study/demo-decision-model", tag = "tdm-v0.1.0" }
```

Nothing here depends on them yet: sw-atlas has no crate that consumes a
bundle until Saga 3, so the dependency is recorded rather than added.

**One compatibility note to carry into Saga 3.** At `tdm-v0.1.0` a
version 3 bundle may arrive with an *empty* `keywords` and `match_order`,
and an empty `parity.matcher`: that is a bundle whose labels are another
program's rules, so it has no keyword yardstick of its own. A reader that
insists on a non-empty keyword list rejects a valid bundle. sw-atlas's A0
matcher is its own code and is unaffected, but anything reading a bundle's
yardstick must tolerate its absence.

**The original ask.** A tagged revision (a git tag is enough; no crates.io
release) at which `crates/tdm-model` and `crates/tdm-trace` build as
standalone dependencies. Keep the bundle `version` field stable within a
tag, and record in the tag's notes the parity tolerance the tests enforce
(today `1e-9`).

**Why.** sw-atlas depends on those crates through a pin. A tag makes it
legible in `Cargo.toml` and in `agentrail audit`, and tells a reader which
bundle versions a given sw-atlas snapshot can load.

## How the ask reaches that repository

Until 2026-09-22 this file was the only record of either request, and
nothing in demo-decision-model referenced it, so an agent working there
could not have known; both items travelled by the owner relaying them by
hand. **That is fixed from their side:** they now carry
`docs/reference/downstream-requests.md`, which records what was asked, what
was delivered and where the answer is. The channel is therefore two files
that cite each other rather than one file and a person, and the next ask
should be written here and expected to be read there -- while still saying
so out loud in a session summary, because neither file is a notification.

## AT01 — they measured our hybrid on our data, unasked

**2026-09-24, their step 024, commit `b78a2e1`, lesson `AT01`.** Nobody asked
for this. They took a read-only copy of this repository's corpus and question
sets -- 642 resources, 386 questions, 72 resources held out entirely -- and
measured the hybrid docent itself, as demo 02 of their own NV01 (a second
domain with `lib/` unchanged). It is the most useful thing this collaboration
has produced, and three of its four findings cost us work.

1. **A `NaN` defect we would have vendored.** `lib/scorer.mlpl` guarded its
   zero-norm case after `sqrt`: correct forward, `NaN` backward. One card whose
   every word is unknown turns every parameter into `NaN` on the first Adam
   step, and training still completes and prints numbers -- accuracy 0.000 with
   MRR exactly 1.000. Their twelve hand-written cards could not produce such a
   card; our 642-resource catalog does. Fixed upstream (epsilon inside the
   root), recorded as their finding Q6 with a probe. **Vendor at or after
   `b78a2e1`.**
2. **The rerank head does not work at 308 questions:** 0.023 warm, 0.032 cold,
   against 0.050 for random, training loss 0.011. Memorised, transferred
   nothing; warm equals cold, so the held-out cards are not the problem. Saga 3
   gains a synthetic-positives step before the rerank head.
3. **Our ceiling is our own recall@k**, 0.63 at k=20 with their stand-in
   matcher. MB02's recall@k is now the headline number of Saga 2.
4. **Intent beat nothing:** 0.697 against a 0.737 always-`FindResource`
   baseline, and the is-off-topic Noul exactly at its always-false baseline,
   because 72% of our questions are `FindResource`. The harness will print
   majority-class baselines beside every accuracy, and the sets need an
   intent-balanced supplement.

Their caveats, kept: the sets were `Unconfirmed` when they ran (confirmed
2026-09-24), the matcher was theirs and not MB02, one configuration per head,
no sweep.

## What their other measurements changed here

Not requests, and worth recording because they moved this project's own
claims:

- **LB01.** On 24 held-out spam and phishing messages, scored by the same
  bounded-choice method with nothing generated, a local `llama3.2:3b` got
  0.792 zero-shot against their trained model's 0.750, `gemma4:31b` got
  1.000, and the 3B model was better calibrated untouched (ECE 0.006) than
  theirs after temperature scaling (0.097). sw-atlas now states plainly
  what it claims -- offline, a few hundred kilobytes, no server, cannot
  invent an exhibit, over a corpus no pretrained model has seen -- and what
  it does not: beating a large model that has the catalog in its prompt.
  [`hybrid-docent.md`](hybrid-docent.md) section 2b.
- **In-browser training.** They ship a flattened program that trains in
  their Live Editor at 512 slots by 24 dimensions, 30 steps, reaching 0.81
  validation against the 0.879 of the model they ship. It is evidence that a
  model this size trains from scratch in seconds, which makes a
  train-it-yourself demo cheap here later; the nightly build stays the
  product.

## What sw-atlas gives back

- **A second demo for the abstraction gate.** NV01 ("demo 02: campus
  navigation, `lib/` unchanged") is close to what sw-atlas Saga 3 builds,
  over the whole corpus (several hundred resources) rather than 9 places.
  Whatever forced sw-atlas to change or extend the vendored `lib/` is
  reported here as a finding, with the diff.
- **Frozen evaluation sets and results.** sw-atlas's paraphrase, off-topic,
  meta and follow-up sets, labelled and hashed before training, plus the
  HT01 rows: matcher alone, model alone, hybrid, ablations, and paired
  intervals on every margin.
