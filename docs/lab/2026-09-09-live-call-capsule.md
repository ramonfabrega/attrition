# A strict replay capsule from a live-derived command package

One probe-created invocation of the original `CommandPackage::clear` now
replays twice in **fresh Unicorn instances**, matching the live general
registers, EFLAGS and every captured output byte. Replay maps only declared
regions, rejects undeclared byte-level accesses even within mapped pages,
permits only the two expected write locations, and has no import adapters.

This is a bounded state-changing call with live-derived inputs. It is not an
automatic capture of arbitrary calls or an entire game checkpoint. No parity
floor moved and no simulation or command decoder implementation changed.

## Live experiment and result

The positive screen/Automation/Accessibility probe passed. Three isolated runs
used temporary executable/DLL copies, with shared settings backed up and
restored between runs. The first pilot installed a hook on the standalone clear
function, but no eligible call occurred in frames 20–35. Its call sites may be
inlined or elsewhere; non-observation does not establish which.

The successful probe copies all 536 bytes of the live local command package
immediately after the existing MoveTo probe issues its command at frame 20.
It invokes the original clear function on that copy, leaving the real package
for the original turn pump. This is explicitly a **probe-created call on a
copied live object**, not a naturally occurring clear call and not reconstructed
scalar fields. The live game singleton remains the real original's singleton.

The capsule records:

- package length **37 before, 0 after**;
- padding seed **12345 before and after**;
- all eight x86 general registers and EFLAGS before and after;
- the package, singleton pointer, game seed, return slot and original code;
- frame, entry, stop and original caller addresses.

The strict replay executes six original instructions. It observes exactly two
writes: two bytes at package +16 set to zero and four bytes at package +532
set to the game seed. The latter is an equal-value write in this capture; the
write event is verified even though a before/after byte diff alone cannot see
it. All nine recorded register/flag values and every retained byte agree with
the live exit. Both fresh instances produce the same access/write record.

The capture file is 1,224 bytes, containing before/after values and metadata;
the mapped captured regions total **572 bytes**. Those figures are not total
Unicorn process memory. Page allocations are larger, but hooks restrict valid
access to the declared byte ranges. Removing the singleton, seed or return-slot
region fails independently. Missing data is never supplied from a full PE image.

A matching normal control, built without the capsule flag and launched with
`-config check.ini -automation`, agrees on **19 complete logged frame bodies**
(UNITS=3, frames 18–35 and closing frame 37) and **37 frame/RNG records**
(frames 0–36). Both record command issuance at 20 and later processing. The
real package remains 37 bytes in the capsule copy receipt. These checks cover
the logged state, not every subsystem or byte of the live process. The earlier
pilot used an incorrect `-ini` argument; it is not the matched control, even
though its measured frame bodies also agree.

## Boundary and provenance

The listing at `CommandPackage::clear@0094c1c0` establishes the six-instruction
leaf, its singleton load at 0xc061ec, its two writes and its plain return. The
opt-in installer checks all 21 instruction bytes before patching. The first six
bytes are displaced to a trampoline; they contain no relative operand. The
remaining original instructions execute in place. The pre-call witness saves
GPRs/EFLAGS and replaces only the return slot with an explicit exit-witness
address. The post-call witness restores those values and resumes the saved
original caller. Thus the capsule's return slot contains the witness address;
replay stops there without executing tracer code.

The local exported `types.txt` identifies CommandPackage size 0x218, its short
size at +0x10, its 512-byte data array at +0x12 and Random padding at +0x214.
Game.info at +0xc plus GameInfo.seed at +4 establishes the seed address used by
the listing. Random contains its 32-bit seed at +0. `docs/FORMATS.md` now corrects
its old 514-byte array label: two alignment bytes precede Random; they are not
payload. These are layout facts backed by the type records and executed
accesses, not an imported implementation.

`replay_capsule.py bind` records SHA-256 identities for the original install
executable, staged original, patched executable and tracer DLL before the
successful run. Replay checks all four, then checks the capsule's code bytes
against the pinned original. Its report includes source and capsule hashes.
Trace framing, health, capture receipt and copy-preservation receipt are also
checked. Artifacts and original bytes remain outside git.

## What this does not establish

The dependency ranges are selected from this leaf's listing and types; they are
not discovered automatically. No heap allocation, callbacks, imports, threads,
exceptions, x87/SIMD state or segment-dependent execution is exercised. The
function's checked instructions do not use those facilities. A different branch
or instruction outside this code range fails rather than inheriting a fabricated
environment. This is a whole-object copy for a leaf that reads no internal
object pointers; it does not demonstrate safe copying of arbitrary object graphs.

No Rust gameplay oracle was added for clearing a transport package. The checks
here compare the original live call with original-code replay and an explicit
write contract. Prior turn-speed comparisons establish a separate Rust scalar
oracle; combining state capsules with a meaningful Rust mutator remains ahead.
There is no mutation corpus, minimizer, dirty-page reuse or throughput claim yet.
Fresh replay correctness comes before those optimizations.

## Reproduce

With the game closed, use a fresh directory outside the install and profile:

```sh
python3 tools/explore/live_session.py stage "$RON_INSTALL" "$OUTPUT" "$PROFILE"
TRACER_DEFS='-DRON_COMMAND_PROBE -DRON_CAPSULE_PROBE' \
  tools/trace/build.sh "$OUTPUT"
uv run tools/explore/replay_capsule.py bind "$RON_INSTALL" "$OUTPUT"
```

Launch the staged executable from its own directory using `winelaunch.sh` and
`-config check.ini -automation`. Use the existing focus/lobby helper to start
Solo / Quick Battle. The staged command file requests quit at frame 36. Exit
the game itself before restoring shared settings:

```sh
python3 tools/explore/live_session.py restore "$OUTPUT"
uv run tools/explore/replay_capsule.py replay "$RON_INSTALL" "$OUTPUT" \
  > /tmp/new-capsule-report.json
```

For a normal control, stage another fresh directory with the same starting
settings and build only `-DRON_COMMAND_PROBE`. Restore afterward, then run
`python3 tools/explore/compare_capsule_runs.py CONTROL OUTPUT`.
The capsule flag requires the command probe and excludes scene suppression;
its receipt IDs overlap the earlier suppression experiment's private markers.

Local evidence: `/tmp/attrition-capsule-live2` is the successful run and
`/tmp/attrition-capsule-control` the matched control. The unsuccessful natural-
call pilot is `/tmp/attrition-capsule-live`. Settings and previously existing
PlayerProfile files were restored byte-for-byte against the first backup.
The helper preserves any newly created game files rather than deleting them.

## Validation

Eleven synthetic guard tests use authored machine-code fixtures, not game data.
They verify fresh-instance agreement, missing dependencies, same-page forbidden
reads, write permissions, unchanged output bytes, registers, instruction limits,
non-executable data, undeclared instructions on executable pages, changed
executable identity and empty input rejection. Each rejection test deliberately
violates the corresponding contract. Five staging tests include rollback and
byte-exact restoration. A live profile contained DumpFileName in two different
INI sections; staging now edits the Logging Options key specifically, preserves
the other section's key, and tests that case.

Default and experimental tracers compile with warnings denied. Default
preprocessed tracer tokens are identical to `be826c7`. No production Rust
behavior changed; the full capture-dependent Rust suite is not rerun for this
opt-in tooling experiment. The prior committed Rust gate remains the baseline.

Final checks also passed workspace clippy with warnings denied, formatting,
seven paperwork guards and the install survey. Both unsupported capsule flag
combinations were deliberately compiled and rejected. The shared settings and
all originally backed-up profile files match the pre-experiment bytes; the
launched game processes are closed.

## Compatibility fingerprint follow-up

The clear-call hook now compares an FNV-1a-64 fingerprint of the installed
21-byte span instead of embedding that whole function as a literal. This is
a compatibility check, not authentication; the capsule manifest's independent
SHA-256 image binding remains required. The current header no longer contains
the body; published Git history has not been rewritten.

Validation: the standalone authored empty/`a`/`foobar` vectors pass. Feeding
the installed span to `test_probe_code_hash.c` produces the expected hash,
and all 168 single-bit mutations differ. The capsule/command tracer also
cross-compiles and links as a freestanding x86 Windows DLL. These checks do
not claim collision resistance or broaden the captured function's closure.
