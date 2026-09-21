# Jev on a real investigation: resumed search and explicit thread state

## Outcome

The question was what prevents an entire suspended path search from replaying
outside Wine. Jev located the decisive existing report, and an executable probe
advanced the **modeled prologue** from its fifth instruction (missing FS state)
to its fourteenth (missing game data). This is not a resumed-search replay:
arguments and exception-chain input are authored, not captured from a game.

No fidelity score, simulation code, main-loop document or capture setting changed.
The experiment is on `codex/jev-lab`, based on `f616bfc`. The old capture files
could not be found, so no new claim of reproducing their live state is made.

## Search experiment

The frozen investigation manifest and observations are retained in
[the compressed evidence](2026-09-19-jev-resumption-observations.json.gz).
It contains only authored project prose/source, responses, source hashes and
numerical observations; no executable, decompile or captured game bytes.

Five questions asked about complete resumption, cleanup versus resumption,
thread context, capture coverage, and missing-input guards. The corpus was
**57 tracked files, 171 passages**: September 9 lab reports, emulator/pathfinder
documentation and selected restore/search/emulator tools. Both searches used
that file set. The exact patterns, queries and content hashes are in the artifact.

Jev made **171 requests**, returned **855 relevance judgments**, and took
**26.24 seconds**, with no errors or cache hits. Reported input usage was
**259,815 tokens**, estimated **$0.010912**. Cumulative estimated Jev spend is
**$0.059800** against the user's $1 ceiling. This uses the previously recorded
price and shared budget ledger, not an invoice. No further API calls were needed.

The lexical baseline used `rg -n` with
`find_upath_restore|TLS|FS:|fs:|exception|recycler|allocator` across the same files:
**69 matching lines in 23 files, about 8 ms**. It found the restore report,
cleanup runner and coarse FS setup. This is a targeted regex baseline, not BM25.
The investigator had read the graph and restore reports before freezing this
experiment; the manifest discloses that familiarity. There is no blind discovery,
human-time speedup, exhaustive relevance labeling or statistically independent
comparison here.

| Question | Jev top-five observation after reading the sources |
|---|---|
| Complete resumption | Restore report's dependency section ranked first; directly useful. |
| Cleanup versus resumption | Cleanup runner ranked first, but the restore report was absent. The shortlist alone does not answer both halves. |
| Thread context | Restore dependency section first, coarse command oracle third, strict runner fifth. Useful cross-checking set. |
| Capture coverage | Restore report first, its verifier third and native collector fourth. Useful navigation to what was actually recorded. |
| Missing-input guards | Relevant negative-control reports and runners, but neither the strict runner nor its tests in the top five. Inspection still needed. |

The result supports an optional semantic source finder, not a replacement for
ordinary search or evidence checking. It did not uncover the new runtime boundary;
executing the probe did. The compound cleanup/resumption miss is a concrete reason
to split questions and inspect both implementations before drawing a conclusion.

## What actually blocks which operation

[Natural graph cleanup](2026-09-09-natural-search-graph.md) and
[search resumption](2026-09-09-restore-entry.md) are different operations.
The former's full cleanup reached an allocator capacity input at `0xc8d81c`;
the latter's delegated search stopped earlier at `0x682f3a`, reading FS offset
zero. Restoring enough graph bytes for disposal does not establish enough inputs
for continued A* search. Those historical measurements are cited, not rerun here.

The documented `/tmp/attrition-restore-entry/restore-prefix.bin` is absent.
An actual verifier invocation failed with `FileNotFoundError`. A filename search
including ignored files under `/private/tmp`, the owned install and the old
`28ba` worktree found neither restore packet nor graph packet. A broader initial
worktree search also found none. This is a local artifact-availability finding,
not a claim that no archive exists elsewhere. No capture was booked to hide it.

## Small executable experiment

`tools/explore/thread_context_probe.py` loads 256 original code bytes from the
user's install at `0x682f30`; bytes remain external. It supplies authored arguments
and a write-before-read scratch stack, runs under an instruction budget, and
requires the exact expected refusal address and PC. An unexpected failure or
successful return fails the experiment. The output binds the source executable
by SHA-256; the observations retain that hash.

`ThreadCall` is an opt-in subclass of the unchanged `BoundedCall`. It declares a
four-byte exception-chain head at a nonzero address, a small host-authored GDT,
an FS descriptor, and flat 32-bit stack/data descriptors. Every data access still
passes the existing byte guard. No page at zero, complete TEB, TLS array, heap,
world snapshot, exception dispatcher or successful callee stub is supplied.
The old exception head is the authored sentinel `0xffffffff`.

The setup follows the segmentation mechanism shown in
[Unicorn's pinned 2.1.4 sample](https://github.com/unicorn-engine/unicorn/blob/2.1.4/samples/sample_x86_32_gdt_and_seg_regs.c),
with independently authored packing and narrower byte regions. Two setup failures
were useful: direct `FS_BASE` writes in the installed 32-bit binding warned and
left the base at zero; setting FS without an explicit 32-bit SS caused stack
address truncation. The tests now exercise high stack addresses and segment reset.
Neither failure is evidence about Wine or the game's search algorithm.

| Setup | Attempted instructions | Refusal PC | Missing read | Mapped memory |
|---|---:|---|---|---:|
| Strict runner, no thread context | 5 | `0x682f3a` | address zero, 4 bytes | 16 KiB |
| Explicit modeled thread context | 14 | `0x682f54` | `0xcae5fc`, 4 bytes | 24 KiB |

The faulting instruction is counted: the second run completed 13 instructions.
It installed the stack exception link, saved the old head and cleared XMM0.
Assertions check those effects. The prefix overwrites XMM0 before use; this says
nothing about later SIMD requirements. The owned install's `sbl/rise_z.map`,
line 36368, associates the next slot with `div_3_table` in `coord.obj` (map SHA-256
`e2a478e181f5cafb1b4533018ca69f80f65e4ec90070c80f785bc4afd5cac0a1`).
This identifies a concrete next dependency, but does not establish its contents,
extent or initialization. The probe invents no table value and stops there.

Eight authored-instruction tests cover FS reads/writes and reset, stack linking
and restoration, null reads, FS padding, missing pointer targets, scratch reads,
descriptor input rejection and SIMD context reset. A deliberate in-memory mutant
removed the byte guard; the FS-padding test failed as intended. This also shows
why the region guard remains necessary rather than relying on segment limits.
All eleven existing bounded-call tests pass unchanged.

## Next useful experiment and limits

The next useful acquisition is a durable, image-bound restore-entry packet that
includes the relevant thread context and the newly observed game-data dependency,
then repeats this refusal-driven process against the live delegation state.
Capture only what execution actually requests, and compare a real return or
intermediate state before claiming a runnable search. Fresh acquisition also
requires reviewing the optional collector's old register-saving wrapper against
[the migration report](2026-09-09-hook-restore-migration.md); this experiment did
not repair it, run Wine, or resume the separately paused runtime matrix.

The current result establishes that a narrow explicit thread model can execute
this prologue while missing dependencies still fail. It does not establish that
the model equals a live thread, that exceptions work, or that a whole search is
close to completion. Building an entire Windows environment would be a different
project; the next captured dependency should determine whether to continue.

## Reproduction and validation

```sh
uv run --offline tools/explore/test_thread_context_probe.py
uv run --offline tools/explore/test_bounded_call.py
uv run --offline tools/explore/thread_context_probe.py /path/to/game
```

The two Python suites and actual-install probe pass. The full release gate passed:
332 rondata tests (two ignored), 855 sim tests, 13 fixed tests and three doctests;
782 fixture requests, none missing; peak monitored tree RSS 12,171 MiB under the
20 GiB cap. Formatting, Clippy, data survey and paperwork checks passed. Reports:
`/tmp/jev-resumption-gate-verified`, log at the corresponding `.log` path.
The initial sandboxed attempt refused before release tests because it could not
sample RSS; the successful rerun retained the monitor and all checks.
