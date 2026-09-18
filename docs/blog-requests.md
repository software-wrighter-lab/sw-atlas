# Requests to `blog`

Notes for agents operating in
[`software-wrighter-lab/blog`](https://github.com/software-wrighter-lab/blog).
sw-atlas reads that repository and cannot write to it.

Raised by sw-atlas Saga 1. The headline is that **nothing is being asked
for.** This file exists so that a change nobody thinks of as breaking does
not quietly break a downstream corpus.

## What the blog already provides

Measured across the 124 posts in `_posts/` on 2026-09-18:

| Front matter field | Posts | Links | What sw-atlas does with it |
|---|---:|---:|---|
| `title`, `date`, `author` | 124 | | resource identity |
| `abstract` | 124 | | the published summary; no model needed to write one |
| `keywords` | 124 | | a hand-written alias list, which is what a matcher needs |
| `tags`, `categories` | 124 | | concept candidates |
| `series`, `series_part` | 123 | | `SeriesNext` relations, and "what should I read next" |
| `video_url` | 75 | 75 | `Demos` relations to video resources |
| `repo_url` | 65 | 65 | `Implements` relations to repositories |
| `papers` | 64 | 204 | `Cites` relations to external work |
| `repo_urls` | 10 | 41 | the same as `repo_url`, several per post |
| `demo_url` | 10 | 10 | `Demos` relations to live demos |
| `video_urls` | 3 | 10 | the same as `video_url`, several per post |

Counted from `_posts/*.md` front matter on 2026-09-18; the Links column
counts list items, not posts. That is **201 declared links to Software
Wrighter repositories, videos and demos**, and 204 citations of outside
work, across 124 posts.

This is the single most valuable finding of sw-atlas Saga 1. Most projects
of this kind begin by running a large model over their corpus to *infer*
which article relates to which repository, which demo, which video. Those
inferences are expensive, they are wrong a few percent of the time, and the
errors are invisible. Here a human wrote them by hand, at publication time,
with full knowledge of what they meant, before a single model has been
run.

The corpus schema records this distinction as `Provenance`: `Declared` for
anything that came from front matter, `Derived` for deterministic analysis,
`Teacher` for anything a model produced. Every metric sw-atlas publishes
can be recomputed with `Teacher` rows excluded, and `Declared` rows are
never second-guessed.

## What would break it

No action needed — this is the watch list, in rough order of damage:

1. **Dropping or renaming `abstract` or `keywords`.** These are the two
   fields that make a post findable without reading its body. An index
   built from body text would be larger, slower and worse.
2. **Moving the cross-corpus links into the body.** A markdown link inside
   prose is a link; `repo_url:` in front matter is an assertion about what
   this post is about. sw-atlas can parse the former, but it cannot know
   the author meant it.
3. **Changing the URL field names** (`video_url` to `youtube`, and so on)
   without a note. The ingester pins the names it reads and its tests fail
   loudly on an unknown field, which is the intended behaviour, but a
   rename costs a session here.
4. **Retiring `series`/`series_part`.** They are the only ordering signal
   in the corpus and the only way to answer "what should I read next".

If any of these is worth doing for the blog's own sake, do it — and please
say so in the commit message so the ingester can be fixed in the same week
rather than discovered broken by a coverage gate.

## Possible future asks, not requested now

- **Video scripts — and this one has already been answered.** 75 posts
  carry a video, and the scripts for them exist as authored text in git
  repositories, not as machine transcripts waiting to be made. That removes
  the objection this entry was originally written around. sw-atlas would
  like the repository names when convenient; the ingester for authored
  scripts is close to the one that already reads front matter, and 75
  videos of written prose is plausibly the largest single knowledge gain
  available to the project. Nothing is needed from the blog repository
  itself — this is recorded here because the blog is where the
  `video_url` relations are declared, so this is where a reader would look.
- **A stable per-post identifier** independent of the filename. sw-atlas
  derives ids from the dated slug today, which is stable in practice
  because `_posts` filenames do not change after publication. If that ever
  stops being true, an explicit id becomes worth the trouble.

## What sw-atlas gives back

The Librarian: a question box on the blog that answers "have you written
about X", "is there a demo for this", "where is the source", and "what
should I read next in this series" — in under 10 MiB and under 10 ms at its
smallest tier, with no server, using the relations the front matter already
declares.
