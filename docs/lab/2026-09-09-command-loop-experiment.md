# Command issuer and memory replay experiment

The next step after the methodology exploration was to turn command issuance
into an executable contract before attempting live injection. This experiment
executes the shipped `CommandManager::issue_move_to@00941720` in Unicorn,
including its acceptance check, object filtering, selection compression and
packet append. No original game function is replaced. The external imported
`memcpy@0055e0ac` uses a bounded host adapter (cdecl, at most 512 bytes).

## Measured result

`tools/explore/command_oracle.py INSTALL` produced 35 cases and 70 original
executions in 0.022915 seconds, measured from fixture construction through
output, excluding interpreter/dependency startup. Each case runs once,
restores the touched data pages and CPU context, and runs again. All 35 pairs
had equal EAX and equal final contents of every captured data page. The union
was 15 pages (60 KiB); this is the data closure for these fixtures, not the
emulator's total allocation or the memory required by a live game.

The packets were piped to `command_oracle_check`, which uses the existing
Rust `commands::decode` and compares every Group and MoveTo field against the
input arguments. All 35 passed. Cases include first selection, selection
reuse, all three queue modes, signed coordinate extremes, angle/formation/
width/disembark fields, package capacity boundaries, playback rejection and
the semaphore rejection bit. Serialization of extreme values is not proof
that those values describe valid gameplay.

The two tools are reproducible without Wine, an installed DLL or a capture.
Original-derived packet rows stay in `/tmp/attrition-command-oracle.txt`,
not in the repository.

## Concrete driver requirements discovered

- A first single-object selection plus MoveTo occupies 27 bytes; reuse plus
  MoveTo occupies 25. Selection compression persists when only packet length
  is reset. A standalone packet therefore does not always describe its own
  selection: the driver needs the preceding selection state.
- Starting with 485 occupied bytes, the new selection and order fit exactly.
  Starting with 486 through 507, the selection fits but the MoveTo does not.
  The public issuer returns void. Calling it is not an acknowledgement that
  an order was emitted.
- At 508 through 512 occupied bytes, neither record fits, but the sender's
  cached selection still changes. The next call after a synthetic packet
  reset emits the reuse form. This was forced by changing the selected
  object's UID; it demonstrates non-transactional sender state, not an
  observed live-game lost command. The real turn pump may maintain invariants
  that avoid this condition. Do not bypass that pump or retry by merely
  clearing packet length.
- Playback mode and the tested semaphore bit reject issuance without packet
  output. Driver success must distinguish acceptance, emission and later
  execution, with explicit frame stamps for each.

These are assertions in the experiment, not just decompiler interpretations.
A canonical scenario driver should let the original own compression and
packet processing, send bounded batches, inspect emission, and observe the
processed command frame. Rust should consume the decoded command stream.

## Evidence and fixture boundary

The local PDB export supplies `CommandPackage` size 0x218, length at +0x10,
data at +0x12, and `CommandManager.local_package` at +0x28. The executable
listing of the issuer supplies the singleton 0xe8ff60 and the ten stack
arguments (`ret 0x28`). `check_accept_issue@00940a70` supplies the game-pointer
operand and semaphore offset. `CommandPackage::add_group@0094bb60` supplies
the group count/owner/object accesses, object-array operands, validity and
captain checks, UID accesses and cache behavior. The local vtable export
supplies the original Unit vtable; its actual original virtual calls run.
`CommandPackage::add_command@0094bae0` supplies the capacity path. The Rust
wire layout remains the existing documented decoder in `docs/COMMANDS.md`.

The fixture has one active captain unit, owner zero, object one, and a
single-player game semaphore initially zero. Five synthetic pages hold the
game prefix, GroupOut storage, object registry, pointer slots and unit.
The fixture only initializes fields this call reads; it is not a constructed
valid whole game. A page at zero supplies the FS exception-chain storage;
no exceptions are exercised. No Windows loader/import environment is built.
The memcpy adapter explicitly includes its reads and writes in tracking.

Memory tracking saves a page before its first emulated data access, including
writes. Replay restores these pages and CPU context in the same mapped image.
This is a prototype of call-state replay with synthesized state, **not a
captured live-call replay** or portable standalone snapshot. It does not yet
validate a different mutated branch, unexpected imports, thread state,
allocation, callbacks, complete output registers, or a fresh emulator. The
existing Machine maps the entire executable, so a missed dependency can still
read mapped data. A future portable replay must trap dependencies outside its
captured closure and pin executable identity.

## Live probe stopped before staging

The authorized live experiment ran the repository's positive permission probe.
Screen capture and the Automation portion passed; cursor requests for 37,41
and 53,59 both read back 2906,1115. The probe exited 1 at Accessibility.
No game launch, install edit, DLL staging, capture replacement or archived-data
mutation occurred. This is a permission-shaped failure, not a conclusion about
Wine or the hardware. User action is required before continuing the live lane.

Once input control works, the next experiment is one real MoveTo through the
local command manager at a measured boundary, recording emission and the
processing frame, then the resulting order and trajectory. The command package
probe provides its expected bytes. Capture the relevant live call state next;
repeat it unchanged before mutating anything. Full checkpoint/save-load work
remains behind those two concrete validations.

Validation: the 35-row positive check passed; changing the first MoveTo opcode
made the Rust checker fail with exit 101. Formatting, targeted clippy with
warnings denied, Python syntax and all seven documentation guards passed.
The full capture gate was not rerun: production code and acceptance floors
are unchanged.
