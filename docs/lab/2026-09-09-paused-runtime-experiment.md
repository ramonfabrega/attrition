# Runtime experiment paused; independent performance work continues

## Why it existed

The proposed comparison would separate two factors in the small standalone
WoW64 reproducer: arming the hook before versus after the competing syscall
thread starts, and bulk versus individual register restores. A prior armed
trial reproduced the startup fault without the game or renderer. It still
combined these factors, so neither is established as the cause. See the
[measured migration report](2026-09-09-hook-restore-migration.md).

This is runtime reliability research, not a prerequisite for Rust simulation,
replay, reader, or renderer development. A positive result would narrow a
runtime investigation; it would not itself deliver headless execution or
prove a Wine repair.

## Pause and exact handoff

The user reported the product's cybersecurity restriction and elected to
sidestep this experiment. Its exact trigger is not visible to this session.
Do not infer that execution alone caused it or move execution to another
agent as a workaround. Resume only within applicable product access and policy.
No four-way native comparison has run; there are no matrix results to interpret.

Last validated and published implementation: `2a13ced` on PR #3. The following
files remain local, uncommitted drafts, deliberately excluded from this
paperwork commit:

- `tools/trace/wow64bop.c` and `wow64bop.sh`: PREARM/SAFESAVE switches.
- `tools/trace/wow64bop_safe.h`: authored individual-save callback stub.
- `tools/explore/test_bop_safe.py`: authored-state ABI fixture.

The already-started ABI check completed: 256 cases passed and a wrong callback
argument mutation was rejected. This is a byte-level emulator check only;
Windows build validation, native behavior, matrix controls and final review of
the drafts remain incomplete. The drafts are not an adoption recommendation.
No executable, DLL, game data or decompiler output belongs in the handoff.

Separately, the last committed tracer completed another two-map run through
frame 1400. Both maps matched all 19 logged bodies and 1401 frame/seed pairs
against the independent baseline; each restored and verified five settings
files. Game processes closed. Receipts and comparisons are under
`/tmp/attrition-dll-confirmation-20260909`. This small successful pair does not
close the intermittent startup failure. Temporary evidence remains ephemeral.

## Independent next target

Refresh the release-test reader profile before changing fixture retention.
Ledger L26 measured run13 being loaded 50 times, but that measurement predates
later streaming changes. The current `capture::read` still allocates a complete
String; setup tests retain several whole-log consumers. Measure the current
call counts, logical input bytes, elapsed spans and sampled process-tree RSS.
Use the existing Rust-only `tools/explore/profile_gate.py` with no concurrent
source editing or compilation: it temporarily instruments two Rust files and
restores them in `finally`. Preserve the old profile log before a new run.

Select the largest remaining repeated reader from that fresh evidence, then
compare complete query/diff results before accepting a bounded reuse or
streaming change. Do not introduce a process-wide cache of raw captures merely
to reduce read counts: it could increase the memory footprint we want to lower.
Elapsed nested spans are not additive CPU costs, and logical bytes are not
physical disk I/O. This work needs existing capture files only, with no live
Wine process, DLL instrumentation, executable patching or GUI control.
