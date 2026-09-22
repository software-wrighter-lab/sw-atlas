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
vendors the MLPL library hash-pinned and pins the Rust crates to a
revision. It edits nothing there.

## PR05 — dynamic choice sets (blocks sw-atlas Saga 3 step 3)

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
measured accuracy on choices held out of training. If sw-atlas reaches
Saga 3 step 3 first, it writes the scorer itself against the same
contract and reports back what it found, so neither side is blocked.

## TAG — a revision to pin

**Ask.** A tagged revision (a git tag is enough; no crates.io release) at
which `crates/tdm-model` and `crates/tdm-trace` build as standalone
dependencies. Keep the bundle `version` field stable within a tag, and
record in the tag's notes the parity tolerance the tests enforce (today
`1e-9`).

**Why.** sw-atlas depends on those crates through a pinned git revision.
A tag makes the pin legible in its `Cargo.toml` and in `agentrail audit`,
and tells a reader which bundle versions a given sw-atlas snapshot can
load.

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
