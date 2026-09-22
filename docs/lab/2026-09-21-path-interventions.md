# A useful intervention needs the right input boundary

L67 established one native-equivalent unit/path result. This follow-up asks
whether that packet supports useful experiments. **It does support changing the
continuation budget; changing its request record does not retarget the saved
search.** The measured adjacent witness is **limit 95 → suspension, 96 →
completion**. These altered-input outcomes are model predictions awaiting a
native check. No live capture, TypeSafe request or main-loop change occurred.
Work starts from `7afe8a5` on the independent lab branch.

## Method and control

`tools/explore/path_intervention.py` prepares the same validated L67 replay,
requires agreement with its native post-state, and retains that runner's declared
memory. An opt-in callback in `payload_replay.explore` makes the prepared runner
available after the baseline report is frozen. The ordinary CLI is unchanged.
The A* return list is copied so later callbacks cannot contaminate that report.

Each trial changes exactly one four-byte captured input: `PathData.to_x`,
`to_y`, `tolerance`, or the delegated `PathFinderData.limit` word. The selector
uses immutable captured region bytes, validates the path header and extent,
and refuses unknown fields, code, scratch, read-only or absent data. It never
edits the packet. Limits are nonnegative and capped at 4,096 for this experiment;
each CLI batch has at most twelve trials. Existing instruction, allocation,
service and byte guards remain active.

Each trial runs twice from captured state, then an unchanged control runs again.
GPRs, all declared non-code backing bytes, initialization, service state and
writes must reproduce exactly; later batches also compare the complete modeled
instruction-address/size stream. Returned trials retain the entire unit and
all path-capacity bytes. Comparisons are explicitly against the **unmodified
native control**, not a native execution of the new input. New dependencies
refuse; no memory region or runtime service is added during a trial.

Four independently prepared batches produced **35 trial cases, 30 distinct
field/value pairs**. Every case returned, repeated identically and recovered
the original control afterward. All four frozen baseline reports match L67's
complete `last` result exactly. No trial needed a new declared region.

## The destination experiment exposed a semantic trap

The captured request is `(x=4550, y=32865, tolerance=0, flags=1)`. Trials changed
X by ±1 and ±48, Y by ±48, and tolerance to 48. All returned 8 with length eight,
capacity forty and 197,650 instructions. Only path slot zero changed; the other
39 slots and the unit record outside its path pointer remained equal to the
control. Representative X+48 and tolerance+48 reruns have exactly the same
instruction stream as the control, not merely the same instruction count.

The owned `find_upath@00682f30` and `astar_path@00683770` reads explain the
observation: this is a resumed call. Saving mode is one; the fresh-goal setup
branch is skipped, and A* reloads saved search state from the unit. Changing
the request record therefore changes the retained endpoint record without
rebuilding the search around that endpoint. These results **do not establish
how a fresh search reacts to a changed destination or tolerance**. A native
check of those edits would test that local mutation, not legitimate retargeting.

This is the experiment's first useful lesson: a successful baseline snapshot
is not permission to treat every plausible input field as an independent knob.

## A continuation-budget intervention changes the outcome

The matched PDB places `PathFinderData.limit` at offset `0x40` within the data
member starting at `pathfinder+0x40`, yielding `0xe85ec0`. The captured native
prefix has wrapper-entry modes `(20,0)` and delegation modes **`(300,1)`**.
An early verbal estimate of 75 was wrong; the intervention log reads its
`before` value from captured bytes and correctly records 300 on every trial.
The saving word is preserved, as are the destination and existing search trees.

Representative measured results:

| Limit | Return (signed) | Guest instructions | Path length/capacity | Outcome versus control |
|---:|---:|---:|---:|---|
| 0 / 1 | −1 | 15,610 | 1 / 10 | Request retained; five tree pointers saved |
| 16 | −1 | 43,278 | 1 / 10 | Saved continuation |
| 64 | −1 | 128,683 | 1 / 10 | Saved continuation |
| 75–79 (tested values) | −1 | 144,013 | 1 / 10 | Saved continuation |
| 84 | −1 | 155,031 | 1 / 10 | Saved continuation |
| **95** | **−1** | **165,934** | **1 / 10** | **Saved continuation** |
| **96** | **8** | **197,666** | **8 / 40** | **All 640 path bytes and non-pointer unit bytes equal** |
| 128 / 300 | 8 | 197,650 | 8 / 40 | Complete output and instruction stream equal |

The final narrowing independently rechecked 84 and 96, then tested 90, 93, 94
and 95. It establishes the adjacent 95/96 witness; it is not an exhaustive proof
of monotonicity over all budgets or other states.

At 95 the five unit words at `+0x104` through `+0x114` retain their captured tree
addresses, rather than becoming zero as in the completed control. The word at
`+0x148` becomes 223; the control retains 127 there. The unit differs from the
completed control at 24 byte offsets outside its path pointer. The path remains
one active record with ten slots of capacity. The reached save branch and these
observations support calling this a modeled suspension, not an emulator refusal.
At 96 the output agrees completely under L67's explicit pointer comparison,
but execution takes sixteen additional instructions. Output and instruction
agreement are different measurements; neither should substitute for the other.

Preparing a baseline took roughly a minute and a half here. The first sweep's
prepared execution plus fingerprint cost 1.57–1.63 seconds per trial; traced
low-budget suspensions ranged from about 0.11 seconds upward. Repeating a trial,
restoring its control, collecting output and initial preparation are additional
costs. These are local observations, not a universal speedup or native execution
benchmark. The benefit is controlled offline screening without reserving the
live lane for each question.

## Validation, retention and next native test

Seventy-two focused tests pass, including one-word selection, immutable inputs,
limit/saving separation, and recovery of the same runner after a deliberately
induced undeclared read following a partial write. Memory-service and lifetime
refusal/reset tests remain passing. The full gate at `/tmp/path-intervention-gate`
passes 332 rondata, 855 sim, 13 fixed and three doc tests, with all 782 requested
fixtures present. Clippy, formatting, the data survey and paperwork checks pass.
Peak tree RSS is 8,315 MiB with two workers. Final validation-note edits also
pass the paperwork guard.

The input is L67's retained `2026-09-21-native-poststate/map-14` packet, payload
SHA-256 `771fee1449962e9c96c45715c1963174493ecfd3880f2beed0d3495c210806eb`.
Sources, four complete reports, summaries, the bounded bisection driver and
validation are retained externally at
`/Users/rf-studio/ron-data/lab-experiments/2026-09-21-path-interventions`.
The first sweep predates instruction tracing and the limit selector; later
representative reruns add that evidence. The final source also reports captured
mode values explicitly. No captured data enters git.

The recommended native candidate is **delegation limit 95**, with 300 as the
already confirmed control and 96 as the optional neighboring completion case.
Prediction: return −1, retain the one-record/ten-slot request and the five saved
tree pointers, with the complete unit/path bytes compared to this model.
Native saved-tree contents are a separate output family: unit pointer agreement
alone would not prove that the continuation can resume correctly.

Before a native run, the collector must explicitly record the input change and
its point relative to the pre-call packet. A packet hash alone must not label an
altered native execution as an unchanged-input witness. That observer change
needs its own offline tests and a fresh capture slot through Ramon. **No native
intervention has been performed or booked.** Fable can review the finding and
choose whether this narrowly scoped pilot serves a current pathfinding question.
