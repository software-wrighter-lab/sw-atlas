# Requests to `moe-microscope`

Work orders for agents operating in
[`sw-ml-study/moe-microscope`](https://github.com/sw-ml-study/moe-microscope).
This document does not authorize changes from this repository, and sw-atlas
cannot make them: every item here is a request to be revalidated against
that repository's own artifacts before its saga begins.

Raised by sw-atlas Saga 1. Reasoning is in [`plan.md`](plan.md) section 11
and [`needle.md`](needle.md).

## SAN01 — is no-FFN real? (highest value, blocks sw-atlas Saga 5)

**Ask.** Take CD01 or CD01b, remove the feed-forward network, keep the
parameter budget matched, retrain on the campus docent corpus that is
already in that repository, and report what changes.

**Why it matters here.** sw-atlas has adopted the Simple Attention Network
shape on the strength of Cactus's claim that MLPs can be dropped entirely
when a model relies on an external knowledge source. That claim is load
bearing: it is what turns "facts live in the snapshot" from a training
policy into an architectural guarantee, and it is why mixture-of-experts
left the Atlas critical path. It has not been checked at a scale where
every tensor can be printed, and that is exactly what the microscope is
for.

**What would make it decisive.** The existing CD01/CD01b rows are the
control. A no-FFN twin at matched stored parameters, on the same corpus,
same split, same held-out paraphrase set, reported in the docent results
table alongside them. Both outcomes are useful. If no-FFN holds, sw-atlas
proceeds with a stronger justification than a vendor's design note; if it
loses, sw-atlas needs to know before Saga 5 rather than after.

**Timing.** Before sw-atlas Saga 5. sw-atlas Sagas 1 to 4 do not depend on
the answer.

## TW01 — tiered weights, reframed

**Ask.** The queued Saga 14 experiment (quantization, the packed file, the
LRU expert cache with capacity curves) is still the right work, but the
workload sw-atlas can supply has changed shape.

**What changed.** Mixture-of-experts routes among feed-forward experts, and
the adopted architecture has none, so sw-atlas cannot provide an expert
bank. What it will provide is a real depth-sharded model:

| Native tier | Browser equivalent | Holds |
|---|---|---|
| RAM | WASM heap, GPU buffer | embeddings, shallow encoder, heads |
| SSD via mmap | OPFS, Cache API | deep encoder layers |
| HDD, cold file | server over HTTP Range | the decoder stack |

The residency question is unchanged by the architecture sitting on top of
it: stored bytes, resident bytes and active bytes still differ, and the
cache policy still decides what a query costs.

**Why sw-atlas is a better workload than a synthetic one.** A MicroMoE
expert is 1,072 parameters, 536 bytes at INT4 — too small for I/O latency
to be visible, which is why a scaled storage fixture was already needed.
sw-atlas's shards are realistically sized because they have to be: the
budget is a visitor's browser.

**File format.** sw-atlas proposes 4 KiB alignment for every tensor in the
packed file, so that an HTTP Range request, an mmap page and an OPFS read
are the same unit and the browser trace can be compared directly against
the native one. If the microscope's format differs, sw-atlas will follow
it rather than fork it.

## MOE-RETURN — hold the fallback

**Ask.** Keep the mixture-of-experts line of research as the microscope's
own. No work is requested now.

**Why.** sw-atlas dropping MoE from its critical path is a bet on SAN01. If
that check comes back negative, the routed variant is the alternative —
already built, already measured, already explained — and it returns to the
Atlas plan as the model rather than the fallback.

## What sw-atlas gives back

Not a request, a statement of the exchange:

- The CD00, CD01, CD01b, MB01 and MB01t rows are carried into the Atlas
  scoreboard with their origin recorded beside them, so the docent work is
  the first published result of a larger project rather than a paused saga.
- The MB01/MB01t matcher is being ported from one campus to a corpus of
  roughly 300 resources; whatever that reveals about the matcher's
  behaviour at scale comes back here.
- The 1442 snapshot-A/snapshot-B experiment is run as designed, and its
  result is reported in both repositories.
