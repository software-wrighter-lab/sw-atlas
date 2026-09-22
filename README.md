# <img src="images/sw-atlas-logo.png" alt="sw-atlas" width="128" align="left" style="margin-right:12px"> sw-atlas

One semantic index over everything Software Wrighter publishes, and a tiny
model that turns a visitor's sentence into a typed decision about it.

<br clear="left"/>

The index holds the facts. The model holds only the language. Every site --
the [campus](https://github.com/software-wrighter-lab/sw-campus), the
[blog](https://blog.softwarewrighter.com/), a live demo -- mounts the same
engine and gives it a different name and face.

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

## Summary

A visitor asks "where was that thing about running experts from disk?".
Answering it well normally costs a large model, a server and a second or
two. Atlas moves almost all of that work to the night before.

```
        NIGHTLY -- expensive, once, on our hardware
   ------------------------------------------------
   blog / campus / repos / videos
              |
              v
     ingest, relate, synthesize questions,
     embed, train, calibrate, regression-test
              |
              v
        published snapshot  (a few MB)

        BROWSER -- cheap, every time, on theirs
   ------------------------------------------------
   question -> typed decision -> catalog lookup -> answer
```

The model never writes prose, a URL, or a resource id. It emits an intent,
a set of concepts, a set of resource kinds and a confidence; ordinary
deterministic code resolves those against the published catalog. A stale
model therefore cannot invent an exhibit or a dead link -- the worst it can
do is route badly, and say so with a low number.

Everything runs in the visitor's browser, and it starts small: a
deterministic matcher in under 10 MiB that works everywhere, promoted one
class at a time up to a routed model at 128 MiB only when the machine can
comfortably afford it, and demoted the moment it cannot.

| Class | RAM target | Capability |
|---|---:|---|
| A0 Minimal | 10 MiB | lexical and graph matching, no model |
| A1 Semantic | 25 MiB | embedding retrieval |
| A2 Intelligent | 64 MiB | intent, ranking, confidence |
| A3 Decoding | 128 MiB | arguments, comparison, reranking |
| A4 Enhanced | 256 MiB | prose synthesis, opt-in only |

The model itself is a Simple Attention Network: an encoder-decoder with no
feed-forward layer, so there is nowhere for a fact to be memorised. It is
shown resource cards from the snapshot the way
[Needle](https://github.com/cactus-compute/needle) is shown a tool list, and
it aligns the question to them. [`docs/needle.md`](docs/needle.md) assesses
that architecture and the open-source alternatives to it.

Read [`docs/plan.md`](docs/plan.md) first; it is the specification.
[`docs/sw-atlas-research.txt`](docs/sw-atlas-research.txt) and
[`docs/jev-os-research.txt`](docs/jev-os-research.txt) are the raw design
conversations it was distilled from -- input, not spec. Where they and the
plan disagree, the plan wins.

## Status

The corpus is built; nothing that answers a visitor is. Four ingesters turn
the blog, the campus, the repository cache and the video scripts into a
validated, canonical corpus with no model in it. There is no matcher over
that corpus yet (Saga 2), no model (Saga 3) and no browser runtime (Saga 9),
so nothing here can be demonstrated to a visitor today, and every quality
number below is inherited from `moe-microscope` rather than measured here.

The honest starting scoreboard, inherited from
[`moe-microscope`](https://github.com/sw-ml-study/moe-microscope)'s campus
docent work: a deterministic keyword matcher scores **0.685** on held-out
paraphrases, and the trained model scores **0.407**. The matcher is the
champion. Nothing neural ships here until it beats that by 20 points on
the same questions.

What the corpus holds today (`just ingest`, and the same bytes and hash on
every rebuild):

| | Blog | Campus | Repos | Videos |
|---|---:|---:|---:|---:|
| Posts | 126 | — | — | — |
| Places | — | 9 | — | — |
| Repositories | 78 named | 4 named | 251 described | — |
| Videos | 84 named | — | — | 84 described, 52 with script |
| Demos named | 6 | 4 | 4 | — |
| Papers cited | 157 | — | — | — |
| Concepts, provisional | 637 | 39 | 21 | 130 |
| Relations, every one `Declared` | 518 | 16 | 4 | — |

Repositories are the public ones across 15 GitHub accounts and
organisations, from a committed cache ([`cache/README.md`](cache/README.md)):
every non-fork, and the nine forks a blog post links to. Of the 80
repositories the blog and campus name, 75 join to that cache by
identifier; the other 5 are owned by other people.

Videos take their scripts from the `shorts` repository: 30 by the Five ML
Concepts rule and 22 through a hand-written, owner-confirmed map; the
other 32 are reached today through the posts that link to them
([`docs/video-sources.md`](docs/video-sources.md)).

Three identifiers already appear in both the blog and the campus, because
a target's identifier is derived from its URL: `repo:sw-comp-history/ibm-1130-rs`,
`demo:sw-comp-history.github.io/ibm-1130-rs` and `repo:sw-embed/sw-cor24-apl`.
A visitor standing in front of the 1130 exhibit can be shown the post about
it without anything having inferred the connection.

| Run | Paraphrase dest | Intent | Unsupported recall | Class |
|---|---:|---:|---:|---|
| MB01t campus text matcher | **0.685** | 0.481 | 0.40 | A0 |
| MB01 campus alias matcher | 0.630 | 0.463 | 1.00 | A0 |
| CD01b docent + word vectors | 0.407 | 0.407 | 0.30 | A2 |
| CD01 dense docent | 0.296 | 0.481 | 0.30 | A2 |

Read the columns rather than the winner: the matchers find destinations
well and read intent badly, and every trained docent was asked to replace
the matcher rather than help it. So the next model is not a rival. In the
hybrid docent (Saga 3, HT01), the matcher proposes candidates, a typed
decision model reused from
[`demo-decision-model`](https://github.com/sw-ml-study/demo-decision-model)
decides intent and reranks those candidates, and ordinary Rust decides
which one to trust. It ships only if the combination beats the matcher by
the same 20 points. The reasoning is in
[`docs/hybrid-docent.md`](docs/hybrid-docent.md).

Progress is driven by [agentrail](CLAUDE.md) sagas. `agentrail status`
says where the current one stands; [`docs/sagas.md`](docs/sagas.md) holds
the queue. The active saga is `atlas-foundation`: the corpus and the
machinery that validates it, with no model in it at all.

## Building

Prerequisites: a stable Rust toolchain (2024 edition, 1.85 or newer),
[just](https://just.systems), and `sw-checklist` (Software Wrighter's
conformance checker, on the `PATH`). Later sagas add
[sw-MLPL](https://github.com/sw-ml-study/sw-mlpl) for training and the
`wasm32-unknown-unknown` target for the browser runtime.

```sh
cargo install just
```

Then:

```sh
just            # list every recipe
just check      # the pre-commit gate: fmt, clippy -D warnings, tests, sw-checklist
```

The gate is green and `sw-checklist` is at zero failures and zero warnings.
Keep it there.

`just ingest` reads the blog, the campus, the repository cache and the
videos, and writes a canonical corpus for each. The remaining
pipeline recipes exist but are not implemented; each exits non-zero naming
the step that lands it, so this list stays honest:

| Recipe | Does | Lands in |
|---|---|---|
| `just ingest-blog` | **done** — 126 posts from the blog's front matter | Saga 1, ingest-blog |
| `just ingest-campus` | **done** — 9 places from the campus catalog | Saga 1, ingest-campus |
| `just ingest-repos` | **done** — 251 public repositories: non-forks, and forks a post names | Saga 1, ingest-metadata and declared-forks |
| `just ingest-videos` | **done** — 84 videos, 52 with scripts from `shorts` | Saga 1, ingest-metadata |
| `just ingest-rest` | the concept graph | Saga 1, concept-graph |
| `just report` | coverage, gated on orphans and dead links | Saga 1, coverage-report |
| `just eval` | score every runtime class on held-out questions | Saga 2 |
| `just snapshot` | write the publishable snapshot with per-file hashes | Saga 9 |

No Python and no hand-written JavaScript, anywhere: the pipeline is Rust
and sw-MLPL, and the browser runtime will be Rust compiled to WebAssembly.

## Copyright

Copyright (c) 2026 Michael A Wright

## License

MIT. See [`LICENSE`](LICENSE) and [`COPYRIGHT`](COPYRIGHT).
