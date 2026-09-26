Read docs/plan.md, then cache/README.md and crates/atlas-repos/.

The corpus currently sends visitors to abandoned repositories. Found while drafting the softwarewrighter/start-here hub, 2026-09-26, verified against the live GitHub API:

Several repositories exist twice, once in `softwarewrighter` and once in an organisation, because the move was done by forking into the organisation and leaving the original behind. GitHub therefore flags the LIVE copy as a fork and the ABANDONED copy as the source. The ingester keeps a fork only when a blog post links to it (owner decision, 2026-09-22), so for these pairs it keeps the dead copy and drops the live one. The pairs, with the public evidence:

- `sw-checklist`: sw-vibe-coding (fork, pushed 2026-06-12) vs softwarewrighter (source, 2026-02-11). The fork is live.
- `sw-cli`: sw-cli-tools (fork) vs softwarewrighter (source), same push date.
- `game-mcp-poc`: sw-game-dev (fork, 2025-11-21) vs softwarewrighter (source, 2025-11-20).
- `sw-install`: sw-vibe-coding (fork, 2025-12-27) vs softwarewrighter (source, 2026-03-22) -- here the SOURCE was pushed later, so which is canonical is an OWNER DECISION. Ask, record the answer in the file, do not guess.
- `rank-wav-rs`: sw-cli-tools (source) vs sw-music-tools (fork). This one is a plain cross-organisation duplicate, not a graduation: the non-fork copy is the live one, so the existing rule already handles it. Assert that with a test rather than adding a declaration.

The mechanism: a new committed declaration, `sources/repo-canonical.ron`, holding one entry per pair -- the canonical `full_name`, the superseded `full_name`, and a sentence of why, in the owner's words where there is one. `atlas-repos` reads it and (a) ingests the canonical repository even when GitHub says it is a fork, (b) leaves the superseded one out of the corpus as a resource, and (c) records the relationship so a visitor who asks about the superseded name still lands on the live resource -- an alias on the canonical resource is the cheap way, a `Supersedes` relation the honest way; pick one and say why in the module doc.

Tests, all offline: a fixture pair where the fork is canonical, asserting the fork is ingested and the source is not; a test that every `full_name` in the file exists in `cache/github-repos.json`, so a typo is a failing build and not a silent no-op; a test that a superseded name still resolves to the canonical resource; and the existing zero-orphan and URL gates still green.

Facts to update in the same commit: `cache/README.md` (the keep/exclude rules and the corpus count), the count pinned in `crates/atlas-repos/tests/repos.rs`, the README status table, and `docs/reference/coverage.md` by regeneration. The corpus count moves, so state the new number and what moved it.

Exit: `sources/repo-canonical.ron` committed with the pairs above, the owner's `sw-install` decision recorded or explicitly listed as outstanding in the summary, `just check` green, `just ingest` byte-stable across two runs, and the README and cache/README.md counts agreeing with the corpus.
