Read docs/plan.md, then look at what https://github.com/softwarewrighter/start-here currently holds: a README and 13 pages under docs/, generated on 2026-09-26 by a throwaway Python script from the GitHub cache.

Replace that with a `just hub` recipe that renders the same pages from `build/corpus/corpus.ron`. Rust only; nothing Python ships. Output goes to `build/hub/`, which is gitignored like every other generated artifact, and is copied into the `start-here` repository by hand -- that repository is not a sibling checkout here, so anything it needs is a work order in `docs/start-here-requests.md` and a line in the session summary.

Why this is worth a step rather than leaving the script: the hub is the second consumer of the corpus after the docent, and it is the one a stranger reads first. Generating it from the corpus makes its claims testable, and the two defects the hand-rolled version shipped were both untestable-by-construction: a link to a repository that does not exist, and a list of repository names taken from clones on the author's machine, which published the names of private repositories. The corpus cannot express either mistake -- it holds only what a visitor can see, and every URL in it is checked by the existing gate. Say that in the crate's module doc, because it is the argument for the design.

What the pages need, and what the corpus does not hold yet: the organisation table wants three numbers per owner -- repositories of the owner's own, forks, and the public total -- so that it agrees with the GitHub page it links to. The corpus deliberately drops forks, so those tallies have to be carried deliberately, either in the corpus or in a small committed facts file beside it. Decide, and note the choice; do not recompute them from the network at render time.

Gates, which are the point of moving this into the workspace:

- Every public non-fork repository appears on exactly one topic page. An orphan is a failing build, exactly as with the corpus gate.
- The topic totals sum to the README headline, and both are tested rather than typed.
- Every link in the rendered output resolves to a resource in the corpus, offline.
- Two runs produce byte-identical output.
- Topic definitions, blurbs and the featured picks live in a committed, hand-editable `sources/hub-topics.ron`; the prose the owner has already approved in the published README is preserved verbatim where it is still true.

Exit: `just hub` renders the README and the docs pages into `build/hub/`, the four gates above are tests, `docs/start-here-requests.md` records what the hub repository must do with the output, `just check` green.
