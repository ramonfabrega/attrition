//! The census by layer, and the board's number: the simulation layer's
//! backed share (item 1467; DECISIONS 63 (iv), the sweep lane's score).
//!
//! `tools/census.py --layers` files every function the Ghidra export lists
//! under a layer — simulation, AI, engine, interface or unknown — by its
//! PDB source path, and marks it **backed** when a document's coverage
//! section cites it in a diff-backed span or a sweep `#[test]` in
//! `crates/sim` (one that runs the original under the emulator) names it.
//! The rule and what it cannot count are the tool's docstring.
//!
//! This module pins the number, as [`crate::blind`] pins the blind list:
//! a document's new coverage citation, or a new sweep, moves it, and the
//! landing that moves it re-pins it here and says so; the handoff's
//! `Census:` line is read against the same two constants
//! (`diff::floors`). The export, the PDB and `llvm-pdbutil` live outside
//! the repo; a machine without them skips and says so.

/// Simulation-layer functions a coverage section's diff or a sweep backs (item 2117: 566 -> 571,
/// `docs/GOLDEN.md` §77.1 — `think_attack`, `add_to_army`, `get_unit`, `go_to_unit`, `go_to`; item
/// 2129: 571 -> 581, the sweep's fifty-first batch; item 2149: 581 -> 591, the fifty-second; item
/// 2131: 591 -> 602, `docs/GOLDEN.md` §78.1 — `get_final_loc`, `get_loc_to`, `action_move_near`,
/// `action_move_to`, `process_move_to`, `compute_form`, `add_group_move_order`, and chapter twelve's
/// follow line, `process_follow`, `action_follow`, `add_follow_order`, `do_follow`; item 2160: 602 -> 612, the sweep's
/// fifty-third batch; item 2170: 612 -> 622, the fifty-fourth; item 2181: 622 -> 632, the
/// fifty-fifth; item 2165: 632 -> 639, `docs/GOLDEN.md` §79.1 — `Unit::init`'s Bark arm,
/// `set_stance`, `fight`'s re-search, `issue_stance`, `process_stance`, `action_stance`,
/// `get_stance_option`, `get_stance_type`; item 2187: 639 -> 649, the sweep's fifty-sixth batch; item
/// 2200: 649 -> 659, a second sweep lane's ten with a port consumer; item 2211: 659 -> 669, the
/// sweep's fifty-seventh batch; item 2205: 669 -> 670, `compare_target`, by golden chapter sixty-four's
/// sea arm, `docs/GOLDEN.md` §80.1; item 2223: 670 -> 680, the sweep's fifty-eighth batch; item 2217:
/// 680 -> 690, the second sweep lane's ten with a port consumer; item 2241: 690 -> 700, the sweep's
/// fifty-ninth batch; item 2253: 700 -> 710, the sweep's sixtieth batch; item 2247: 710 -> 720, the second
/// sweep lane's next ten, the territory family through `obj --sparse`; item 2235: 720 -> 721,
/// `UnitData::get_action`, by the city alarm's worker arm, `docs/GOLDEN.md` §81.1; item 2271: 721 ->
/// 731, the sweep's sixty-first batch, the backups and restores through `obj --ptrarg-null`; item
/// 2276: 731 -> 741, the second sweep lane's ten, the squad and city chains through `obj --link`; item
/// 2281: 741 -> 751, the sweep's sixty-second batch, seven of its ten through `obj --record`; item
/// 2298: 751 -> 761, the sweep's sixty-third batch, the unbuilt lists, the speed and the spells;
/// item 2287: 761 -> 771, the second sweep lane's ten with a port consumer, five through `obj --record`;
/// item 2309: 771 -> 782, the sweep's sixty-fourth batch, the leaders-table walks through `obj --record IDX@S`;
/// item 2325: 782 -> 793, the sweep's sixty-fifth batch, eleven, `get_inside`'s out-pointer through
/// `obj --outarg` and `memcpy` through `--stub-cdecl`;
/// item 2314: 793 -> 803, the second sweep lane's ten with a port consumer, four through `obj --rows N`;
/// item 2336: 803 -> 813, the sweep's sixty-sixth batch, a `bool`'s low byte through `obj --ret al` and
/// `timeGetTime` through `--stub iat`;
/// item 2331: 813 -> 815, chapter sixty-seven's coverage, `train_time` and `get_cost` diff-backed by run845;
/// item 2346: 815 -> 825, the sweep's sixty-seventh batch, `edx` beside `ecx` through `obj --edx`, a table
/// index through `--tame`, a container emptied through `--stub-write`;
/// item 2341: 825 -> 835, the second sweep lane's next ten with a port consumer, the order adders and the
/// gaia walks, five through `obj --record-order`;
/// item 2362: 835 -> 845, the sweep's sixty-eighth batch, eight `vtordisp` thunks through `fwd`'s
/// `vtordisp` token, `set_shooter_vis` and `issue_unqueue`;
/// item 2374: 845 -> 873, the sweep's sixty-ninth batch, the twenty-eight `vtordisp` thunks left, one `fwd`
/// line each, the band empty on `census.py --unbacked`;
/// item 2357: 873 -> 874, chapter sixty-eight's coverage, `get_military_level_slow` diff-backed by run852;
/// item 2379: 874 -> 884, the sweep's seventieth batch, ten `CommandManager::issue_*` whose command
/// bytes `obj --deref` reads at the call;
/// item 2368: 884 -> 894, the second sweep lane's ten with a port consumer, `Caravan::trade_value`'s eight
/// windows among them, on `--regen`'s kept port block and `--seq`'s terminator;
/// item 2395: 894 -> 904, the sweep's seventy-first batch, ten `issue_*` issuers left after 2379
/// (`CommandManager` six, `GroupOut` four), re-pinned by the commander at booking;
/// item 2400: 904 -> 916, the port-consumer sweep's eleven on `--record-edx`, `--ring` and `--ecx-answer`,
/// `compute_trade` and `kill_group_move` regenerated without hand edits;
/// item 2421: 916 -> 926, the sweep's seventy-second batch, ten `GroupOut::issue_*`, seven of them
/// forwarders written as hand `--port`s;
/// item 2443: 926 -> 936, the sweep's seventy-third batch, eight `GroupOut` and two `CommandManager`
/// issuers, `--fwd-gate` built and most of the list `issue_halt`'s shape after all;
/// item 2454: 936 -> 946, the sweep's seventy-fourth batch, ten issuers on the new `--cmd-gate`,
/// `issue_camera`'s truncation ported as integers;
/// item 2464: 946 -> 956, the sweep's seventy-fifth batch, the `this+OFF` token and ten `CommandManager`
/// issuers, six on `--cmd-gate` and four hand `--port`s reading an object argument;
/// item 2469: 956 -> 963, the sweep's seventy-sixth batch, `--cmd-obj` built (the three object-argument
/// issuers regenerated from it) and seven group-first issuers on `--cmd-gate`;
/// item 2481: 963 -> 965, the sweep's seventy-seventh batch, three of ten, `--cmd-byte` built and three
/// group-first issuers on `--cmd-gate`, the rest of the list read and not the shape;
/// item 2486: 965 -> 974, the sweep's seventy-eighth batch, `--cmd-seh` built and the treaty family's seven
/// on it, `_chat_set` and `_player_speed` by hand on existing flags;
/// item 2495: 974 -> 984, the sweep's seventy-ninth batch, `--chain` built and `PathFinder`'s four
/// forwarders on it, `issue_cannon_time`'s by-value `String` arm and five more, ten of ten;
/// item 2507: 984 -> 994, the sweep's eightieth batch, `mix`'s `stale` token and the three `walk_data` on it,
/// `TurnControl`'s three issuers, two `PathFinder` forwarders and `MapFairness`'s two, ten of ten;
/// item 2513: 994 -> 997, the sweep's eighty-first batch, `--deref`'s call-time cell read and the three `find_*`
/// regenerated on it, `Border::build_border`, `find_global_render_z` and `air_alt_speed`, three of ten;
/// item 2537: 997 -> 1003, the sweep's eighty-second batch, `obj --ret xmm0` and `GuyData`'s five render getters
/// and `get_game_fps_average` on it, six of ten).
/// **The commander re-pins this constant in the booking commit** (Loop 2367,
/// the thirty-fourth pass): it is the two sweep lanes' shared pin, seven of
/// thirteen merges backed out on it, and the lane gate forgives its red
/// (`release_gate.py`'s `COMMANDERS_LINES`); a lane reports the count
/// `tools/census.py --layers` measures on its tree.
pub const SIMULATION_BACKED: usize = 1003;
/// Simulation-layer functions: those whose PDB line record starts in a
/// `game\` file the layer rule files under simulation.
pub const SIMULATION_FUNCTIONS: usize = 3611;

#[cfg(test)]
mod tests {
    use super::*;

    /// The number the census measures from `json`'s key `"<key>": N`.
    fn field(json: &str, key: &str) -> Option<usize> {
        let at = json.find(&format!("\"{key}\":"))? + key.len() + 3;
        json[at..]
            .trim_start()
            .split(|c: char| !c.is_ascii_digit())
            .next()?
            .parse()
            .ok()
    }

    /// **The simulation layer's backed share is the pinned one**, measured
    /// by `tools/census.py --layers --json` on this tree. Made to fail first
    /// with the pin moved by one.
    #[test]
    fn the_census_s_simulation_share_is_pinned() {
        let home = std::env::var("HOME").unwrap_or_default();
        let index = format!("{home}/ghidra-projects/decomp/INDEX.tsv");
        if !std::path::Path::new(&index).is_file() {
            eprintln!("skipping: no {index} (the Ghidra export is not on this machine)");
            return;
        }
        let Some(install) = crate::testenv::install_root() else {
            eprintln!("skipping: no install (set RON_INSTALL) for sbl/rise.pdb");
            return;
        };
        let pdb = format!("{install}/sbl/rise.pdb");
        if !std::path::Path::new(&pdb).is_file() {
            eprintln!("skipping: no {pdb}");
            return;
        }
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
        let out = std::process::Command::new("python3")
            .arg(format!("{root}/tools/census.py"))
            .args(["--layers", "--json", "--index", &index, "--pdb", &pdb])
            .output()
            .expect("python3 runs");
        let text = String::from_utf8_lossy(&out.stdout);
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            if err.contains("no llvm-pdbutil") {
                eprintln!("skipping: {err}");
                return;
            }
            panic!("census.py --layers failed: {err}");
        }
        let measured = (
            field(&text, "simulation_backed").expect("simulation_backed"),
            field(&text, "simulation_functions").expect("simulation_functions"),
        );
        assert_eq!(
            measured,
            (SIMULATION_BACKED, SIMULATION_FUNCTIONS),
            "the census moved: re-pin `census::SIMULATION_BACKED` and \
             `SIMULATION_FUNCTIONS` to (backed, functions) as measured, and the \
             handoff's `Census:` line with them"
        );
    }

    #[test]
    fn a_measured_field_is_read_whole() {
        let json = r#"{"layers": {}, "simulation_backed": 45, "simulation_functions": 3611}"#;
        assert_eq!(field(json, "simulation_backed"), Some(45));
        assert_eq!(field(json, "simulation_functions"), Some(3611));
        assert_eq!(field(json, "simulation"), None);
    }
}
