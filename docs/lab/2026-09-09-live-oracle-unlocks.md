# Live oracle: command injection, call replay, and presentation suppression

## What now works

Four short original-game runs were made in an isolated directory, with shared
configuration backed up and restored. Installed executables, existing tracers,
archived captures and the Ghidra project were not changed. The branch adds
opt-in probes, a replay/mutation runner, a comparison checker and a staging /
restoration tool. Default tracer behavior remains unchanged.

**A real command without gameplay input.** The DLL calls the original
`CommandManager::issue_move_to@00941720` once at simulation-frame 20, selecting
owner zero's object one. The package grew from 10 to 37 bytes. The original
turn pump, not the probe, processed the command: `process_group` reports frame
21; `process_move_to` appears after dump FRAME 21 and before FRAME 22. The
next do_frame-entry observation (21) already has the new order destination.
Issuance time therefore must not be substituted for execution time in Rust's
scenario stream. The experiment does not yet recover an archived recording
or drive this whole scenario through Rust's general input harness.

The request was a nearby destination relative to the unit's decoded position.
The game applied its own destination adjustment; the raw requested coordinate
is not necessarily the final order coordinate. Run1 intentionally remains in
the local record as a failed probe: its first implementation forgot the
coordinate XOR mask and requested an out-of-map move. Run2 corrected that
using `docs/FORMATS.md`'s existing evidence. Run2 and run3 then produced **19
identical logged frame bodies** despite adding call proxies in run3.

**Live call -> original replay -> Rust.** Run3 captured 23 real
`GuyData::turn_speed@005de340` calls. Six normal-member calls (five distinct
input tuples) were replayed outside Wine using the captured fields, and every
answer matched both the live return and Rust's existing turn-speed function.
Seventeen crew calls were explicitly marked excluded: their track fields are
not captured by this probe. A missing/excluded case is not silently treated
as a match. This is field reconstruction for a known function, not a complete
register/memory snapshot of an arbitrary call.

The new runner also mutates the live scalar states with a fixed LCG. **10,006
original/Rust comparisons passed** (six live cases plus 10,000 mutations).
The Python measurement was **0.142578 seconds**, including trace reading,
fixture creation, original execution and table output, excluding interpreter /
dependency startup and the separate Rust checker. This is not a whole-engine
throughput measurement or a claim that each mutation was unique. The mutator
is deterministic and boundary-oriented, not coverage-guided yet.

**Simulation while scene presentation is absent.** Run4 temporarily set
`Scene.display_mode` to 3 at frame 18 and restored 0 at frame 35. The listing
of `Game::loop_render@005917e0` skips both Scene::render branches for that
value while retaining the rest of the function. This is an experimental
unsupported display value, not a discovered official headless mode.

A proxy on `Scene::render@008b3270` witnessed:

- **zero render calls over simulation frames 18 through 34**;
- **115 calls outside that interval**, including after restoration;
- **19 identical complete logged frame bodies** versus run3 (UNITS=3,
  frames 18–35 and the closing frame 37, not the entire game's state);
- **37 identical frame/RNG records**, frames 0–36, including their caller
  and seed fields.

`compare_live_runs.py` asserts every one of these conditions, along with the
command receipt and processed-command text. A no-op tracer or an empty pair
of files cannot pass. This proves a small presentation-free interval after
normal initialization. **It does not yet prove headless startup, removal of
the graphics device, long-run equivalence, lower RSS, or faster ticks.** The
normal fixed-rate scheduler was still in use, so no speedup is claimed.

## Why the boundary matters

`Game::loop@00591570` already separates TurnControl/do_frame from loop_render.
But loop_render is not just drawing: it services ImageIO, incremental loads,
GraphicPieces loading, networking, tasks, camera/input-related work and sound.
Replacing the entire function with a return would remove those effects too.
Even `Game::do_frame@00591ef0` includes GraphicEvents processing and scene
capture work. The next extraction must measure dependencies, not classify
functions as harmless based on their names.

The single most useful architecture direction is **state capsules**: acquire
a valid original state once, then run many deterministic experiments from it
without menus, map generation or rendering in the inner loop. Our earlier
command experiment restored the pages touched by a synthetic call; this
experiment adds actual live scalar inputs. The missing generalization is a
portable capture of registers, readable dependencies, writes, allocation and
external effects, with strict failure on uncaptured dependencies.

## Concrete next experiments, ordered by evidence they would add

1. Extend the rendering comparison to longer fixed-seed windows, two maps,
   richer commands and all relevant dumped records. Run with the existing
   fast-forward control, measure wall time/RSS, and retain a normal control.
   Keep startup graphics for now; removing presentation and removing device
   initialization are separate experiments.
2. Expand the live call closure to crew turn-speed, then an actual mutator
   such as order insertion. Capture entry and exit memory, not just a return.
   Replay unchanged in a fresh emulator with uncaptured reads/imports failing.
   Snapshot before mutation, restore dirty pages, and verify repeated calls.
3. Add edge coverage and a semantic mismatch objective to the deterministic
   mutation loop. Corpus retention should reward new branches and original /
   Rust disagreement. Minimize changed fields and command sequence length;
   a crash alone is insufficient, and structurally invalid object graphs
   should be reported separately from valid-game behavior.
4. Move the safe real-command issuer into a validated scenario adapter:
   frame, player, object identities, full argument fields, acceptance,
   emitted package, processed frame. Use one representation for original and
   Rust, preserving the original's selection and turn semantics. The present
   probe is deliberately one fixed command, not that finished adapter.
5. At a larger boundary, compare a restored subsystem step against its live
   effects. Only then expand toward do_frame. Log the *first uncaptured
   dependency* so harness construction becomes a repeatable fault-driven task
   instead of rebuilding a guessed whole game object graph.

## Other avenues worth borrowing, rather than building from scratch

These are candidate techniques, not installed or benchmarked backends here.

- [Snapchange](https://github.com/awslabs/snapchange) loads memory/register
  snapshots and restores dirty pages through KVM. Its lifecycle is useful for
  the capsule design, but its KVM execution backend is not a drop-in for this
  Apple Silicon/macOS host.
- [WTF](https://github.com/0vercl0k/wtf) targets snapshot fuzzing of Windows
  programs and supplies multiple execution backends. Evaluate it against a
  concrete captured subsystem before investing in a custom whole-process
  emulator. Build support and useful throughput on this host remain untested.
- [Nyx-Net](https://schumilo.de/publications/nyx-net/paper.pdf) demonstrates
  incremental snapshots for stateful sequences. The transferable idea is a
  checkpoint after an expensive command prefix, allowing many suffixes to be
  explored. A game's evolving scenario benefits from the same structure.
- [rev.ng's model](https://docs.rev.ng/references/model/) and
  [import pipeline](https://docs.rev.ng/user-manual/key-concepts/artifacts-and-analyses/)
  provide a possible second analysis/lifting path using known types and
  function boundaries. Benchmark a small function against live execution
  before replacing any Ghidra workflow. Lifted original code belongs in local
  oracle tooling, never in the clean Rust sim or committed derived artifacts.

An additional local unlock is using the PDB as a **harness schema**: generate
capture descriptors for fields and calling conventions, then let actual read
faults refine the required object graph. The specification remains the human
understanding and differential contracts, while repetitive offset plumbing
becomes generated, identity-pinned local data. Any generated pointer walk must
validate readable ranges and cycles, not recursively dereference every PDB
pointer.

## Reproduce and review

Local captures: `/tmp/attrition-live-command/run1` through `run4`. The directory
also contains settings backups, screenshots and the isolated executable/DLL.
These are disposable local artifacts, not durable archived captures. Do not
re-run into the same output directory.

For a new session, with the game closed:

```sh
python3 tools/explore/live_session.py stage INSTALL NEW_OUTPUT PROFILE_DIRECTORY
TRACER_DEFS='-DRON_COMMAND_PROBE -DRON_TURN_PROBE' tools/trace/build.sh NEW_OUTPUT
```

Launch NEW_OUTPUT/riseofnations_trace.exe from NEW_OUTPUT using the existing
Wine environment in `tools/gamelog/winelaunch.sh`; drive Solo / Quick / Start
with the focus-and-settle lobby helper. Keep the launch in a managed process
session: this command runner discarded a detached child on its first attempt.
The command file requests in-game quit at frame 36. Close the app afterward,
then `python3 tools/explore/live_session.py restore NEW_OUTPUT`.

For the suppression variant, add `--hide-scene` to staging and
`-DRON_HIDE_SCENE` to TRACER_DEFS. The staging option prints the required build
flags; it does not build. The tracer checks the hooked prologues; the movement
probe checks the issuer's prologue and object/capacity preconditions. Both
probes must remain opt-in. No hidden configuration silently changes a normal
capture lane.

```sh
python3 tools/explore/compare_live_runs.py CONTROL_OUTPUT HIDDEN_OUTPUT
uv run tools/explore/replay_live_turn.py INSTALL CONTROL_OUTPUT/rontrace.log \
  --mutations 10000 > /tmp/live-turn-table.txt
cargo run -p rondata --release --example turn_oracle_check < /tmp/live-turn-table.txt
```

The existing lobby configuration, not merely the `check.ini` text, determined
this run's human Nubians / random opponent setup. Any new controlled A/B must
pin and record the actual lobby and initial seed; an INI filename alone is
not provenance. Here the measured equal frame state and RNG corroborate the
controls. Original-produced packet tables and memory/trace contents stay
outside version control.

## Validation and handoff

Both default and experimental tracer variants compile with `-Werror`.
Preprocessing the default tracer yields the same tokens as branch parent
`e473cf0`, so the opt-in additions do not change the normal lane. Workspace
clippy with warnings denied, formatting and seven documentation guards pass.
The full captured-game Rust gate was not rerun: no Rust or production sim
logic changed in this tranche.

The live replay checker was made to fail by flipping one captured return bit;
it rejected the mismatch at frame 21. The rendering comparison was made to
fail by mutating one logged coordinate. The staging helper has four retained
fixture tests, including byte-exact restoration of CRLF settings, failure
rollback after a partial shared write, rejection of reused outputs and
rejection of an output nested under the install. No fixture launches Wine.

The game processes launched for these experiments are closed. The three
shared INI files were restored byte-for-byte, and backed-up PlayerProfile
files restored. The helper deliberately does not delete newly created game
files. No queue item, parity floor or phase-completion claim changed.
