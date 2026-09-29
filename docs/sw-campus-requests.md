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

## EASEL-CONTRACT v1 — the query contract, concrete (2026-09-29)

**This supersedes the advance notice below.** The campus plan queues
`docent-pins` "after the sw-atlas query contract stabilizes", and the advance
notice promised a proposal "when Saga 9 opens", which is five sagas away. The
campus does not need a runtime to design pins; it needs the shape of an
answer. That shipped in sw-atlas Saga 2 step 011 and is what follows. Every
number here was measured on 2026-09-29 against corpus
`d77185…`, and can be reproduced with `just ask --measure`.

### What a query returns

```rust
enum Outcome {
    One,            // the evidence is decisive: one resource
    Several,        // the usual case: the closest few, offered as a choice
    NotYet(String), // the subject is real and not in the index yet
    Rephrase,       // none of those words is in the index
    NothingHere,    // the words are known; nothing they point at is here
}

struct Reply {
    outcome: Outcome,
    offer:   Vec<ResourceId>, // in the order a visitor should see them
    because: String,          // for a trace panel; NOT machine-readable
}
```

Three of the five outcomes are refusals. That is the design, not a gap: the
measurement says a confident single answer is not available (below).

**Which outcomes a pin may render.** `One` and `Several` carry an `offer`;
pin the entries whose identifier starts with `campus:` and ignore the rest,
which are posts, repositories, videos and demos the campus has no anchor for.
`NotYet` carries a sentence written for a visitor and must be rendered as
words — there is nothing to pin, and inventing a pin for an exhibit that does
not exist is the one failure this project will not tolerate. `Rephrase` and
`NothingHere` carry an empty offer; render the apology, pin nothing.

An offer may legitimately contain **no** campus places at all: a question
answered best by a blog post gets posts. A pin layer must therefore degrade to
"no pins this time" without looking broken.

### Identity

Identifiers derive from URLs, never from titles, so renaming an exhibit does
not break a join:

| Kind | Rule | Example |
|---|---|---|
| Campus place | `campus:` + the last path segment of the place URL | `campus:ibm-1130-emulator` |
| Repository | `repo:owner/name` | `repo:sw-comp-history/ibm-1130-rs` |
| Demo | `demo:host/path` | `demo:sw-comp-history.github.io/ibm-1130-rs` |
| Post | `blog:YYYY-MM-DD-slug` | `blog:2026-02-26-ibm-1130-system-emulator` |

Three identifiers already join the two repositories with nothing inferred,
because both sides derive them the same way. An identifier changes only when
the URL changes; if a place moves in the hierarchy, tell sw-atlas, because
that is a redirect rather than a rename.

### Guarantees the campus can rely on

- **Nothing is generated.** Every visitor-facing string is a fixed frame with
  catalog text quoted into it. The docent cannot describe an exhibit that does
  not exist, because it has no way to write a sentence that is not already a
  frame.
- **The model, when it exists, emits no URL and no identifier.** It emits an
  intent, concepts, resource kinds and a confidence; deterministic code
  resolves those against the catalog. A stale model cannot invent an exhibit.
  This is a hard constraint in sw-atlas's CLAUDE.md, not a current
  implementation detail.
- **Ties break by identifier**, so two machines rank the same way.
- **A0 answers in 0.64 ms at p50 and 0.96 ms at p95**, and its whole index is
  budgeted at 10 MiB. A pin layer can query on every keystroke if it wants to.

### The honest limits

- **No percentages on a pin.** The scores are not probabilities. Calibration
  is sw-atlas Saga 8; until it lands, any number rendered as a confidence
  would be decoration.
- **No single confident pin.** Over 386 confirmed questions, no combination of
  score and margin names one resource above **0.54** precision. `One` fires on
  0.005 of questions. Design for a small cluster, not a winner.
- **What an offer is worth.** The docent commits to an offer on **0.969** of
  confirmed questions, and the offer holds the expected destination **0.414**
  of the time.
- **What a *place-seeking* question is worth, which is the number that matters
  for pins.** Of the 21 confirmed questions whose expected destination is a
  campus place, the offer contains that place **13 times (0.62)**. Better than
  the corpus-wide figure, and not good enough to pin silently: a visitor who
  asks for a place and gets no pin must still see the answer as words.

### The five misses, and what would fix them

The eight failures above are vocabulary, not ranking. "The machine with the
toggle switches and the row of little lights" and "which part of the campus
holds the language toys" describe exhibits in words no catalog entry uses, and
"take me to the APL exhibit" loses `campus:apl` to three blog posts that say
APL more often.

This is the strongest argument yet for the campus catalog's `Docent` block:
its **aliases** and **example queries** are exactly the missing vocabulary, and
they are knowledge only the campus has. Shipping CATALOG-EXPORT with those
fields intact would likely move that 0.62 more than anything sw-atlas can do
on its own side, because sw-atlas is not allowed to invent names for exhibits
it did not write.

### What is not decided yet, and must not be designed against

- **Packaging and the mount point** — sw-atlas Saga 9. Whether the campus
  embeds a WASM crate, calls a worker, or reads a snapshot and matches locally
  is open. The *answer shape* above is stable; how it arrives is not.
- **The model tier** — Saga 3. A0 is the permanent floor and stays the
  champion until beaten by 20 points on held-out paraphrases. Pins built
  against the five outcomes keep working whatever wins.
- **Calibrated confidence** — Saga 8.

### What sw-atlas asks the campus to build against

Render up to three pins from the `campus:` identifiers in `offer`, in the
order given; show the `because` line only in a trace or debug view; render
`NotYet` text verbatim; and treat an empty offer as an ordinary answer rather
than an error. If the campus wants a pin for a destination that is not a
place — a post, a demo — say so and sw-atlas will describe how those resolve
to the place that declares them.

## EASEL-CONTRACT — what the Atlas runtime will offer (advance notice, superseded)

**Kept for the record; read the contract above instead.** sw-atlas Saga 9 produces `atlas-runtime`, a Rust
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

That proposal arrived early, above, because the campus was blocked on it.

## What the ingester found, 2026-09-22 (Saga 1) and 2026-09-29 (Saga 2)

**`dist/catalog.json` still does not exist**, so the ingester reads
`pages/docent/snapshot-a.json` and records which file it used on every
resource it produced. CATALOG-EXPORT above is therefore still the live
request, and the switch will be visible in a commit rather than silent.

**Update, 2026-09-29.** The campus MVP saga closed without it, so both
CATALOG-EXPORT and DEMO-NAMES are still open. Neither blocks sw-atlas: the
corpus builds, both gates pass, and snapshot A is enough to measure against.
What they now block is the *quality* of `docent-pins`: the contract above
measures campus places reaching a visitor 13 times in 21 for place-seeking
questions, and the misses are missing vocabulary that the `Docent` block's
aliases and example queries would supply. That makes CATALOG-EXPORT the
highest-value thing the campus could ship for the docent, and DEMO-NAMES the
cheapest.

The corpus is larger than when this section was written: 648 artifacts, 129
posts, 9 places, 246 repositories from the cache, 84 videos, 12 demos, 737
concepts, 552 declared relations, zero unreachable resources, zero dead
URLs.

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

## DEMO-NAMES — the demos have no names of their own

**Ask.** Where the catalog links to a live demo, give the link a name
rather than a verb. `[run](...)` beside the 1130 exhibit tells a reader what
to click and tells an index nothing; `[IBM 1130 System Emulator](...)`, or a
`title` field beside the URL, tells both.

**Why.** sw-atlas ingested three demos titled `run` and one titled
`io demo`, which are labels on a button rather than names of a thing. It now
drops such labels and leaves the title empty, because an empty title is
honest and a resolver can fall back to the name of the exhibit that declared
the link -- but that means a visitor asking for the emulator by name is
matched through the exhibit rather than the demo. The campus and the blog
are the only places that know what these demos are called.

Cheap, and worth doing in the same pass as CATALOG-EXPORT above: if
`dist/catalog.json` carries a name per link, sw-atlas needs no rule at all.

## Not asked for

- No change to the campus's existing keyword matcher. It is the champion
  at 0.685 and sw-atlas keeps its own port of it as a permanent runtime
  class, not a temporary fallback.
- No content restructuring, no new metadata fields, no docent-specific
  authoring burden. The campus catalog as designed is already a better
  corpus than most of what this project will ingest.
