# First natural pointer-graph capsule

## What passed

A naturally occurring `Unit::update_order@006179d0` call at frame 0 was captured
without a probe-created invocation or copied input object. Two fresh bounded
emulators and a reset invocation match all eight GPRs, EFLAGS, and every captured
data byte at the live exit. The replay executes 14 instructions and three stores.
Removing any of its five required input regions fails, including fields sharing
an otherwise mapped page. There are 98 declared bytes, mapped onto four pages
(16 KiB). This is guest mapping size, not total process RSS. The capture file is
240 bytes including metadata and before/after snapshots.

**This was an equal-value cache refresh. Zero captured unit bytes changed.**
The emulator's three-store sequence is checked against the pinned function's
store contract, including equal-value stores; the live witness records entry
and exit memory, not hardware store events. This proves replay of a natural
pointer-graph call, not observation of a nonzero live state transition.

The matched normal command-probe control agrees on all 19 logged UNITS=3 frame
bodies (18–35 and closing 37) and all 37 frame/RNG records (0–36). That checks the
logged behavior after instrumentation; it is not a claim about all game memory.
No sim rule, Rust production code, or parity floor changed.

## Why this entry

A streaming scan of existing HIT records in runs 53, 54 and 906 found this entry
at frame 0 in all three. `Guy::turn_towards@005d9720` appears at frame 1 in all
three; `Unit::kill_current_path@005e31d0` first appears at 200, 69 and 200.
`Guy::turn_angles@005d98c0` has no HIT in these traces. HITs are coverage witnesses,
not call counts: outside a rearmed window they record only the first entry.
Absence is not a proof of unreachability. The next natural-call target should
have such a witness before building instrumentation for it.

The local executable listing shows update_order is a leaf, with no stack
writes. Its only stores are the order-cache fields documented in ORDERS §1.4:
`current_node` (+0xd4, four bytes), `current_data` (+0xcc, four bytes), and
`current_metric` (+0xd0, one byte). The root's head pointer leads to the previous
node, whose data and metric supply the cache. The 20-byte snapshot +0xcc..+0xdf
also includes padding and length, which must remain unchanged. The executor
splits that snapshot into regions to grant write access only to the three
actual fields. Node data includes its surrounding padding, also checked unchanged.
These are existing documented layouts, not new inferred formats.

This is not yet a Rust differential oracle. Rust's `Sim::current_order` uses
`VecDeque::front` and deliberately does not mirror this cache or its pointers.
A meaningful Rust comparison needs a decoded order-list/identity contract;
recreating the original's cache layout in Rust would add no gameplay fidelity.

## Witness and strict replay

`RON_ORDER_CAPSULE` enables `tools/explore/live_order_capsule.h`. Coverage must
be disabled and the image must have its expected base. The witness checks the
seven-byte displaced prologue, saves the original code, and intercepts the
first nonempty call in frames 0–35. It saves/restores all GPRs and EFLAGS, and
redirects only that call's return slot through its exit callback. The original
leaf does not write below ESP, so the exit callback's stack use cannot overwrite
captured call data. The callback returns to the recorded natural caller.

`replay_capsule.py bind` pins the original, staged original, patched executable,
and tracer DLL before launch. `replay_order_capsule.py` verifies all four hashes,
code bytes against the source image, pointer consistency, stack effect, capture
framing and trace receipts before matching the live result. The executable
range is 62 bytes; the two captured padding bytes are checked against the image
but are not executable in the bounded replay. Undeclared imports, memory or
instructions fail. Source and output pointers retain the original addresses.

The contract permits only the three stores. It compares untouched data too.
Negative controls changing a live register, untouched memory, or the expected
store sequence each fail. The whole-run comparator also rejects a changed logged frame body and a missing
capture receipt. Five missing-input controls remove code, the root's
head field, the previous-node link, node data, and the return slot individually.

## Live-derived stale-cache mutations

`replay_order_capsule.py --mutations 4096` also passes 4,096 deterministic
mutations of the three cached output fields, including zero, all-one and seeded
random bit patterns. Authoritative head/node inputs remain the captured bytes.
Every invocation must restore the complete live exit state and exact store
sequence, and every mutated invocation is compared with a fresh emulator.
The report is `/tmp/order-capsule-fuzz-report.json` (seed 12345).

These are synthetic mutations of a live input, not more natural captures. They
establish independence from stale cached pointers/data/metric for this nonempty
list path. They do not widen order-dispatch coverage or test list corruption,
empty lists, queue insertion/removal, or Rust's different representation.
The driver retains only one baseline and the current result in this loop.

## Runs and a control-loop fix

- `/tmp/attrition-order-capsule`: completed 37 frames, with the normal frame-20
  command receipt and no hook error. No call met the initial changed-cache-only
  filter in frames 20–35. It supplies no successful capsule.
- `/tmp/attrition-order-capsule2`: broadened to frame 0 and allowed equal-value
  refreshes. Wine reported a page fault before any FRAME record. This is an
  unresolved startup failure, not evidence about the witness's replay. Only the
  identified experiment processes were stopped, then settings restored.
- `/tmp/attrition-order-capsule3`: the same pinned experimental executable/DLL
  retried successfully; natural frame-zero capsule and complete bounded run.
  `/tmp/order-capsule-report.json` and `/tmp/order-capsule-control-report.json`
  contain the replay and matched-control results. The control is the retained
  `/tmp/attrition-capsule-control` normal command-only run.

The failed launch exposed a separate defect: `focus.sh` returned success for
no matching window, and `lobby_click` ignored focusing failures. Input could
therefore go to a different foreground application. Focus mode now fails for
an absent window and propagates Automation errors; `--title` preserves its
empty-query behavior. Lobby click/start propagate failure before issuing input.
Four mocked tests require no desktop; both no-click tests fail against the old
scripts and pass after the fix. This corrects the capture control loop without
asserting that it fixes Wine's startup fault.

All five originally backed-up shared settings/profile files were restored
byte-for-byte against the first run's backup. New game-created files are retained.
The launched games are closed; captures, code bytes and reports stay outside git.

## Reproduction and validation

From the repository root, with the game closed, stage a fresh output using
`live_session.py stage`. Build into it with
`TRACER_DEFS='-DRON_COMMAND_PROBE -DRON_ORDER_CAPSULE -Werror'`, then bind images
before launch. Use the usual `-config check.ini -automation` launch and inspect
the lobby before starting. The staged command file quits at frame 36.

```sh
uv run tools/explore/replay_capsule.py bind /path/to/install /tmp/new-run
uv run tools/explore/replay_order_capsule.py /path/to/install /tmp/new-run --mutations 4096
python3 tools/explore/compare_order_capsule_runs.py /tmp/normal-control /tmp/new-run
# Close the game before restoring its shared settings.
python3 tools/explore/live_session.py restore /tmp/new-run
```

Eight new order-capsule framing tests, 11 bounded executor tests, 11 existing
capsule tests, five staging tests, and four focus tests pass. Default and
experimental tracers compile with warnings denied. Default preprocessed tokens
match the preceding tip; incompatible capsule/suppression flags are rejected.
Clippy, formatting, the install survey and documentation guards pass. The
pre-commit full release gate also passes: 269 rondata (one ignored), 821 sim,
13 fixed and three doctests. Rondata took 228.98 s; the memory watchdog reported
9,710 MiB across the process tree under its 20 GiB ceiling. The gate log is
`/tmp/order-capsule-release.log`. No Rust behavior changed.

The next milestone is a natural call with an observed nonzero semantic mutation
and a decoded Rust comparison. This result should not be presented as that
milestone, nor as automatic pointer-graph capture or a generic Windows process
snapshot system.
