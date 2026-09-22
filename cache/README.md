# cache/

Network-derived inputs, committed so that every corpus rebuild and every
test runs offline and gives the same bytes.

## `github-repos.json`

Every public repository owned by `softwarewrighter` and by each GitHub
organisation that account belongs to, one JSON object per repository with
the fields `atlas-ingest repos` reads: `full_name`, `description`,
`topics`, `language`, `homepage`, `pushed_at`, `fork`, `archived`,
`stargazers_count`. Sorted by `full_name`, unique.

| | Count |
|---|---:|
| Repositories in the cache | 283 |
| ... forks, excluded by the ingester | 41 |
| ... public, not forks: the corpus | 242 |
| Organisations and users represented | 15 |

Fetched 2026-09-19 (the file's timestamp; the script prints the time but
does not store it). Counts from `jq` over the committed file.

**Refreshing.** `scripts/fetch-repos` is the only step in this repository
that touches the network. It needs an authenticated `gh`. Run it on
purpose, never from the gate: a changed cache is a corpus change, and
belongs in its own commit that states the new counts and updates the
count pinned in `crates/atlas-repos/tests/repos.rs`.

**What is kept.** The cache keeps forks, so it remains a complete record
of what GitHub reported. The ingester drops them, because a fork is
someone else's work until the owner says otherwise. Nine forks are linked
from blog posts and so exist in the blog corpus as stubs with no GitHub
metadata; see the step 011 summary.
