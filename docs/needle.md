# Needle, Simple Attention Networks, and what they change

Assessment of the open-source alternatives raised in
[`jev-os-research.txt`](jev-os-research.txt), against the architecture in
[`plan.md`](plan.md). Read against a local clone of
[cactus-compute/needle](https://github.com/cactus-compute/needle) (MIT,
JAX/Flax, 9,335 lines of Python, 52 MB bfloat16 checkpoint).

The short answer: **Needle matters, and the rest mostly do not.** Needle is
the same architecture Atlas was going to invent, already built, already
trained, already measured, under a licence that allows anything. It
simplifies the plan in four places and makes one of its assumptions
testable much sooner and much more cheaply.

## 1. What Needle is

A 26M-parameter encoder-decoder for function calling. Per its README:
distilled from Gemini 3.1, pretrained on 200B tokens across 16 TPU v6e in
27 hours, post-trained on 2B tokens of single-shot function-call data in 45
minutes. Weights are open; the data generator, trainer, evaluator and a
finetuning UI are in the repository. These are the authors' claims, not
numbers reproduced here.

```
d=512, 8H/4KV, BPE=8192, max_seq_len=128

  query text                    tools JSON
      |                              |
  Embedding  <--- shared --->   Embedding
      |                              |
  Encoder x12                        |
  (self attn, GQA+RoPE,              |
   gated residual, NO FFN)           |
      |         \                    |
      |          \                   v
      |           \            Decoder x8
      |            \           (masked self attn,
      |             `--------->  cross attn,
      |                          gated residual,
      v                          NO FFN)
 contrastive head                    |
 (mean pool -> 128d,                 v
  L2 normalised)              typed tool call
      |                       {"name":..., "arguments":{...}}
      v
 top-k tool retrieval
```

Two heads on one encoder: a CLIP-style contrastive head that ranks
candidate tools by cosine similarity, and a decoder that emits the call.
The design notes say why there is no feed-forward network: *"MLPs can be
completely dropped from transformer networks, as long as the model relies
on an external knowledge source."*

## 2. Why this is the Atlas architecture

Needle's tool list is Atlas's catalog. The correspondence is not an
analogy; it is the same problem with different nouns.

| Needle | Atlas |
|---|---|
| tools JSON passed in context | resource cards from the published snapshot |
| no FFN, so no memorised facts | invariant 2: facts in the snapshot, language in the weights |
| contrastive tool-selection head | the A1 retrieval tier |
| decoder emits a typed tool call | `Decision` |
| constrained decoding over the schema | schema-valid decisions by construction |
| INT4 QAT during training | the quantisation step, already solved |
| selectable depth | the A0-A4 ladder from one checkpoint |
| finetune on 120 examples per tool | Saga 3's generated questions |
| ~50% of training batches use *synthesised novel tools* | new resources may need no retraining at all |

The last row is the one with consequences.

## 3. Four changes to the plan

### 3.1 "Facts in the snapshot" stops being a discipline

The plan asserted that the model must not learn facts, and enforced it by
choosing what to train on. Needle enforces it *architecturally*: with no
feed-forward network there is nowhere to memorise. The design notes argue
that attention alone is the right primitive because routing a query to a
target is alignment and copying, not per-position feature transformation,
and that at this scale FFN parameters -- about two thirds of a normal
transformer -- are wasted on a structured task.

If that holds for Atlas, a stale model cannot invent an exhibit *because
it cannot hold one*, which is a far stronger guarantee than a training
policy.

### 3.2 MoE leaves the critical path

This is the uncomfortable one, and it follows directly. Mixture-of-experts
routes among *FFN experts*. A network with no FFN has no experts to route
among. Saga 6's expert sharding and the knowledge-shard scheme in the
published artifact both assumed a dense-FFN model.

So: **MoE comes off the Atlas critical path and stays in
`moe-microscope`, which is where the research belongs.** Atlas keeps the
tiered-weight handoff -- an encoder-decoder with 20 attention blocks still
has separable layers, and the A-class ladder can load depth incrementally
rather than experts. The residency question survives; the expert framing
does not. If the no-FFN result does not hold for this corpus, the MoE
variant comes back as the alternative it always was.

### 3.3 A1 and A2 merge into one artifact

The plan had an embeddings tier (EM01) and a separate decision transformer
(AT01), each trained, exported and budgeted on its own. Needle gets both
from one encoder: the contrastive head is the retrieval tier, the decoder
is the decision tier, and the decoder is only paid for when it runs.

```
question
   |
   v
encoder  (one pass, always)
   |
   +--> contrastive head -> cosine over resource vectors -> top-k     [A1]
   |                                                          |
   |                                                          v
   +--> decoder over query + top-k resource cards -> typed Decision   [A2]
                                                    (only when needed)
```

Most questions stop at the first head. "Where is APL?" is a ranking
problem; it needs no decoding at all. Decoding is for questions whose
answer has arguments -- comparisons, status about a named subject,
filtered finds.

That split is also what makes the latency budget survivable. Cactus reports
6,000 tokens/s prefill and 1,200 decode in their native runtime; WASM on a
CPU will be far slower, and a 30-token JSON answer at, say, 100 tokens/s is
300 ms -- over the plan's 200 ms A2 limit. One encoder pass plus a cosine
scan is not. **A2 is the contrastive head; the decoder is A3.**

The 128-token context window forces the same shape: the whole catalog
cannot be in context, so the contrastive head has to prefilter. Needle's
design notes describe exactly this use ("when the tool set is large and you
want to filter to the top-k most relevant tools").

### 3.4 The retraining question gets answered early and cheaply

The plan's Saga 9 was: publish snapshot B with the 1442 card reader,
retrain, and measure what was learned against what was forgotten. That
assumed a retrain is how new knowledge arrives.

Needle's tools are supplied at inference time, and its data generator
deliberately synthesises *novel* tool definitions for about half of its
training batches so the base model handles tools it has never seen. If that
transfers, then a new blog post is a new card in the index and **nothing is
retrained at all**.

So the experiment inverts, and gets better:

```
   snapshot A model, unchanged
            |
            +--> ask the 1442 questions with the 1442 card in context
            |
    zero-shot correct?
      /            \
    yes             no
     |               |
  publish is      finetune on generated
  an index        1442 questions, then
  rebuild;        measure forgetting as
  nightly         originally planned
  retraining
  is deleted
```

Either outcome is a publishable result, one of them deletes a saga's worth
of machinery, and both are cheaper than retraining first and asking later.
This also reshapes the nightly pipeline: the interesting branch becomes
"does the index alone suffice?", with training the exception.

## 4. What Needle does not change

- **The bar.** 0.685 paraphrase destination accuracy from `moe-microscope`'s
  MB01t matcher. A 26M-parameter model distilled from Gemini is not exempt;
  if it cannot beat 284 keyword signals on this corpus, that is the finding.
- **The catalog is the authority.** Needle returns a name and arguments, not
  prose and not a URL. Deterministic code still resolves them.
- **The A-class budgets.** 52 MB at bfloat16, ~26 MB at INT8, ~13 MB at
  INT4. That fits A2/A3 and does not fit A0 or A1, so the matcher and a
  small embedding path remain the low tiers. Budgets stay tests.
- **Calibration.** Needle optimises exact match. Atlas needs to know when it
  does not know; the contrastive head gives a margin to calibrate, but the
  work in Saga 7 is unchanged.

## 5. The one real conflict: Needle is Python

**Resolved 2026-09-18 by the repository owner: no Python. Rust and/or
sw-MLPL only.** The section below is kept as the reasoning that was put to
that decision, not as a live recommendation.

Needle is JAX/Flax. `sw-campus` forbids Python "anywhere in the pipeline",
and `moe-microscope` has zero `.py` files across 165 sw-MLPL programs.
Three ways to resolve it were put forward:

| | Approach | First number in | Cost |
|---|---|---|---|
| **A** | Port the SAN forward and backward pass to sw-MLPL/Rust; train in-house | months | full ownership, house-pure, no borrowed weights |
| **B** | Finetune upstream Needle offline in JAX, export weights, write only the Rust *inference* path | days | Python in the offline pipeline; nothing Python ships |
| **C** | Needle as an external measured baseline only; build A as planned | days for the number, months for the product | keeps the rule, spends the time anyway |

The recommendation made here was B first, A as the destination, on the
argument that open weights already trained on 200B tokens answer the
project's gating question in days rather than months. **The owner chose A**:
the lab's tools build the lab's products, and a JAX dependency in the
nightly pipeline — even one that ships nothing — is not the trade this
project is willing to make. That is the decision; the counter-argument was
heard and rejected on its merits.

What survives the decision is everything in §1 to §4: the architecture, the
no-FFN guarantee, the two heads, the merged milestone, and the reframed
generalisation experiment. What is lost is the cheap measurement. Three
consequences follow, and they belong to Saga 3:

1. **A Rust forward pass comes first.** No FFN means the whole inference
   path is attention, ZCRMSNorm, a gated residual, RoPE, a cross-attention
   block and a dequant path. That was Saga 8 work; it moves forward.
2. **The published checkpoint is a pickle.** `needle.pkl` is 52 MB of
   pickled JAX arrays. Reading it from Rust, reading whatever format the
   Hugging Face copy offers instead, or declaring the weights unreachable
   and training from scratch on this corpus are three different projects;
   Saga 3 step 1 is to determine which one it is and record the answer.
3. **The design notes are the reusable artifact, not the code.** What
   `simple_attention_networks.md` documents — why the FFN can go, why
   encoder-decoder, gated residuals initialised at sigmoid(0), ZCRMSNorm,
   Muon for an attention-only stack, INT4 QAT as regularisation, token-level
   loss weighting — is a specification precise enough to reimplement from,
   under a licence that permits it, and it is cited wherever this work is
   published.

## 6. The rest of the field, and why it is mostly rejected

From [`jev-os-research.txt`](jev-os-research.txt):

| Project | Verdict | Why |
|---|---|---|
| **SetFit** | adopt as an offline teacher, if anything | Few-shot contrastive finetuning of a sentence encoder, retrainable on CPU in seconds. That is genuinely the cheapest possible A1. But it is Python/HuggingFace, and Needle's contrastive head gives the same thing from a model we already need. Worth one measurement as a floor. |
| **Outlines / SGLang** | not needed | Grammar-constrained decoding matters when a general LLM must be forced into a schema. Atlas's output is a ranked list plus a small typed record; Needle already ships `constrained.py`. SGLang's RadixAttention is a server-side concern and there is no server. |
| **LanceDB** | reject, with a number | 300 resources at 384-dimension INT8 is 115 KiB. Ten thousand chunks would be 3.7 MiB. A flat cosine scan over that in WASM is faster than any index structure and needs no dependency. Revisit at ~100k chunks, which this corpus will not reach. |
| **HippoRAG 2 / A-Mem** | reject for now | Both infer a knowledge graph from text. This corpus does not need inference: 65 posts declare `repo_url`, 75 declare `video_url`, 10 declare `demo_url`, 64 declare `papers`, 123 declare a series. The relations are hand-written and correct. Revisit if derived relations ever outnumber declared ones. |
| **LLMRouter** | reject, note the confirmation | KNN and SVM routing backends over an embedding cache -- precisely the A1 baseline, in a box. Nothing to adopt; it is a useful independent confirmation that the cheap tier is the right first tier. |
| **MobileBERT / TinyBERT** | keep as comparison rows | 10-30 MB encoders for classification. A fair baseline for A1/A2 and a sanity check that a 26M no-FFN model is worth its weight. |

The pattern across all of them, and the reason the plan does not change
more than it does: they are retrieval and serving infrastructure for
corpora far larger and far more volatile than this one. Atlas's corpus is
~300 hand-curated artifacts that change a few times a week. The scarce
resource is not index throughput. It is the visitor's RAM.

## 7. What this looks like in the saga queue

- Saga 4 (embeddings) and Saga 5 (dense decision model) **merge** into one
  saga producing one encoder with two heads.
- Saga 6 (MoE) **leaves**, and becomes a `moe-microscope` work order plus a
  fallback here if no-FFN disappoints.
- Saga 9 (snapshot A to B) **runs earlier and smaller**: zero-shot first,
  finetuning only if zero-shot fails.
- Saga 11 (nightly) **shrinks**: index rebuild is the normal path, training
  the exception, and the quality gate guards both.
- A new, early, cheap saga appears ahead of all of them: **run upstream
  Needle against the campus corpus and the MB01t questions, unmodified, and
  put the number in the table.** It costs a day and it decides the shape of
  everything after it.
