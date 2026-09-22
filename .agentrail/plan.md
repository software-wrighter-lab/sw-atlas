# Saga: atlas-baseline (MB02)

Port `moe-microscope`'s MB01/MB01t matcher from one campus to the whole
corpus, and build the evaluation harness before there is anything to
evaluate. Exit: the number every later saga must beat, measured over 643
artifacts, and the frozen question sets that make the comparison honest.

No model is trained here either. This saga produces the yardstick, and the
yardstick has to exist before the hybrid docent (Saga 3) can claim
anything. docs/plan.md section 10 holds the architecture; docs/reference/
results.md holds the rows this saga fills; docs/hybrid-docent.md section 6
holds the verification rules this saga has to satisfy.

The order matters and is deliberate. The evaluation sets are authored and
frozen FIRST, before the matcher is measured and long before any training
row exists, because a question set written after seeing what a matcher gets
right is not a test. demo-decision-model froze its 96 probe labels before
writing any v2 training frame, and that is why its model-2 comparison is
believable; this saga adopts the same discipline.

Sources are read-only siblings. Every step ends green: `just check`, then
commit, then `agentrail complete`, then push.

## Steps

1. authored-questions -- the frozen evaluation sets. At least 300
   cross-corpus paraphrases (no alias appearing verbatim), plus exact-name
   questions, off-topic questions, ambiguous questions with their expected
   sets, meta questions (corpus, self, why) and multi-turn follow-ups. Each
   row carries its text, its expected answer, its intent, its split and its
   origin. Drafted rows are marked unconfirmed and are not usable as a
   frozen set until the repository owner has passed over them; the file
   records who confirmed what and when. A committed hash freezes each set,
   and a leakage check by hash and by 8-gram overlap fails the build if a
   frozen row ever appears in a training row.

2. matcher -- the A0 runtime over the whole corpus: alias, concept, title
   and abstract signals with the MB01 weights, the MB01t text variant, plus
   graph traversal over relations. Ported, not reinvented: the weights are
   moe-microscope's and any deviation from them is recorded as a deviation.

3. eval-harness -- `just eval`: intent accuracy, destination accuracy,
   top-3, MRR, unsupported recall, ambiguous top-2, ECE, Brier, p50/p95
   latency and bytes, over every frozen set, for every arm. Plus
   MB02-oracle@k, the share of questions whose answer is in the matcher's
   top k -- the ceiling for anything that reranks, and the cheapest
   decisive measurement in the project. Paired statistics, because every
   arm answers the same questions: McNemar on per-question correctness and
   a paired bootstrap interval on every margin, reported beside it.

4. scoreboard -- the measured MB02 and MB02-oracle@k rows written into
   docs/reference/results.md by the harness rather than by hand, the README
   quoting them, and a plain statement of what the oracle number means for
   Saga 3: whether a reranker has room to win at all, or whether a second
   candidate source (A1 retrieval) has to come first.

5. close -- request files brought up to date with whatever the harness
   found, Saga 3 queued with what it needs and from whom, and the saga
   closed with `agentrail complete --done`.
