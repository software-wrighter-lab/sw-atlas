# Saga: atlas-foundation

Build the corpus and the machinery that validates it: a Rust workspace with
a canonical schema for Resource, Concept, Relation, Question and Decision;
four ingesters that turn the existing public artifacts into that schema
without any model, GPU or teacher; a concept graph; and a coverage report
that makes "Atlas knows about everything" a number with a gate behind it.

No model is trained in this saga. Nothing neural is written. The point is
to produce the thing a model will later have to earn the right to improve
on, and the harness that will judge it.

The full architecture, runtime classes, data model, snapshot layout and the
saga queue are in docs/plan.md. Read it before every step; where it and
docs/sw-atlas-research.txt or docs/jev-os-research.txt disagree, the plan
wins. docs/needle.md holds the model-architecture assessment and is
background for this saga, not instruction.

Sources are read-only siblings. This repository may not modify them:
../sw-campus (pages/docent/snapshot-a.json, later dist/catalog.json),
../blog (_posts/*.md front matter), and the GitHub API for repository and
video metadata. Anything they need to change is a work order recorded in
docs/handoffs.md, not an edit.

Every step ends green: `just check` (fmt, clippy -D warnings, tests,
sw-checklist with ZERO failures and ZERO warnings) before commit, then
`agentrail complete`, then push.

## Steps

1. scaffold-workspace -- Cargo workspace (Rust 2024), `atlas-core` crate
   stub, justfile with check/ingest/report/eval/snapshot recipes (the last
   three may be stubs that fail loudly), .gitignore audit, `just check`
   green with sw-checklist at zero/zero. README build section becomes true.

2. corpus-schema -- `atlas-core`: Resource, Concept, Relation, Question,
   Decision with ResourceId/ConceptId newtypes, ResourceKind, RelationKind,
   Provenance, Maturity, Split, Origin; RON serde; a canonical writer with
   a stable content hash; cross-reference validation (every relation
   endpoint and every concept reference resolves) as a native test that
   fails on a broken fixture.

3. ingest-blog -- `atlas-ingest blog ../blog`: 124 posts to Resources from
   front matter alone. title, abstract, date, categories, tags, keywords ->
   aliases, series/series_part -> SeriesNext relations, repo_url/repo_urls,
   video_url(s), demo_url, papers -> Declared relations to Repo/Video/Demo/
   Paper resources. No model, no body text beyond what front matter gives.
   Test: a pinned sample of posts round-trips to expected ids and relations.

4. ingest-campus -- `atlas-ingest campus ../sw-campus`: snapshot-a.json's
   nine places, kinds, parents, titles, taglines, status, links, aliases,
   concepts to Resources and PartOf relations; the 21 stories kept as
   catalog text, never as training text. The ibm-1442 exclusion is asserted
   by a test, because Saga 6 depends on it. Reads dist/catalog.json with its
   SHA-256 when sw-campus publishes one, snapshot-a.json until then.

5. ingest-metadata -- `atlas-ingest repos` and `atlas-ingest videos`:
   repository metadata (id, org, name, description, topics, language,
   homepage, pushed_at) from the GitHub API with a committed cache, and
   video metadata from the blog's own video_url/video_title fields plus a
   committed supplement. Metadata only: no source files, no transcripts.

6. concept-graph -- normalize the union of blog tags/keywords, campus
   concepts and repo topics into a Concept set with aliases and parents;
   an alias collision report; a hand-curated override file for the cases
   the normalizer gets wrong, committed and applied deterministically.

7. coverage-report -- `just report`: the coverage table from docs/plan.md
   section 10 (artifacts indexed per kind, tagged, related, orphans, broken
   URLs), a link checker over every Resource url, and the two gates: zero
   orphans and zero broken URLs or the command exits non-zero. Written to
   docs/reference/coverage.md by the run, not by hand.

8. results-and-handoffs -- docs/reference/results.md seeded with the four
   transferred moe-microscope rows (MB01, MB01t, CD01, CD01b) and the empty
   columns Saga 2 will fill; docs/handoffs.md with the three moe-microscope
   work orders from plan.md section 11 and anything the ingesters found
   that sw-campus or blog should change; README status updated; Saga 2
   queued.
