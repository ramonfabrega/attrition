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

    /// Where game `k`'s file and take sit: the client's recording under
    /// `~/ron-golden/recordings/`, and `run1700 + k`'s `map-*/rontrace.log`.
    /// **Both are asked of the fixture audit**, the take under `map-0` when
    /// the run's directory holds no map: until the pass of 2026-11-19,
    /// second, a lost recording or a lost run directory filed no request,
    /// so the scores below skipped with `0 missing fixtures` on the gate's
    /// log. Seen with `HOME` pointed at an empty directory: sixteen requests
    /// missing, where the audit had counted none.
    fn paths(k: usize) -> (Option<std::path::PathBuf>, Option<std::path::PathBuf>) {
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let rcx = crate::testenv::experiment(
            &home.join("ron-golden/recordings"),
            &format!("item4413-row{:03}-6000.rcx", HELD_OUT[k].row),
        );
        let run = home.join(format!("ron-golden/run{}", 1700 + k));
        let map = std::fs::read_dir(&run)
            .ok()
            .and_then(|d| {
                d.filter_map(Result::ok).map(|e| e.path()).find(|p| {
                    p.file_name()
                        .is_some_and(|n| n.to_string_lossy().starts_with("map-"))
                })
            })
            .unwrap_or_else(|| run.join("map-0"));
        (rcx, crate::testenv::experiment(&map, "rontrace.log"))
    }

    /// **The held-out scores** (item 4413, `docs/RECGAME.md` §10.3): each of
    /// the eight games the original has played, scored by
    /// `checksum::playback` on every state to the take's last, against its
    /// pinned score: [`FirstScore::moved`] where a later fix has moved it,
    /// else the score as found. A game with no take skips, and says so; a
    /// take with no pin fails and prints the score to pin.
    #[test]
    fn the_held_out_games_score_as_pinned() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let mut wrong: Vec<String> = Vec::new();
        for (k, held) in HELD_OUT.iter().enumerate() {
            let (Some(rcx), Some(take)) = paths(k) else {
                eprintln!(
                    "skipping: row {} has no file or no run{} take",
                    held.row,
                    1700 + k
                );
                continue;
            };
            let pin = &SCORES[k];
            let until = pin.as_ref().and_then(|p| p.last);
            let score =
                match crate::checksum::playback::compare_until(inst.root(), &rcx, &take, until) {
                    Ok(s) => s,
                    Err(e) => {
                        wrong.push(format!("row {} run{}: not scored: {e}", held.row, 1700 + k));
                        continue;
                    }
                };
            let first: Vec<First> = crate::checksum::CHECKSUM_LINES
                .iter()
                .zip(&score.first)
                .filter_map(|(l, f)| f.map(|f| (l.0, f)))
                .collect();
            eprintln!(
                "HELD OUT row {} run{}: states {} agreed {:?} first {first:?} over {:?}",
                held.row,
                1700 + k,
                score.states,
                score.agreed,
                score.over
            );
            for (line, d) in crate::checksum::CHECKSUM_LINES.iter().zip(&score.details) {
                if let Some(d) = d {
                    eprintln!("  {}: {d}", line.0);
                }
            }
            let Some(pin) = pin else {
                wrong.push(format!(
                    "row {}: run{} is taken and SCORES[{k}] is unpinned",
                    held.row,
                    1700 + k
                ));
                continue;
            };
            let want = pin.moved.unwrap_or(pin.found);
            let got = (score.states, score.agreed, first, score.over);
            if got != (want.states, want.agreed, want.first.to_vec(), want.over) {
                wrong.push(format!(
                    "row {} run{}: states, agreeing, first partings, ours' end: got {got:?}",
                    held.row,
                    1700 + k
                ));
            }
        }
        assert!(
            wrong.is_empty(),
            "the held-out games against their pins:\n{}",
            wrong.join("\n")
        );
    }

    /// **Row 120's barge is born with `form` 0** (item 4435, `docs/RECGAME.md`
    /// §10.3): run1716, the ranges take of run1706's file, holds the units
    /// line's every range on state 2694, where the computer's transport
    /// barge `1/15` (type 320) is born holding a move. Range 521, its
    /// `collide_frame..play`, reads `form` (`+0xaa`, byte 98) 0 on theirs,
    /// `Unit::init@00612100:81`–`86`'s birth value for every type outside
    /// the four citizen ids; this crate's `transport_boat` had left
    /// `Unit::new`'s −1. Every units range on 2694, ours against theirs,
    /// byte for byte: red on the tree before the fix, at range 521.
    #[test]
    fn row_120_s_barge_is_born_with_form_zero_on_run1716_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(6) else {
            eprintln!("skipping: row 120's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1716/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1716's ranges record is missing");
            return;
        };
        const STATE: i64 = 2694;
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 120's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        while built.sim.frame < STATE {
            stream.apply(built.sim.frame as i32, &mut built);
            built.sim.command().tick();
        }
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        let set = record
            .first_of(STATE - 1)
            .expect("state 2694's set")
            .clone();
        let theirs = record.ranges(&set).expect("the set's ranges");
        let theirs: Vec<_> = theirs.iter().filter(|r| r.line == 0).collect();
        let ours = built.sim.checksum_units();
        assert_eq!(ours.ranges.len(), theirs.len(), "units ranges on 2694");
        for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
            assert_eq!(
                a.bytes, b.bytes,
                "units range {k} on 2694, record {} {}",
                a.record, a.field
            );
        }
        // The barge's own range, by name: `form` at byte 98 of range 521.
        assert_eq!(theirs[521].bytes.len(), 111);
        assert_eq!(theirs[521].bytes[0xaa - 0x48], 0, "theirs' form");
    }

    /// **Row 120's scout lands with its figure's `last` on its point**
    /// (item 4439, `docs/RECGAME.md` §10.3): on 3250 the barge `1/15`
    /// unloads the scout `1/0` at (19080, 39768). `Unit::come_out@00617c10`
    /// seats it through `set_new_location(this, x, y, 1, 1)`
    /// (`:519`), whose `param_3` reaches `Guy::set_new_location(guy 0, x,
    /// y, 1)` and copies the point and `z` into `last_x`/`last_y`/`last_z`
    /// (`005d86f0:66`–`70`), and then turns it to the host's angle with
    /// `Unit::set_angle(…, 1)` (`:629`), whose `Guy::set_angle(guy 0, a,
    /// 1)` writes `last_angle` (`005d9010:13`–`16`). Every units and guys
    /// range on 3250, ours against theirs, byte for byte: red on the tree
    /// before the fix, at guys range 51, the scout's figure, `last_*` ours
    /// (30849, 41470, −37, −1553072128), theirs (19080, 39768, −30,
    /// −664928256).
    #[test]
    fn row_120_s_scout_lands_with_its_figure_s_last_on_its_point_on_run1716_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(6) else {
            eprintln!("skipping: row 120's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1716/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1716's ranges record is missing");
            return;
        };
        const STATE: i64 = 3250;
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 120's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        while built.sim.frame < STATE {
            stream.apply(built.sim.frame as i32, &mut built);
            built.sim.command().tick();
        }
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        let set = record
            .first_of(STATE - 1)
            .expect("state 3250's set")
            .clone();
        let all = record.ranges(&set).expect("the set's ranges");
        for (line, ours) in [
            (0u8, built.sim.checksum_units()),
            (6u8, built.sim.checksum_guys()),
        ] {
            let theirs: Vec<_> = all.iter().filter(|r| r.line == line).collect();
            assert_eq!(
                ours.ranges.len(),
                theirs.len(),
                "line {line}'s ranges on 3250"
            );
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "line {line} range {k} on 3250, record {} {}",
                    a.record, a.field
                );
            }
        }
    }

    /// **Row 120's MLRS reads `cavarch_o` −2 in the units line** (item
    /// 4443, `docs/RECGAME.md` §10.3): on 5864 the computer's fire-on-the-move
    /// MLRS `1/35` (type 273) steps an `ExploreTo` with an empty search after
    /// an empty one, and `Unit::cavarch_fight@005ff4b0:34` writes `cavarch_o`
    /// (`+0xa2`) `0xfffe`. This crate held the −2 as `cavarch_idle` and gave
    /// it to the dump differ, but the units line's `collide_frame..play`
    /// range wrote −1. Every units range on 5864, ours against theirs, byte
    /// for byte: red on the tree before the fix, at range 1139, record 35,
    /// byte 90, ours `ff ff`, theirs `fe ff`.
    #[test]
    fn row_120_s_mlrs_reads_cavarch_o_minus_two_on_run1716_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(6) else {
            eprintln!("skipping: row 120's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1716/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1716's ranges record is missing");
            return;
        };
        const STATE: i64 = 5864;
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 120's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        while built.sim.frame < STATE {
            stream.apply(built.sim.frame as i32, &mut built);
            built.sim.command().tick();
        }
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        let set = record
            .first_of(STATE - 1)
            .expect("state 5864's set")
            .clone();
        let theirs = record.ranges(&set).expect("the set's ranges");
        let theirs: Vec<_> = theirs.iter().filter(|r| r.line == 0).collect();
        let ours = built.sim.checksum_units();
        assert_eq!(ours.ranges.len(), theirs.len(), "units ranges on 5864");
        for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
            assert_eq!(
                a.bytes, b.bytes,
                "units range {k} on 5864, record {} {}",
                a.record, a.field
            );
        }
        // The MLRS's own range, by name: `cavarch_o` at byte 90 of 1139.
        assert_eq!(theirs[1139].bytes.len(), 111);
        assert_eq!(
            &theirs[1139].bytes[0xa2 - 0x48..0xa4 - 0x48],
            &[0xfe, 0xff],
            "theirs' cavarch_o"
        );
    }

    /// **The held-out line**, from the table alone: the games whose score
    /// agrees on every line on every state, of the eight, and how many are
    /// taken.
    #[test]
    fn the_held_out_line_reads_as_pinned() {
        let taken = SCORES.iter().flatten().count();
        let whole = SCORES
            .iter()
            .flatten()
            .filter(|p| {
                let s = p.moved.unwrap_or(p.found);
                s.first.is_empty() && s.agreed == [s.states; 12]
            })
            .count();
        assert_eq!(
            (whole, taken),
            HELD_OUT_LINE,
            "parity held out: N of 8, T taken"
        );
    }

    /// The queue's line for [`SCORES`]: the games whole of the eight, the
    /// games taken, the lines whole of twelve a game as found and now
    /// ([`FirstScore::moved`], else as found), and each game's lowest first
    /// parting in row order, as found and now (`whole` for a game with
    /// none, `-` for one not taken).
    fn queue_line() -> String {
        let lines = |s: &Score| s.agreed.iter().filter(|&&a| a == s.states).count();
        let lowest = |s: &Score| {
            s.first
                .iter()
                .map(|f| f.1.0)
                .min()
                .map_or("whole".to_string(), |v| v.to_string())
        };
        let col = |now: bool| -> (usize, String) {
            let scores = SCORES.iter().map(|p| {
                p.as_ref().map(|p| {
                    if now {
                        p.moved.unwrap_or(p.found)
                    } else {
                        p.found
                    }
                })
            });
            let n = scores.clone().flatten().map(|s| lines(&s)).sum();
            let firsts: Vec<String> = scores
                .map(|s| s.map_or("-".to_string(), |s| lowest(&s)))
                .collect();
            (n, firsts.join(" "))
        };
        let (found, found_firsts) = col(false);
        let (now, now_firsts) = col(true);
        format!(
            "Parity held out: {} of {} whole \u{b7} {} taken \u{b7} lines {found} of {} as found, \
             {now} now \u{b7} first partings {found_firsts} as found, {now_firsts} now",
            HELD_OUT_LINE.0,
            SCORES.len(),
            HELD_OUT_LINE.1,
            12 * HELD_OUT_LINE.1,
        )
    }

    /// **The handoff's parity-held-out line is the pinned scores** (the
    /// pass of 2026-11-19, second; DECISIONS 145 §1): `docs/QUEUE.md`
    /// carries [`queue_line`] beside the older battery's `Held-out:` (parked
    /// 4413a: two lines were named "held out" and only the battery's was
    /// parsed). The found half never moves; a fix that moves a game's second
    /// column moves the `now` half, and the commander writes it at the
    /// booking. Made to fail first on a handoff with no such line.
    #[test]
    fn the_handoff_s_parity_held_out_line_is_the_pinned_scores() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/QUEUE.md");
        let q = std::fs::read_to_string(path).expect("docs/QUEUE.md");
        let want = queue_line();
        let line = q.lines().find(|l| l.starts_with("Parity held out:"));
        assert_eq!(
            line.map(str::trim_end),
            Some(want.as_str()),
            "docs/QUEUE.md's handoff carries `{want}` (`diff::held_out::SCORES`), beside the \
             Held-out line"
        );
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

/// A line's first parting: `(line, (state, ours, theirs))`, ours `None`
/// where the record is unemitted.
#[cfg(test)]
type First = (&'static str, (i32, Option<u32>, u32));

/// One score: the states compared, each line's agreeing states in
/// `CHECKSUM_LINES`' order, and the lines' first partings.
#[cfg(test)]
#[derive(Clone, Copy)]
pub(crate) struct Score {
    pub states: u32,
    pub agreed: [u32; 12],
    pub first: &'static [First],
    /// The first state our walk holds `Sim::game_over` on, if any.
    pub over: Option<i32>,
}

/// A held-out game's pins: its first score, as found and never re-pinned in
/// place, and the second column a later fix moves.
#[cfg(test)]
pub(crate) struct FirstScore {
    /// The last state compared, where the original ended the game before
    /// the file's closing package.
    pub last: Option<i32>,
    pub found: Score,
    pub moved: Option<Score>,
}

/// **The held-out scores**, `HELD_OUT[k]` played as `run1700 + k`: `None`
/// until the original has played it.
#[cfg(test)]
pub(crate) const SCORES: [Option<FirstScore>; 8] = [
    // run1700
    // row 0, the fuzz lobby (seed 424242), as found 2026-11-19: ten lines whole;
    // units 5,435 of 6,001, first parting state 2990; guys 5,998, first 3157.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 6001,
            agreed: [
                5435, 6001, 6001, 6001, 6001, 6001, 5998, 6001, 6001, 6001, 6001, 6001,
            ],
            first: &[
                ("units", (2990, Some(3067758760), 2623096745)),
                ("guys", (3157, Some(3930404420), 1713190284)),
            ],
            over: None,
        },
        // Moved by item 4435 (a transport's birth `form`, 0 not -1): units
        // 5,435 -> 5,998, first 2990 -> 3157 (the barge 1/15, born on 2990).
        // Moved by item 4439 (a passenger's figure lands with `last` on its
        // seat): units and guys 5,998 -> 6,001, whole; the game is whole.
        moved: Some(Score {
            states: 6001,
            agreed: [6001; 12],
            first: &[],
            over: None,
        }),
    }),
    // run1701
    // row 20, New World (seed 12345), as found 2026-11-19: ten lines whole;
    // units 5,073 of 6,001, first parting state 3694; guys 5,997, first 4180.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 6001,
            agreed: [
                5073, 6001, 6001, 6001, 6001, 6001, 5997, 6001, 6001, 6001, 6001, 6001,
            ],
            first: &[
                ("units", (3694, Some(1358059058), 1889097011)),
                ("guys", (4180, Some(797978321), 3921096337)),
            ],
            over: None,
        },
        // Moved by item 4435: units 5,073 -> 5,997, first 3694 -> 4180 (the
        // barge 1/14, born on 3694). Moved by item 4439: units and guys 5,997
        // -> 6,001, whole; the game is whole.
        moved: Some(Score {
            states: 6001,
            agreed: [6001; 12],
            first: &[],
            over: None,
        }),
    }),
    // run1702
    // row 40, East Indies at Medieval (seed 12345), as found 2026-11-19: the
    // original ended the game itself on 4973 (ours holds no `game_over` there);
    // seven lines whole; units 1,858 of 4,974, first 1858; groups 1,843, first
    // 711; guys 4,824, first 2444; cities 4,967, first 4127; world 4,859, first
    // 4859.
    Some(FirstScore {
        last: Some(4973),
        found: Score {
            states: 4974,
            agreed: [
                1858, 4974, 4974, 4974, 4974, 1843, 4824, 4974, 4967, 4974, 4974, 4859,
            ],
            first: &[
                ("units", (1858, Some(2347968956), 4143655355)),
                ("groups", (711, Some(3431986133), 1658910664)),
                ("guys", (2444, Some(4226756071), 1757622212)),
                ("cities", (4127, Some(3606914667), 2610767417)),
                ("world", (4859, Some(155879679), 1707773025)),
            ],
            over: None,
        },
        // Moved by origin/main at 0b34ad2ac (merged into item 4413's branch): groups
        // 1,843 -> 4,964, its first parting 711 -> 1854. Moved by item 4439:
        // guys 4,824 -> 4,876, first 2444 -> 2487.
        // Moved by item 4443 (the units line folds `cavarch_o` -2): units
        // 1,858 -> 2,487, first 1858 -> 2487. On 1858 (our walk) the
        // fire-on-the-move 1/32 (type 156) is the one unit idle at -2.
        moved: Some(Score {
            states: 4974,
            agreed: [
                2487, 4974, 4974, 4974, 4974, 4964, 4876, 4974, 4967, 4974, 4974, 4859,
            ],
            first: &[
                ("units", (2487, Some(1142976455), 3877923716)),
                ("groups", (1854, Some(284774603), 2949074744)),
                ("guys", (2487, Some(2678849350), 3953000265)),
                ("cities", (4127, Some(3606914667), 2610767417)),
                ("world", (4859, Some(155879679), 1707773025)),
            ],
            over: None,
        }),
    }),
    // run1703
    // row 60, Great Sahara at Gunpowder (seed 12345), as found 2026-11-19: the
    // original ended the game itself on 4074 (ours holds no `game_over` there);
    // five lines whole; units 1,055 of 4,075, first 1055; builds 4,073, first 4019;
    // groups 3,863, first 3598; guys 3,958, first 2014; leaders 4,068, first 969;
    // cities 4,066, first 2253; world 3,984, first 3984.
    Some(FirstScore {
        last: Some(4074),
        found: Score {
            states: 4075,
            agreed: [
                1055, 4073, 4075, 4075, 4075, 3863, 3958, 4068, 4066, 4075, 4075, 3984,
            ],
            first: &[
                ("units", (1055, Some(2835614280), 2603158087)),
                ("builds", (4019, Some(202745570), 1178445602)),
                ("groups", (3598, Some(3778361523), 962934390)),
                ("guys", (2014, Some(4240840485), 61413159)),
                ("leaders", (969, Some(2042929290), 243376476)),
                ("cities", (2253, Some(2126061725), 1479221375)),
                ("world", (3984, Some(402934873), 4267790267)),
            ],
            over: None,
        },
        // Moved by item 4444 (the caps' republic term read every frame,
        // run1722): leaders 4,068 -> 4,075, whole -- who=1's seven
        // `resource_cap` words ours 2400, theirs 3200 on 969, its republic
        // taken on 968. The game's first parting 969 -> 1055 (units,
        // `cavarch_o`, item 4443's leaf).
        // Moved by item 4443 (the units line folds `cavarch_o` -2): units
        // 1,055 -> 3,662, first 1055 -> 1561. On 1055 (our walk) the
        // fire-on-the-move 1/21 (type 139) is the one unit idle at -2.
        // Moved by item 4447 (an empty building at its ceiling ejects,
        // run1722): builds 4,073 -> 4,075, whole -- the human's city
        // `0/2000`'s `build_masks` on 4019 ours 4144, theirs 20528
        // (`0x4000`, set by `take_damage`'s `eject_contents(1, -1, 0, 1)`
        // with no garrison to read).
        moved: Some(Score {
            states: 4075,
            agreed: [
                3662, 4075, 4075, 4075, 4075, 3863, 3958, 4075, 4066, 4075, 4075, 3984,
            ],
            first: &[
                ("units", (1561, Some(3913862560), 4278570550)),
                ("groups", (3598, Some(3778361523), 962934390)),
                ("guys", (2014, Some(4240840485), 61413159)),
                ("cities", (2253, Some(2126061725), 1479221375)),
                ("world", (3984, Some(402934873), 4267790267)),
            ],
            over: None,
        }),
    }),
    // run1704
    // row 80, Great Lakes at Enlightenment (seed 12345), as found 2026-11-19: the
    // original ended the game itself on 3836 (ours holds no `game_over` there);
    // eight lines whole; units 2,239 of 3,837, first 2239; builds 3,736, first
    // 2376; groups 854, first 854; world 0, first parting on state 0.
    Some(FirstScore {
        last: Some(3836),
        found: Score {
            states: 3837,
            agreed: [
                2239, 3736, 3837, 3837, 3837, 854, 3837, 3837, 3837, 3837, 3837, 0,
            ],
            first: &[
                ("units", (2239, Some(3211665055), 2108038814)),
                ("builds", (2376, Some(2962723024), 945131824)),
                ("groups", (854, Some(3368547131), 1595471662)),
                ("world", (0, Some(3924883285), 3379296129)),
            ],
            over: None,
        },
        // Moved by origin/main at 0b34ad2ac: groups 854 -> 3,837, whole.
        // Moved by item 4436 (run1720's ranges): world 0 -> 3,837, whole --
        // state 0's tile masks, eleven tiles of bit 0x4, the Chinese Town
        // 1/2000's third behind-ring, now marked at its start.
        // Moved by item 4443 (the units line folds `cavarch_o` -2): units
        // 2,239 -> 3,835, first 2239 -> 2367. On 2239, run1720's units range
        // 3003, record 86 (1/86, type 140), `cavarch_o` read ours -1, theirs -2.
        moved: Some(Score {
            states: 3837,
            agreed: [
                3835, 3736, 3837, 3837, 3837, 3837, 3837, 3837, 3837, 3837, 3837, 3837,
            ],
            first: &[
                ("units", (2367, Some(2679787322), 784355304)),
                ("builds", (2376, Some(2962723024), 945131824)),
            ],
            over: None,
        }),
    }),
    // run1705
    // row 100, Great Lakes at Information (seed 12345), as found 2026-11-19: the
    // original ended the game itself on 3330 (ours holds no `game_over` there);
    // five lines whole; units 1,611 of 3,331, first 1611; builds 2,230, first 1976;
    // ammo 3,172, first 2618; deaths 2,756, first 2756; groups 2,400, first 2400;
    // guys 2,895, first 2646; world 134, first 134.
    Some(FirstScore {
        last: Some(3330),
        found: Score {
            states: 3331,
            agreed: [
                1611, 2230, 3331, 3172, 2756, 2400, 2895, 3331, 3331, 3331, 3331, 134,
            ],
            first: &[
                ("units", (1611, Some(1862838836), 884976179)),
                ("builds", (1976, Some(3652536594), 4264774034)),
                ("ammo", (2618, Some(1076236747), 891350609)),
                ("deaths", (2756, Some(1136466598), 3621657254)),
                ("groups", (2400, Some(1692465100), 225246143)),
                ("guys", (2646, Some(1999790736), 3081855676)),
                ("world", (134, Some(3348711314), 341395346)),
            ],
            over: None,
        },
        // Moved by origin/main at 0b34ad2ac: groups 2,400 -> 3,331, whole; guys
        // 2,895 -> 3,316, its first parting unmoved.
        // Moved by item 4441 (run1721's ranges): world 134 -> 2,618 -- state
        // 134's tile masks, `BAD_PATH` on (211, 108) ours and (208, 111)
        // theirs around the computer's University 1/2006, now masked column
        // by column as `BuildType::mask_me` walks it; world parts next on 2618.
        // Moved by item 4443 (the units line folds `cavarch_o` -2): units
        // 1,611 -> 2,696, first 1611 -> 1793. On 1611, run1721's units range
        // 1170, record 31 (1/31, type 170), `cavarch_o` read ours -1, theirs -2.
        moved: Some(Score {
            states: 3331,
            agreed: [
                2696, 2230, 3331, 3172, 2756, 3331, 3316, 3331, 3331, 3331, 3331, 2618,
            ],
            first: &[
                ("units", (1793, Some(4040064246), 1794145779)),
                ("builds", (1976, Some(3652536594), 4264774034)),
                ("ammo", (2618, Some(1076236747), 891350609)),
                ("deaths", (2756, Some(1136466598), 3621657254)),
                ("guys", (2646, Some(1999790736), 3081855676)),
                ("world", (2618, Some(4234786203), 1600898917)),
            ],
            over: None,
        }),
    }),
    // run1706
    // row 120, Islands (seed 12345), as found 2026-11-19 on the tree merged with
    // origin/main at 0b34ad2ac: ten lines whole to 6000; units 4,815 of 6,001,
    // first 2694; guys 5,998, first 3250.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 6001,
            agreed: [
                4815, 6001, 6001, 6001, 6001, 6001, 5998, 6001, 6001, 6001, 6001, 6001,
            ],
            first: &[
                ("units", (2694, Some(1165909129), 10705802)),
                ("guys", (3250, Some(2633326924), 47538376)),
            ],
            over: None,
        },
        // Moved by item 4435: units 4,815 -> 5,861, first 2694 -> 3250. On
        // 2694, run1716's units range 521, the barge 1/15's `form` (+0xaa),
        // read ours -1, theirs 0. Moved by item 4439: units 5,861 -> 5,864,
        // first 3250 -> 5864; guys 5,998 -> 6,001, whole. On 3250, run1716's
        // units range 159, the scout 1/0's figure, `last_x`, `last_y`,
        // `last_z`, `last_angle` read ours 30849, 41470, -37, -1553072128,
        // theirs 19080, 39768, -30, -664928256. Moved by item 4443 (the units
        // line folds `cavarch_o` -2): units 5,864 -> 6,001, whole; the game is
        // whole. On 5864, run1716's units range 1139, record 35 (the MLRS
        // 1/35, type 273), `cavarch_o` read ours -1, theirs -2.
        moved: Some(Score {
            states: 6001,
            agreed: [6001; 12],
            first: &[],
            over: None,
        }),
    }),
    // run1707
    // row 140, East Indies at difficulty 4 (seed 12345), as found 2026-11-19 on the
    // tree at 1e94d6275 (the drain's gate): nine lines whole to 6000; units 1,568
    // of 6,001, first 1568; guys 5,994, first 3081; leaders 6,000, first parting on
    // state 0.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 6001,
            agreed: [
                1568, 6001, 6001, 6001, 6001, 6001, 5994, 6000, 6001, 6001, 6001, 6001,
            ],
            first: &[
                ("units", (1568, Some(3165276990), 354175805)),
                ("guys", (3081, Some(2868511280), 3461022379)),
                ("leaders", (0, Some(3890907082), 2955970510)),
            ],
            over: None,
        },
        // Moved by item 4439: guys 5,994 -> 6,000, first 3081 -> 4026.
        // Moved by item 4443 (the units line folds `cavarch_o` -2): units
        // 1,568 -> 5,999, first 1568 -> 3937. On 1568 (our walk) the
        // fire-on-the-move 1/2 (type 12) is the one unit idle at -2.
        moved: Some(Score {
            states: 6001,
            agreed: [
                5999, 6001, 6001, 6001, 6001, 6001, 6000, 6000, 6001, 6001, 6001, 6001,
            ],
            first: &[
                ("units", (3937, Some(595040496), 264476909)),
                ("guys", (4026, Some(1242658676), 1019574129)),
                ("leaders", (0, Some(3890907082), 2955970510)),
            ],
            over: None,
        }),
    }),
];

/// **Parity held out, N of 8** (DECISIONS 144 §2): the games whole on every
/// line and state, and the games taken.
#[cfg(test)]
pub(crate) const HELD_OUT_LINE: (usize, usize) = (3, 8);
