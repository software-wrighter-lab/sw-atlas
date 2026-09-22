# Requests to `sw-campus`

Work orders for agents operating in
[`software-wrighter-lab/sw-campus`](https://github.com/software-wrighter-lab/sw-campus).
sw-atlas reads that repository and cannot write to it; everything here is a
request, to be revalidated against the campus plan before it is acted on.

Raised by sw-atlas Saga 1.

## CATALOG-EXPORT — publish `dist/catalog.json` canonically

**Ask.** The campus plan's docent saga, step 1, already commits to this: a
test that writes `dist/catalog.json` canonically with its SHA-256, so a
consumer reads the live catalog rather than a snapshot copied by hand.
sw-atlas is that consumer. Nothing new is being asked for — only that this
step is known to have a second reader downstream.

**What sw-atlas needs from the format.** Stable place identifiers; kind and
parent, so the hierarchy survives; title, tagline, status and links; the
`Docent` block's aliases, concepts and example queries; and the stories
with their ids and kinds. The content hash is what lets an Atlas snapshot
declare which campus edition it was built from and show a stale badge
honestly.

**Until then.** sw-atlas ingests `pages/docent/snapshot-a.json` and records
in the corpus which source was used, so the switch is visible when it
happens rather than silent.

## SNAPSHOT-B-EVENT — publishing the 1442 is an experiment, not just content

**Ask.** When the IBM 1442 card read punch and its radio demo are added to
the campus, treat that publication as a dated, announced event rather than
an ordinary content commit, and say so in the commit message.

**Why.** The 1442's absence from snapshot A was deliberate, and it has
since become load bearing for two repositories. moe-microscope reserved it
as the snapshot A to B incremental-learning experiment; sw-atlas Sagas 4 and
7 (NP01, SN01) both use it as the one piece of the world that a trained model
provably has never seen. It is the only clean held-out resource in the
entire corpus, and it can only be spent once.

**What sw-atlas will do with it.** Ask the withheld questions ("where can I
hear the card reader play music?") of a model trained before the exhibit
existed, then again after publication, and report whether new knowledge
needed retraining at all or only an index rebuild. That result decides the
design of the nightly pipeline.

**The one thing that would spoil it.** Adding the 1442 incrementally —
first a placeholder, then a title, then a demo — over several commits, so
that no model can be said to have been trained before it existed. One
commit, one date, one announcement.

**Already noted, no action needed.** Snapshot A's 1130 wing arrival story
already names the 1442 in passing, as one of the peripherals plugged into
the machine. That is correct and should stay: the wing is about the machine
and everything attached to it. It simply means the withheld knowledge is
the exhibit and its demo — a destination — rather than the word, and
sw-atlas's experiment is scored that way. A test in this repository pins
that single mention so a later reader does not mistake it for a leak.

## EASEL-CONTRACT — what the Atlas runtime will offer (advance notice)

**Not a request yet.** sw-atlas Saga 9 produces `atlas-runtime`, a Rust
crate a Yew app mounts with a role parameter. The campus docent saga's
steps 4 and 5 currently plan a `Predictor` trait with a deterministic
matcher behind it and an `mlpl-wasm` bridge in front. Those two designs
should meet before either is built.

What the runtime intends to provide, so the campus can design against it:

- A single mount point taking a role (`Guide` inside the campus, `Docent`
  in the museum) that changes vocabulary and defaults, not knowledge.
- A ranked list of resources with calibrated confidences, plus an abstain
  signal, rather than a single answer.
- A "why this answer?" payload: matcher signals at A0, the hybrid docent's
  candidates, typed decisions and policy rule (sw-atlas Saga 3, HT01),
  nearest neighbours and cosine margins at A2, the decoded record at A3.
- Progressive capability, starting at a deterministic matcher in under
  10 MiB that works on any device, promoted only when the machine can
  afford it and demoted the moment it cannot. The campus page must render
  and be useful before Atlas has loaded anything.

sw-atlas will send a concrete interface proposal when Saga 9 opens.

## What the ingester found, 2026-09-22

**`dist/catalog.json` still does not exist**, so the ingester reads
`pages/docent/snapshot-a.json` and records which file it used on every
resource it produced. CATALOG-EXPORT above is therefore still the live
request, and the switch will be visible in a commit rather than silent.

**Snapshot A ingested cleanly.** 9 places, 4 repositories, 4 demos, 39
concepts, 16 `PartOf` relations, and the 21 stories kept as catalog text
rather than training text. Every campus URL resolves. The 1442 exclusion is
asserted by a test in `crates/atlas-ingest/tests/campus.rs`, including the
subtler half: the 1130 wing's arrival story mentions the card read punch in
prose, so what is withheld is the exhibit and its demo, not the string.

**Three identifiers already join the campus to the blog** with nothing
inferred, because both derive an identifier from the URL:
`repo:sw-comp-history/ibm-1130-rs`,
`demo:sw-comp-history.github.io/ibm-1130-rs` and
`repo:sw-embed/sw-cor24-apl`. A visitor standing at the 1130 exhibit can be
shown the post about it today.

**One small ask for when the catalog is published: keep concept labels as
prose.** The campus writes `machine learning`; the blog writes
`machine-learning`. sw-atlas normalizes both to one concept and then has to
choose which spelling to show a visitor, and it prefers the prose one --
`Chain of Thought` over `chain-of-thought`. If `dist/catalog.json`
slugified its concepts on the way out, the corpus would lose the only
human-readable spellings it has for several hundred concepts. Emit them as
written.

## Not asked for

- No change to the campus's existing keyword matcher. It is the champion
  at 0.685 and sw-atlas keeps its own port of it as a permanent runtime
  class, not a temporary fallback.
- No content restructuring, no new metadata fields, no docent-specific
  authoring burden. The campus catalog as designed is already a better
  corpus than most of what this project will ingest.
