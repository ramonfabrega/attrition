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

/// **The second battery's rows** (DECISIONS 145 §2, item 4462): the same
/// order, the rows between the first's.
#[cfg(test)]
pub(crate) const HELD_OUT_SECOND_ROWS: [usize; 8] = [10, 30, 50, 70, 90, 110, 130, 150];

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
        rows_are_the_table(&HELD_OUT_ROWS, &HELD_OUT);
    }

    /// **The second declaration** (item 4462's first act; DECISIONS 145
    /// §2): rows 10, 30, …, 150, written out before any take, held the
    /// same way. `RON_HELD_OUT_PRINT=2` prints this table as Rust.
    #[test]
    fn the_second_held_out_eight_are_the_rows_the_rule_names() {
        rows_are_the_table(&HELD_OUT_SECOND_ROWS, &HELD_OUT_SECOND);
    }

    /// `rows` of `lobbies_on_disk` against `table`, column for column;
    /// `RON_HELD_OUT_PRINT` (`1` the first battery, `2` the second) prints
    /// the table instead.
    fn rows_are_the_table(rows: &[usize; 8], table: &[HeldOut; 8]) {
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
        let which = if rows[0] == HELD_OUT_SECOND_ROWS[0] {
            "2"
        } else {
            "1"
        };
        let print = std::env::var("RON_HELD_OUT_PRINT").is_ok_and(|v| v == which);
        for (k, &row) in rows.iter().enumerate() {
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
            let want = &table[k];
            assert_eq!(want.row, row, "the table's row {k}");
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
    fn paths(b: &Battery, k: usize) -> (Option<std::path::PathBuf>, Option<std::path::PathBuf>) {
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let rcx = crate::testenv::experiment(
            &home.join("ron-golden/recordings"),
            &format!("item{}-row{:03}-6000.rcx", b.item, b.table[k].row),
        );
        let run = home.join(format!("ron-golden/run{}", b.run + k));
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
        games_score_as_pinned(&FIRST);
    }

    /// **The second battery's scores** (item 4462, DECISIONS 145 §2):
    /// `HELD_OUT_SECOND[k]` played as `run1730 + k`, held as the first's
    /// are; the found column is the estimate.
    #[test]
    fn the_second_held_out_games_score_as_pinned() {
        games_score_as_pinned(&SECOND);
    }

    fn games_score_as_pinned(b: &Battery) {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let mut wrong: Vec<String> = Vec::new();
        for (k, held) in b.table.iter().enumerate() {
            let (Some(rcx), Some(take)) = paths(b, k) else {
                eprintln!(
                    "skipping: row {} has no file or no run{} take",
                    held.row,
                    b.run + k
                );
                continue;
            };
            let pin = &b.scores[k];
            let until = pin.as_ref().and_then(|p| p.last);
            let score =
                match crate::checksum::playback::compare_until(inst.root(), &rcx, &take, until) {
                    Ok(s) => s,
                    Err(e) => {
                        wrong.push(format!(
                            "row {} run{}: not scored: {e}",
                            held.row,
                            b.run + k
                        ));
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
                b.run + k,
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
                    "row {}: run{} is taken and {}[{k}] is unpinned",
                    held.row,
                    b.run + k,
                    b.scores_name
                ));
                continue;
            };
            let want = pin.moved.unwrap_or(pin.found);
            let got = (score.states, score.agreed, first, score.over);
            if got != (want.states, want.agreed, want.first.to_vec(), want.over) {
                wrong.push(format!(
                    "row {} run{}: states, agreeing, first partings, ours' end: got {got:?}",
                    held.row,
                    b.run + k
                ));
            }
        }
        assert!(
            wrong.is_empty(),
            "the held-out games against their pins:\n{}",
            wrong.join("\n")
        );
    }

    /// **Row 140's Dutch seat starts with both stats bits raised** (item
    /// 4457, `docs/RECGAME.md` §10.3): run1725, the ranges take of run1707's
    /// file, holds setup's set. `Leader::init@006e3930:544` gives the Dutch
    /// (`has_tribe_bonus(0x16)`, `dutch_free_commerce`) their next Commerce
    /// epoch through `gain_tech(…, 1)`, which passes the tech gate
    /// (`gain_tech@006dcb60:935`) and reaches the tail's `leader_flags |=
    /// 0xc000000` (`:2313`). This crate laid the epoch into the tree and put
    /// both stats bits back as they stood. Every leaders range on state 0,
    /// ours against theirs, byte for byte: red on the tree before the fix,
    /// at range 128, record 1, `flags` byte 3, ours `0b`, theirs `0f`
    /// (`unit_stats_dirty`, `0x4000000`).
    #[test]
    fn row_140_s_dutch_seat_starts_with_both_stats_bits_on_run1725_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIRST, 7) else {
            eprintln!("skipping: row 140's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1725/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1725's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 140's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let built = crate::checksum::playback::recording_start(&loaded, &rec);
        assert_eq!(built.sim.frame, 0);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        let set = record.first_of(-1).expect("setup's set").clone();
        let theirs = record.ranges(&set).expect("the set's ranges");
        let theirs: Vec<_> = theirs.iter().filter(|r| r.line == 7).collect();
        let ours = built.sim.checksum_leaders();
        assert_eq!(ours.ranges.len(), theirs.len(), "leaders ranges on state 0");
        for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
            assert_eq!(
                a.bytes, b.bytes,
                "leaders range {k} on state 0, record {} {}",
                a.record, a.field
            );
        }
        // The Dutch seat's `leader_flags`, by name: byte 3 of range 128.
        assert_eq!(theirs[128].bytes[3], 0x0f, "theirs' leader_flags byte 3");
    }

    /// **Row 140's Dutch merchants keep their cached armour for a frame**
    /// (item 4464, `docs/RECGAME.md` §10.3): on 3937 the Dutch seat (who=1)
    /// reaches its first age (`ages` 0 -> 1) inside the objects pass, and
    /// the `gain_tech` tail raises `0x4000000`. `ObjectData::armor@00647db0`
    /// would add `ages × DUTCH_ATTACK_BONUS` for its Armed Merchants (type
    /// 0x3e). But `myarmor` (`UnitData +0x9c`) is a cache that only
    /// `Unit::update_armor@006054c0` writes, and the next frame's
    /// `Leader::process` reaches it through `calc_unit_stats@006cf970`. So
    /// theirs reads 1 on 3937 and 2 on 3938. Every units range on 3937 and
    /// 3938, ours against run1725, byte for byte: red on the tree before
    /// the fix, at range 173 (`1/1`), byte 84, ours 2, theirs 1.
    #[test]
    fn row_140_s_dutch_merchants_keep_their_cached_armour_on_run1725_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIRST, 7) else {
            eprintln!("skipping: row 140's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1725/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1725's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 140's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for (state, want) in [(3937, 1u8), (3938, 2)] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let theirs = record.ranges(&set).expect("the set's ranges");
            let theirs: Vec<_> = theirs.iter().filter(|r| r.line == 0).collect();
            let ours = built.sim.checksum_units();
            assert_eq!(ours.ranges.len(), theirs.len(), "units ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "units range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            // The Armed Merchant `1/1`'s `myarmor`, by name: byte 84 of range 173.
            assert_eq!(
                theirs[173].bytes[0x9c - 0x48],
                want,
                "theirs' myarmor on {state}"
            );
        }
    }

    /// **Row 140's boarder keeps its walk's `last_z` on the frame it
    /// boards** (item 4467, `docs/CHECKSUM.md` §23.9): on 4026 who=1's
    /// `1/0` (`TypeIndex` 69, land) walks its one figure onto its `des`
    /// (29565, 22740) and then boards the carrier, after its own `process`.
    /// `Unit::process@00610bc0` chose its arm on `inside_up` at its head,
    /// so the figure ran `Guy::move`, whose head copied `last_z = z` (−33,
    /// the ground the 4025 step wrote at `last` (29583, 22730)); the inside
    /// arm's `last_z = z` (`:582`) runs from 4027. This crate keyed the
    /// arm on the unit's state at the checksum and wrote the new `z` (−30)
    /// on the boarding frame. Every units and guys range on 4026 and 4027,
    /// ours against theirs, byte for byte: red on the tree before the fix,
    /// at units range 163 and guys range 52, record 0, `last_z` (byte 104),
    /// ours `e2`, theirs `df`.
    #[test]
    fn row_140_s_boarder_keeps_its_walk_s_last_z_on_run1725_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIRST, 7) else {
            eprintln!("skipping: row 140's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1725/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1725's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 140's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for (state, want) in [(4026, -33i8), (4027, -30)] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            for (line, ours) in [
                (0u8, built.sim.checksum_units()),
                (6u8, built.sim.checksum_guys()),
            ] {
                let theirs: Vec<_> = all.iter().filter(|r| r.line == line).collect();
                assert_eq!(
                    ours.ranges.len(),
                    theirs.len(),
                    "line {line}'s ranges on {state}"
                );
                for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                    assert_eq!(
                        a.bytes, b.bytes,
                        "line {line} range {k} on {state}, record {} {}",
                        a.record, a.field
                    );
                }
                if line == 6 {
                    // `1/0`'s figure's `last_z`, by name: byte 104 of guys range 52.
                    assert_eq!(
                        theirs[52].bytes[104] as i8, want,
                        "theirs' last_z on {state}"
                    );
                }
            }
        }
    }

    /// **Row 60's pushed caravan keeps its crew's `des` on guy 0's body**
    /// (item 4460, `docs/RECGAME.md` §10.3): run1722, the ranges take of
    /// run1703's file. On tick 2106 the computer's Caravan `1/40` (type 59)
    /// is pushed (−2, −2) to (37942, 19990) by a collision.
    /// `Unit::set_new_location@005f8d20` writes `des` on the squad alone
    /// (`+0xb5`, one figure), and with `param_3` 0 guy 0's body stays on
    /// (37944, 19992). The two trackless crew figures' `des` is written only by
    /// `Guy::set_new_location@005d86f0`'s crew loop, on guy 0's last point.
    /// This crate reported it as the unit's point. Every units and guys range on
    /// 2107 and 2108, byte for byte: red on the tree before the fix at units
    /// range 1429 and guys range 399, record 40, `des_x` (byte 84), ours
    /// 37942, theirs 37944.
    #[test]
    fn row_60_s_pushed_caravan_keeps_its_crew_s_des_on_run1722_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIRST, 3) else {
            eprintln!("skipping: row 60's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1722/map-7"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1722's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 60's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [2107, 2108] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            for (line, ours) in [
                (0u8, built.sim.checksum_units()),
                (6u8, built.sim.checksum_guys()),
            ] {
                let theirs: Vec<_> = all.iter().filter(|r| r.line == line).collect();
                assert_eq!(
                    ours.ranges.len(),
                    theirs.len(),
                    "line {line}'s ranges on {state}"
                );
                for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                    assert_eq!(
                        a.bytes, b.bytes,
                        "line {line} range {k} on {state}, record {} {}",
                        a.record, a.field
                    );
                }
                if line == 6 && state == 2107 {
                    // `1/40`'s first crew figure's `des_x`, by name: bytes
                    // 84..88 of guys range 399.
                    assert_eq!(
                        &theirs[399].bytes[84..88],
                        &37944i32.to_le_bytes(),
                        "theirs' crew des_x on {state}"
                    );
                }
            }
        }
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
        let (Some(rcx), _) = paths(&FIRST, 6) else {
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
        let (Some(rcx), _) = paths(&FIRST, 6) else {
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
        let (Some(rcx), _) = paths(&FIRST, 6) else {
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

    /// **Row 100's missile walks its strike** (item 4459, `docs/CHECKSUM.md`
    /// §18): on tick 2586 the computer lays an `AIR_ATTACK_GROUND` on the
    /// missile `1/84` (TI 316) in its silo, and
    /// `AirAttackGroundOrder::walk_data@004875a0` hands over five ranges
    /// after the node byte. Every units range on 2587: red without the
    /// walker, 3,804 ranges against 3,809, the first unequal at range 2809,
    /// record 84, where ours walked `guys.length` (`01 00 00 00`) and
    /// theirs the flags byte (`04`).
    #[test]
    fn row_100_s_missile_walks_its_air_attack_ground_order_on_run1721_s_ranges() {
        let Some(theirs) = units_ranges_agree(5, "run1721", 2587, 2812) else {
            return;
        };
        assert_eq!(
            theirs,
            [-1i32, -1, 0x640, 0, 0, 0].map(i32::to_le_bytes).concat(),
            "theirs' AirOrder row"
        );
    }

    /// **Row 100's Caravan crew keep guy 0's point** (item 4471,
    /// `docs/CHECKSUM.md` §18): on tick 2645 the computer's Caravan `1/46`
    /// (TI 59, standing blocked on its trade route) is pushed, and its
    /// point moves from (40382, 12377) to (40382, 12355) on both sides while
    /// guy 0's body stays. Guy 0's `des_y` takes the unit's 12355, but the
    /// two crew figures' `des_y` is what `Guy::set_new_location`'s crew loop
    /// last wrote, guy 0's point 12377. Every guys range on 2646: red
    /// without it at range 435, record 46, `des_y` ours `43 30`, theirs
    /// `59 30`.
    #[test]
    fn row_100_s_caravan_crew_keep_guy_0_s_point_on_run1721_s_ranges() {
        let Some(theirs) = line_ranges_agree(5, "run1721", 2646, 6, 435) else {
            return;
        };
        assert_eq!(
            &theirs[0x60 - 0x8..0x64 - 0x8],
            &12377i32.to_le_bytes(),
            "theirs' crew des_y"
        );
    }

    /// PROBE 4474: record 91 on both sides over 2740..=2744.
    #[test]
    fn probe_4474_record_91() {
        if std::env::var("RON_PROBE_4474").is_err() {
            return;
        }
        let Some(inst) = crate::testenv::install() else { return };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIRST, 5) else { return };
        let ranges = crate::testenv::experiment(&home.join("ron-golden/run1721/map-14"), "rontrace.ranges").unwrap();
        let rec = crate::recgame::read(&rcx.to_string_lossy()).unwrap();
        let loaded = crate::load::load(&inst).unwrap();
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).unwrap();
        let rno: i16 = std::env::var("RON_PROBE_REC").ok().and_then(|v| v.parse().ok()).unwrap_or(91);
        let lo: i64 = std::env::var("RON_PROBE_LO").ok().and_then(|v| v.parse().ok()).unwrap_or(2738);
        let hi: i64 = std::env::var("RON_PROBE_HI").ok().and_then(|v| v.parse().ok()).unwrap_or(2744);
        while built.sim.frame <= hi {
            let state = built.sim.frame;
            if state >= lo {
                for u in built.sim.units.iter().filter(|u| u.alive() && i64::from(u.index) == i64::from(rno)) {
                    crate::diff::harness::debug_unit(&built, u, state);
                }
                let set = record.first_of(state - 1).unwrap().clone();
                let theirs = record.ranges(&set).unwrap();
                for line in [0u8, 6] {
                    let t: Vec<_> = theirs.iter().filter(|r| r.line == line).collect();
                    let ours = if line == 0 { built.sim.checksum_units() } else { built.sim.checksum_guys() };
                    let idx: Vec<usize> = ours.ranges.iter().enumerate().filter(|(_, r)| r.record == rno).map(|(i, _)| i).collect();
                    let (Some(&a), Some(&b)) = (idx.first(), idx.last()) else {
                        eprintln!("s{state} line {line}: ours holds no record {rno} ({} ranges, theirs {})", ours.ranges.len(), t.len());
                        continue;
                    };
                    eprintln!("s{state} line {line}: ours {} ranges, theirs {}; record {rno} at {a}..={b}", ours.ranges.len(), t.len());
                    for i in a.saturating_sub(1)..=(b + 3) {
                        let o = ours.ranges.get(i);
                        let th = t.get(i);
                        let ob = o.map(|r| r.bytes.iter().map(|x| format!("{x:02x}")).collect::<String>()).unwrap_or_default();
                        let tb = th.map(|r| r.bytes.iter().map(|x| format!("{x:02x}")).collect::<String>()).unwrap_or_default();
                        let mark = if ob == tb { " " } else { "*" };
                        eprintln!("  {mark}{i} r{} {}\n     ours   {ob}\n     theirs {tb}", o.map_or(-1, |r| r.record), o.map_or("", |r| r.field));
                    }
                }
            }
            stream.apply(built.sim.frame as i32, &mut built);
            built.sim.command().tick();
        }
    }

    /// Game `k`'s units line on `state`, ours against the ranges take
    /// `run`'s, every range byte for byte; returns theirs' range `range`.
    fn units_ranges_agree(k: usize, run: &str, state: i64, range: usize) -> Option<Vec<u8>> {
        line_ranges_agree(k, run, state, 0, range)
    }

    /// The same for line `line` (0 units, 6 guys).
    fn line_ranges_agree(
        k: usize,
        run: &str,
        state: i64,
        line: u8,
        range: usize,
    ) -> Option<Vec<u8>> {
        let inst = crate::testenv::install()?;
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIRST, k) else {
            eprintln!("skipping: game {k}'s recording is missing");
            return None;
        };
        let Some(ranges) = crate::testenv::experiment(
            &home.join(format!("ron-golden/{run}/map-14")),
            "rontrace.ranges",
        ) else {
            eprintln!("skipping: {run}'s ranges record is missing");
            return None;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("the game's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        while built.sim.frame < state {
            stream.apply(built.sim.frame as i32, &mut built);
            built.sim.command().tick();
        }
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        let set = record.first_of(state - 1).expect("the state's set").clone();
        let theirs = record.ranges(&set).expect("the set's ranges");
        let theirs: Vec<_> = theirs.into_iter().filter(|r| r.line == line).collect();
        let ours = match line {
            0 => built.sim.checksum_units(),
            6 => built.sim.checksum_guys(),
            _ => unreachable!("line {line}"),
        };
        assert_eq!(
            ours.ranges.len(),
            theirs.len(),
            "line {line}'s ranges on {state}"
        );
        for (i, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
            assert_eq!(
                a.bytes, b.bytes,
                "line {line} range {i} on {state}, record {} {}",
                a.record, a.field
            );
        }
        Some(theirs[range].bytes.clone())
    }

    /// **A bumped guard's `dest_angle`** (item 4448, `docs/RECGAME.md`
    /// §10.3): on tick 2366 the computer's Fusiliers `1/73` (type 136)
    /// turns at its guard spot (`Unit::do_guard`'s `set_angle`) after
    /// `Unit::work`'s `update_action` has already copied the old heading,
    /// and a unit that walks into it runs `detect_unit_collision`'s `0xc`
    /// arm, whose `update_action(blocker)` copies the new one. Every
    /// units range on 2367: red without that call at range 2407, record
    /// 73, byte 18, ours `15 a7`, theirs `b5 b5`.
    #[test]
    fn row_80_s_fusiliers_take_the_guard_turn_s_dest_angle_on_run1720_s_ranges() {
        let Some(theirs) = units_ranges_agree(4, "run1720", 2367, 2407) else {
            return;
        };
        assert_eq!(
            &theirs[0x5a - 0x48..0x5c - 0x48],
            &[0xb5, 0xb5],
            "theirs' dest_angle"
        );
    }

    /// The same call on row 100 (item 4448): run1721's Assault Infantry
    /// `1/43` (type 104) turns at its guard spot on tick 1792. Red without
    /// it at range 1581, record 43, byte 18, ours `00 c0`, theirs `f2 cb`.
    #[test]
    fn row_100_s_assault_infantry_take_the_guard_turn_s_dest_angle_on_run1721_s_ranges() {
        let Some(theirs) = units_ranges_agree(5, "run1721", 1793, 1581) else {
            return;
        };
        assert_eq!(
            &theirs[0x5a - 0x48..0x5c - 0x48],
            &[0xf2, 0xcb],
            "theirs' dest_angle"
        );
    }

    /// **The held-out line**, from the table alone: the games whose score
    /// agrees on every line on every state, of the eight, and how many are
    /// taken.
    #[test]
    fn the_held_out_line_reads_as_pinned() {
        line_reads_as_pinned(&FIRST);
    }

    /// The second battery's line, from its table alone (item 4462).
    #[test]
    fn the_second_held_out_line_reads_as_pinned() {
        line_reads_as_pinned(&SECOND);
    }

    fn line_reads_as_pinned(b: &Battery) {
        let taken = b.scores.iter().flatten().count();
        let whole = b
            .scores
            .iter()
            .flatten()
            .filter(|p| {
                let s = p.moved.unwrap_or(p.found);
                s.first.is_empty() && s.agreed == [s.states; 12]
            })
            .count();
        assert_eq!((whole, taken), b.line, "{}: N of 8, T taken", b.label);
    }

    /// The queue's line for [`SCORES`]: the games whole of the eight, the
    /// games taken, the lines whole of twelve a game as found and now
    /// ([`FirstScore::moved`], else as found), and each game's lowest first
    /// parting in row order, as found and now (`whole` for a game with
    /// none, `-` for one not taken).
    fn queue_line(b: &Battery) -> String {
        let lines = |s: &Score| s.agreed.iter().filter(|&&a| a == s.states).count();
        let lowest = |s: &Score| {
            s.first
                .iter()
                .map(|f| f.1.0)
                .min()
                .map_or("whole".to_string(), |v| v.to_string())
        };
        let col = |now: bool| -> (usize, String) {
            let scores = b.scores.iter().map(|p| {
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
            "{}: {} of {} whole \u{b7} {} taken \u{b7} lines {found} of {} as found, \
             {now} now \u{b7} first partings {found_firsts} as found, {now_firsts} now",
            b.label,
            b.line.0,
            b.scores.len(),
            b.line.1,
            12 * b.line.1,
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
        handoff_line_is_the_pinned_scores(&FIRST);
    }

    /// **The handoff's second parity-held-out line** (item 4462): the
    /// second battery's [`queue_line`], `Parity held out, second:`, in the
    /// first's grammar. Its found half is the estimate (DECISIONS 145 §2).
    #[test]
    fn the_handoff_s_parity_held_out_second_line_is_the_pinned_scores() {
        handoff_line_is_the_pinned_scores(&SECOND);
    }

    fn handoff_line_is_the_pinned_scores(b: &Battery) {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/QUEUE.md");
        let q = std::fs::read_to_string(path).expect("docs/QUEUE.md");
        let want = queue_line(b);
        let head = format!("{}:", b.label);
        let line = q.lines().find(|l| l.starts_with(&head));
        assert_eq!(
            line.map(str::trim_end),
            Some(want.as_str()),
            "docs/QUEUE.md's handoff carries `{want}` (`diff::held_out::{}`), beside the \
             Held-out line",
            b.scores_name
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

/// **The second eight, as declared** (item 4462, before any take; DECISIONS
/// 145 §2): rows 10, 30, …, 150 of the same order, each the human (tribe 4,
/// slot 0) against the computers its lobby seats, `MAP_SIZE 2`, played as
/// the lobby stands (adding seats or a team is Ramon's):
///
/// | row | first capture | `MAP_STYLE` | seed | computer tribes | `DIFFICULTY` | ages |
/// |---|---|---|---|---|---|---|
/// | 10 | run1016 Lakota peace | 18 | 12345 | 19 | 3 | 8..7, `RUSH_RULES 10` |
/// | 30 | run1152 East Indies | 18 | 12345 | 15 | 5 | 5..5 |
/// | 50 | run1253 Great Sahara | 7 | 12345 | 7 | 5 | 3..3 |
/// | 70 | run1328 Great Lakes | 14 | 12345 | 15 | 5 | 2..2 |
/// | 90 | run139 East Indies | 18 | 12345 | 11 | 0 | 0..7 |
/// | 110 | run1482 East Indies melee | 18 | 12345 | 10, 13 | 0 | 7..7 |
/// | 130 | run651 East Indies all-tech | 18 | 12345 | 23 | 5 | 8..7 |
/// | 150 | run977 Great Sahara | 7 | 12345 | 10 | 4 | 0..7 |
///
/// Row 110 seats three players (`PLAYERS 2`), the one row of the sixteen
/// that does. Each is played to 6,000 frames with the human idle, as
/// `run1730 + k` for `HELD_OUT_SECOND[k]`.
#[cfg(test)]
pub(crate) const HELD_OUT_SECOND: [HeldOut; 8] = [
    HeldOut {
        row: 10,
        capture: "gamelog-run1016-lakota-peace-cover.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 18; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 3; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 10; CANNON_TIMES 0; STARTING_TECHNOLOGY 8; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 19; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Sitting Bull",
        ],
        digest: 0x2dc0d92836db2607,
    },
    HeldOut {
        row: 30,
        capture: "gamelog-run1152-eastindies-japanese-industrial-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 18; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 5; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 5; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 15; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Shogun Takauji Ashikaga",
        ],
        digest: 0x4c9996d38f35b638,
    },
    HeldOut {
        row: 50,
        capture: "gamelog-run1253-greatsahara-egyptian-gunpowder-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 7; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 3; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 3; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 7; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Pharaoh Ramesses II",
        ],
        digest: 0xf96cd391ec4ba84f,
    },
    HeldOut {
        row: 70,
        capture: "gamelog-run1328-greatlakes-japanese-medieval-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 14; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 2; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 2; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 15; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Shogun Takauji Ashikaga",
        ],
        digest: 0xcfc0c20cc95e7c1a,
    },
    HeldOut {
        row: 90,
        capture: "gamelog-run139-eastindies-makelist.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 18; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 0; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 11; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; William the Conqueror",
        ],
        digest: 0x9391b199dcc55b5e,
    },
    HeldOut {
        row: 110,
        capture: "gamelog-run1482-eastindies-information-melee-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 18; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 0; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 7; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 10; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Emperor Napoleon",
            "flags 1; tribe 13; who 2; team 8; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; Tsar Ivan the Terrible",
        ],
        digest: 0xfbf3f6e14ffc47b6,
    },
    HeldOut {
        row: 130,
        capture: "gamelog-run651-eastindies-persian-alltech-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 18; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 8; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 23; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Cyrus the Great",
        ],
        digest: 0x4e3334b9feb6c070,
    },
    HeldOut {
        row: 150,
        capture: "gamelog-run977-greatsahara-french-diff4-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 7; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 4; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 10; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Emperor Napoleon",
        ],
        digest: 0x04c04c23cd57f7c9,
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
        // Moved by item 4449 (`set_new_location(…, 1)` copies each figure's
        // point into `last`): guys 4,876 -> 4,870, a named fall past its
        // first parting, which stays on 2487.
        // Moved by item 4450 (`City::calc_gather`'s bytes kept to the
        // reassembly, a finished granary's written at `Build::activate`):
        // cities 4,967 -> 4,974, whole.
        // Moved by item 4453 (a push after the pushed unit's walk leaves
        // `guy_flags & 2`): guys 4,870 -> 4,949, its first parting on 2487.
        // Moved by item 4467 (the guys line's inside arm is chosen at the
        // unit's own `process`): guys 4,949 -> 4,961, first 2487 -> 3107;
        // units stands on 2487, its sum there moved.
        // Moved by item 4460 (a trackless crew figure's `des` is guy 0's body
        // point, `Guy::set_new_location`'s crew loop): guys
        // 4,961 -> 4,965, first 3107 -> 3190.
        moved: Some(Score {
            states: 4974,
            agreed: [
                2487, 4974, 4974, 4974, 4974, 4964, 4965, 4974, 4974, 4974, 4974, 4859,
            ],
            first: &[
                ("units", (2487, Some(3429527498), 3877923716)),
                ("groups", (1854, Some(284774603), 2949074744)),
                ("guys", (3190, Some(2877806205), 2189546945)),
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
        // Moved by item 4448 (`detect_unit_collision`'s guard arm runs
        // `update_action` on the blocker): units 3,662 -> 3,672, first
        // 1561 -> 2014 -- a bumped guard's `dest_angle` takes its turn.
        // Moved by item 4450 (run1722): cities 4,066 -> 4,075, whole --
        // who=1's city 0 `granary` ours 50, theirs 20 on 2253
        // (`City::calc_gather`'s byte kept to the reassembly on 2256), and
        // 20 on 1713 (`Build::activate` writes it at once).
        // Moved by item 4453 (a push after the pushed unit's walk leaves
        // `guy_flags & 2`; 4450a's gaia animal `8/2` on 2014, 16 ours, 18
        // theirs): units 3,662 -> 3,714; guys 3,958 -> 4,010, first 2014 -> 2107.
        // On item 4448's tree, beside its guard arm: units 3,672 -> 3,724, first
        // 2014 -> 2107.
        // Moved by item 4460 (a trackless crew figure's `des` is guy 0's body
        // point, `Guy::set_new_location`'s crew loop): units
        // 3,724 -> 3,788, first 2107 -> 3757; guys 4,010 -> 4,075, whole. On
        // 2107, run1722's units range 1429 and guys range 399, record 40 (the
        // Caravan 1/40, TI 59, pushed (-2, -2) on 2106), its first crew
        // figure's `des_x` read ours 37942, theirs 37944 (`des_y` 19990, 19992).
        moved: Some(Score {
            states: 4075,
            agreed: [
                3788, 4075, 4075, 4075, 4075, 3863, 4075, 4075, 4075, 4075, 4075, 3984,
            ],
            first: &[
                ("units", (3757, Some(89157210), 2279304812)),
                ("groups", (3598, Some(3778361523), 962934390)),
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
        // Moved by item 4448 (`detect_unit_collision`'s guard arm runs
        // `update_action` on the blocker): units 3,835 -> 3,837, whole. On
        // 2367, run1720's units range 2407, record 73 (the Fusiliers 1/73,
        // type 136), `dest_angle` read ours 0xa715.., theirs 0xb5b5...
        // Moved by item 4455 (`check_orphaned_buildings` marks a site with
        // no builder and no recruit, `build_masks |= 0x2000`): builds 3,736
        // -> 3,837, whole, and the game with it. On 2376, run1720's builds
        // range 1883, record 2036 (the computer's site 1/2036, never
        // started), `build_masks` read ours 0, theirs 0x2000.
        moved: Some(Score {
            states: 3837,
            agreed: [
                3837, 3837, 3837, 3837, 3837, 3837, 3837, 3837, 3837, 3837, 3837, 3837,
            ],
            first: &[],
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
        // Moved by item 4448 (`detect_unit_collision`'s guard arm runs
        // `update_action` on the blocker): units 2,696 -> 2,697, first
        // 1793 -> 2587. On 1793, run1721's units range 1581, record 43 (the
        // Assault Infantry 1/43, type 104), `dest_angle` read ours
        // 0xc000.., theirs 0xcbf2...
        // Moved by item 4449 (`set_new_location(…, 1)` copies each figure's
        // point into `last`): guys 3,316 -> 3,308, a named fall past its
        // first parting, which stays on 2646.
        // Moved by item 4455 (`check_orphaned_buildings` marks a site with
        // no builder and no recruit, `build_masks |= 0x2000`): builds 2,230
        // -> 2,588, first 1976 -> 2588. On 1976, run1721's builds range
        // 1451, record 2027 (the computer's site 1/2027, never started),
        // `build_masks` read ours 0, theirs 0x2000.
        // Moved by item 4459 (the units line walks `AirAttackGroundOrder`'s
        // five ranges): units 2,697 -> 2,728, first 2587 -> 2646. On 2587,
        // run1721's units range 2809, record 84 (the missile 1/84, TI 316,
        // laid in its silo): ours walked `guys.length` 01 00 00 00 after the
        // order's node byte, theirs the order's flags byte 04.
        // Moved by item 4465 (a hangar's `launching` walks its pointer from
        // the first launch and its `SimpleArray<int>`; a nuke's strike
        // writes the silo's own `visible` 0xff and relights its fog):
        // builds 2,588 -> 3,331, whole, and world 2,618 -> 3,331, whole. On
        // 2588, run1721's builds range 1270, record 2022 (the silo 1/2022):
        // ours walked the next `must_walk` 01, theirs `launching.length`
        // 01 00 00 00, then size 4, increment -1, flags 0 and [84]. On 2618,
        // range 1268, record 2022, `visible` read ours 0, theirs 0xff.
        // Moved by item 4468 (the nuke strike's round keeps
        // `calc_nuke_spline`'s spline and the ammo line walks its
        // `SplineData`): ammo 3,172 -> 3,291, first 2618 -> 3210. On 2618,
        // run1721's ammo range 1, record 0, byte 75, `traj` read ours 1,
        // theirs 2; theirs walks `has_path` 01 and the spline's 278 ranges
        // after it (header, 12 control points, 17 weights, 16 knots, no
        // weights, 121 points and 121 normals, `total_spline_length`
        // 0x4765a7a2), ours none. Ammo parts next on 3210, ours refused.
        // Moved by item 4460 (a trackless crew figure's `des` is guy 0's body
        // point, `Guy::set_new_location`'s crew loop): units
        // 2,728 -> 2,743, first 2646 -> 2743; guys 3,308 -> 3,323, first 2646
        // -> 2744.
        moved: Some(Score {
            states: 3331,
            agreed: [
                2743, 3331, 3331, 3291, 2756, 3331, 3323, 3331, 3331, 3331, 3331, 3331,
            ],
            first: &[
                ("units", (2743, Some(4227177816), 316449748)),
                ("ammo", (3210, None, 252233938)),
                ("deaths", (2756, Some(1136466598), 3621657254)),
                ("guys", (2744, Some(3271434361), 1542266661)),
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
        // Moved by item 4457 (the Dutch free Commerce's `gain_tech` tail):
        // leaders 6,000 -> 6,001, whole. run1725's setup set, range 128,
        // record 1, `leader_flags` byte 3: ours 0x0b, theirs 0x0f.
        // Moved by item 4464 (`myarmor` is `Unit::update_armor`'s cache):
        // units 5,999 -> 6,000, first 3937 -> 4026. run1725's 3937, range
        // 173, record 1 (who=1's Armed Merchant), `myarmor` ours 2, theirs 1.
        // Moved by item 4467 (the guys line's inside arm is chosen at the
        // unit's own `process`): units and guys 6,000 -> 6,001, whole; the
        // row is whole. run1725's 4026, units range 163 and guys range 52,
        // record 0 (who=1's `1/0`, TI 69, boarding), `last_z` ours -30,
        // theirs -33.
        moved: Some(Score {
            states: 6001,
            agreed: [
                6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001,
            ],
            first: &[],
            over: None,
        }),
    }),
];

/// **Parity held out, N of 8** (DECISIONS 144 §2): the games whole on every
/// line and state, and the games taken. Item 4455: 3 -> 4, row 80 whole
/// (`build_masks & 0x2000` on its site 1/2036). Item 4467: 4 -> 5, row 140
/// whole (a boarder's `last_z` on 4026).
#[cfg(test)]
pub(crate) const HELD_OUT_LINE: (usize, usize) = (5, 8);

/// **The second battery's scores**, `HELD_OUT_SECOND[k]` played as
/// `run1730 + k` (item 4462): `None` until the original has played it. The
/// found column is pinned once and never re-pinned; it is the estimate
/// (DECISIONS 145 §2).
#[cfg(test)]
pub(crate) const SCORES_SECOND: [Option<FirstScore>; 8] = [
    // run1730
    // row 10, as found 2026-11-20: 4 lines whole; units 2,496 of 6,001, first 1747; builds 537
    // of 6,001, first 537; ammo 5,591 of 6,001, first 4542; deaths 5,350 of 6,001, first 4686;
    // groups 3,066 of 6,001, first 3066; guys 5,932 of 6,001, first 1747; leaders 3,976 of
    // 6,001, first 1576; world 537 of 6,001, first 537.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 6001,
            agreed: [
                2496, 537, 6001, 5591, 5350, 3066, 5932, 3976, 6001, 6001, 6001, 537,
            ],
            first: &[
                ("units", (1747, Some(1368369504), 1685366186)),
                ("builds", (537, Some(1374294361), 1810502041)),
                ("ammo", (4542, Some(2344817575), 281265979)),
                ("deaths", (4686, Some(4192616794), 999637338)),
                ("groups", (3066, Some(2106809472), 4177681537)),
                ("guys", (1747, Some(978400498), 3434034420)),
                ("leaders", (1576, Some(2345254394), 1305656828)),
                ("world", (537, Some(993370751), 2782374025)),
            ],
            over: None,
        },
        // Moved by item 4467 (the guys line's inside arm is chosen at the
        // unit's own `process`): guys 5,932 -> 5,942, first 1747 -> 2449;
        // units stands on 1747, its sum there moved.
        // Moved by item 4468 (a nuke strike's round keeps
        // `calc_nuke_spline`'s spline and the ammo line walks its
        // `SplineData`): ammo 5,591 -> 5,907, first 4542 -> 5135, ours
        // refused there. No ranges are on disk for run1730: on 4542 ours'
        // ammo sum was 2344817575 and is now theirs' 281265979.
        // Moved by item 4460 (a trackless crew figure's `des` is guy 0's body
        // point, `Guy::set_new_location`'s crew loop): guys
        // 5,942 -> 5,963, first on 2449.
        moved: Some(Score {
            states: 6001,
            agreed: [
                2496, 537, 6001, 5907, 5350, 3066, 5963, 3976, 6001, 6001, 6001, 537,
            ],
            first: &[
                ("units", (1747, Some(2268899682), 1685366186)),
                ("builds", (537, Some(1374294361), 1810502041)),
                ("ammo", (5135, None, 990220286)),
                ("deaths", (4686, Some(4192616794), 999637338)),
                ("groups", (3066, Some(2106809472), 4177681537)),
                ("guys", (2449, Some(2995814966), 2693759490)),
                ("leaders", (1576, Some(2345254394), 1305656828)),
                ("world", (537, Some(993370751), 2782374025)),
            ],
            over: None,
        }),
    }),
    // run1731
    // row 30, as found 2026-11-20, ended by the original on 4547: 8 lines whole; units 2,334 of
    // 4,548, first 2334; guys 4,519 of 4,548, first 2334; leaders 3,748 of 4,548, first 1776;
    // world 4,423 of 4,548, first 1662.
    Some(FirstScore {
        last: Some(4547),
        found: Score {
            states: 4548,
            agreed: [
                2334, 4548, 4548, 4548, 4548, 4548, 4519, 3748, 4548, 4548, 4548, 4423,
            ],
            first: &[
                ("units", (2334, Some(1174910604), 994488935)),
                ("guys", (2334, Some(700966203), 3856065750)),
                ("leaders", (1776, Some(740584264), 2366335817)),
                ("world", (1662, Some(3453374911), 2282901965)),
            ],
            over: None,
        },
        // Moved by item 4467 (the guys line's inside arm is chosen at the
        // unit's own `process`): guys 4,519 -> 4,525; units and guys stand
        // on 2334, their sums there moved.
        // Moved by item 4460 (a trackless crew figure's `des` is guy 0's body
        // point, `Guy::set_new_location`'s crew loop): guys
        // 4,525 -> 4,542, first on 2334.
        moved: Some(Score {
            states: 4548,
            agreed: [
                2334, 4548, 4548, 4548, 4548, 4548, 4542, 3748, 4548, 4548, 4548, 4423,
            ],
            first: &[
                ("units", (2334, Some(2937566862), 994488935)),
                ("guys", (2334, Some(4256556349), 3856065750)),
                ("leaders", (1776, Some(740584264), 2366335817)),
                ("world", (1662, Some(3453374911), 2282901965)),
            ],
            over: None,
        }),
    }),
    // run1732
    // row 50, as found 2026-11-20, ended by the original on 4454: 11 lines whole; groups 4,415
    // of 4,455, first 3352.
    Some(FirstScore {
        last: Some(4454),
        found: Score {
            states: 4455,
            agreed: [
                4455, 4455, 4455, 4455, 4455, 4415, 4455, 4455, 4455, 4455, 4455, 4455,
            ],
            first: &[("groups", (3352, Some(4005438802), 229385035))],
            over: None,
        },
        moved: None,
    }),
    // run1733
    // row 70, as found 2026-11-20, ended by the original on 5140: 11 lines whole; world 5,054
    // of 5,141, first 5054.
    Some(FirstScore {
        last: Some(5140),
        found: Score {
            states: 5141,
            agreed: [
                5141, 5141, 5141, 5141, 5141, 5141, 5141, 5141, 5141, 5141, 5141, 5054,
            ],
            first: &[("world", (5054, Some(2368706929), 2281807025))],
            over: None,
        },
        moved: None,
    }),
    // run1734
    // row 90, as found 2026-11-20: 10 lines whole; units 6,000 of 6,001, first 4021; guys 6,000
    // of 6,001, first 4021.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 6001,
            agreed: [
                6000, 6001, 6001, 6001, 6001, 6001, 6000, 6001, 6001, 6001, 6001, 6001,
            ],
            first: &[
                ("units", (4021, Some(1032385785), 3868390707)),
                ("guys", (4021, Some(1818783002), 2641849684)),
            ],
            over: None,
        },
        moved: None,
    }),
    // run1735
    // row 110, as found 2026-11-20: 5 lines whole; units 2,702 of 6,001, first 2184; builds
    // 4,221 of 6,001, first 4221; ammo 4,472 of 6,001, first 3303; groups 3,544 of 6,001, first
    // 1608; guys 5,977 of 6,001, first 2464; leaders 4,995 of 6,001, first 3176; world 3,999 of
    // 6,001, first 3809.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 6001,
            agreed: [
                2702, 4221, 6001, 4472, 6001, 3544, 5977, 4995, 6001, 6001, 6001, 3999,
            ],
            first: &[
                ("units", (2184, Some(1779962534), 330632884)),
                ("builds", (4221, Some(3943548441), 1202439769)),
                ("ammo", (3303, None, 3094479036)),
                ("groups", (1608, Some(2931878699), 617474571)),
                ("guys", (2464, Some(451796102), 1018289160)),
                ("leaders", (3176, Some(2352774769), 1606385266)),
                ("world", (3809, Some(2182316411), 2894561759)),
            ],
            over: None,
        },
        // Moved by item 4460 (a trackless crew figure's `des` is guy 0's body
        // point, `Guy::set_new_location`'s crew loop): guys
        // 5,977 -> 6,000, first on 2464.
        moved: Some(Score {
            states: 6001,
            agreed: [
                2702, 4221, 6001, 4472, 6001, 3544, 6000, 4995, 6001, 6001, 6001, 3999,
            ],
            first: &[
                ("units", (2184, Some(1779962534), 330632884)),
                ("builds", (4221, Some(3943548441), 1202439769)),
                ("ammo", (3303, None, 3094479036)),
                ("groups", (1608, Some(2931878699), 617474571)),
                ("guys", (2464, Some(451796102), 1018289160)),
                ("leaders", (3176, Some(2352774769), 1606385266)),
                ("world", (3809, Some(2182316411), 2894561759)),
            ],
            over: None,
        }),
    }),
    // run1736
    // row 130, as found 2026-11-20, ended by the original on 4730: 4 lines whole; units 1,704
    // of 4,731, first 1570; builds 553 of 4,731, first 553; ammo 4,392 of 4,731, first 1601;
    // deaths 4,084 of 4,731, first 1746; groups 4,553 of 4,731, first 2811; guys 4,566 of
    // 4,731, first 1736; leaders 1,976 of 4,731, first 1376; world 553 of 4,731, first 553.
    Some(FirstScore {
        last: Some(4730),
        found: Score {
            states: 4731,
            agreed: [
                1704, 553, 4731, 4392, 4084, 4553, 4566, 1976, 4731, 4731, 4731, 553,
            ],
            first: &[
                ("units", (1570, Some(834499912), 2753003137)),
                ("builds", (553, Some(2297078769), 1047897137)),
                ("ammo", (1601, Some(2349929217), 941472936)),
                ("deaths", (1746, Some(1419733817), 1134652217)),
                ("groups", (2811, Some(2634708853), 2723050627)),
                ("guys", (1736, Some(4103418160), 1070804956)),
                ("leaders", (1376, Some(73709351), 1700509480)),
                ("world", (553, Some(699267042), 2987457272)),
            ],
            over: None,
        },
        // Moved by the merge of 4459 (`AirAttackGroundOrder`'s five ranges
        // walked), measured on the merged tree before this item's gate:
        // units 1,704 -> 1,735, first 1570 -> 1735.
        // Moved by item 4467 (the guys line's inside arm is chosen at the
        // unit's own `process`): guys 4,566 -> 4,591, first on 1736.
        // Moved by item 4468 (a nuke strike's round keeps its spline and
        // the ammo line walks its `SplineData`): ammo 4,392 -> 4,511, first
        // 1601 -> 2367. No ranges are on disk for run1736: on 1601 ours'
        // ammo sum was 2349929217 and is now theirs' 941472936.
        // Moved by item 4460 (a trackless crew figure's `des` is guy 0's body
        // point, `Guy::set_new_location`'s crew loop): guys
        // 4,591 -> 4,695, first on 1736.
        moved: Some(Score {
            states: 4731,
            agreed: [
                1735, 553, 4731, 4511, 4084, 4553, 4695, 1976, 4731, 4731, 4731, 553,
            ],
            first: &[
                ("units", (1735, Some(3081680851), 796971071)),
                ("builds", (553, Some(2297078769), 1047897137)),
                ("ammo", (2367, Some(1123882197), 1189418202)),
                ("deaths", (1746, Some(1419733817), 1134652217)),
                ("groups", (2811, Some(2634708853), 2723050627)),
                ("guys", (1736, Some(4103418160), 1070804956)),
                ("leaders", (1376, Some(73709351), 1700509480)),
                ("world", (553, Some(699267042), 2987457272)),
            ],
            over: None,
        }),
    }),
    // run1737
    // row 150, as found 2026-11-20: 12 lines whole.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 6001,
            agreed: [
                6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001,
            ],
            first: &[],
            over: None,
        },
        moved: None,
    }),
];

/// **Parity held out, second, N of 8** (item 4462): the second battery's
/// games whole on every line and state, and the games taken. As found,
/// 2026-11-20: 1 of 8, row 150 (run1737).
#[cfg(test)]
pub(crate) const HELD_OUT_SECOND_LINE: (usize, usize) = (1, 8);

/// One battery: its declared table, its pins, where its files and takes
/// sit, and its line in the handoff.
#[cfg(test)]
pub(crate) struct Battery {
    /// The handoff line's head, before its colon.
    pub label: &'static str,
    /// The item whose recordings these are (`item<N>-row<RRR>-6000.rcx`).
    pub item: u32,
    /// The run number of game 0; game `k` is `run + k`.
    pub run: usize,
    pub table: &'static [HeldOut; 8],
    pub scores: &'static [Option<FirstScore>; 8],
    /// The pins' name, for a failure's message.
    pub scores_name: &'static str,
    pub line: (usize, usize),
}

/// The first battery (item 4413).
#[cfg(test)]
pub(crate) const FIRST: Battery = Battery {
    label: "Parity held out",
    item: 4413,
    run: 1700,
    table: &HELD_OUT,
    scores: &SCORES,
    scores_name: "SCORES",
    line: HELD_OUT_LINE,
};

/// The second battery (item 4462).
#[cfg(test)]
pub(crate) const SECOND: Battery = Battery {
    label: "Parity held out, second",
    item: 4462,
    run: 1730,
    table: &HELD_OUT_SECOND,
    scores: &SCORES_SECOND,
    scores_name: "SCORES_SECOND",
    line: HELD_OUT_SECOND_LINE,
};
