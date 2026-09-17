# A natural semantic mutation compared with Rust

## Result and boundary

A natural `Unit::kill_current_path@005e31d0` invocation at frame 36, during the
bounded run's shutdown, reduced a two-waypoint path to zero. It was not a
probe-created invocation or an edited live object. Two fresh emulators and a
reset invocation match the live GPRs/EFLAGS and all captured gameplay data.
Replay executes 35 original instructions and verifies the two length stores,
2 → 1 → 0. All nine missing-region controls fail.

The existing Rust path-deletion operation agrees with the observed result.
Another 4,096 live-derived synthetic cases vary active length from zero to two,
FINAL flags and unrelated flag bits, coordinates, tolerances and padding. Every
case matches a fresh emulator and Rust; 682 preserve at least one waypoint.
The Rust comparison checks **all surviving coordinates, tolerances and flags**,
not only length. The original replay also requires every waypoint byte,
including unused capacity and padding, to remain unchanged.

This is the first natural nonzero mutation in this branch's capsule experiments
with a direct Rust semantic comparison. It does not extend the parity headline.
The sampled call is shutdown teardown, not a witnessed in-match cancellation.
The live input has no suspended pathfinding search; cleanup of allocator-owned
search trees is explicitly outside this capsule. Lengths above two are not
covered by the mutation run, even though the witness can capture up to 64.

## The production operation is the oracle's operation

ORDERS §3.2 already specifies discarding the top path segment through the first
popped waypoint with FINAL set. The existing loop moved from Sim's private
`kill_current_path` body into `Unit::discard_current_path_segment`; the Sim
method delegates to it. No arithmetic or ordering changed.

`crates/rondata/examples/path_capsule_check.rs` uses that public unit operation.
It constructs neither a world nor a simulation, and compares the full surviving
`PathData` sequence. This is an extraction of production behavior for testing,
not a second Rust formula written to agree with the original. Empty input and
a deliberately wrong survivor count are rejected by the checker.

## Capture and execution contract

`RON_PATH_CAPSULE` enables `tools/explore/live_path_capsule.h`. It selects the
first call in frames 0–240 with a nonempty path of at most 64 entries and no
suspended search (`openlist == 0`). The standard staged run still quits at 36;
the larger eligible window is a bound, not a claim that those frames ran.
Existing HIT records had previously witnessed the entry at 200/69/200 in runs
53/54/906; the short command-probe run reached it during shutdown.

The witness captures the path header, all initially active 16-byte PathData
records, the null search pointer, GPRs/EFLAGS, and the redirected return slot,
before and after. The layout is already documented in ORDERS and PATHFINDER.
The original path array remains allocated for this no-search call; deletion
changes length without freeing or overwriting the waypoints.

The local instruction listing establishes three allowed code spans: the
80-byte path-deletion entry; the 17-byte null-search test at
`Unit::clear_partial_path@005e3920`; and its two-byte return at `0x005e3bc0`.
The intervening search-cleanup body is **not mapped**. Source bytes are checked
against the bound executable. No imports or allocator shims are supplied.

The original aligns ESP and uses temporary stack space for saved registers and
a nested return address. Unlike the preceding leaf capsule, its dead stack
cannot be captured by the existing exit callback without being overwritten.
This capsule therefore compares live **gameplay memory**, not live scratch.
A separate 16-byte scratch region permits writes but requires a preceding
write to every byte read in the current replay. Fresh/reset comparisons include
that scratch and the complete store sequence. Missing scratch fails; initial
zeros and prior-call contents cannot supply implicit dependencies.

The successful replay declares 167 bytes: 99 executable bytes, 52 captured data
bytes, and 16 authored scratch bytes, occupying 16 KiB of guest mappings. These
are not process RSS figures. The fixed-capacity capture file is 2,296 bytes.
Only path length and scratch are writable; header/search pointers and the whole
path array are read-only. A 1,024-instruction cap bounds the captured domain.

Nine controls remove each declared region individually, including each code
span, stack input/scratch, and pointer/data dependencies. The parser rejects
truncation, oversize, invalid lengths/pointers, a non-null search, unexpected
stack effects and a capture with no observed deletion. Image identities and
trace receipts are checked before replay.

## Control, artifacts and reproduction

The normal command-only control agrees on all 19 logged UNITS=3 frame bodies
(18–35 and closing 37) and all 37 frame/RNG records (0–36). This remains a check
of the logged behavior, not a proof that every byte of game memory agrees.
The path capture is `/tmp/attrition-path-capsule`; the control is the retained
`/tmp/attrition-capsule-control`. Replay, mutation and control reports are
`/tmp/path-capsule-report.json`, `/tmp/path-capsule-fuzz-report.json` and
`/tmp/path-capsule-control-report.json`. Decoded rows are
`/tmp/path-capsule-oracle.txt`. All original-derived artifacts stay outside git.

Stage a fresh output with `live_session.py`, build with
`TRACER_DEFS='-DRON_COMMAND_PROBE -DRON_PATH_CAPSULE -Werror'`, bind images with
`replay_capsule.py bind`, and use the normal bounded launch. After inspecting
and running the lobby, close the game and restore its shared settings.

```sh
# Do not accept a partial producer's output as a passing table.
set -e
uv run tools/explore/replay_path_capsule.py /path/to/install /tmp/new-run --mutations 4096 --oracle-rows > /tmp/path-rows.txt
cargo run --release -p rondata --example path_capsule_check < /tmp/path-rows.txt
python3 tools/explore/compare_natural_capsule_runs.py /tmp/control /tmp/new-run --kind path
```

`compare_natural_capsule_runs.py` shares the order/path control checks; the old
order-specific command remains a compatible wrapper. These checks still target
the standard 36-frame scenario, even though the witness permits a longer one.
All five backed-up shared settings/profile files were restored byte-for-byte;
the launched game is closed. No alternate desktop input was needed.

## What remains

This establishes one no-search deletion and synthetic variations within its
two-entry capacity. It does not cover live in-match deletion, larger stacks,
suspended searches, allocator cleanup, or automatic pointer-graph discovery.
The Rust checker compares semantic fields; it intentionally has no original
pointers, capacity padding, or ABI register model. Those are checked on the
original replay side. The next useful extension is failure minimization into
small semantic counterexamples, followed by a live in-match sample and wider
path-length coverage without constructing a full simulation.

## Validation

Eight path-capsule framing tests pass, alongside eight order-capsule tests,
11 bounded executor tests, 11 prior capsule tests, four focus tests and five
staging tests. The old order comparison command still passes its retained control.
Changing a captured register, waypoint byte or expected length fails replay.
The whole-run comparator rejects an altered frame body, missing receipt and
receipt naming a frame absent from the frame/RNG records.

Clippy with warnings denied, formatting, seven documentation guards and the
install survey pass. Default and experimental tracer builds use warnings denied;
default preprocessed tokens match the preceding tip, and all three incompatible
capsule/suppression combinations are rejected. Full `cargo test --release`
passes: 269 rondata (one ignored), 821 sim, 13 fixed and three doctests. Rondata
took 230.02 s; the memory watchdog recorded 9,258 MiB across the process tree
under its 20 GiB ceiling. The gate log is `/tmp/path-capsule-release.log`.
