//! **Parity held out** (item 4413, `docs/RECGAME.md` §10.3, DECISIONS 144
//! §2): eight lobbies named before the original plays them.

/// One held-out game, as the rule names it: a row of the lobbies on disk in
/// the order `Start:` counts them (`lobby::lobbies_on_disk`, the order
/// `every_lobby_s_start_built_from_its_choices` enumerates), its block
/// written out.
#[cfg(test)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct HeldOut {
    /// The row in `lobbies_on_disk`'s order.
    pub row: usize,
    /// The row's first capture, by file name.
    pub capture: &'static str,
    /// The `GAMEINFO` block's own lines, from `TEAM_STYLE` to `flags`, `; `
    /// between them.
    pub info: &'static str,
    /// Each `PLAYER` row's lines, `synced_*` off, `; ` between them.
    pub players: &'static [&'static str],
    /// FNV-1a 64 of the whole block's lines, `\n` between them: what the
    /// two written-out columns leave out.
    pub digest: u64,
}

/// The rows the rule names (DECISIONS 144 §2).
#[cfg(test)]
pub(crate) const HELD_OUT_ROWS: [usize; 8] = [0, 20, 40, 60, 80, 100, 120, 140];

#[cfg(test)]
fn fnv(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// A lobby's block in [`HeldOut`]'s columns: `(info, players, digest)`.
#[cfg(test)]
fn columns(lines: &[String]) -> (String, Vec<String>, u64) {
    let start = lines
        .iter()
        .position(|l| l.starts_with("TEAM_STYLE"))
        .unwrap_or(0);
    let first_player = lines
        .iter()
        .position(|l| l == "BEGIN PLAYER")
        .unwrap_or(lines.len());
    let info = lines[start..first_player].join("; ");
    let mut players: Vec<Vec<&str>> = Vec::new();
    for l in &lines[first_player..] {
        if l == "BEGIN PLAYER" {
            players.push(Vec::new());
        } else if !l.starts_with("synced_")
            && let Some(p) = players.last_mut()
        {
            p.push(l);
        }
    }
    (
        info,
        players.iter().map(|p| p.join("; ")).collect(),
        fnv(&lines.join("\n")),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The declaration** (item 4413's first act): the eight lobbies, named
    /// and written out before any take, against what `lobbies_on_disk`
    /// yields today, so the set cannot drift silently. `RON_HELD_OUT_PRINT=1`
    /// prints the table as Rust.
    #[test]
    fn the_held_out_eight_are_the_rows_the_rule_names() {
        let Some(dir) = crate::testenv::gamelog_dir() else {
            eprintln!("skipping: no gamelog directory (set RON_GAMELOG_DIR)");
            return;
        };
        let lobbies = crate::lobby::lobbies_on_disk(&dir, crate::corpus::Ledger::of_tree())
            .expect("the logs");
        assert_eq!(
            lobbies.len(),
            crate::diff::START_LOBBIES,
            "the rows are of 161"
        );
        let print = std::env::var_os("RON_HELD_OUT_PRINT").is_some();
        for (k, &row) in HELD_OUT_ROWS.iter().enumerate() {
            let l = &lobbies[row];
            let (info, players, digest) = columns(&l.lines);
            if print {
                println!("    HeldOut {{");
                println!("        row: {row},");
                println!("        capture: {:?},", l.captures[0]);
                println!("        info: {info:?},");
                println!("        players: &[");
                for p in &players {
                    println!("            {p:?},");
                }
                println!("        ],");
                println!("        digest: {digest:#018x},");
                println!("    }},");
                continue;
            }
            let want = &HELD_OUT[k];
            assert_eq!(want.row, row, "HELD_OUT[{k}]'s row");
            assert_eq!(
                (l.captures[0].as_str(), info.as_str(), players, digest),
                (
                    want.capture,
                    want.info,
                    want.players.iter().map(|s| s.to_string()).collect(),
                    want.digest
                ),
                "row {row}: the lobby the rule named"
            );
        }
    }
}

/// **The eight, as declared** (item 4413, before any take): rows 0, 20, …,
/// 140 of the 161 lobbies in `lobbies_on_disk`'s order, each the human
/// (tribe 4, slot 0) against one computer at difficulty 2, `MAP_SIZE 2`:
///
/// | row | first capture | `MAP_STYLE` | seed | computer tribe | `DIFFICULTY` | ages |
/// |---|---|---|---|---|---|---|
/// | 0 | run fuzz-424242 | 18 | 424242 | 16 | 0 | 0..7 |
/// | 20 | run1123 New World | 16 | 12345 | 11 | 0 | 0..7 |
/// | 40 | run1203 East Indies | 18 | 12345 | 19 | 5 | 2..2 |
/// | 60 | run1284 Great Sahara | 7 | 12345 | 5 | 5 | 3..3 |
/// | 80 | run1349 Great Lakes | 14 | 12345 | 14 | 5 | 4..4 |
/// | 100 | run1410 Great Lakes | 14 | 12345 | 1 | 5 | 7..7 |
/// | 120 | run346 Islands | 18 | 12345 | 11 | 5 | 0..7 |
/// | 140 | run915 East Indies | 18 | 12345 | 22 | 4 | 0..7 |
///
/// Each is played to 6,000 frames with the human idle, as `run1700 + k` for
/// `HELD_OUT[k]`. The columns below are the test's; the table above is
/// their reading.
#[cfg(test)]
pub(crate) const HELD_OUT: [HeldOut; 8] = [
    HeldOut {
        row: 0,
        capture: "gamelog-fuzz-424242-early.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 18; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 0; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 424242; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player 1",
            "flags 1; tribe 16; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; King Dangun",
        ],
        digest: 0x2dc921584dc15885,
    },
    HeldOut {
        row: 20,
        capture: "gamelog-run1123-newworld-cover1-clickfree-diagonal.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 16; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 0; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 11; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; William the Conqueror",
        ],
        digest: 0xdba9f8185b59ec20,
    },
    HeldOut {
        row: 40,
        capture: "gamelog-run1203-eastindies-lakota-medieval-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 18; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 2; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 2; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 19; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Sitting Bull",
        ],
        digest: 0x2996a3d30281b30f,
    },
    HeldOut {
        row: 60,
        capture: "gamelog-run1284-greatsahara-greek-gunpowder-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 7; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 3; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 3; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 5; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Pericles",
        ],
        digest: 0xf48a448f02c0980a,
    },
    HeldOut {
        row: 80,
        capture: "gamelog-run1349-greatlakes-chinese-enlightenment-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 14; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 4; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 4; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 14; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Emperor Wu Di",
        ],
        digest: 0x9757262322138b86,
    },
    HeldOut {
        row: 100,
        capture: "gamelog-run1410-greatlakes-maya-information-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 14; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 7; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 1; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Ahmok",
        ],
        digest: 0xa742edf3f4748a61,
    },
    HeldOut {
        row: 120,
        capture: "gamelog-run346-islands-toughest-24k-trace.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 18; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 11; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; William the Conqueror",
        ],
        digest: 0x9a1298f644fbc845,
    },
    HeldOut {
        row: 140,
        capture: "gamelog-run915-eastindies-dutch-diff4-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 18; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 4; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 22; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Beatrix Wilhelmina Armgard",
        ],
        digest: 0xafdc08ec613c25c1,
    },
];
