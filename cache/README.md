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
| Repositories in the cache | 307 |
| ... not forks | 264 |
| ... forks | 43 |
| ... of those, forks of the owner's own work | 14 |
| ... excluded as not artifacts | 2 |
| ... excluded as organisation profiles (`.github`) | 14 |
| In the corpus | 252 |
| Organisations and users represented | 15 |

Fetched 2026-10-01 (the file's timestamp; the script prints the time but
does not store it). Counts from `jq` over the committed file.

**Refreshing.** `scripts/fetch-repos` is the only step in this repository
that touches the network. It needs an authenticated `gh`. Run it on
purpose, never from the gate: a changed cache is a corpus change, and
belongs in its own commit that states the new counts and updates the
count pinned in `crates/atlas-repos/tests/repos.rs`.

**What is kept.** The cache keeps every fork, so it remains a complete
record of what GitHub reported. The ingester keeps a fork when the owner
has written about it -- when a blog post or campus place links to it
(owner decision, 2026-09-22) -- or when `sources/repo-canonical.ron`
declares it the real copy of work that exists twice. Any other fork is
someone else's work sitting in an owner's account, and stays out.

**Why a declaration is needed at all.** GitHub's `fork` field records
which button was pressed, not who wrote the code. Work moved into an
organisation by forking is marked as somebody else's forever, while the
copy left behind keeps the flag off -- so a rule that reads the flag
alone keeps the abandoned copy and drops the live one. Before the
declarations, seven pieces of work were in the corpus only as the copy
nobody pushes to, and three were in twice. Owner rule, 2026-09-26: **a
fork of my own work is my work; a fork of somebody else's is not.** The
cache does not yet record each fork's parent, which is the field that
would let the ingester apply that rule without a list; adding it is part
of the next cache refresh.

**Counts after the declarations.** 252 repositories from the cache. The
ten superseded copies are out; the ten canonical copies are in whatever
GitHub's fork flag says; the fourteen `.github` organisation profile
repositories are out as infrastructure rather than artifacts; and four more
forks are in because `sources/accounts.ron` says their parent is the owner's
own -- three continuing work from the earlier `wrightmikea` account, and one
(`sw-embed/sw-cor24-x-assembler`) that began as a fork of `cor24-rs` and
became a different program.

Two repositories are excluded for a different reason: they are public but
are not artifacts a visitor could be sent to. They are named with their
reasons in `sources/repo-exclusions.ron`, and `just report` states the
count, so the exclusion cannot quietly shrink a coverage number.

## `url-status.json`

What the last link check found for every URL the corpus names: 642 URLs,
635 resolving, none gone, six hosts refusing a robot and one not answering.
Written by `just report-check`, which is the second and last step in this
repository that touches the network. `just report` reads it and makes no
request, so the coverage gate cannot fail because someone else's site is
down. Refreshing it is a corpus change and belongs in its own commit with
its counts.
