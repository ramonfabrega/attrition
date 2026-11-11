# high-buildings: adjudication

- Item 4152, wave 3. Adjudicator: Claude Opus 5.5, 2026-11-11.
- Inputs: `high-buildings-reading.md` (first), `high-buildings-second.md` (blind second), the decompile export, and the full .text listing of `riseofnations.exe` (llvm-objdump).
- Leaves: `territory_high` (+0x9dc int), `reg_terr` (+0x145e ushort[64]), `high_buildings` (+0x5660 ushort[129]).

## Leaf 1: `territory_high` (+0x9dc, int)

### Claims lined up

| claim | first | second | status |
|---|---|---|---|
| writer: `World::compute_all_territory (006b5700)`, store `006b5880` via `edx = 0xe3ad68` (`&territory`), `0x4(%edx)` | yes | yes | agree, **confirmed** (`006b5834`, `006b5877`..`006b5880`) |
| writer: `GameDaemon::check_borders (00732060)`, same fold | yes (store not given) | yes, store `00732160` | agree, **confirmed** (`0073211b`, `00732157`..`00732160`) |
| no direct store at `0x9dc(` on a leader, no absolute `0xe3ad6c` | yes | yes | agree, **confirmed** (below) |
| predicate: slots 0..7 only (bound `0xe724c8`), `leader_flags & 1` | yes | yes | agree, **confirmed** (`testb $0x1,-0x9d8(%edx)` at `006b5840`/`00732120`; `cmpl $0xe724c8,%edx` at `006b588f`/`0073216f`) |
| step: `territory = Σ reg_terr[r]` over r in 0..63 with `Region.size (+0x14) != 0`, then `high = max(high, territory)`, signed | yes | yes | agree, **confirmed** (`cmpl $0x0,0x14(%ecx,%eax)` at `006b5856`; `cmovgl` at `006b587c`, signed) |
| `check_borders` folds only when a call this frame returned bit 2 and no sized region 0..63 has cursor `+0x2c` < size | yes | yes | agree, **confirmed** (`testb $0x2,%dl` at `007320c8`; the scan `007320f8`..`00732117`, `jl 0x732177` at `00732109`) |
| plain game: setup (`Setup::build_cities`, `Setup::build_game` twice) and every frame via `GameDaemon::process_all` | yes | yes | agree, **confirmed** (both readers; not re-read beyond the callers) |
| readers: `AchieveChartPane::set_chart_data (007f2c40)` (UI float), `LeaderData::log_data`; no sim reader | yes | yes | agree, **confirmed**: every `0x9dc(` access in `Unit::think_spellcaster`, `ObjectsOut::draw_underlays`, `Options::do_center`, `IFaceSelected::update_visibility`, `SelectGroup::get_item`, `ConquestSetupWin::on_combo_fill_items` is a load on a non-leader class |
| **reset** | `Leader::init (006e3930)` tribe<0 arm memsets `[+0, +0x6929)`; ordinary path skips +0x9dc | **"nothing resets it"**; no whole-struct clear; carries across games in one process | **disagree; first confirmed and extended, second corrected** |
| crate site: `Sim::update_territory_holdings` (`territory/upkeep.rs`), the fold at the two `border_pass.rs` sites | yes | yes | agree, checked below |

### The reset, settled from the listing

1. **`Leader::init (006e3930)`, tribe<0 arm.** `movl 0xc(%ebp),%esi` (`006e393f`) loads `param_2`; `jns 0x6e3a36` at `006e394a` skips the arm when it is >= 0. Inside: `pushl $0x6929; xorl %eax,%eax; pushl %eax; pushl %ebx; calll 0x55e0b2` at `006e398c`..`006e3995`. `0x55e0b2` is a `jmpl *0xac5424` thunk, and `rise_z.map` names it `_memset` (`0055e0b2`, `vcruntime:VCRUNTIME140.dll`; `__imp__memset` at `00ac5424`). cdecl, last push first: `memset(this, 0, 0x6929)`, so `[+0, +0x6929)` is zeroed and +0x9dc lies inside. The arm then falls through into the ordinary path at `006e3a36`. `param_2` is the tribe: the ordinary path stores it with `movl %esi,0xc(%ebx)` at `006e3aff`, and `types.txt` line 42092 names `LeaderData +0xc` `tribe`.
2. **`Leaders::clear (006ed5d0)`**, a writer neither reader named: `movl $0xe40fa4,%esi` (`006ed5d9`, `= 0xe3a390 + 0x6c14`), then `pushl $0x6929; leal -0x6c14(%esi),%eax; pushl $0; pushl %eax; calll 0x55e0b2` at `006ed5e0`..`006ed5ee`, which is `memset(&leaders.list[i], 0, 0x6929)`. The loop steps `0x1bbb` dwords (`0x6eec`) up to `0xe864dc` (decompile `funcs/Leaders/clear@006ed5d0.c` line 13 and the loop tail), so it covers **all ten slots**. Its callers are `Game::close (00594b80)` (`00594faa`) and `Game::run_oos_recovery (005869e0)` (`00586dd6`).
3. The ordinary path of `Leader::init` does skip +0x9dc. Its run is `0x9d4`, `0x9d8`, `0x9e0`, ... at `006e4a78`..`006e4a96`, and none of its `rep stos` runs (`+0x68f6` 9 dwords, `+0x480` 6, `+0x6d8` 0x2c, `+0x5762`, `+0x5a22`, `+0x6070`, `+0x6274`) spans it. Both readers had this right. The second reader's net for a whole-struct clear looked for `0x6ee4`/`0x6eec`/`0x45538` sizes, and the clear is `0x6929` bytes, the struct less its non-POD tail. That is why the net missed it.

Who reaches the tribe<0 arm: `Leaders::init (006ed850)` calls `Leader::init(i, -1, i)` with `pushl %edi; pushl $-0x1; pushl %edi` at `006ed8c5`..`006ed8cb` for every slot, and so does `WalkDataGame::walk_data (005a2360)` at `005a30db`..`005a30e6` on the one record at `0xe789dc`. `Setup::build_leader (005abc70)`, `Setup::build_game (005ac190)` (`005acee9`..`005acef7`) and `ScenarioRead::load_leader_chunk (009ac080)` pass **the slot's current tribe** (`pushl 0xe3a39c(%reg)`) as `param_2`, so they also memset any slot whose tribe is still negative. `Leaders::init` is called only from `Game::init_rules_and_teams (00589bb0)` (`00589d04`). In `Game::run (00584590)` that call (decompile line 312) comes before `Game::init` (line 460). `Game::init` reaches `Setup::build_game` (`0058d1b3`), and `Setup::build_game` runs setup's `compute_all_territory`. **So every game starts with `territory_high = 0` before its first fold, and `Game::close` zeroes it again at teardown.** The second reader's "a second game in the same process inherits the previous high-water mark" is wrong.

Verdict: **corrected (second), confirmed and extended (first)**. Deciding lines: `006e398c`..`006e3995` (`Leader::init`'s memset), `006ed5e0`..`006ed5ee` (`Leaders::clear`), and `0055e0b2` = `_memset` in `rise_z.map`.

### Writer completeness (own nets)

- Export: `->territory_high` is read only in `LeaderData::log_data`. `field_0x9dc` hits no leader writer.
- Listing, displacement `0x9dc(`: 60 hits. I mapped each non-`%ebp` store and load to its function: `HotKeyGroup*`, `SelectGroup*`, `Constants::init`, `Game::init` (`0058c50c`, a `Game` field), `Console::*`, `TileSet::init`, `TerrainMap*`, `ConquestSetupWin*`, `CommandPackage::process_hotkey`. Every store is on a non-leader class.
- Listing, absolutes `0xe3ad4x`..`0xe3ad7x`: `$0xe3ad68` appears only as the two folds' base. `0xe3ad68(%eax)` at `006ec5f4` and `0xe3ad68(%edx)` at `009e9006` are `territory` reads (`imull`). There is no `0xe3ad6c` absolute.
- The writers are the two folds plus the two memsets. No writer is missing.

### Crate site

`Sim::update_territory_holdings` is at `crates/sim/src/territory/upkeep.rs:129` (`grep -n`; the reader cited the doc line 128), called from `territory/border_pass.rs:56` (`start_border_pass`, pre-play), `:170` (`settle_borders`) and `:248` (`advance_border_pass` tail). Both readers put the fold there, and that is right: it is called at the setup site and at the pass-complete tail of `advance_border_pass`. **Reset**: the first reader's "only at sim creation" is right for the crate, because one `Sim` is one game, so the second reader's "deliberate cross-game divergence" note is moot. A crate that ever reuses a `Sim` across games must zero the field at game start.

### Rule as a builder should port it

- Field: `territory_high: i32` per player, in tiles (the unit of `territory`).
- Writers: (a) setup's wholesale territory recompute (`compute_all_territory`), and (b) the border pass's tail on a frame where some region finished this frame and no sized land region still has an open cursor (`check_borders`).
- Predicate: slots 0..7 with `leader_flags & 1`.
- Step: after `territory` is re-summed from `reg_terr` over land regions 0..63 with size != 0, set `territory_high = max(territory_high, territory)` (signed).
- Reset: zero at game creation (`Leaders::init` gives every slot `Leader::init(i, -1, i)`, whose tribe<0 arm memsets `[+0, +0x6929)`), and zero at game teardown and at OOS recovery (`Leaders::clear`). It is never lowered in between.
- No sim reader. It is checksummed: the row at `crates/sim/src/checksum/leaders.rs:236`.
- Plain game: yes.
- Confidence: high on all of it, all listing-confirmed. Open: nothing on the rule. Whether `territory`'s crate units equal the original's tiles decides whether the checksum value agrees. That is a property of `territory`, not of this leaf.

