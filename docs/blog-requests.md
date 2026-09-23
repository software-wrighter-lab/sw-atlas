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

## What the ingester found, 2026-09-22

Step 002 wrote this file from the plan. These are the things only running
the ingester over all 126 posts could have turned up.

**The front matter held up.** 126 posts parsed with no exceptions and no
special cases, and every one of the 126 post URLs resolves. 508 declared
relations came out of it: 78 repositories, 84 videos, 6 demos and 157 cited
papers, plus the series chain. Nothing was inferred and no model was
involved, which was the bet this file opened with.

**One tiny ask: `video_url: ""`.** The post
`2026-03-13-rabbit-hole-rust-to-unsupported-isa` writes an empty string for
`video_url`. The ingester used to turn that into a resource with the
identifier `video:` and no address; it now treats an empty URL as no link,
so nothing is broken either way. Omitting the key when there is no video
would say the same thing more clearly, and this is the only post that does
it. Not worth a commit of its own.

**A real ask, worth thinking about: whose repository is it?** `repo_url`
and `repo_urls` mix two different things. 73 of the 78 repositories named
are Software Wrighter's own; five belong to other people —
`badlogic/pi-mono`, `crackanimad0r/MesaOS`, `lukehinds/nono`,
`weagan/Engram` and `XSkill-Agent/XSkill`. To a reader that distinction is
obvious from the owner; to an index it is a guess, and it matters: a
visitor asking "show me your code for this" should not be sent to somebody
else's repository, and "what have you built" should not count five projects
that are not yours. Today sw-atlas separates them by comparing the owner
against the list of accounts it fetches, which works only because that list
happens to be complete. A convention would be better — a separate key, or
listing third-party repositories under `papers` where citations already
live. The blog should pick whichever is least annoying to write; sw-atlas
will follow it.

**Nine repositories the blog links to are GitHub forks**, among them
`sw-game-dev/game-mcp-poc`, `sw-embed/sw-cor24-pascal` and
`softwarewrighter/bdh`. The repository owner decided they belong in the
corpus because he has written about them. Nothing is needed here; it is
recorded because a reader of this file would otherwise wonder why a fork
appears in the index.

**Demo links, when there are any: a name rather than a verb.** The same ask
as the campus's DEMO-NAMES. A post's `demo_url` carries no title today, and
where a link's text is `run` or `source` sw-atlas now treats it as a button
label and leaves the resource's title empty rather than inventing one. Six
posts declare a demo; naming them costs a few words each.

**Paper links: six hosts refuse robots and one is unreachable.** ACM,
doi.org and openreview answer 403 to a link check, and
`yann.lecun.org/exdb/publis/pdf/lecun-06.pdf` did not answer at all. None
of them fails sw-atlas's link gate -- a host that dislikes robots is not a
dead link -- and no change is requested. If a cited paper ever does rot, a
DOI link survives longer than a publisher's PDF path.

## What sw-atlas gives back

The Librarian: a question box on the blog that answers "have you written
about X", "is there a demo for this", "where is the source", and "what
should I read next in this series" — in under 10 MiB and under 10 ms at its
smallest tier, with no server, using the relations the front matter already
declares.
