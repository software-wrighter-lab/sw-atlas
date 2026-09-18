# Video sources

Survey of `~/github/softwarewrighter/shorts` and
`~/github/softwarewrighter/video-publishing`, read on 2026-09-18, against
what this project needs to index 75 published videos.

**The short answer.** `shorts` is a real text corpus and gives Atlas the
words spoken in the videos. `video-publishing` is a production plan and
gives Atlas almost nothing. Neither records a YouTube URL per project, so
the blog remains the authority for *which* videos exist and *where* they
are; `shorts` becomes the authority for *what was said in them*. The join
between the two is the work.

**The gitignored media does not matter.** Drafts, audio, clips and the
rendered videos are excluded from both repositories and would have to be
recovered from an older machine. None of it is needed here: Atlas indexes
text, and the text is tracked on purpose. `shorts/.gitignore` says so
outright — it excludes `*.mp4`, `*.mp3`, `*.wav`, and then lists what is
deliberately kept: "`*.txt` (scripts, title, description)".

## What `shorts` contains

| | Count |
|---|---:|
| Projects under `projects/` | 78 |
| ... with `narration.md` or `narration.txt` (the spoken script) | 39 |
| ... with `description.md` or `description.txt` (the published blurb) | 63 |
| ... with `title.txt` | 14 |
| Tracked `.txt` files in total | 1,350 |

Two files are worth more than their size suggests:

- **`docs/5mlc-concepts.md`** — every Five ML Concepts episode with the
  five concepts it covers and a category for each (Classic, Core, Modern,
  Essential, Practical).
- **`docs/concepts-status.txt`** — the inverse: 130 concepts, each mapped
  to the episode that covers it, generated 2026-02-26.

That is a hand-built concept-to-resource index over the video corpus, in
exactly the shape [`plan.md`](plan.md) §7 defines `Concept` for, written by
the person who made the videos. It is `Provenance::Declared` material, and
it is the same windfall the blog's front matter was: work a model would
otherwise be paid to do worse.

The narration is the real prize. `projects/5MLC-10/work/narration.md` is
the full spoken text, sectioned by concept — an authored paragraph per idea,
written to be said aloud, which is close to how a visitor would ask about
it. `description.md` beside it is the published YouTube description with
the numbered concept list and the outbound links.

## The join to published videos

The blog declares 75 videos with `video_url` and `video_title`. Joining
them to `shorts` projects splits cleanly in two.

**29 join deterministically.** Thirty of the blog's videos are titled
`Five ML Concepts - #N`, for N from 1 to 29 — and `shorts/projects/` holds
`5MLC-2` through `5MLC-30`. Episode 1 is the outlier, living in
`projects/daily5-20260203`, and episode 30 is produced but not yet
published. So the rule is: *episode N maps to `projects/5MLC-N`, except
episode 1.* No human judgement, no fuzzy matching, 29 videos with their
full scripts.

**The remaining ~46 need a hand-written map**, because titles changed
between production and publication. The clearest example:

| `shorts` project | Its `description.txt` opens | The blog's `video_title` |
|---|---|---|
| `projects/trm` | "Under 1000 Parameters Beats GPT-4 at Mazes" | "976 parameters is more than billions?!" |
| `projects/pocket-llm` | "AI on your phone. All day. No internet required." | "AI in Your Pocket" |
| `projects/many-eyes-learning` | "Given enough eyeballs, all bugs are shallow." | "Given enough eyeballs..." |

The second and third would survive fuzzy matching; the first would not, and
would match wrongly and silently. Guessing here produces exactly the kind
of plausible-but-wrong relation this project exists to avoid, so the answer
is a committed map of about 46 lines — `video_url` to `shorts` project —
written once by hand, carrying `Provenance::Declared`.

## What `video-publishing` contains

Not an index of published videos.

- **`video-tracker.csv`** — 111 rows, columns for `publish_date`, `url` and
  `views`. Every row is `Backlog` or `Plan`; **zero have a URL** and none
  have a publish date. The "Published to sw-game-dev" notes refer to the
  *repository* being published, not the video. It is a backlog of videos
  not yet made.
- **`for-daily-release.org`** — a schedule of intentions by weekday theme.
- **`tools/`** — a Rust workspace of `vid-*` binaries (composite, concat,
  lipsync, montage, scale). Production infrastructure, not corpus.
- **`org-plans/`** — planned video counts per GitHub organisation.

One thing it does offer, marginally: the weekday theme taxonomy (Machine
Learning Mondays, Technology Tuesdays, Throwback Thursdays,
Sharpen-the-Saw Sundays) is the same vocabulary the blog's `series` field
uses, so it corroborates concepts rather than adding them.

## What this changes

1. **Video resources get a `body`.** The schema already anticipated this
   ([`plan.md`](plan.md) §12.3): `Resource.body` is empty rather than
   absent, and an ingester fills it with the narration. No type change.
2. **A video's concepts are declared, not inferred**, for the 26 episodes
   `concepts-status.txt` covers — 130 concept-to-episode edges.
3. **Step 009 (`ingest-metadata`) expects a committed join map** rather
   than matching titles at ingest time. The deterministic 5MLC rule can be
   code; everything else is data a human wrote.
4. **Nothing needs recovering from the old machine** for indexing. The
   media is regenerable and irrelevant here.

## What is still missing

No YouTube URL or video id is recorded per project in either repository —
only eleven YouTube links exist across all of `shorts`, none of them a
per-project publication record, and `docs/5mlc-concepts.md` carries only
three `PUBLISHED` date markers. The blog's `video_url` fields are the only
complete record of what is published and where, which is why the join map
is keyed on them.

If a per-project `published.txt` (url, date, id) ever becomes part of the
shorts workflow, the hand-written map stops being necessary. That is a
suggestion for that repository, not a requirement: 46 lines written once
is cheaper than changing a pipeline.
