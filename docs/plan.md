# sw-atlas — plan

One semantic index over everything Software Wrighter publishes, and a tiny
model that turns a visitor's sentence into a typed decision about it. The
index holds the facts. The model holds only the language. Every site — the
campus, the blog, a live demo — mounts the same engine and gives it a
different name and face.

Distilled from [`sw-atlas-research.txt`](sw-atlas-research.txt), and
reconciled against what the sibling repositories actually contain today
(`sw-campus`, `blog`, `moe-microscope`, `sw-mlpl`, `sw-os-ml`).

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
2. **Facts live in the snapshot, language lives in the weights.** A status
   change (`planned` -> `working`), a new URL, an edited abstract rebuilds
   the index and retrains nothing. A new concept, a new alias, a new
   relationship kind is what earns a retrain.
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

| Class | RAM target | Transferred | Capability | Model |
|---|---:|---:|---|---|
| **A0** Minimal | ≤ 10 MiB | ≤ 2 MB | lexical + graph matching | none |
| **A1** Semantic | ≤ 25 MiB | 2–10 MB | embedding retrieval | tiny encoder |
| **A2** Intelligent | ≤ 64 MiB | 10–30 MB | intent / concept / ranking | dense decision |
| **A3** Routed | ≤ 128 MiB | 30–75 MB | routed experts, reranking | MoE + shards |
| **A4** Enhanced | ≤ 256 MiB | opt-in | prose synthesis | small generator |

**A2 at 64 MiB is the design target. A3 at 128 MiB is the self-imposed
ceiling.** A4 is never entered without an explicit click.

Latency budgets, measured in the browser, not inferred:

| Operation | Target |
|---|---|
| page load attributable to Atlas | 0 ms |
| A0 match | < 10 ms |
| A1 retrieval | < 50 ms |
| A2 decision | < 100 ms preferred, 200 ms limit |
| A3 rerank | < 500 ms |
| expert shard fetch | asynchronous, never blocking |
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

## 6. Stack

This is where the research note and the house standards disagree, and the
house standards win.

The research proposed PyTorch → safetensors → ONNX → ONNX Runtime Web.
But `moe-microscope` contains zero `.py` files and 165 sw-MLPL programs;
`sw-campus` states "no hand-written JavaScript beyond what Trunk emits; no
Python anywhere in the pipeline"; sw-MLPL is a Rust-implemented ML language
with autograd, MLX on Apple Silicon, CUDA via Candle, and WASM execution.
Adopting a Python/ONNX pipeline would make sw-atlas the only repository in
the lab that cannot be built with the lab's own tools.

| Layer | Choice |
|---|---|
| Ingest, corpus build | Rust CLI (`atlas-ingest`), RON/JSON output |
| Teacher enrichment | external model, called offline, output validated against a strict schema before it is allowed into the corpus |
| Training | sw-MLPL, MLX/CUDA backends, batch, native |
| Export | packed `.atlas` weight file written by MLPL, plus a manifest |
| Browser runtime | Rust/Yew/WASM crate, inference in a Web Worker |
| Content | RON in-repo, JSON published |
| Gate | `just check`: fmt, clippy, tests, `sw-checklist` zero failures zero warnings |

Two consequences worth stating plainly:

- **We write the inference kernel.** A 5–10M INT8 encoder with four to
  eight experts is a few matmuls, a softmax, a router and a quantised
  dequant path. Writing it in Rust is less work than bridging ONNX Runtime
  Web from Yew, and it is the only way to control *which bytes are resident
  when* — which is the actual research question.
- **ONNX and Transformers.js stay as external comparison baselines.**
  Measured once, in a scratch harness, to answer "what would Qwen2.5-0.5B
  Q4 on WebGPU have cost us?" That number belongs in the results table.
  Neither ships.

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
    core.atlas           header, tokenizer, embeddings, shared body, heads
    experts/
      ml.atlas
      history.atlas
      languages.atlas
      systems.atlas
      hardware.atlas
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

Caching: Cache API / OPFS keyed by file hash. The manifest lists a hash per
file, so a nightly publish where only the ML expert and the index changed
transfers only those two. That is what makes a nightly cadence acceptable
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

### Saga 3 — question generation (`atlas-questions`)

1. **templates.** Per resource and concept: navigation, explanation,
   status, story, recommendation, find-resource and compare templates over
   aliases and concepts. 300 resources × 20 templates × 5 aliases is
   ~30,000 rows before any model is involved.
2. **teacher.** Offline enrichment, schema-validated: vague references
   ("that little old RCA CPU", "the post about models thinking
   recursively"), cross-resource questions, adversarial near-misses,
   unanswerable questions. Target 100K–500K rows.
3. **ambiguity.** Questions with genuinely multiple correct answers, with
   the expected set. Trained as a distribution, never one-hot.
4. **hygiene.** Every generated row carries origin and provenance; the
   paraphrase and off-topic evaluation sets are frozen, never trained on,
   and checked for leakage against the training rows by hash.

Exit: a corpus with declared splits; leakage check clean; a human spot-read
of 100 teacher rows recorded with its error rate.

### Saga 4 — A1, embeddings (**EM01**)

1. **offline-embed.** Resource embeddings computed by a teacher, quantised
   to INT8, packed.
2. **query-encoder.** A small encoder trained to match the teacher's
   resource embeddings (distillation onto the index, not general semantic
   similarity), exported and run in WASM.
3. **retrieval.** Top-k cosine over `embeddings.bin` in a worker.

Exit: top-1, top-3, MRR against MB02 on the same sets; A1 inside 25 MiB and
50 ms. **If A1 already beats the bar, say so loudly** — it would mean the
decision transformer is unnecessary for retrieval and only needed for
intent, and the plan should shrink accordingly.

### Saga 5 — A2, the decision model (**AT01**)

Dense first, MoE never before dense is understood.

```
vocab            4K–8K
context          128 tokens
embedding        128–192
encoder blocks   2–4
heads            4
shared FFN       512
heads out        intent | concepts | resource_kinds | confidence
parameters       5–10M target, INT8
```

1. **tokenizer.** Corpus-fitted, small vocabulary, committed with a hash.
2. **train.** sw-MLPL, native batch, recorded run.
3. **decode.** `Decision` + the deterministic resolution step: concepts and
   kinds → ranked resources through the index.
4. **export.** `core.atlas` + manifest + an MLPL inference twin asserting
   parity with the trained model on every corpus row.

Exit: AT01 in the scoreboard against MB02 and EM01. Under budget:
≤ 64 MiB resident, ≤ 200 ms per query on CPU-only WASM.

### Saga 6 — A3, routed experts (**AT02**)

1. **moe.** Router over 4 then 8 experts, top-1 and top-2, balance term;
   technique vendored from `moe-microscope` (MX01/MX02/LD01), not
   reinvented.
2. **specialization.** Do experts actually separate by campus domain —
   history, architecture, languages, ML, systems? Measured, not asserted.
   If they do not, the knowledge-shard split in §9 loses its basis and the
   plan says so.
3. **shards.** Experts split into separately fetchable `.atlas` files;
   router decides what to fetch; LRU with an eviction trace.
4. **residency.** Report stored / resident / active / transferred /
   executed per query — `moe-microscope`'s RB01 vocabulary, unchanged.

Exit: stored > resident > active demonstrated in a browser with real
numbers, inside 128 MiB.

### Saga 7 — calibration (**CB01**)

The most Jev-shaped part, and the one that makes the whole thing honest.

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
2. **loader.** Manifest, hash-keyed Cache API/OPFS, Range fetch per tensor,
   resumable, no blocking.
3. **classes.** The A0–A4 ladder, the capability probe, promotion,
   demotion, the `Basic · Smart · Full · Auto` control.
4. **budget-tests.** Memory and latency budgets asserted in a headless
   browser. Over budget fails the build.
5. **why.** The explanation panel: signals for A0, neighbours for A1,
   intent and concept distributions for A2, router weights for A3.

Exit: one crate that a Yew app mounts in a few lines, with a role
parameter, that degrades to A0 on a 2015 laptop.

### Saga 9 — snapshot A → B (**SN01**)

The experiment the campus already set up by withholding the 1442.

```
Snapshot A            ask: "where can I hear the card reader play music?"
   |                  expect: low confidence / unsupported
   | publish 1442 + radio demo
   v
Snapshot B            ask again
                      expect: navigate -> ibm-1442, high confidence
                      and: "where can I try APL?" unchanged
```

Measured: new knowledge acquired, old knowledge retained, catastrophic
forgetting, examples required, training time, size delta, calibration
drift.

Exit: a table with those eight numbers. This is the result the blog post is
about.

### Saga 10 — integration (`atlas-in-the-wild`)

1. **campus.** The Guide/Docent easel mounts `atlas-runtime`. Replaces the
   mockup matcher only when the bar is met; `?atlas=0` hides it.
2. **blog.** The Librarian: "have you written about X", "is there a demo",
   "what should I read next in this series".
3. **roles.** Role definitions as data; one shared snapshot; a test that
   the same question from two roles resolves to the same resource.

Exit: two sites, one snapshot, one runtime, three personas.

### Saga 11 — nightly (`atlas-nightly`)

```
        git change detected
                |
        classify the change
           /          \
   factual only    semantic
        |               |
  rebuild index    regenerate questions
        |          retrain candidate
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

The classifier matters as much as the trainer: a status flip or a URL edit
must cost zero GPU. Rejection is a normal outcome and must leave yesterday's
snapshot serving.

Exit: a night that adds a post and publishes only the changed shards; a
night that rejects a regression and says why.

### Saga 12 — A4, optional prose (**GN01**)

Only now, and only opt-in. A small instruct model in a worker that receives
*retrieved facts only* and is instructed to use nothing else, for the
genuinely generative questions ("why is APL historically interesting
compared with Forth?"). Never downloaded before it is asked for. Measured
against the same harness so its cost is visible next to A2's.

## 11. Handoff to moe-microscope — TW01

Atlas gives the microscope a real workload for the tiered-weight experiment
already in its queue (Saga 14, "quantization and the packed file with the
expert cache").

```
                weights.bin / .atlas
                        |
                      mmap
                        |
                +-------+-------+
                |  expert cache |
                +-------+-------+
                        |
                  active experts
```

Three tiers, then the same picture in a browser:

| Native | Browser |
|---|---|
| RAM — shared weights, router, hot experts | WASM heap / GPU buffer |
| SSD via mmap — warm experts | OPFS / Cache API |
| HDD / file — cold experts | server over HTTP Range |

Measured both sides: expert bytes, cache hit rate, bytes read per query,
load latency, compute time, total latency, evictions. A MicroMoE expert is
1,072 parameters / 536 bytes at INT4 — too small to show I/O cost — so the
microscope needs a scaled storage fixture with identical logical routing.
Atlas's experts are the realistic size.

The connection to `sw-os-ml` is not decorative. MLOS's open gate G4 is
"known-next-use beats LRU", and nothing writes `next_use` yet. A docent
session has unusually legible next-use structure: a visitor in the museum
asks museum questions. Atlas can emit a residency trace with the true next
use labelled afterwards — the first real workload for that policy, in a
browser rather than a kernel, but the same problem.

## 12. Open decisions

Flagged rather than assumed. Work proceeds on the stated default.

1. **Training language.** Default: sw-MLPL, per §6. The risk is real —
   `moe-microscope` has filed upstream findings (F19, F21, F22) against
   MLPL's training path, one of which is that `adam` inside a user function
   trains local copies. A 10M-parameter model is 400× the docent's 26K. If
   MLPL cannot train at that scale on MLX today, the fallback is Candle in
   Rust — still no Python.
2. **Model scale.** The research says 15–30M, then revises to 5–10M for
   browser citizenship. Default: start at 5M, grow only when a measurement
   demands it.
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
| Experts do not specialise | flat specialization map in Saga 6 | drop knowledge sharding, keep dense A2 |
| Teacher questions are unlike real ones | high eval scores, poor live behaviour | log real queries (locally, opt-in), measure the gap |
| Budgets quietly slip | A2 creeping past 64 MiB | budgets are tests; the build fails |
| Nightly churn annoys visitors | large transfers on repeat visits | per-file hashes; measure bytes per returning visit |
| Scope | 12 sagas | Sagas 1–3 have standalone value: a validated corpus and a cross-corpus matcher are a working site feature with no model at all |

## 14. Non-goals

- No general chatbot. No open-domain questions.
- No prose from the weights, at any tier below A4.
- No Python, no hand-written JavaScript, no ONNX in the shipping path.
- No repository source ingestion, no video transcription, in the first
  corpus.
- No server-side inference. Everything after the nightly build runs on the
  visitor's machine.
- No reproduction of Jev or RLCD. TypeSafe has not published enough to
  build it. What is borrowed is the shape — AI as a fast probabilistic
  decision inside ordinary reliable software — not the architecture.

## 15. First session

Not training. Saga 1 step 1–2: the `atlas-core` schema and the blog
ingester, because 124 posts of hand-written front matter can become a
validated corpus today with no model, no GPU and no teacher — and because
until the corpus exists there is nothing for a model to earn the right to
replace.
