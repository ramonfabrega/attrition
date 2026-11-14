# Other Rise of Nations projects (2026-11-14)

Gathered by a read-only scout (Opus 5.5) from the GitHub API and shallow clones read in place (nothing built or run):
the two named repos, this repo's issues, and the starred lists of both stargazers (VictorTaendy, MuntezV2).

**Issue thread.** The first and only issue is #11, "Discontinued?" (MuntezV2, 2026-11-03): a well-wisher asking whether
the project stopped. No comments, no links; closed by its author. It points to no project.
**Starred lists.** VictorTaendy stars 11 repos, MuntezV2 3 (both include attrition). Kept: don, OpenRiseOfNations,
empire_earth(_next), gta-reversed, horizon. OpenTTD and OpenTTDLab are already-known prior art. 5 unrelated (Barotrauma,
CC-Tweaked, Mindustry, scikit-learn, one personal repo).

| project | approach | state (runs / claims / last commit) | licence |
|---|---|---|---|
| [emberian/don](https://github.com/emberian/don) "Descent of Nations" | Rust reimplementation + retail differential oracle + RL gym; save-state codec transcribed from the disassembly | Runs: browser integration build, RL env over partial dynamics. Saves: 55/55 round-trip, 15/15 checksum channels *from a loaded retail save*. Replays: rules channel matches, every dynamic channel 0 substantive matches. Last commit 2026-10-04; 1,003 commits; emberian 933 + a `claude` account 22 | GPL-3.0-or-later |
| [ben-wright-1/OpenRiseOfNations](https://github.com/ben-wright-1/OpenRiseOfNations) | C++17 reimplementation, native macOS (Metal/AppKit), spec-first from the PDB + Ghidra | Runs: v0.1.1 .dmg; tutorials, scenarios, skirmish vs AI, full HUD, audio, BHS scenario scripts. The README says the sim has *not* been checked against recorded games. 2026-10-14; 3 commits (one dump); 1 contributor | GPL-3.0 |
| [Protryon/empire_earth](https://github.com/Protryon/empire_earth) + [_next](https://github.com/Protryon/empire_earth_next) | Matching decompilation of *Empire Earth: AoC* (2002, not BHG); _next is a modern clang/x64 port with D3D7 on wgpu | _next: 1,840 commits since 2026-11-04, pushed today; 1 contributor | none stated |
| [gta-reversed/gta-reversed](https://github.com/gta-reversed/gta-reversed) | Function-by-function reimplementation injected into the running original (ASI DLL replaces functions one at a time) | Claims 50-60 % done; 2,196 commits, 68 contributors; 2026-11-13 | none stated |
| [horizon-rust/horizon](https://github.com/horizon-rust/horizon) | Rust reimplementation of Minecraft Java, checked against vanilla by differential harnesses | Ships a launcher; README says 0 open "player-visible gaps" of 833 from a method-by-method port ledger; 30,678 commits, 100+ contributors; 2026-11-14 | MIT |

## emberian/don: the closest peer, and the one that differs most in method

Started 2026-08-08, eleven days before attrition. Its aims overlap ours (a fidelity reference built from the same
`rise.pdb`, lockstep checksums as the oracle) and add an RL environment and a browser client. What it holds that we don't:
- **A `.svx` save-game codec**, verified in source: `crates/don-state/` ("every byte of every capture is produced by a
  typed walker"; `tests/roundtrip.rs`, `tests/check_all.rs`). It covers all 15 `CheckSums::check_*` channels on 55 captured
  saves, plus the `SaveGame::verify_save` trailer. Attrition saves as seed + orders and has no `.svx` reader. The saves
  themselves are gitignored (`schema/live/frame-pairs/`).
- **A native i686 oracle**: it maps `riseofnations.exe`, applies relocations and calls shipped functions directly on
  x86 Linux, with no emulator (`crates/oracle/`, `docs/oracle-architecture.md`). It classifies functions as
  ISLAND / SELF-CALL / DATA-ONLY, a scheme taken from sushi-shi/gruntz-decomp. `schema/oracle-regression.json`
  records 23 cases and 18.7 M trials with zero mismatches, damage and turning among them. Ours is unicorn
  (`tools/emu/callfn.py`). The native route is claimed to be 10³–10⁴× faster; that figure is marked [reported] in
  their own document.
- **A `.rcx` decoder over 61 multiplayer recordings**, with the per-turn checksum packets (`crates/don-replay/`).
  `schema/replay-validation.json` (2026-10-04) shows only `rules`, `scenario_data` and `script_run_time` matching
  substantively. Every dynamic channel (units, guys, builds, cities, goods, world, leaders) is at 0, so their sim does
  not replay a game.
- **The full BHS toolchain**: a compiler with 5/5 fixtures byte-identical to the shipped compiler, a VM and a builtin
  registry (`crates/don-bhs*`). Attrition has a tree-walker for the AI scripts only (`crates/sim/src/ai/bhs.rs`).
- **Live-process tooling** (RoNtoy: injection into the running retail game, a command-ingress path, network
  observation) and an RL API (Gymnasium/PettingZoo).

**Constraint flags.** It commits PDB dumps: `re/symtab.json` (11.5 MB), `schema/symbols.json` (17.6 MB),
`schema/types.json`, `schema/pdb-types.json`, `schema/vtables.json`. It also describes its state walkers as
"a literal transcription of the retail function's op sequence" (`crates/don-state/README.md`), and recent commits read
"transcribe … `Game::do_frame` step bodies". Both are exactly what CLAUDE.md forbids us. The project has been silent
since 2026-10-04; its last commit reads "checkpoint some uncommitted work after usage wall". Its 2026-10-04 `tick.rs`
is mostly stubs: three steps ported of 29.

## ben-wright-1/OpenRiseOfNations: what a playable RoN looks like now

The only repo here you can download and play. The work came in as 3 commits, built with Claude Code per its README.
It is ~61.6 k lines of C++/ObjC++, 41 k of them sim, plus ~30 k lines of specs in `docs/`. Verified in source:
- **Asset readers**: `engine/assets/` has `.bh3`, `.bha`, chunk, `.tga`, fonts, cursors, interface layouts, tileset,
  scenario `.scx` and `graphics_txt`.
- **A Metal renderer covering the whole frame**: `platform/metal/`, `engine/render/` (terrain transitions, rivers,
  forests, farms, fog overlay, lighting).
- **Two render specs**: `docs/lighting-spec.md` (the EE HDR → tone-map → bloom → FXAA pass order, read from `d3dgl.dll`)
  and `docs/renderstates.md` (the 50 `Scene::init_render_states` blocks). These bear directly on our open colour rows:
  grey buildings, pale mountain.
- **A full BHS compiler/VM** with an 873-entry builtin table, matching its own JS parser on all 365 shipped scripts.
- **Scenario/tutorial script hosting and the original HUD and menus.**
- **JS reference tools** (`re/tools/rcxdump.mjs`, `savdump.mjs`, `combat-ref.mjs`, …) and docs `replay-spec.md`,
  `state-spec.md`.

What it does not have: verification against the original's behaviour. Damage "matches on 169,312 pairs", but the other
side is its own JS reference, derived from the same reading. The one comparison against the original is the rules
tables, checked against a recording's rules block (`ron-rules`). Missing by its own account: random maps, saves and
replays, multiplayer, attrition and healing, transports, aircraft. `re/jobs/` holds Ghidra job lists (function names),
not decompiler output. `re/decomp/` and `re/iface/` are gitignored. The one bundled art file is a fan-made logo.

## The method references (not RoN)

- **empire_earth / _next** is a matching decompilation of a 2002 RTS, rebuilt with a modern toolchain. Notable pieces:
  its D3D7 subset reimplemented on wgpu ("GX"), `tools/open_data.py` turning retail data into glTF/PNG/CSV, and a
  headless live-game harness (`tools/harness/`). The whole thing is decompiler-derived source, so it is a reading
  source only, for both licence and constraint reasons.
- **gta-reversed** reimplements function by function inside the running original. This is the hook-replacement way of
  verifying a port incrementally; attrition's equivalent is the original's own dumps.
- **horizon** is the closest analogue to attrition's method at scale. Its "port ledger" reads every vanilla method
  against the port (`tools/port_ledger.py`, compare our census/rubric). It has 281 differential harnesses against
  vanilla and a gap count published in its README.

## Worth taking (all Ramon's call; nothing adopted)

1. **Ideas from don's oracle design**: native PE mapping and the ISLAND/SELF-CALL/DATA-ONLY reachability scheme, as a
   faster sweep path beside unicorn. This would mean reading `docs/oracle-architecture.md` and gruntz-decomp, never
   their code: GPL into attrition's MIT OR Apache-2.0 is one-way incompatible. It also needs an x86 host; we are on Apple Silicon, so this
   is low priority.
2. **A `.svx` save codec as a second oracle.** don shows one save gives all 15 checksum channels at any paused frame,
   which is a state oracle independent of the logger. Re-derive it from our own PDB/decompile. Do **not** read don's
   transcribed walkers or its committed symbol dumps into anything we write.
3. **OpenRiseOfNations' `lighting-spec.md` / `renderstates.md`** as a cross-check *after* our own reading of the render
   path. Its prose is GPL-3.0 and itself a reading of the same decompile. Treat it as a second reader's report, never
   a source. It must not reach a blind reader (DECISIONS 21/134).
4. **Format pointers**: OpenRoN's `bh3/bha/chunk/iface_layout` readers and don's `.rcx` notes can be compared against
   `docs/FORMATS.md` claims, as evidence of agreement only.
5. **Horizon's published gap ledger** as a presentation model for TRACKS/README. MIT, so no constraint concern.

None of these projects ships game assets or decompiler output in git, except don's committed PDB dumps. All of them are
reading sources at most. Nothing from them enters our repo.

## Where attrition stands beside them

Attrition is the only one of these that replays the original's own games in lockstep for their full length (every captured
game, to 24,000 frames). It is also the only one whose minted recording the original plays back in lockstep, and
whose sim reproduces 8 of the original's 12 counted state-checksum lines by simulation rather than by reloading
retail state. don reproduces all 15 channels only from a loaded retail save, and its own replays diverge on every
dynamic channel. OpenRoN has not been compared with the original's behaviour at all.

Attrition is behind on the playable surface. OpenRiseOfNations already plays tutorials, scenarios and skirmish natively
on macOS, with full art, HUD, audio and scenario scripts. Attrition's client draws captured games (59 of 60 pieces) and
has a plain lobby and HUD. don is ahead on a save-game codec, a full BHS compiler, a native fast oracle, live-process
tooling and an RL API.

Attrition's README line "No open-source Rise of Nations engine existed before this one" is no longer accurate. don
predates it by eleven days and OpenRiseOfNations shipped a playable build on 2026-10-14.
