# A sparse pointer-graph call with reset

## Result and scope

`Guy::turn_angles@005d98c0` and its nested
`GuyData::turn_speed@005de340` execute with 453 declared bytes in 18 regions,
occupying 45,056 bytes (44 KiB) of guest mappings. The code accounts for 336 of
those declared bytes. This is neither total Unicorn RSS nor a live capture.
The inputs are authored normal-squad-member fixtures, using the same field
graph as the preceding turn-speed oracle. No executable-wide mapping, game
startup, graphics device, or imported-function adapter is needed.

All 4,096 deterministic cases agree with existing Rust `movement::turn_speed`
and `movement::turn_towards` on the written heading and returned remaining
angle. Cases mix random headings with wraparound, half-circle, and facing
threshold neighbors; vary the packed/instant-stop predicates, body/unit mode,
half rate, speed divisor boundaries, and signed type-rate bit patterns.
This adds a composed function oracle; production arithmetic did not change.
It does not prove the fixture graph is the state reached by a natural caller.

## Bounds and reset contract

`tools/explore/bounded_call.py` maps only declared regions. Byte checks reject
access to adjacent page padding, even when page permissions allow it. Code is
immutable and bounded to the two function extents from the local listing:
134 bytes at the entry and 202 bytes at the nested entry, excluding padding.
Every invocation has a 256-instruction budget and must reach the declared stop.

The object graph consists of individual fields and pointer slots, rather than
zero-filled object allocations. Only the output word and a 40-byte scratch
stack are writable. The return address and four arguments are read-only.
Scratch reads require a preceding write to every byte **in the current call**;
the emulator's initial zeros and a prior invocation's stack contents are not
accepted as dependencies. Thus a missing caller input below ESP fails rather
than acquiring an accidental default. Crew fields are absent deliberately.

Before each invocation, restore the initial CPU context, all declared data,
and supplied GPRs/EFLAGS; clear the per-call scratch initialization set. Code
mappings and translated code can be retained. This restores the whole small
data set, not a dirty-page algorithm. Outputs include all GPRs/EFLAGS, all data
regions (including scratch), and the ordered memory-write events.

All cases were rerun in reverse order on the reused emulator and compared
against their complete first results. Each was then compared with a fresh
emulator. All 18 regions were individually removed from a case chosen to touch
the whole graph; every omission failed. The output must receive exactly one
four-byte write; callee-saved registers and `ret 16` stack cleanup are checked.

Eleven authored machine-code tests exercise nested pointer mutation/reset,
read-before-write scratch, stale scratch across calls, missing pointer bytes
on a mapped page, pointer escape, read-only writes, execution in page padding,
instruction budget, empty execution, region overlap, and forbidden code overrides. These need
no original executable or captured data.

## Measurements and artifacts

Local report `/tmp/turn-angles-report.json`: 4,096 reused calls took 0.868 s;
the same 4,096 cases in fresh emulators took 2.384 s, approximately 2.75 times
as long. Both timings include strict hooks and full output collection; the
fresh timing includes construction and comparison with retained results.
This is one in-process exploratory measurement, with reused execution first,
not a general end-to-end fuzzing speed claim. A final rerun after adding an
empty-entry guard took 0.986 s reused and 2.698 s fresh (about 2.74 times);
`/tmp/turn-angles-report-final.json` and `/tmp/turn-angles-oracle-final.txt`
contain that run's report and 4,096 passing rows. The release gate was running
concurrently with these measurements. The driver retains results for
reset verification, so its host memory use is not bounded to one capsule.

Original-derived rows are `/tmp/turn-angles-oracle.txt`, outside git. The source
SHA-256 is `30478a44b577cb11ebcbbbf53d3e93ba02fd2aacf3bdefa6552c9b6449625079`.
The driver hashes the source for provenance and reads function bytes from that
install; unlike the live capsule workflow, it does not bind an independently
recorded expected executable identity. A changed executable must earn the same
bounds and Rust comparisons again.

Reproduce from the repository root (keep output outside the repository):

```sh
uv run tools/explore/test_bounded_call.py
uv run tools/explore/turn_angles_oracle.py /path/to/install > /tmp/turn-angles.txt
cargo run --release -p rondata --example turn_angles_oracle_check < /tmp/turn-angles.txt
```

## Next boundary

Natural live entry/exit capture is still outstanding. A whole-export search for
`Guy::turn_angles(` found one caller body, `Unit::detect_boat_collision@005fa8b0`,
besides the helper's own definition. There was no named call in Unit's move-step
export. This is a static search result, not proof against indirect callers or
inlining. Do not assume the earlier frame-20 land-unit MoveTo exercise reaches
this entry: obtain a call witness for a boat-collision scenario first, or choose
a different mutator already observed in the trace.

Its witness must preserve
the pointer graph and arguments and must not overwrite the callee's dead stack
before recording it. The previous leaf witness's pushfd/pushad exit callback
would overwrite this function's scratch bytes. Reusing that witness unchanged
would manufacture a memory mismatch. Either move the exit witness to a separate
stack before recording, or explicitly limit the live comparison to semantic
memory and label that narrower contract. No live agreement is claimed here.

The reusable runner is deliberately experimental: no automatic dependency
closure, arbitrary x87/SIMD state capture, multithreading, self-modifying code,
coverage-guided corpus, or failure minimization. The next useful result is a
natural-call witness checked with this sparse executor and the existing Rust
oracle, followed by minimization of a deliberately introduced disagreement.

## Validation

The 11 new authored executor tests and 11 existing live-capsule guard tests
pass. The Rust checker rejects both an empty table and a deliberately changed
heading. Clippy with warnings denied, formatting, seven documentation guards,
and the local install survey pass. Full release-gate results are recorded in
the journal after completion; none of this tranche changes production sim code.
