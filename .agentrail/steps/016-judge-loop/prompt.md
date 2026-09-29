Read docs/plan.md, then docs/hybrid-docent.md sections 6 and 9.

Owner request, 2026-09-25: "I imagine a test/distillation/feedback-loop harness where a LLM interacts with our Jev-like-plus-matcher Docent, and evaluates its responses for helpfulness and accuracy. The LLM can record actionable feedback for you to correct. Can we set this up using an already installed Ollama model?"

Build it, as `atlas-judge`. A local Ollama model asks the docent questions, reads what atlas-answer replied, and scores the reply. Nothing Python: Rust plus a small HTTP client against http://localhost:11434, offline-only tooling, and the crate must degrade to a clear error when Ollama is not running so `just check` on a machine without it still passes.

The hard rule this step exists to respect: **the judge is never ground truth.** Its verdicts are Provenance::Teacher, exactly like the drafted question rows, and no number a judge produced may enter docs/reference/results.md or the README scoreboard as a measurement. What it produces is a queue of candidate defects for a human to confirm -- the same status ladder the question sets use (Drafted -> Confirmed). A judge that rates the docent 0.8 helpful is a model's impression, which the measurement discipline in CLAUDE.md excludes by name.

Shape:

1. **Questions.** The judge generates them from the catalog it can see, and asks the hard kinds on purpose: a thing that exists, a thing that does not, a thing in several places, a meta-question, and nonsense. Generated questions are candidate rows for sources/questions/, written to a review file in the form the owner already edits (` :: ` separated), never merged into a frozen set automatically.

2. **Judging.** For each question, the docent's reply plus the resources it offered (title, summary, url, maturity) go to the model, which returns a bounded, parseable verdict: helpful yes/no, accurate yes/no, the failure kind from a closed list (WRONG_DESTINATION, SHOULD_HAVE_SAID_NOT_YET, SHOULD_HAVE_ASKED_TO_REPHRASE, MISSING_BETTER_MATCH, UNGRAMMATICAL, INVENTED_FACT), and one sentence of actionable feedback. Reject any reply that does not parse rather than coercing it; an unparseable verdict is a recorded refusal, not a zero.

3. **Provenance.** Every verdict records the model name and digest, the prompt revision, the sampling settings, the corpus hash, and the date, because a judge's verdicts are only comparable to each other under an identical judge. Use the local models already installed (`ollama list`); llama3.2:3b for iteration, the largest available for the recorded run.

4. **INVENTED_FACT is the one verdict that is a bug in us, always.** The docent cannot invent -- every sentence is a frame over catalog text -- so a confirmed INVENTED_FACT means a frame is wrong or the catalog is wrong. Make that verdict loud in the report.

5. **Output.** `just judge` writes a report: the verdict distribution, the failure kinds ranked by count, and the individual rows with the docent's reply beside the judge's sentence, so the owner can read it as a to-do list. Generated, gitignored, regenerable. A short summary of what it found goes in docs/, with the judge's provenance beside it and the word "unconfirmed" on every number.

Exit: `just judge --limit 20` runs against a local model and produces the report; the unparseable-verdict path and the no-Ollama path are both tested without a network; docs/hybrid-docent.md gains a section on what the judge is for and what its numbers may never be used for; just check green.
