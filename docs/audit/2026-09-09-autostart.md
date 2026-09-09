# Native unattended capture driver

## Scope and evidence

This exploration removes mouse/keyboard input from bounded original-game
acquisition. Wine and the display/device initialization remain required. It
moves no simulation fidelity floor and does not resume a saved native search.
The runner takes the existing profile's match rules and pins map and seed;
it does not claim those rules equal the headline human-versus-AI captures.

`tools/explore/unattended_capture.py` stages a fresh directory for each map,
builds the opt-in tracer, launches through `winelaunch.sh`, checks the lifecycle,
reads back `MAP_STYLE` and `(int)seed`, requires the closing dump at endpoint+1,
and restores and byte-verifies every backed-up settings file. The default
endpoint is 36, with 14 then 18 as the map styles. Longer runs schedule native
fast-forward at 37 using the existing command channel. Each run writes a JSON
receipt with binary/config hashes, timings, process status, and cleanup result.
Install-derived files and logs remain outside Git.

The shared profile has a cooperative `.attrition-capture.lock`: another copy
of this runner refuses while it is held. This does not lock out Fable's manual
scripts or mouse. The live capture lane still requires an explicit handoff.
Only the runner's launch process group is terminated on timeout; restoration
refuses if a game process remains. A failure retains its log and receipt and
stops the pair; there is no automatic retry that could hide a failure.

## Native boundary

`TopMenu::main_menu@0059bf70` and `TopMenu::sub_menu@0059c030` are replaced only
in a `RON_AUTOSTART` build. The first selection returns `GAME_SOLO` (1). The
surrounding `TopMenu::exec@0059fc90` and `Main::run_game_loop@00595f40` remain
native, including the route through `Game::run_solo@00587830`. The enum values
are checked in the shipped listing: dispatch table entry 1 selects the solo
arm; the loop's exit comparison is 21. These are menu choices, not sim writes.

Within `SetupWin::exec@005c67f0`, the virtual modal call at 0x5c7e37 is wrapped
without changing any original argument. The driver notes the instance and
waits for `System::modal_idle@005994a0`. It first runs the native idle function,
then, once only, confirms `SetupWin::current` still names the same instance and
calls `SetupWin::on_button_clicked@005c5b50` with the engine's Start string.
The handler's validation and setup work remain native. The current-pointer
store at 0x5c7cd2 and the handler's string operand identify the dependencies;
no structure layout is inferred from the decompiler's padding labels.

**The first wrapper was wrong and its live run failed.** `Window::set_modal@00a55460`
takes a by-value callback object and co-modal window after its mode argument.
A C wrapper forwarding only mode discarded them. The fault receipt recorded
return address 0xa5556e, just after the co-modal virtual call. The replacement
saves registers/flags, observes this/mode, restores everything, and jumps to
the original virtual target with the original stack intact. An executable
regression compiles the real wrapper, runs it under Unicorn on 256 authored
states, and checks all integer registers, flags, and 256 bytes of native
arguments. A deliberately wrong argument offset is rejected.

Every patch site is byte-checked before any driver patch, the preferred image
base is required, and coverage instrumentation is excluded. The driver is
absent from the default tracer build. INFO 170..175 describe selection, modal
entry, Start dispatch/return, refusal, and installation. INFO 176..177 retain
bounded exception context/stack words and continue the original exception
handling; they do not recover from a fault. The lifecycle reader rejects
faults, refusals, missing/repeated transitions, instance mismatch, nonzero exit,
and any missing/repeated frame through the requested endpoint.

## Exit boundary and limits

The first successful 37-frame capture reached the closing dump and returned
to the menu, then faulted at 0xa2c457 during application teardown. This is
consistent with the traced-executable shutdown failure already documented in
ORACLE, but its underlying cause is not established here. The driver now emits
the second menu selection, flushes the trace, and calls `ExitProcess` after
native match shutdown. This is a **controlled post-match process exit**, not
proof that the full application destructor path works. A code-0 exit alone is
never accepted: the runner also needs exact lifecycle, contiguous frame range,
map/seed read-back, closing dump, and verified settings restoration.

Temporary evidence:

- `/tmp/attrition-autostart`: ABI failure, corrected capture, application
  teardown failure, then verified controlled exit. Earlier failed traces and
  the first gamelog were retained separately in this directory.
- `/tmp/attrition-unattended-pair`: Great Lakes passed; East Indies failed
  before menu entry with exit 40 and a Wine exception at 0x7bf21139. The runner
  rejected it and restored five settings files. The startup fault is open.
- `/tmp/attrition-unattended-pair-repeat`: both maps passed at endpoint 36,
  37 frame records each, code-0 exits, five restored settings files each.
  This shows the first pair's second-launch failure is not inevitable; it does
  not establish a reliability rate or explain that failure.

- `/tmp/attrition-unattended-1400`: both maps passed, each with 1,401 frame
  records, seed 12345 read back, closing frame 1401, code-0 exit, and five
  restored files. Launch-to-exit was 23.212 s / 24.219 s; including build,
  staging, validation and restoration, 30.734 s / 31.888 s. These are single
  observations, not a comparative speedup or a renderer cost measurement.
- `/tmp/attrition-unattended-prefix-comparison.json`: on each map, the
  independent short/long runs agree on all 18 complete logged frame bodies
  (18..35) and all 37 frame/seed pairs (0..36). `samegame.digests` excludes
  closing records; no record category or field was excluded. This confirms
  the shared observed prefix only, not complete state or the long tail.
- `/tmp/attrition-unattended-1400-repeat`: the next independent pair stopped
  on Great Lakes during startup, exit 9, before menu selection. The receipt
  records failure and five restored files. Thus longer-run repeatability is
  not established. There are two distinct pre-menu failures to investigate;
  neither is silently retried or relabelled as a successful capture.

## What remains

Passing a lifecycle receipt is not a fidelity result.
Measure logging, simulation, startup, and render costs separately. Device-free
startup, full application teardown, arbitrary profiles with introductory
prompts, and same-process match reset are not established. The original game
still owns a visible window during acquisition.

## Landing validation

Commit 04684a6 passed the release suite: 269 rondata tests (one ignored),
822 sim tests, 13 fixed tests, and three doctests. Clippy with warnings denied,
format checking, the install survey, and repository guards passed. Twelve
new lifecycle/runner tests and the compiled 256-state modal ABI regression
passed, including deliberately malformed inputs and a wrong argument offset.
The default tracer's preprocessed source was byte-identical to its parent.
Final sampled gate peak was 3838 MiB under the 20 GiB cap; this observation
is not attributed to the capture change. The game was closed and all five
backed-up settings files verified unchanged before the branch was pushed.
