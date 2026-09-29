Read docs/plan.md, then docs/sw-campus-requests.md (the EASEL-CONTRACT section) and docs/hybrid-docent.md section 9.

sw-campus is blocked on this. Its plan queues `docent-pins` -- ephemeral push-pin overlays for campus, building and wing destinations -- "after the sw-atlas query contract stabilizes", and its MVP saga is closed, so that queue is what it picks up next. The EASEL-CONTRACT section in docs/sw-campus-requests.md promises a concrete interface proposal "when Saga 9 opens", which is five sagas away. The campus cannot wait for a runtime that does not exist, and it does not need to: what it needs is the shape of an answer, and that shipped in step 011.

Write the contract. Not a plan for one -- the thing the campus can build against this week, in docs/sw-campus-requests.md, superseding the advance notice rather than deleting it.

What it has to say, all of it true today and measured:

The answer shape. Five outcomes, and three of them are refusals: one resource, the closest few, not yet, nothing here, please rephrase. The offer is a list of stable resource identifiers, never prose. The evidence line is for a trace panel and is not machine-readable. Say which of those the campus should render as a pin (an offer with campus place identifiers in it) and which it must render as words (a refusal has nothing to pin).

Identity. Identifiers are derived from URLs and are already shared between the two repositories -- three of them join campus places to blog posts today with nothing inferred. The campus needs the rule, not the list: how a place identifier is formed, and the guarantee that it does not change when a title changes.

The guarantees, which are the reason a campus can trust this: every visitor-facing string is a frame with catalog text quoted into it, so the docent cannot invent an exhibit; the model tier, when it exists, emits an intent and concepts and never a URL or an identifier; A0 answers in under a millisecond at p50 and fits in 10 MiB.

The honest limits. The scores are not probabilities -- calibration is Saga 8 -- so a pin must not render a percentage. Over the confirmed questions the docent commits to an offer on 0.969 of them and the offer holds the expected destination 0.431 of the time, which is what a "closest few" pin cluster is worth. A single confident pin is not available at any threshold: no score-and-margin combination names one resource above 0.54 precision.

What is still undecided, marked as such: packaging and the mount point (Saga 9), the model tier (Saga 3), calibrated confidence (Saga 8).

While you are in the file, update the "What the ingester found" section: `dist/catalog.json` still does not exist and the campus MVP has closed without it, so CATALOG-EXPORT and DEMO-NAMES are both still live requests, and the ingester's numbers are now Saga 2 numbers rather than Saga 1 numbers. Say whether either is now blocking anything.

Exit: the contract is committed and pushed, it states nothing sw-atlas has not measured or built, the session summary names it as the unblock for sw-campus, and `just check` is green (documentation-only commits still hold the gate).
