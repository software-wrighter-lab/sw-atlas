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

**Answered 2026-09-22, relayed by the repository owner.** The tag exists
(see TAG below) and upstream owns PR05. Both items below record what was
asked and what came back; neither is an open ask any more.

## PR05 — dynamic choice sets (upstream owns it; Saga 3 step 3 waits)

**Status, 2026-09-22: accepted upstream and in progress.** The owner
relayed that demo-decision-model is mid-build on PR05 and that sw-atlas
must **not** write its own scorer. Saga 3 step 3 therefore waits on their
delivery instead of carrying a fallback, and sw-atlas contributes the
evaluation rather than the trainer.

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

**The fallback is withdrawn.** An earlier version of this file said that
sw-atlas would write the scorer itself if it got there first. It will not:
two implementations of one contract is the duplication these request files
exist to prevent.

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
could not have known. Both items travelled by the owner relaying them by
hand. Upstream is adding a request file on its own side so the next ask
survives without a relay; sw-atlas keeps this file as its half of the
record and should not assume it is read.

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
