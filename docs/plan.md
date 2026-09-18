# sw-atlas — plan

One semantic index over everything Software Wrighter publishes, and a tiny
model that turns a visitor's sentence into a typed decision about it. The
index holds the facts. The model holds only the language. Every site — the
campus, the blog, a live demo — mounts the same engine and gives it a
different name and face.

Distilled from [`sw-atlas-research.txt`](sw-atlas-research.txt), and
reconciled against what the sibling repositories actually contain today
(`sw-campus`, `blog`, `moe-microscope`, `sw-mlpl`, `sw-os-ml`).

Revised after [`needle.md`](needle.md): Cactus's Needle is an open,
MIT-licensed, 26M-parameter encoder-decoder with no feed-forward network,
built for exactly this shape of problem. It is the architecture this plan
was going to invent. Sections 5, 6, 9, 10 and 12 carry its consequences;
`needle.md` carries the reasoning and the assessment of the alternatives
that were rejected.

---

## 1. The thesis

A visitor asks a question. Almost all of the work of answering it can be
done *before* they ask.

```
        NIGHTLY — expensive, once, on our hardware
   ────────────────────────────────────────────────
   blog / campus / repos / videos
              |
              v
     ingest, relate, synthesize questions,
     embed, train, calibrate, regression-test
              |
              v
        published snapshot  (a few MB)

        BROWSER — cheap, every time, on theirs
   ────────────────────────────────────────────────
   question -> typed decision -> catalog lookup -> answer
```

A general assistant interprets, searches, reasons and generates on every
question. Atlas interprets and relates the corpus expensively once, then
classifies and retrieves cheaply many times. The research objective, stated
so it can be failed:

> How much offline computation can we spend to minimise online inference
> cost while keeping useful natural-language discovery over an evolving
> personal corpus?

The second thesis is the one that decides whether anything ships:

> A 10M-parameter model plus a 10–30 MB knowledge artifact should be a
> better navigator of this corpus than a 500M-parameter general model, and
> must first be a better navigator than 284 keyword signals.

It is not better today. `moe-microscope`'s deterministic matcher scores
0.685 on held-out paraphrases; the trained docent scores 0.407. That is the
starting scoreboard, not a footnote.

## 2. Name and roles

The repository is the engine, the corpus and the index. `Docent` is not the
system; it is one costume.

```
                    sw-atlas
                       |
        +--------------+--------------+
        |              |              |
      engine         corpus         model
        |
   +----+-----+-----------+-----------+----------+
   |          |           |           |          |
 Docent     Guide     Librarian     Tutor    Archivist
 museum     campus       blog     live demo    repos
```

A role is a UI persona: vocabulary, greeting, the default intent, which
resource kinds it prefers, how it says "I don't know". A role is data, not
code, and roles do not get their own knowledge. Asked from the blog whether
there is a demo for an article, the Librarian answers out of the same index
the Guide uses.

## 3. Ownership boundaries

| Repository | Owns | Does not own |
|---|---|---|
| **sw-atlas** | schema, ingest, nightly pipeline, training, evaluation, the published snapshot, the browser runtime crate | any site's UI, any mechanism research |
| `moe-microscope` | MoE mechanisms, tiered weights, expert cache, quantization — the *why it works* | the corpus, the product, the shipped snapshot |
| `sw-campus` | campus content and UI; mounts the runtime as Guide/Docent | the model, the index |
| `blog` | posts and front matter; mounts the runtime as Librarian | the model, the index |
| `sw-os-ml` | residency and known-next-use in a kernel | anything in a browser |

Two rules keep this honest. `moe-microscope` never grows a corpus; sw-atlas
never invents a mechanism. When a mechanism proves out in the microscope it
arrives here as a vendored, hash-pinned technique; when Atlas needs a
mechanism that does not exist yet, it files a work order there and ships
without it in the meantime.

The existing `moe-microscope` Saga 4 docent work is not superseded — it
becomes Atlas's first baseline and its first published result. CD00, CD01,
CD01b, MB01 and MB01t transfer as rows in the Atlas scoreboard.

## 4. Invariants

These do not change with any milestone. Each one is a test, not an
intention.

1. **The model never emits prose, a URL, or a resource ID.** It emits
   `intent`, `concepts[]`, `resource_kinds[]`, `confidence`. The catalog
   resolves those to places and links. A stale model therefore cannot
   invent an exhibit or a dead link — the worst it can do is route badly.
2. **Facts live in the snapshot, language lives in the weights**, and the
   architecture enforces it rather than the training policy. A network
   with no feed-forward layer has nowhere to memorise a fact; resources
   reach it as cards in its context, the way Needle receives a tool list.
   A status change (`planned` -> `working`), a new URL, an edited abstract
   rebuilds the index and retrains nothing. Whether a *new resource* also
   retrains nothing is the open question Saga 6 answers first.
3. **The page never waits for Atlas.** Campus content, blog content and
   demos render first. Atlas arrives progressively and can be absent.
4. **A0 always works and is never deleted.** The deterministic matcher is a
   permanent runtime class, not a fallback to be thrown away when the
   neural tier lands. It is currently the champion.
5. **A tier ships only when it beats the tier below it**, measured on the
   same held-out questions, and only while it stays inside its memory and
   latency budget. Budget violation is a failing test.
6. **Calibrated, not confident.** Ranked alternatives with probabilities
   beat a wrong single answer. Abstention is a correct output.

## 5. Runtime classes

The budgets are Atlas's own working set — not browser-process memory. They
are deliberately far below what browsers permit (ONNX Runtime documents a
~2 GB ArrayBuffer ceiling; that is a hardware boundary, not a licence).

| Class | RAM target | Transferred | Capability | What runs |
|---|---:|---:|---|---|
| **A0** Minimal | ≤ 10 MiB | ≤ 2 MB | lexical + graph matching | no model |
| **A1** Semantic | ≤ 25 MiB | 2–10 MB | embedding retrieval | precomputed vectors, cosine scan |
| **A2** Intelligent | ≤ 64 MiB | 10–30 MB | intent, ranking, confidence | encoder + contrastive head |
| **A3** Decoding | ≤ 128 MiB | 30–75 MB | arguments, comparison, rerank | encoder + decoder |
| **A4** Enhanced | ≤ 256 MiB | opt-in | prose synthesis | small generator |

**A2 at 64 MiB is the design target. A3 at 128 MiB is the self-imposed
ceiling.** A4 is never entered without an explicit click.

A2 and A3 are two heads on one artifact, not two models (§6). The split is
where the cost is: A2 is a single encoder pass plus a cosine scan, which is
bounded and fast; A3 decodes tokens autoregressively, which is neither. Most
questions — "where is APL?", "have you written about MoE?" — are ranking
questions and stop at A2. A3 is for questions whose answer carries
arguments: comparisons, status about a named subject, filtered finds.

Latency budgets, measured in the browser, not inferred:

| Operation | Target |
|---|---|
| page load attributable to Atlas | 0 ms |
| A0 match | < 10 ms |
| A1 retrieval | < 50 ms |
| A2 decision (encode + rank) | < 100 ms preferred, 200 ms limit |
| A3 decode (short typed record) | < 500 ms |
| weight shard fetch | asynchronous, never blocking |
| anything > 1 s | visible progress or do not do it |

### Selecting a class

`navigator.deviceMemory` is coarse, privacy-clamped and not universally
supported. It is one weak signal among several, never the selector.

```
         start at A0, always
                  |
         WASM SIMD? WebGPU? deviceMemory hint?
                  |
         20 ms microbenchmark in a worker
                  |
          propose a ceiling class
                  |
         promote one class at a time,
         only after the previous one is
         measured inside budget
                  |
         watch: p95 latency, jank, eviction rate
                  |
          over budget or slow -> demote
```

Demotion must work all the way back to A0 with no loss of correctness,
only of quality. The user-visible control is `Basic · Smart · Full` with
`Auto` as the default. Never "your machine has 16 GB, so I took 400 MB".

**CPU-only is a primary target.** A2 must be good without WebGPU. WebGPU is
an accelerator for A3/A4, never a requirement.

## 6. Stack and model

### The model

One encoder, two heads, no feed-forward network — the Simple Attention
Network shape that Needle demonstrates and `needle.md` assesses.

```
question                     top-k resource cards
   |                                  |
   v                                  |
 encoder  (self attn, GQA + RoPE,     |
           gated residual, NO FFN)    |
   |                                  |
   +--> contrastive head              |          A2
   |      mean pool -> 128d, L2       |     bounded cost
   |      cosine over resource vectors|     one forward pass
   |      -> ranked resources          |
   |                                  v
   +--> decoder (masked self attn, cross attn,   A3
                 gated residual, NO FFN)     unbounded cost
          -> typed Decision record            pay only when asked
```

No FFN is not an economy; it is the guarantee. Two thirds of a normal
transformer's parameters are feed-forward, and they are where facts get
memorised. Removing them makes invariant 2 structural: the model can align
a question to a card it is shown, and it cannot hold a fact it was not
shown. It also removes the mixture-of-experts story from this repository,
because MoE routes among FFN experts and there are none — see §11.

Starting configuration, to be revised by measurement, not by argument:

| | |
|---|---|
| vocabulary | 8,192 BPE |
| context | 128 tokens (query + prefiltered cards) |
| d_model | 256–512 |
| encoder / decoder layers | ladder, 2 to 12 encoder, 2 to 8 decoder |
| heads | 8 query / 4 key-value |
| parameters | 5–26M, INT4 or INT8 at rest |
| output | ranked resources (A2); typed `Decision` (A3) |

The depth ladder is how one checkpoint serves several runtime classes: a
shallower encoder for A2 on a small machine, full depth where it fits. That
replaces expert sharding as the mechanism behind the A-class ladder.

### The pipeline

| Layer | Choice |
|---|---|
| Ingest, corpus build | Rust CLI (`atlas-ingest`), RON/JSON output |
| Teacher enrichment | external model, called offline, output validated against a strict schema before it is allowed into the corpus |
| Training | **open decision 1** — see §12 |
| Export | packed `.atlas` weight file plus a manifest |
| Browser runtime | Rust/Yew/WASM crate, inference in a Web Worker |
| Content | RON in-repo, JSON published |
| Gate | `just check`: fmt, clippy, tests, `sw-checklist` zero failures zero warnings |

The house rules are unambiguous about what *ships*: `sw-campus` forbids
hand-written JavaScript and Python, `moe-microscope` holds zero `.py` files
across 165 sw-MLPL programs. Nothing here challenges that — the browser
runtime is Rust compiled to WebAssembly and the snapshot is data.

What the arrival of Needle does challenge is the *training* side. Needle is
JAX, its weights are open under MIT, and it is already pretrained on 200B
tokens. Porting the architecture to sw-MLPL before knowing whether it beats
a keyword matcher on this corpus spends months to answer a question that
open weights answer in days. §12 states the three options and the
recommendation; work proceeds on the default recorded there.

Two things are settled regardless:

- **We write the inference kernel in Rust.** An encoder-decoder with no
  FFN is attention, a norm, a gated residual and a dequant path. Writing it
  is less work than bridging ONNX Runtime Web from Yew, and it is the only
  way to control which bytes are resident when — the actual research
  question.
- **ONNX, Transformers.js and a 0.5B generative baseline stay external
  comparison rows.** Measured once, to answer "what would the conventional
  approach have cost?" Neither ships.

`sw-checklist` governs: ≤ 25 LOC per function, ≤ 4 functions per module,
≤ 4 modules per crate. Design to the gate — several small crates.

## 7. Data model

Five types. They are the contract between every milestone.

```rust
Resource {
    id: ResourceId,          // "blog:2026-09-17-tbt-apl-360-revisited"
    kind: ResourceKind,      // Blog | Campus | Repo | Video | Demo | Paper
    title, url, date,
    abstract_: String,       // <= 60 words
    concepts: Vec<ConceptId>,
    aliases: Vec<String>,
    maturity: Option<Maturity>,
    source_hash: String,     // what this was derived from
}

Concept {
    id: ConceptId,           // "mixture-of-experts"
    label, aliases: Vec<String>,
    parents: Vec<ConceptId>,
    resources: Vec<ResourceId>,
}

Relation {
    from: ResourceId,
    kind: RelationKind,      // Discusses | Implements | Demos | Cites
                             // | PartOf | RelatedTo | SeriesNext
    to: ResourceId,
    weight: f32,
    provenance: Provenance,  // Declared | Derived | Teacher
}

Question {                   // training data only, never published
    text: String,
    decision: Decision,
    origin: Origin,          // Authored | Template | Teacher | Adversarial
    split: Split,
}

Decision {                   // what the model emits
    intent: [f32; N_INTENT],
    concepts: SparseVec,
    resource_kinds: [f32; N_KIND],
    confidence: f32,
    ambiguity: f32,
}
```

Intents, extending the campus docent's six: `navigate`, `explain`,
`recommend`, `story`, `status`, `find_resource`, `compare`, `unsupported`.
`find_resource` and `compare` are what cross-corpus questions need and the
campus-only docent never had.

**`Provenance` is load-bearing.** A `Declared` relation came from front
matter the author wrote. A `Derived` one came from deterministic analysis.
A `Teacher` one came from a large model and is the only kind that may be
wrong in an interesting way. Every publish reports the mix, and any metric
can be recomputed with `Teacher` relations excluded.

## 8. What the corpus already is

The single most encouraging finding from the survey: most of this corpus is
already structured. The first ingest needs no model at all.

**Blog** — 124 posts in `blog/_posts`. Every one has `title`, `abstract`,
`categories`, `tags`, `keywords`, `author`, `date`. 123 have `series` and
`series_part`; 75 have `video_url`/`video_title`; 65 have `repo_url` and 10
more have `repo_urls`; 64 have `papers`; 10 have `demo_url`. The
`repo_url`, `video_url` and `demo_url` fields *are* declared cross-corpus
relations, already written by hand, already correct. `keywords` is a
hand-written alias list. `abstract` is a hand-written summary.

**Campus** — `sw-campus/pages/docent/snapshot-a.json`: 9 places with
kinds, parents, titles, taglines, status, links, aliases, concepts, 9
classified query groups, 21 stories, 9 ambiguous queries with expected
destination sets, 10 unsupported queries. The 1442 card reader and its
radio demo are deliberately excluded and reserved for snapshot B — that
exclusion is the incremental-learning experiment, and it must be preserved.
The campus plan already commits to writing `dist/catalog.json` canonically
with its SHA-256; Atlas consumes that, not the mockup's JSON, once it
exists.

**Repos and videos** — metadata only, to begin with: id, name,
description, topics, language, homepage, last update for a repository;
id, title, description, date, duration, URL, associated project for a
video. No source files, no transcripts. This keeps the claim precise:
"I know *about* this repository", not "I know what is in it". Transcripts
and selected repo documentation become richer source kinds later, behind
their own milestone.

So the first corpus is: 124 blog posts + 9–12 campus places (growing) +
the public repositories (the research note's illustration says 86; Saga 1
step 4 counts them) + ~75 videos — on the order of 300 resources. At 384-dimension INT8, 300
resource embeddings are 115 KiB. Even 10,000 chunks would be 3.7 MiB. The
index is not the expensive part; the query encoder is.

## 9. Published artifact

```
snapshot/2026-09-18/
  manifest.json          version, hashes per file, min runtime class
  catalog.json           resources: id, kind, title, url, abstract, concepts
  concepts.json          concept graph with aliases and parents
  relations.bin          packed relation edges
  signals.bin            A0 matcher signals (aliases, concepts, weights)
  embeddings.bin         INT8 resource embeddings + dims + scale
  model/
    core.atlas           header, tokenizer, embeddings, encoder L0-L3,
                         contrastive head          <- A2 needs only this
    encoder-deep.atlas   encoder L4-L11            <- A2 full depth
    decoder.atlas        decoder stack             <- A3
```

`.atlas` file layout — ours, not ONNX's external-data convention, because
the point is to control residency:

```
  offset  contents
  0x0000  magic "ATLS", format version, snapshot id, flags
  0x0020  tensor directory: name, offset, length, shape,
          quantization, scale/zero-point, checksum
  ....    payload, each tensor aligned to 4 KiB
```

A 4 KiB alignment is not aesthetic: it makes an HTTP `Range` request, a
`mmap` page and an OPFS read the same unit, so the browser trace and the
native trace in `moe-microscope` can be compared directly.

The shards are cut by *what a runtime class needs*, not by subject. That is
the change the no-FFN architecture forces: there are no domain experts to
shard by, but there is still a depth ladder, and a machine that can only
afford A2 never transfers the decoder.

Caching: Cache API / OPFS keyed by file hash. The manifest lists a hash per
file, so a nightly publish where only the index changed transfers only the
index. That is what makes a nightly cadence acceptable
to a repeat visitor.

## 10. Milestones

Each is one agentrail saga, each step one session, each ending with a
measured row in the results table and a `just check`-clean commit. IDs
follow the `moe-microscope` convention so the two scoreboards can be read
side by side.

### Saga 1 — foundation (`atlas-schema`)

1. **schema.** `atlas-core` crate: the five types, RON serde, a canonical
   writer with a content hash, cross-reference validation as a unit test.
2. **ingest-blog.** `atlas-ingest blog`: 124 posts from front matter alone.
   No model. Declared relations from `repo_url(s)`, `video_url(s)`,
   `demo_url`, `papers`, `series`.
3. **ingest-campus.** Snapshot A verbatim, then `dist/catalog.json` when
   the campus writes it. Preserves the 1442 exclusion explicitly.
4. **ingest-metadata.** Repositories and videos, metadata only, from the
   GitHub API and the blog's own video fields.
5. **corpus-report.** Coverage, printed and gated:

```
Knowledge coverage
──────────────────────────────
Blog posts          124 / 124
Campus places         9 / 9
Repositories         ?? / 86
Videos               ?? / 75

Resources tagged        ???
Resources related       ???
Orphan resources          ?      <- gate: must be 0 to publish
Broken URLs               ?      <- gate: must be 0 to publish
```

Exit: every public artifact has a Resource; every Resource is reachable
through concepts and relations; zero orphans, zero broken URLs. "Atlas
knows about everything" becomes a number.

### Saga 2 — the yardstick (`atlas-baseline`, **MB02**)

Port `moe-microscope`'s MB01/MB01t matcher from one campus to the whole
corpus, and build the evaluation harness before building anything to
evaluate.

1. **matcher.** A0 runtime: alias, concept, title and abstract signals with
   the MB01 weights, plus graph traversal over relations.
2. **eval-harness.** Held-out authored questions, a cross-corpus paraphrase
   set (no alias appearing verbatim), off-topic questions, ambiguous
   questions with expected sets. Reports intent accuracy, destination
   accuracy, top-3, MRR, unsupported recall, ambiguous top-2, ECE, Brier,
   p50/p95 latency, and bytes in all five senses.
3. **scoreboard.** `docs/reference/results.md`, seeded with the transferred
   rows:

| Run | Paraphrase dest | Intent | Unsupported recall | Stored | Class |
|---|---:|---:|---:|---:|---|
| MB01 campus matcher | 0.630 | 0.463 | 1.00 | 284 B | A0 |
| MB01t campus text matcher | **0.685** | 0.481 | 0.40 | 1010 B | A0 |
| CD01 dense docent | 0.296 | 0.481 | 0.30 | 26 KB | A2 |
| CD01b + word vectors | 0.407 | 0.407 | 0.30 | 26 KB | A2 |
| MB02 corpus matcher | — | — | — | — | A0 |

Exit: MB02 measured over ~300 resources. **This is the number every later
milestone must beat.** Adopt `moe-microscope`'s bar unchanged: +20 points
of paraphrase destination accuracy over the stronger matcher, +20 points of
intent accuracy, unsupported recall ≥ 0.8.

### Saga 3 — the Needle probe (**NP01**)

New, early, and the cheapest decisive experiment available. Upstream Needle
is MIT, its weights are open, and it runs on a Mac. Before building
anything, find out whether a tiny no-FFN model can do this job at all.

1. **probe.** Run upstream Needle unmodified, offline, with Atlas resource
   cards in place of tool definitions and the campus catalog as the tool
   list. Score its contrastive head (recall@1..5) and its decoder (exact
   match) on Saga 2's held-out questions and the MB01t paraphrase set.
2. **zero-shot-new-resource.** The 1442 card reader and its radio demo are
   excluded from snapshot A on purpose. Put the 1442 card in the context of
   a model that has never been trained on it and ask the withheld
   questions. This is the whole retraining question, answered in an
   afternoon.
3. **cost.** Measure what a 26M parameter model actually costs: bytes at
   bfloat16, INT8 and INT4; encoder-only latency against
   encoder-plus-decode latency; and the same numbers projected onto a
   WASM CPU path.

Exit: three rows in the scoreboard — the matcher, Needle's retrieval head,
Needle's decoder — and a recorded answer to "does an unmodified tiny model
beat 284 keyword signals on this corpus?". A *no* is as valuable as a yes
and is published either way. Everything after this saga is shaped by the
answer.

### Saga 4 — question generation (`atlas-questions`)

1. **templates.** Per resource and concept: navigation, explanation,
   status, story, recommendation, find-resource and compare templates over
   aliases and concepts. 300 resources × 20 templates × 5 aliases is
   ~30,000 rows before any model is involved.
2. **teacher.** Offline enrichment, schema-validated: vague references
   ("that little old RCA CPU", "the post about models thinking
   recursively"), cross-resource questions, adversarial near-misses,
   unanswerable questions. Needle's own generator synthesises *novel* tool
   definitions for about half its batches to force generalisation; do the
   same here with held-out resources, because that is what makes a new blog
   post work without retraining.
3. **ambiguity.** Questions with genuinely multiple correct answers, with
   the expected set. Trained as a distribution, never one-hot.
4. **hygiene.** Every generated row carries origin and provenance; the
   paraphrase and off-topic evaluation sets are frozen, never trained on,
   and checked for leakage against the training rows by hash.

Exit: a corpus with declared splits; leakage check clean; a human spot-read
of 100 teacher rows recorded with its error rate. Needle's guidance of ~120
examples per tool sets the floor: ~300 resources means ~36,000 rows at
minimum, which the templates alone supply.

### Saga 5 — the model (**AT01**, retrieval and decision in one)

The merged milestone: what were separate embedding and decision tiers are
two heads on one encoder.

1. **tokenizer.** Corpus-fitted BPE, 8,192, committed with a hash.
2. **train-or-finetune.** Per open decision 1: either finetune upstream
   Needle on the Saga 4 corpus, or train the ported architecture in-house.
   Same corpus, same eval, either way.
3. **retrieval-head (A2).** Contrastive head scored as recall@1..5 and MRR
   against MB02 and against Saga 3's unmodified-Needle row.
4. **decision-head (A3).** Typed `Decision` decoding with a constrained
   grammar, scored on exact match, and only for the intents that need
   arguments.
5. **export.** `core.atlas` + depth shards + manifest, INT8 and INT4, and an
   inference twin that reloads the export and asserts parity with the
   trained model on every corpus row.

Exit: AT01 in the scoreboard against MB02, EM-floor and NP01. Under budget:
A2 inside 64 MiB and 200 ms on CPU-only WASM; A3 inside 128 MiB and 500 ms.

### Saga 6 — new resources without retraining (**SN01**)

Saga 3 asked this of a model that had never seen the corpus. Ask it now of
the trained model, properly, with the machinery to act on the answer.

```
   trained on snapshot A
            |
            +--> 1442 card in context, withheld questions asked
            |
    zero-shot correct?
      /            \
    yes             no
     |               |
  publishing is   finetune on generated 1442 questions,
  an index        then measure: knowledge acquired,
  rebuild;        knowledge retained, forgetting,
  retraining      examples required, time, size delta,
  is deleted      calibration drift
  from the
  nightly path
```

Exit: a recorded answer and, whichever branch it takes, the nightly
pipeline's design decided by measurement rather than assumption.

### Saga 7 — calibration (**CB01**)

The most Jev-shaped part, and the one that makes the whole thing honest.
Needle optimises exact match; it does not optimise knowing when it is
wrong. The contrastive head's cosine margin is the raw material.

1. **measure.** Reliability diagram, ECE, Brier, per-intent.
2. **abstain.** Train the threshold; below it Atlas offers ranked
   alternatives or says it does not know.
3. **report.** Published in the snapshot manifest so the UI can show it.

```
confidence bucket   n     accuracy
0.9–1.0           ???        ???
0.8–0.9           ???        ???
0.7–0.8           ???        ???
```

Exit: ECE recorded; a 0.9 answer is right about 90% of the time or the
number is published showing that it is not.

### Saga 8 — the runtime crate (`atlas-runtime`)

1. **worker.** Inference off the main thread; message protocol.
2. **loader.** Manifest, hash-keyed Cache API/OPFS, Range fetch per shard,
   resumable, no blocking.
3. **classes.** The A0–A4 ladder over the depth shards, the capability
   probe, promotion, demotion, the `Basic · Smart · Full · Auto` control.
4. **budget-tests.** Memory and latency budgets asserted in a headless
   browser. Over budget fails the build.
5. **why.** The explanation panel: signals for A0, neighbours and cosine
   margins for A2, the decoded record and its grammar for A3.

Exit: one crate that a Yew app mounts in a few lines, with a role
parameter, that degrades to A0 on a 2015 laptop.

### Saga 9 — integration (`atlas-in-the-wild`)

1. **campus.** The Guide/Docent easel mounts `atlas-runtime`. Replaces the
   mockup matcher only when the bar is met; `?atlas=0` hides it.
2. **blog.** The Librarian: "have you written about X", "is there a demo",
   "what should I read next in this series".
3. **roles.** Role definitions as data; one shared snapshot; a test that
   the same question from two roles resolves to the same resource.

Exit: two sites, one snapshot, one runtime, three personas.

### Saga 10 — nightly (`atlas-nightly`)

Shaped by Saga 6. If new resources work without retraining, the normal path
is an index rebuild and training is the exception.

```
        git change detected
                |
        classify the change
           /          \
   factual or       semantic and
   new resource     zero-shot fails
        |               |
  rebuild index    regenerate questions
  (the normal      finetune candidate
   path)               |
        |               |
        +-------+-------+
                |
        old QA | new QA | adversarial QA | calibration
                |
           regression?
           /         \
         yes          no
          |            |
        reject      publish (hashes per file)
```

Rejection is a normal outcome and must leave yesterday's snapshot serving.

Exit: a night that adds a post and publishes only the changed files; a
night that rejects a regression and says why.

### Saga 11 — A4, optional prose (**GN01**)

Only now, and only opt-in. A small instruct model in a worker that receives
*retrieved facts only* and is instructed to use nothing else, for the
genuinely generative questions ("why is APL historically interesting
compared with Forth?"). Never downloaded before it is asked for. Measured
against the same harness so its cost is visible next to A2's.

### Removed: the MoE saga

The previous draft had a routed-experts milestone between the decision
model and calibration. It is gone, because mixture-of-experts routes among
feed-forward experts and this architecture has none. The research does not
disappear — it returns to `moe-microscope`, which is where it belongs, and
comes back here only if the no-FFN result disappoints. See §11.

## 11. Work orders for moe-microscope

Three asks, in the order Atlas needs them. None of them is a change this
repository can make; each is a request to be revalidated there.

**TW01 — tiered weights, reframed.** The microscope's queued experiment
(Saga 14, "quantization and the packed file with the expert cache") is
still the right work, but Atlas can no longer supply an expert bank as its
workload. What it can supply is a real *depth-sharded* model:

| Native | Browser |
|---|---|
| RAM — embeddings, shallow encoder, heads | WASM heap / GPU buffer |
| SSD via mmap — deep encoder layers | OPFS / Cache API |
| HDD / file — decoder stack | server over HTTP Range |

Measured both sides: shard bytes, hit rate, bytes read per query, load
latency, compute time, total latency, evictions. A MicroMoE expert is 1,072
parameters / 536 bytes at INT4 — too small to show I/O cost — so the
microscope still needs a scaled storage fixture; Atlas's shards are the
realistic size, and the residency question is unchanged by the architecture
that sits on top of it.

**SAN01 — is no-FFN real?** The claim that feed-forward layers can be
dropped entirely when the model relies on an external knowledge source is
exactly the kind of claim the microscope exists to check, at a scale where
every tensor can be printed. The campus docent corpus is already there, and
CD01/CD01b are dense-FFN models trained on it: strip the FFN, keep the
budget, and report what changes. If no-FFN loses on this corpus, Atlas
needs to know before Saga 5, not after.

**MOE-RETURN — the fallback.** Mixture-of-experts leaves the Atlas critical
path because there are no FFNs to route among (§6). That is a bet on
SAN01. The microscope keeps the MoE line of research as its own; if Atlas's
no-FFN model underperforms, the routed variant is the alternative already
built and measured, and it comes back.

The connection to `sw-os-ml` is not decorative. MLOS's open gate G4 is
"known-next-use beats LRU", and nothing writes `next_use` yet. A docent
session has unusually legible next-use structure: a visitor in the museum
asks museum questions. Atlas can emit a residency trace with the true next
use labelled afterwards — the first real workload for that policy, in a
browser rather than a kernel, but the same problem.

## 12. Open decisions

Flagged rather than assumed. Work proceeds on the stated default.

1. **How the model gets trained — the decision that shapes the year.**
   Needle is JAX, MIT-licensed, open-weighted, and already pretrained on
   200B tokens. The lab's standard is that the lab's tools build the lab's
   products, and `sw-campus` forbids Python in the pipeline.

   | | Approach | First number | Cost |
   |---|---|---|---|
   | **A** | Port the SAN to sw-MLPL/Rust, train in-house | months | house-pure, full ownership |
   | **B** | Finetune upstream Needle offline in JAX, export weights, write only the Rust inference path | days | Python offline; nothing Python ships |
   | **C** | Needle as an external baseline only; build A anyway | days for the number, months for the product | keeps the rule, spends the time |

   **Default: B, with A as the destination** — and Saga 3 deliberately
   costs nothing either way, because running upstream Needle unmodified is
   a measurement, not an adoption. The reasoning, and the counter-argument,
   are in [`needle.md`](needle.md) §5. This one is the repository owner's
   call, not an implementation detail; say the word and the default changes
   before Saga 3 closes.
2. **Model scale.** Needle is 26M at d=512. The earlier target was 5–10M
   for browser citizenship. Default: measure Needle's 26M first (13 MB at
   INT4 fits A3), then use the depth ladder to find the smallest
   configuration that still beats the matcher — rather than picking a size
   in advance.
3. **Corpus boundary.** Default: metadata only for repos and videos. Adding
   video transcripts (75 videos) would likely be the single largest
   knowledge gain available, and is deferred, not rejected.
4. **Teacher model.** Which one, run where, at what cost per night, and
   whether its output is committed to the repository for reproducibility.
   Default: committed, hash-pinned, regenerable.
5. **Publishing host.** Where the snapshot is served from, given CORS and
   Range requests from two different origins (`campus.softwarewrighter.com`
   and `blog.softwarewrighter.com`). Default: one Pages repository serving
   the snapshot with permissive CORS.

## 13. Risks

| Risk | Signal | Response |
|---|---|---|
| The matcher keeps winning | AT01 below MB02 again | that *is* the finding; publish it, keep A0, stop at A1 |
| No-FFN does not hold on this corpus | AT01 far below its dense twin; SAN01 negative | the MoE variant returns from moe-microscope; §11 |
| Decoder too slow in WASM | A3 over 500 ms on CPU | A2 is the product, A3 is opt-in; the split in §5 already assumes this |
| Teacher questions are unlike real ones | high eval scores, poor live behaviour | log real queries (locally, opt-in), measure the gap |
| Budgets quietly slip | A2 creeping past 64 MiB | budgets are tests; the build fails |
| Nightly churn annoys visitors | large transfers on repeat visits | per-file hashes; measure bytes per returning visit |
| Scope | 11 sagas | Sagas 1–2 have standalone value: a validated corpus and a cross-corpus matcher are a working site feature with no model at all, and Saga 3 is one day |

## 14. Non-goals

- No general chatbot. No open-domain questions.
- No prose from the weights, at any tier below A4.
- No Python, no hand-written JavaScript, no ONNX in the shipping path.
- No repository source ingestion, no video transcription, in the first
  corpus.
- No server-side inference. Everything after the nightly build runs on the
  visitor's machine.
- No reproduction of Jev or RLCD. TypeSafe has not published enough to
  build it. What is borrowed from them is the shape — AI as a fast
  probabilistic decision inside ordinary reliable software — not the
  architecture. The architecture comes from Needle, which is open, and is
  credited as such wherever this work is published.
- No vector database, no knowledge-graph inference engine, no serving
  framework. 300 resources at INT8 is 115 KiB and the relations are
  hand-declared in front matter; [`needle.md`](needle.md) §6 records what
  was considered and why each was rejected.

## 15. First session

Not training, and not Needle. Saga 1 step 1–2: the `atlas-core` schema and
the blog ingester, because 124 posts of hand-written front matter can
become a validated corpus today with no model, no GPU and no teacher — and
because until the corpus exists there is nothing for a model to earn the
right to replace, and nothing to measure Needle against.

The whole of Sagas 1 and 2 is a useful product on its own: a validated
index of every public Software Wrighter artifact, and a cross-corpus
matcher that answers "where did you write about X?" in under 10 ms and
under 10 MiB, on any device, with no model at all. Everything after it has
to be better than that.
