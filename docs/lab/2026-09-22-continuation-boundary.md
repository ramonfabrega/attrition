# Second-call boundary: immediate re-entry works, native chaining is unproven

The saved-tree checkpoint is packaged as draft
[PR #6](https://github.com/ramonfabrega/attrition/pull/6), stacked on PR #5 and
frozen at `af4ed73`. Further research runs on `codex/continuation-lab` from that
tip. Fable's review is an adoption boundary, not a block on independent research.
No capture was requested or run for this work; the lane remains released.

An immediate second modeled invocation now completes from the validated limit-95
suspended state. Both **95 then 300** and **95 then 95** return 8, with all
**640 path-capacity bytes equal to the single-call 300 control**. This is a
frozen-world counterfactual, not a native second-call fidelity result. Two unit
words differ, and the current suspended-graph observer refuses the completed
state. Those limits are part of the result.

Follow-up: [L77](2026-09-22-absent-search-state.md) closes the absent-container
observation gap, identifies and traces the two counters, and graduates the
recurring probe into reusable lab tools. Native second-call fidelity remains open.

## Disk before capture

The three retained paired-call captures each contain only one full pre-call
payload and one post-unit/path witness. Only the latest has a post-graph witness.
Their proxy traces continue beyond that event, so the later call can be located
without a new run. Matching the same path-header address identifies a candidate
continuation, not a fresh proof of object identity for every later frame.

| retained capture | first witness frame | next call using same path-header address | intervening other A* calls | later internal A* result |
|---|---:|---:|---:|---:|
| unchanged native post-state, September 21 | 224 | 466 | 529 | 0 |
| native limit 95, September 22 | 224 | 225 | 1 | 1 |
| native post-graph, September 22 | 224 | 225 | 1 | 1 |

In both altered runs, another unit's search executes in frame 224 before this
path is searched again in frame 225. The later proxy event provides the path
header, step/anti arguments and internal return, not the later outer return,
complete argument/register boundary, intervening world writes, unit/path bytes
or saved graph. Treating immediate re-entry at frame 224 as a replay of that
native frame 225 would invent those missing inputs. The two altered traces agree
on this limited event pattern; their later output state has not been compared.

## Bounded offline experiment

The existing `.run()` deliberately resets captured memory, scratch definedness,
allocator cursor, allocation history and retired objects. Calling it twice
therefore cannot test stateful continuation. A one-off authored driver, retained
outside git, instead enters the same declared callee a second time after a
successful first return, without restoring heap/global memory or model histories.
It uses the existing instruction budget and byte/lifetime/service guards.

The second boundary is explicit: original captured GPR/arithmetic-flag values,
original six callee argument words, and a selected limit with saving still one.
The modeled stack's initializedness is cleared so an unwritten local cannot
inherit proof from the previous invocation. Heap/global initializedness, allocator
cursor, allocation records, retirement records and cumulative service quotas
survive. Existing world/frame inputs remain frozen. Extended state is whatever
the first modeled call left; native FXSAVE is still not imported.

Only two invocations occur in each trial. The first is the native-validated
limit-95 model and takes 165,934 instructions. Before the second, it has 100
modeled allocations and an arena cursor of 3,116 bytes. Each second call takes
33,494 instructions, adds 13 allocations and leaves the cursor at 6,832 bytes.
Six frees are recorded across completion; retirement stays enforced. Both
trials return 8 with path length 8 and capacity 40. The complete path-capacity
bytes, including inactive slots, equal the single-call 300 control.

Each chain is repeated from a fresh reset of the captured first boundary. Its
full second-call model fingerprint repeats exactly. The unchanged single-call
control is restored after each experiment and agrees with the prepared baseline.
The first call's native unit/path witness is rechecked before the trials; no new
native second-call comparison is claimed.

## Whole-unit differences and completed-state gap

The path match does not imply whole-unit agreement. Compared with the single-call
300 control, both chains differ at two byte offsets, corresponding to these
little-endian unit words:

| unit offset | single-call 300 | either two-call chain |
|---|---:|---:|
| +0x134 | 83 | 123 |
| +0x148 | 127 | 223 |

These are observed values, not newly named fields or formulas. The latter value
was already observed after suspension in L68/L70. No semantic explanation for
both words is established by this experiment. All other unit bytes agree with
the control, including the path pointer. These differences remain evidence;
they are not masked to make the two ways of reaching the path equivalent.

All five saved-tree pointers are null after completion. The current L72 observer
was deliberately scoped to a suspended graph with five present containers. It
retains the unit view and then refuses `invalid graph record extent`, rather than
inventing empty header records. Thus this experiment has a complete unit/path
observation, not a complete post-completion recycler/graph comparison. L75's
suspended native comparison remains unchanged and valid within its stated scope.

## Guard controls and next falsifier

Six authored-program tests check the one-off re-entry mechanism. Heap values and
allocation ownership survive; removing ownership or initializedness causes the
expected read refusal; retired objects remain inaccessible; prior stack writes
do not authorize a new invocation's uninitialized read; malformed boundary inputs
refuse before writes. An explicit bad-model control erases retirement and wrongly
permits a freed-object read. This is why carrying bytes alone is insufficient:
the guard needs the retained lifetime history as well.

The experiment source and tests stay one-off, per the lab's convention; there is
no new supported runner API and no change to the reset semantics of existing
call tools. The result does not demonstrate multi-frame or native chained replay.
Before booking another capture, the next concrete work is an explicit completed-
state observation contract (including the unit view and recycler capacity), with
negative controls for partially missing containers. Then choose between a paired
immediate native re-entry experiment and observing the natural next invocation.
The latter must account for the intervening call and frame change; matching only
its return would be too weak. No capture slot is currently requested.

Evidence and authored one-off sources are retained under
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-continuation-boundary`.
No main score changed. PR #6 and its predecessor remain untouched by this work.

Validation: six authored re-entry tests pass, and the final release gate exits
zero with 332 rondata, 855 sim, 13 fixed-point and three doc tests. All 782
fixture requests are present; clippy, formatting, install survey and paperwork
checks pass. Peak process-tree memory is 8,069 MiB. The validation-note edit
is followed by the fast guard and whitespace check.
