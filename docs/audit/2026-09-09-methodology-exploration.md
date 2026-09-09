# Exploring the route to a complete engine

Independent exploration requested by the user; not a queue execution or a
mechanic audit. Started from `20d340d` on `codex/methodology-exploration`.
The earlier capture microbenchmark was too narrow. This pass read the sim,
input adapter, differ, evidence readers, fuzzing scripts, trace injector,
Ghidra exporter, and selected original functions in the local export.
No game was launched, no existing capture or Ghidra project was changed,
and no simulation rule or score was changed.

## Assessment

The project has unusually good access to its reference implementation. The
main opportunity is to turn that access into more **independent, executable
contracts**. Currently a large part of integration progress depends on two
particular games reaching their next RNG disagreement. That is valuable
integration evidence, but a narrow way to discover and retire all the
remaining work.

Keep long lockstep runs. Add a second route that does not wait for their
frontier: drive the same real commands into both engines, restore or construct
local state at useful boundaries, and compare individual transitions and
function results. The tools for this are largely already present. The missing
part is how they connect, what they measure, and which results count as progress.

The strongest immediately demonstrated opportunity is broader emulated
conformance. The strongest likely multiplier is a common scenario/command
interface. The most promising deeper investigation is the original's state
visitor, with important qualifications below.

## 1. The original has a command API; the current scenario system does not use it

`tools/fuzz/scenario.py` explicitly limits scenarios to console staging.
Its `move` command teleports; it cannot issue ordinary game orders. Its
`--no-stage` mode varies maps but removes the state changes the Rust harness
cannot reproduce. `tools/fuzz/ledger.tsv` contains four rows: one existing
baseline and three entries for the same numeric seed, including a re-diff.
This is not an ongoing coverage-guided differential campaign.

The original's `CommandManager` has **68 `issue_*` entries** in `INDEX.tsv`.
I read `issue_move_to` at `0x00941720` and `check_accept_issue` at
`0x00940a70`. The former constructs a command, adds the group selection to
`local_package`, and appends the command. The latter checks playback and
multiplayer/connection state. This is a real command path, unlike a teleport.
`docs/FORMATS.md` already records the group-building pattern in the shipped
obsolete script examples, so the ingredients were known but not connected
to the fuzzer.

On the other side, `crates/rondata/src/commands.rs` decodes a broad command
vocabulary. Yet `input::Stream::one` in `input.rs` only updates selection and
applies `MoveTo`. It skips production, repair, market, and other commands;
some explanations are stale relative to mechanics now present in `sim`.
Even `MoveTo` ignores fields beyond destination and queue mode and lowers
straight to unit orders. This observation is about the general replay
adapter, not a claim that specialized tests cannot drive those mechanics.

**Opportunity:** a canonical scenario stream with explicit frame, player,
selection, command, and arguments. One adapter submits commands through the
original's command manager at an established turn boundary; the other lowers
them into the sim using the same group/queue semantics. Record acceptance,
rejection, actual processing frame, and the resulting order list. Keep staging
operations explicit and implement their corresponding Rust state transitions.

**First discriminating experiment:** one selected unit receives a move through
the injected command manager; verify that it becomes an order, appears in the
recording, and is processed on the same frame as the equivalent UI order.
Then add patrol, attack, garrison, and production lifecycle scenarios.
Do not infer correct scheduling merely because the command was accepted.
This would enable repeatable coverage expansion, meaningful shrinking of
failing scenarios, and input replay beyond ordinary movement.

## 2. Stateful function conformance is cheaper than the current boundary suggests

`docs/EMULATOR.md` limits the next step mostly to pure functions, describing
`GuyData::turn_speed` as roughly an hour of singleton-state synthesis. I used
it as a probe of that boundary, rather than assuming that estimate.

`tools/explore/turn_oracle.py` reuses the existing PE loader and Unicorn
machine. A fixture connects one guy, its unit slot, its unit, its type, and
constants. Field offsets came from `decomp/types.txt`; the two absolute
singleton load operands and calling convention were checked in the listing.
No decompiled function body was translated into the probe: its answers come
from executing the original bytes.

The sweep crosses type-rate truncation boundaries, packed/unpacked state,
stop behavior, average speeds, both modes, and two tuning values. It covers
normal squad members, not the extra crew branch. It is an input-domain
experiment, not a claim that every chosen value arises in a normal match.

**Measured:** 960 calls, 0.030874 seconds from machine construction through
output generation; nine executed basic-block addresses; zero emulated writes
outside the stack. Dependency resolution and Python/import startup are outside
that timer. All 960 results matched the existing Rust `movement::turn_speed`.
Changing one expected bit made the Rust checker fail with exit 101. This found
no gameplay bug; it demonstrated an inexpensive additional oracle surface.

```sh
uv run --offline tools/explore/turn_oracle.py "$RON_INSTALL" \
  > /tmp/attrition-turn-oracle.txt
cargo run -p rondata --release --example turn_oracle_check \
  < /tmp/attrition-turn-oracle.txt
```

The executable listing was checked with:

```sh
/opt/homebrew/opt/llvm/bin/llvm-objdump -d \
  --start-address=0x5de340 --stop-address=0x5de440 \
  "$RON_INSTALL/riseofnations.exe"
```

**Opportunity:** classify functions by the memory they need and mutate, not
just pure versus stateful. Small read-only object graphs are already practical.
For larger graphs, capture input memory and registers at a real call boundary,
then replay locally. Track reads, writes, and original basic-block coverage;
compare return values *and effects* against the Rust function. Restore dirty
memory between trials. Do not treat synthetic invalid pointers or violated
preconditions as simulation bugs.

This is a natural entry to coverage-guided differential fuzzing: reward new
original branches and save a case when the two implementations disagree.
[AFL++ Unicorn mode](https://github.com/AFLplusplus/AFLplusplus/blob/stable/unicorn_mode/README.md)
already documents state preparation, context-dump helpers, comparison feedback,
and persistent harnesses. It still requires a valid harness; installing it
alone would not produce an engine oracle.

## 3. Examine the original's state visitor before designing a raw whole-process snapshot

The executable already knows how to traverse substantially more state than
its text logger prints:

- `SaveGame::do_save` (`0x005a81f0`) invokes `WalkDataGame::walk_data`.
- `SaveGame::walk_function` (`0x0043d730`) writes supplied memory ranges.
- `WalkDataGame::walk_data` (`0x005a2360`) visits leaders, groups, objects,
  armies, cities, world, command manager, and script-related state among its
  other systems.
- `Unit::walk_data` (`0x0060cf40`) includes a path stack and the order list,
  under visitor flags; `OrderList::walk_data` (`0x00730270`) traverses the
  concrete polymorphic orders.
- `SaveGame::verify_save` (`0x005a76b0`) computes subsystem checksums through
  the same general visitor interface.
- `Game::do_frame` (`0x00591ef0`) contains an `auto_save_load` branch that
  saves and reloads periodically after incrementing the frame. Its value is
  read by `Game::solo_checks` (`0x00588110`). I did not resolve or activate
  the preference key.

This does **not** make save/load a transparent per-instruction snapshot.
I followed the writers: `OrderList::walk_data` updates traversal cursor fields
in its output arm; `WalkDataGame` changes `playing` and touches UI/network
machinery; `RunTimeEnv::walk_data` (`0x009c41a0`) unconditionally calls
`close`, whose body (`0x009c40a0`) clears/frees runtime stacks before walking
script files. Thus the visitor is a serious lead, but invoking the whole save
path casually inside an arbitrary function would be wrong.

**Opportunity:** first try selected subsystem visitors with an instrumented
`DataWalk` that records ranges and section identity at a safe boundary. They
may provide binary observations and a state inventory without inventing an
object-graph walker from scratch. For checkpoint/restart, use the existing
save path only after testing continuation fidelity: compare unbroken and
save/reloaded games for commands, state, and RNG at the next frames.

For local function replay, a captured memory closure can be smaller and less
intrusive than a full save. A whole-process memory dump would retain raw pointer
identity, but does not automatically restore OS handles, threads, or the Rust
representation. Those are different engineering problems.
[Qiling's snapshot facilities](https://docs.qiling.io/en/latest/snapshot/)
illustrate the established separation of memory, CPU, registers, and file state;
they are not evidence that RoN's process is already restorable under Qiling.

A successful local-state bridge would allow many independent one-step checks
past the first whole-match divergence. Begin with one subsystem, not a promise
that every frame can immediately be reconstructed.

## 4. The completion counters need an external denominator

Decision 29 measures long captures, cited functions entered, and parsed fields
compared. All three are useful regression measures. None enumerates all
required gameplay behavior independently of what has already been inspected.

The `report.py blind` denominator is an address-citation regex over Markdown.
Running that exact extraction against the current index finds 802 recognized
addresses. The index contains 48,233 entries, many irrelevant to the sim; their
ratio is not a completion percentage. As an illustration of the citation
boundary, none of `CommandManager`'s 82 indexed entries is in that regex's
recognized cited set, despite command issuance being discussed in the docs.
Formatting and what authors chose to investigate influence the counter.

Likewise, entering a function once is not exercising its predicates, and
`DATALAYER` explicitly says its widening ledger checks identifier appearances,
not proof of assertion coverage. A parsed field can be named but unasserted;
a never-parsed record remains outside the denominator.

The general harness also has assisted modes: `Built::tick` can install the
original RNG and figure state when frame seeds are supplied. Those are useful
localizing tools. They must stay visibly distinct from autonomous continuation
claims; this observation does not establish that the long-word floor secretly
uses those corrections.

**Opportunity:** retain the three counters and add a capability inventory tied
to the original's command dispatch and subsystem lifecycles: issue, execute,
complete/cancel, interact, survive save/reload if in scope. For every capability,
record implementation, executable contract, exercised branches, observed fields,
and unsupported/exempt status. Add an explicit autonomous/assisted provenance
label to each test result. Treat the two long games as integration fixtures,
not an exhaustive definition of engine completeness.

Also strengthen the determinism oracle. The hand-written `soak::digest` omits
RNG and broad AI/world/group state. In its current code, changing only
`sim.rng.seed` cannot change the digest. That is a source-level observation,
not a discovered nondeterministic execution. A canonical full-state encoding
with named exclusions, checked across processes and platforms, would give a
stronger guarantee. Keep the present soak for its successful termination work.

## 5. Make the observation pipeline machine-readable and self-validating

The current Ghidra export is an effective reading library: C, types, vtables,
and an address/name/path index. It discards much of the program database's
structure from the downstream interface. The project repeatedly reconstructs
field readers/writers, call relationships, virtual dispatch, widths, and
loader scales through text searches and prose.

**Opportunity:** an additional generated, versioned facts export containing
function extents/signatures/calling conventions, direct references, basic
blocks, typed field offsets/widths including base paths, and unresolved indirect
calls. Keep provenance to the binary/PDB and distinguish facts from inferred
relationships. Use it to generate *test scaffolds and coverage inventories*,
not to force the Rust structs to copy the original engine's memory layout.

For difficult data-flow questions, export/use p-code rather than asking C text
to supply machine-width semantics it has obscured.
[Ghidra's p-code reference](https://ghidra.re/ghidra_docs/languages/html/pcoderef.html)
describes its data-flow representation; its
[emulation caveats](https://ghidra.re/ghidra_docs/GhidraClass/Debugger/B2-Emulation.html)
also explain why it should not be assumed to be an infallible execution oracle.
A second decompiler or symbolic executor is useful for a disputed small slice,
not as a replacement for behavioral comparison.

Trace trust has a concrete gap. `Trace::parse` validates the magic but accepts
unknown versions, ignores a partial final record, and drops INFO records,
including the writer's explicit lost-record notification. This is demonstrated
by the synthetic `trace_acceptance` example:

```sh
cargo run -p rondata --release --example trace_acceptance
# header_only: accepted=true
# unknown_version: accepted=true
# partial_record: accepted=true
# 123_dropped_records: accepted=true
```

A permissive live reader is reasonable; a gate consuming a finalized golden
record needs a separate completeness/health contract. The writer's
`flush_locked` also ignores `WriteFile`'s result and byte count before clearing
the buffer. Expanded function proxies would need thread/call identity: CALL/RET
records currently lack thread IDs and the Rust reader pairs them on one stack.
That is a limit on expanding them to concurrent functions, not a demonstrated
mispaired call in today's captures.

**Measured corpus check:** 86 `rontrace-run*.log` files, 1,311,028,608 bytes,
40,969,558 records after headers, no partial tails, reported dropped records,
hook mismatches, or protection failures. The scan took 3.658457 seconds. Versions
were 84 at v1 and two at v2. None carried a detach marker, so requiring that
marker retroactively would reject the entire corpus without proving corruption.
This scan does not prove absence of complete-record truncation or failed writes.

```sh
python3 tools/explore/trace_health.py "$RON_GAMELOG_DIR"
```

New captures should carry executable/PDB identity, tracer build and schema,
lobby/config/command hashes, frame ranges, record counts, loss state, and an
explicit completed-capture artifact. Content identity would also make derived
indexes and caches trustworthy. Preserve the existing text as the historical
oracle while validating new binary observations against it over overlaps.

## 6. Runner and injection: change the boundary before changing the platform

The PE import-extension technique is a sensible way to preload this tool into
an owned executable copy. Replacing it solely to adopt another injector has
no demonstrated payoff. The recent Rosetta workaround also has a reproducible
basis; I did not run a new platform comparison.

The better near-term investments are selective instrumentation, richer call
records, and differential tests of the instrument itself. `funcs.py` already
relocates prologues with Capstone. Use the existing emulator to compare original
and displaced execution on register/flag/memory fixtures, including both stub
paths. A successful game trace only exercises a subset of the relocated forms.
For long runs, compare a subsystem allowlist against whole-image coverage,
checking identical oracle results and actual elapsed time.

I read the logger as well. `Log::say` performs wide-character formatting and
writes indentation a character at a time; `GameLog`'s dump paths contain many
flushes. The game log is initialized with its keep-open flag, so there is no
basis here to claim it necessarily opens/closes the file for every ordinary
integer field. Binary state-range capture could remove formatting and repeated
text parsing, but neither its speed nor its observational equivalence was
measured in this pass. Sample the original around formatting/flush/renderer
before choosing which cost to remove.

Native x86 Windows is worth an A/B runner experiment if screen/Wine reliability
or throughput remains limiting. Hold executable, configuration, commands, and
instrumentation fixed and compare both performance and results. Do not assume
cross-host x87 behavior is identical. A full headless shim for the original's
renderer is a larger project than using its command boundary.
[WinAFL](https://github.com/googleprojectzero/winafl) supplies mature target-function
instrumentation on Windows; [Frida Stalker](https://frida.re/docs/stalker/)
supports selective tracing. Neither is a tested drop-in replacement for this
32-bit executable under Wine/Rosetta.

## 7. Gate cost

The earlier report measured repeated reads and queries. This pass additionally
ran the entire rondata library suite with temporary wall-time instrumentation
around capture reads, `Log::parse`, and `scan_children`; results follow below.
The source instrumentation was restored after the process exited. This measures
elapsed spans, not CPU samples or an optimized implementation.

**Measured full suite:** 248 passed, zero failed, one ignored; test execution
289.66 s, compilation 8.41 s, wrapper wall time 300.238539 s. No `skipping:`
line appeared in the captured output. Explicit `RON_INSTALL` pointed to the
main checkout's owned install. The command was:

```sh
python3 /tmp/attrition_profile_gate.py
# temporarily times capture::read, Log::parse, and scan_children; runs:
# RON_INSTALL=/Users/rf-studio/code/fun/attrition/game \
#   zsh tools/memcap.sh 20 cargo test -p rondata --release --lib -- --nocapture
# output: /tmp/attrition-profile-gate.log; then restores both source files
```

The instrumentation runner is preserved as `tools/explore/profile_gate.py`;
its retained version additionally preflights `ps` and propagates the child exit
code. Use a context allowed to sample processes before rerunning it.

| Measured operation | Calls | Sum of elapsed spans |
|---|---:|---:|
| capture text reads | 561 | 20.672 s |
| `Log::parse` | 571 | 116.610 s |
| whole-child scans | 605 | 345.644 s |

Those sums span two workers and must not be presented as percentages of CPU
or as additive wall-clock savings. Read calls consumed **154,693,938,770 bytes
of logical input over 100 distinct paths**, not 154 GB of physical disk I/O.
run13 was read 50 times (32,105,026,200 logical bytes), run38 49 times, run12
47 times, run3 44 times, and run11 43 times. Repeated sibling setup is therefore
measured behavior, not a guess based on filename mentions.

The two shutdown tests `the_census_is_the_corpus` and
`a_closing_dump_and_its_own_block_are_one_state` alone accumulated 53.059 s and
36.117 s in scans, respectively; the former also accumulated 18.250 s indexing.
These identify concrete places to start reusing owned query products or
combining traversals. The scan span includes the callback's work, so this is
not a pure tokenization benchmark and a CPU profile can refine it further.

**Memory result unavailable:** memcap printed a zero peak despite the suite
running. A direct `ps -o rss= -p $$` then failed with `operation not permitted`
in this sandbox. Its pipeline converts missing samples into zero, so this run
also demonstrates that a memory ceiling can silently become ineffective.
Do not call it a measured low-memory run or proof of cap enforcement. Future
runs need a successful sampling preflight and failure on lost sampling.
The earlier primer's shortcut “peak zero means nothing ran” is false in this
environment.

A separate promising memory experiment is process-per-test execution at a
bounded worker count. It releases the process's allocator state between tests
instead of relying on allocator tuning to return old pages. That is the model
[cargo-nextest documents](https://nexte.st/docs/design/how-it-works/). It would
need its own measured time/RSS comparison; do not raise concurrency on the
assumption that isolation makes heavy individual tests cheap.

For a durable reader improvement, expose owned initial state and owned frame
records behind an indexed capture interface. A callback that loads/parses one
frame can preserve the current parser within that scope, avoiding a simultaneous
rewrite of every accessor. Derive the data from content-identified captures,
and compare every resulting record and the existing printed diff. This is a
candidate architecture, not an implemented or measured speedup.

## What I would try first

| Investigation | Why it can accelerate completion | First result that would justify expansion |
|---|---|---|
| Shared scenario/command interface | Reaches many missing behaviors without hand-driving each run | One original/Rust order lifecycle with matching scheduling and selection semantics |
| Small stateful emulated contracts | Finds widths/predicates without waiting for a match to reach them | Expand the working turn probe to another small object graph and obtain new branch coverage |
| Subsystem visitor/state bridge | Makes later transitions independently testable and may reduce dump cost | One subsystem observed equivalently; checkpoint continuation matches the unbroken original |
| Capability and evidence inventory | Stops completion depending only on two games and author-selected citations | Enumerated command/lifecycle support with executable tests and explicit gaps |
| Query reuse / process-isolated gate | Shortens feedback and may remove allocator retention | Same suite outcomes with measured lower wall time/RSS |
| Selective, validated instrumentation | Reduces capture tax while increasing confidence in the oracle | Stub equivalence probes and an unchanged A/B trace |

The larger change I recommend is allowing these independent contracts and
coverage gains to count as completion progress even when neither long-game
frontier moves that day. Require executable evidence, keep the integration
floors, and stop making one capture's next disagreement the only available
source of productive work.

## Reproduction and validation notes

Source/index census command (no decompile content is copied into the repo):

```sh
python3 - <<'PY'
from pathlib import Path
import re
rows = [s.split('\t') for s in
        (Path.home()/'ghidra-projects/decomp/INDEX.tsv').read_text().splitlines()]
known = {int(r[0], 16) for r in rows}
pat = re.compile(r'([A-Za-z_][A-Za-z0-9_:~<>]*)@(00[0-9a-f]{6})')
cited = {int(m.group(2), 16) for p in Path('docs').rglob('*.md')
         for m in pat.finditer(p.read_text())}
print('indexed', len(rows), 'recognized citations', len(cited & known))
print('CommandManager', sum(r[1].startswith('CommandManager::') for r in rows))
print('issue methods', sum(r[1].startswith('CommandManager::issue_') for r in rows))
PY
```

The turn fixture's field evidence in `decomp/types.txt` is `GuyData`:
`last_speed +0x80`, `avg_speed +0x84`, `o +0x8c` (short),
`guy_flags +0x9a` (short), `who +0xa1` and `guy_num +0xa2` (char);
`UnitTypeData`: `turn_speed +0x2c4`, `squad_size +0x304`;
`Constants`: `unit_turn_speed +0x8`, `unit_pack_turn_bonus +0xc`.
The listing command in section 2 independently shows those accesses plus
unit type at `+0x18`, masks at `+0x68`, and the singleton operands. Zeroed
fixture memory supplies owner/object/guy zero; no undocumented random memory
is required by the exercised branch.

The numeric gate summary is preserved in
`2026-09-09-methodology-measurements.json`. Raw output remains at
`/tmp/attrition-profile-gate.log`. Its timings can be summed from the
`EXPLORE (read|parse|scan) <nanoseconds> ...` records; read records also include
byte length and input path. Test-thread names identify parse/scan consumers.

Validation completed:

- Instrumented rondata library gate: 248 passed, one ignored, exit zero.
  All temporary production-source changes restored byte for byte. This is
  assertion validation, not a before/after printed-output parity experiment.
- Turn oracle: 960 comparisons passed; a one-bit expected-result mutation
  failed with exit 101.
- `cargo clippy --all-targets -- -D warnings` and
  `cargo fmt --all -- --check`: passed.
- `cargo test -p sim --release`: 821 passed. This command did not explicitly
  set the install path; it should not be read as additional original-backed
  coverage. The separate turn probe and rondata gate did use the owned install.
- Python probes parsed successfully. The retained profiling preflight was
  exercised under the sandbox's denied `ps`: it exited nonzero and left both
  source files byte-identical. That guard was observed failing, not just added.

No proposed new injection, save/load capture, full-state reconstruction,
platform switch, persistent cache, or coverage-guided fuzzer was claimed as
implemented. The branch contains the report and experiments, not a ratified
rewrite of the project's method.
