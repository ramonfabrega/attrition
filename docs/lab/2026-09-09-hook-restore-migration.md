# RNG/frame and modal register-save migration

## Prior evidence, and the implemented boundary

The existing `tools/trace/wow64bop.c` and ORACLE's "Coverage is back" section
already identify the WoW64 fault signature and record instruction-isolation
experiments. The previous lab plan missed this prior work. Its measured
compatibility rule had been applied to coverage stubs; ORACLE explicitly left
the RNG/frame hooks as a follow-up. This change completes that follow-up for
those hooks and the lab's modal wrapper. It does not repair Wine/Rosetta.

`tools/trace/hook_stub.h` replaces bulk register/flags saves with individual
volatile-register saves and LAHF/SETO/SAHF arithmetic-flag preservation. The
cdecl observer preserves callee-saved registers and control flags. Original
arguments, caller address, EBP, integer registers, stack, and arithmetic flags
reach the displaced prologue unchanged. The modal wrapper uses the same
technique while retaining the complete original native argument stack.

`test_hook_stub.py` executes bytes emitted by the production builder on 256
authored states. `test_autostart_abi.py` executes the actual compiled modal
wrapper on another 256. Each covers all 64 arithmetic-flag combinations and
both values of EFLAGS.ID. Callback models obey the cdecl control-flag contract:
they clobber arithmetic flags, EAX, ECX, and EDX, while preserving IF/DF/ID and
callee-saved registers. A wrong stack operand fails in each test. The emitted
instruction guard was made to reject each of PUSHAD, POPAD, PUSHFD, and POPFD;
it now checks the real generated hook and compiled modal bytes.

Optional capsule-acquisition wrappers still use bulk restores. They are not
covered by this migration or the post-change reliability sample. Their native
acquisition path needs its own register-image-preserving conversion; previously
captured offline replay remains separately testable. No simulation rule changed.

Update, September 19: the restore-specific wrapper now has an
[tested register-image migration and one successful native capture](2026-09-19-restore-collector-abi.md),
including x87/SSE preservation and an explicit captured-flags mask. Other optional
wrappers and general native reliability remain outside that result.

## Segment diagnostics

The fault recorder now emits INFO 178 with CS, SS, EFLAGS, CONTEXT flags, and a
fault ordinal; INFO 176 carries the matching ordinal in its formerly reserved
last payload. The ordinal is allocated atomically. The bounded reader joins
records by ordinal, not adjacency, and retains at most eight fault contexts.
An authored interleaving fixture prevents associating one thread's segments
with another fault. The x86 layout is also declared by `wow64bop.c`.

A frozen-source baseline used two fixed diagnostic pairs with the segment
probe and loader logging. All four launches succeeded, so it supplied no new
fault segment state. Those results are not evidence against the fault. The
frozen source tree is `/tmp/attrition-segment-baseline-source`; its cohort is
`/tmp/attrition-segment-baseline-two-pairs`.

## Reproducer repairs and current limitation

Three inline-assembly strings used Intel spellings under the pinned compiler's
AT&T parser. Use PUSHAL/POPAL and PUSHFL/POPFL there; emitted instructions are
the intended original classes. The NOARM verdict also incorrectly required
ROUNDS hook hits even though that mode never arms a hook. It now requires zero
hits with the same target result and no declined exceptions, and labels the
unarmed control separately. Both are separable changes to the existing tool.

The first new three POPAD and three safe-sequence NOARM trials all reached
phase C, zero hits, zero declines, and target result 42. Their original exit-1
verdicts are retained in `/tmp/attrition-bop-results.json`; they were verdict
errors, not reproduced runtime faults. The old historical observations remain
historical: this small fresh instruction-only batch did not reproduce them.

## Capture result

The post-change cohort is fixed at ten pairs, with no retries, endpoint 1400,
seed 12345, map 14 then 18, 180-second timeout, ordinary Wine logging, and no MF
probe. It uses the same protocol as the earlier 6/10-pair batch. Every attempted
launch remains in the denominator, and an unattempted second map is separate.
At least one pre-menu WoW64 fault persists after removing bulk restores from
these hooks. The migration therefore does not solve startup reliability.

The batch completed **9/10 pairs, 19/20 launches**. Pair 1/map 18 failed
before menu entry at the same WoW64 instruction; the other 19 launches passed
and matched all 19 logged bodies and all 1401 frame/seed pairs against the
independent pre-change baseline. Every attempted launch restored and verified
five profile files. Compared with the prior 6/10-pair, 15/19-launch sample,
this is encouraging but is neither a controlled causal estimate nor a promise
of long-term reliability. No simulation score moved.

This cohort ran ordinary unattended builds. At that point the extra segment
packet was still limited to the MF probe, so its single failure has no CS/SS
witness. The final fault recorder enables that packet in every unattended
build; this only executes after an access violation. It does not backfill
missing evidence or change any pre-fault execution. Retained evidence:
`/tmp/attrition-safe-hooks-ten-pairs/cohort.json` and `comparison.json`;
recipe `/tmp/run-attrition-safe-hooks.py`, comparison
`/tmp/compare-attrition-safe-hooks.py`.

## What remains open

A linear byte sweep alone is not an execution trace. Disassembly and the
function index identify POPFD in the original CPU-detection functions
`wincpuid@0054cea0`, `wincpufeatures@0054cf90`, and `check_80386@0054d100`.
Whether those sites execute before a failing launch is not established here.
A targeted startup witness is needed before changing that path. Do not patch
CPU detection merely because it contains an instruction from the historical
reproducer.

The follow-up standalone batch supplies a current positive reproduction without
the game, its renderer, or its assets. Two ID-enabled POPAD instruction trials,
two ID-enabled safe-sequence trials, and two unarmed controls passed. Of two
armed callback trials, one passed and one exited 5 with an unhandled read at
`0x4ecd`, fault PC `0x7bf21139`. The first x86 exception dispatch records
CS `0x0107`, SS `0x0023`, and EFLAGS `0x202`; Wine's debugger labels it WoW64
32-bit code and decodes the instruction as an absolute load from `0x4ecd`.
The earlier x64 exception-dispatch record reports a different restored RIP
and CS `0x002b`; these are distinct exception contexts, not interchangeable
snapshots. Together with the installed-image instruction check, this supports
the wrong-mode interpretation for this reproducer. It does not establish the
mechanism that caused the transition failure, or supply the missing game-run
segment context.

The armed shape includes code patching while the syscall thread runs as well
as the callback's bulk restores. One failure in two armed trials versus zero
in two controls does not isolate either cause. The instruction-only negative
results must stay alongside this positive result. Next isolate those two
factors in this small executable before another game-startup patch. No renderer
or menu flow is needed to reproduce this particular runtime fault.

Evidence: `/tmp/attrition-bop-followup-results.json`, recipe
`/tmp/run-attrition-bop-followup.py`, and
`/tmp/attrition-bop-callback/run-1.log`. Builds use STUB=1 and THREADS=1;
ID instruction lanes use NOARM=1, IDFLAG=1, and LOOPOP=2 or 4; armed trials use
NORESTORE=1 and ROUNDS=10000, with NOARM=1 added for their controls. The runtime
mechanism, long-term reliability effect, and full renderer independence remain
open.
