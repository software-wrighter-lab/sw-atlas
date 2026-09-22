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

# Public, non-fork repositories from the committed GitHub cache (no network).
ingest-repos *args:
    cargo run --quiet -p atlas-ingest -- repos cache/github-repos.json --out build/corpus/repos.ron {{args}}

# The blog's videos, joined to their scripts in the shorts checkout.
# Override the checkout with SHORTS=/path/to/shorts.
ingest-videos *args:
    cargo run --quiet -p atlas-ingest -- videos ../blog "${SHORTS:-../../softwarewrighter/shorts}" --out build/corpus/videos.ron {{args}}

# Build every corpus that has an ingester.
ingest: ingest-blog ingest-campus ingest-repos ingest-videos

# Where the remaining source lands (atlas-foundation step 12).
ingest-rest:
    @echo "just ingest-rest: not implemented yet." >&2
    @echo "The concept graph lands in step 012." >&2
    @exit 1

# Regenerate the coverage report and gate on orphans and dead links (step 13).
report:
    @echo "just report: not implemented yet." >&2
    @echo "Lands in atlas-foundation step 013 (coverage-report): it writes" >&2
    @echo "docs/reference/coverage.md and exits non-zero on any orphan" >&2
    @echo "resource or broken URL." >&2
    @exit 1

# Score every runtime class on the held-out questions (saga atlas-baseline).
eval *args:
    @echo "just eval {{args}}: not implemented yet." >&2
    @echo "Lands in saga atlas-baseline, which builds the harness before" >&2
    @echo "there is anything to evaluate. See docs/sagas.md." >&2
    @exit 1

# Write the publishable snapshot with per-file hashes (saga atlas-runtime).
snapshot:
    @echo "just snapshot: not implemented yet." >&2
    @echo "Lands in saga atlas-runtime. See docs/plan.md section 9 for the" >&2
    @echo "artifact layout it has to produce." >&2
    @exit 1
