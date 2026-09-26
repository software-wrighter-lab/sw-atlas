set shell := ["sh", "-cu"]

# Show available repository tasks.
default:
    @just --list

# The pre-commit gate: fmt, clippy, tests, sw-checklist. See CLAUDE.md.
check:
    cargo fmt --all -- --check
    cargo clippy --all-targets -- -D warnings
    cargo test
    sw-checklist

# Format every tracked Rust source in place.
fmt:
    cargo fmt --all

# Build the blog corpus from the sibling checkout.
ingest-blog *args:
    cargo run --quiet -p atlas-ingest -- blog ../blog --out build/corpus/blog.ron {{args}}

# Build the campus corpus from the sibling checkout.
ingest-campus *args:
    cargo run --quiet -p atlas-ingest -- campus ../sw-campus --out build/corpus/campus.ron {{args}}

# Every non-fork, plus the forks a post or campus place links to.
# Public repositories from the committed GitHub cache (no network).
ingest-repos *args:
    cargo run --quiet -p atlas-ingest -- repos cache/github-repos.json --out build/corpus/repos.ron {{args}}

# Override the checkout with SHORTS=/path/to/shorts.
# The blog's videos, joined to their scripts in the shorts checkout.
ingest-videos *args:
    cargo run --quiet -p atlas-ingest -- videos ../blog "${SHORTS:-../../softwarewrighter/shorts}" --out build/corpus/videos.ron {{args}}

# Count, freeze and check the frozen evaluation sets.
questions *args:
    cargo run --quiet -p atlas-questions -- {{args}}

# Reads what the ingest recipes wrote, so run those first (or `just ingest`).
# One concept vocabulary over the four corpora, with its collision report.
concepts *args:
    cargo run --quiet -p atlas-ingest -- concepts --out build/corpus/corpus.ron {{args}}

# Build every corpus, then unify their concepts into build/corpus/corpus.ron.
ingest: ingest-blog ingest-campus ingest-repos ingest-videos concepts

# Offline by default: reads cache/url-status.json, makes no network call.
# Regenerate the coverage report and gate on orphans and dead links.
report *args:
    cargo run --quiet -p atlas-coverage --bin atlas-report -- --today "$(date -u +%F)" {{args}}

# Re-check every URL over the network and rewrite the committed cache.
# A corpus change, so commit cache/url-status.json with its counts.
report-check *args:
    just report --check {{args}}

# Ask the docent a question; --measure reports the outcome distribution.
ask *args:
    cargo run --quiet --release -p atlas-ask -- {{args}}

# Needs build/corpus/corpus.ron, so run `just ingest` first.
# Score every arm on the confirmed question sets, with baselines and intervals.
eval *args:
    cargo run --quiet --release -p atlas-eval-cli -- {{args}}

# Write the publishable snapshot with per-file hashes (saga atlas-runtime).
snapshot:
    @echo "just snapshot: not implemented yet." >&2
    @echo "Lands in saga atlas-runtime. See docs/plan.md section 9 for the" >&2
    @echo "artifact layout it has to produce." >&2
    @exit 1
