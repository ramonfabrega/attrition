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

/// **The third battery's rows** (DECISIONS 147 §2, item 4487): the same
/// order, the rows halfway between the first's and the second's.
#[cfg(test)]
pub(crate) const HELD_OUT_THIRD_ROWS: [usize; 8] = [5, 25, 45, 65, 85, 105, 125, 145];

/// **The fourth battery's rows** (DECISIONS 148 §2, item 4510): the same
/// order, the rows halfway between the second's and the third's.
#[cfg(test)]
pub(crate) const HELD_OUT_FOURTH_ROWS: [usize; 8] = [15, 35, 55, 75, 95, 115, 135, 155];

/// **The fifth battery's rows** (DECISIONS 149 §3, 150 §2, item 4527): the
/// same order, the rows three past the first's.
#[cfg(test)]
pub(crate) const HELD_OUT_FIFTH_ROWS: [usize; 8] = [3, 23, 43, 63, 83, 103, 123, 143];

/// **What the fifth adds to its row's lobby** (DECISIONS 150 §2): one more
/// seat and one team of two, made on the client's page after the lobby's
/// own events (`lobby::events_for`) as a person makes them: a seat added
/// (the computer), its nation, then the team on each of two seats. Every
/// row is `MAP_SIZE 2`, which `rules.xml`'s `mapsizes` names "Small
/// (2/3-player)", so three seats is the most the row's own map holds; a
/// fourth would move `MAP_SIZE` and leave the lobby. Teams `0..4` ally
/// (`LeaderData::is_team`, `relations.rs`), and the row's `TEAM_STYLE 1`
/// locks them.
#[cfg(test)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Seated {
    /// The added computer's nation (`PLAYER tribe`); its name is the
    /// nation's first leader, as `Event::AddSeat` and `Event::Nation` give.
    pub tribe: u8,
    /// The two seats on the team, by place.
    pub mates: [usize; 2],
    /// The team both take.
    pub team: i32,
}

#[cfg(test)]
impl Seated {
    /// The page's events, after a lobby of `seats` seats.
    pub(crate) fn events(&self, seats: usize) -> Vec<crate::lobby::Event> {
        use crate::lobby::Event;
        let mut e = vec![
            Event::AddSeat,
            Event::Nation {
                seat: seats,
                tribe: self.tribe,
            },
        ];
        for &seat in &self.mates {
            e.push(Event::Team {
                seat,
                team: self.team,
            });
        }
        e
    }
}

/// The fifth's additions, row for row with [`HELD_OUT_FIFTH_ROWS`]. Every
/// row's lobby is the human (seat 0, tribe 4) against one computer (seat
/// 1). The team alternates: on rows 3, 43, 83 and 123 the human and the
/// added computer are team 0, against the row's computer; on rows 23, 63,
/// 103 and 143 the two computers are team 1, against the human. The added
/// nations are 0, 1, 2, 3, 5, 6, 7 and 8, none already seated on its row.
#[cfg(test)]
pub(crate) const HELD_OUT_FIFTH_SEATS: [Seated; 8] = [
    Seated {
        tribe: 0,
        mates: [0, 2],
        team: 0,
    },
    Seated {
        tribe: 1,
        mates: [1, 2],
        team: 1,
    },
    Seated {
        tribe: 2,
        mates: [0, 2],
        team: 0,
    },
    Seated {
        tribe: 3,
        mates: [1, 2],
        team: 1,
    },
    Seated {
        tribe: 5,
        mates: [0, 2],
        team: 0,
    },
    Seated {
        tribe: 6,
        mates: [1, 2],
        team: 1,
    },
    Seated {
        tribe: 7,
        mates: [0, 2],
        team: 0,
    },
    Seated {
        tribe: 8,
        mates: [1, 2],
        team: 1,
    },
];

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

    /// PROBE 4540 (reverted whole): row 110's leaders on 5409.
    #[test]
    fn probe_4540_row_110_leaders() {
        if std::env::var("RON_PROBE_4540").is_err() {
            return;
        }
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SECOND, 5) else {
            return;
        };
        let ranges =
            crate::testenv::experiment(&home.join("ron-golden/run1742/map-18"), "rontrace.ranges")
                .expect("ranges");
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 110's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        let lo: i64 = std::env::var("P_LO").ok().and_then(|v| v.parse().ok()).unwrap_or(5395);
        let hi: i64 = std::env::var("P_HI").ok().and_then(|v| v.parse().ok()).unwrap_or(5410);
        while built.sim.frame < lo - 1 {
            stream.apply(built.sim.frame as i32, &mut built);
            built.sim.command().tick();
        }
        for state in lo..=hi {
            let techs_before: Vec<Vec<bool>> = built.sim.tech.iter().map(|t| t.tech.clone()).collect();
            let act_before: Vec<(usize, bool)> = built.sim.buildings.iter().enumerate().map(|(i, b)| (i, b.active)).collect();
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let sim = &built.sim;
            for (w, t) in sim.tech.iter().enumerate() {
                for (k, (&a, &b)) in techs_before[w].iter().zip(&t.tech).enumerate() {
                    if a != b {
                        eprintln!("P {state} who {w} tech {k} {} -> {}", a, b);
                    }
                }
            }
            for (i, b) in sim.buildings.iter().enumerate() {
                let was = act_before.get(i).map(|x| x.1);
                if was != Some(b.active) {
                    eprintln!(
                        "P {state} building {i} owner {} o {} ty {:?} wonder {} active {:?} -> {}",
                        b.owner, b.index, b.ty, b.ty.is_some_and(|t| sim.is_wonder_building(t)), was, b.active
                    );
                }
            }
            let Some(set) = record.first_of(state - 1).cloned() else { continue };
            let all = record.ranges(&set).expect("the set's ranges");
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 7).collect();
            let ours = built.sim.checksum_leaders();
            eprintln!(
                "P {state} r128 ours {:02x?} theirs {:02x?} field {} rec {}",
                ours.ranges.get(128).map(|r| r.bytes.clone()),
                theirs.get(128).map(|r| r.bytes.clone()),
                ours.ranges.get(128).map(|r| r.field).unwrap_or(""),
                ours.ranges.get(128).map(|r| r.record).unwrap_or(-1)
            );
            let n = ours.ranges.len().min(theirs.len());
            for k in 0..n {
                let (a, b) = (&ours.ranges[k], theirs[k]);
                if a.bytes != b.bytes {
                    eprintln!(
                        "PART {state} range {k} record {} {} ours {:?} theirs {:?}",
                        a.record, a.field, a.bytes, b.bytes
                    );
                }
            }
            if ours.ranges.len() != theirs.len() {
                eprintln!("PART {state} lens ours {} theirs {}", ours.ranges.len(), theirs.len());
            }
        }
    }

    /// **The declaration** (item 4413's first act): the eight lobbies, named
    /// and written out before any take, against what `lobbies_on_disk`
    /// yields today, so the set cannot drift silently. `RON_HELD_OUT_PRINT=1`
    /// prints the table as Rust.
    #[test]
    fn the_held_out_eight_are_the_rows_the_rule_names() {
        rows_are_the_table(&HELD_OUT_ROWS, &HELD_OUT, "1");
    }

    /// **The second declaration** (item 4462's first act; DECISIONS 145
    /// §2): rows 10, 30, …, 150, written out before any take, held the
    /// same way. `RON_HELD_OUT_PRINT=2` prints this table as Rust.
    #[test]
    fn the_second_held_out_eight_are_the_rows_the_rule_names() {
        rows_are_the_table(&HELD_OUT_SECOND_ROWS, &HELD_OUT_SECOND, "2");
    }

    /// **The third declaration** (item 4487's first act; DECISIONS 147
    /// §2): rows 5, 25, …, 145, written out before any file is recorded,
    /// held the same way. `RON_HELD_OUT_PRINT=3` prints this table as Rust.
    #[test]
    fn the_third_held_out_eight_are_the_rows_the_rule_names() {
        rows_are_the_table(&HELD_OUT_THIRD_ROWS, &HELD_OUT_THIRD, "3");
    }

    /// **The fourth declaration** (item 4510's first act; DECISIONS 148
    /// §2): rows 15, 35, …, 155, written out before any file is recorded,
    /// held the same way. `RON_HELD_OUT_PRINT=4` prints this table as Rust.
    #[test]
    fn the_fourth_held_out_eight_are_the_rows_the_rule_names() {
        rows_are_the_table(&HELD_OUT_FOURTH_ROWS, &HELD_OUT_FOURTH, "4");
    }

    /// **The fifth declaration** (item 4527's first act; DECISIONS 149 §3,
    /// 150 §2): rows 3, 23, …, 143, each with the seat and the team
    /// [`HELD_OUT_FIFTH_SEATS`] adds, written out before any file is
    /// recorded. The columns are the page's block: the row's own lobby set
    /// by `events_for` on a blank page, then the additions, each through
    /// `Choices::apply`, which refuses what the page cannot hold.
    /// `RON_HELD_OUT_PRINT=5` prints this table as Rust.
    #[test]
    fn the_fifth_held_out_eight_are_the_rows_the_rule_names_seated() {
        let Some(dir) = crate::testenv::gamelog_dir() else {
            eprintln!("skipping: no gamelog directory (set RON_GAMELOG_DIR)");
            return;
        };
        let Some(root) = crate::testenv::install_root() else {
            eprintln!("skipping: no install (set RON_INSTALL)");
            return;
        };
        let roster = crate::lobby::Roster::read(std::path::Path::new(&root)).expect("the roster");
        let lobbies = crate::lobby::lobbies_on_disk(&dir, crate::corpus::Ledger::of_tree())
            .expect("the logs");
        assert_eq!(
            lobbies.len(),
            crate::diff::START_LOBBIES,
            "the rows are of 161"
        );
        let print = std::env::var("RON_HELD_OUT_PRINT").is_ok_and(|v| v == "5");
        for (k, &row) in HELD_OUT_FIFTH_ROWS.iter().enumerate() {
            let l = &lobbies[row];
            let lobby = crate::lobby::Lobby::parse(&l.block).expect("the row's block");
            let mut c = crate::lobby::Choices::blank();
            for e in crate::lobby::events_for(&lobby, &roster) {
                c.apply(&e, &roster).expect("the row's own lobby");
            }
            assert_eq!(
                c.lobby().partings(&lobby),
                Vec::<String>::new(),
                "row {row}: the page"
            );
            assert_eq!(c.seats.len(), 2, "row {row}: two seats as it stands");
            let add = &HELD_OUT_FIFTH_SEATS[k];
            for e in add.events(c.seats.len()) {
                c.apply(&e, &roster)
                    .unwrap_or_else(|why| panic!("row {row}: {e:?} refused: {why}"));
            }
            assert_eq!(c.seats.len(), 3, "row {row}: three seats");
            let teamed: Vec<usize> = (0..3)
                .filter(|&s| c.seats[s].team != crate::lobby::NO_TEAM)
                .collect();
            assert_eq!(teamed, add.mates.to_vec(), "row {row}: one team of two");
            let (info, players, digest) = columns(&c.lines());
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
            let want = &HELD_OUT_FIFTH[k];
            assert_eq!(want.row, row, "the table's row {k}");
            assert_eq!(
                (l.captures[0].as_str(), info.as_str(), players, digest),
                (
                    want.capture,
                    want.info,
                    want.players.iter().map(|s| s.to_string()).collect(),
                    want.digest
                ),
                "row {row}: the lobby the rule named, seated"
            );
        }
    }

    /// `rows` of `lobbies_on_disk` against `table`, column for column;
    /// `RON_HELD_OUT_PRINT=which` (`1` the first battery, `2` the second,
    /// `3` the third, `4` the fourth) prints the table instead.
    fn rows_are_the_table(rows: &[usize; 8], table: &[HeldOut; 8], which: &str) {
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

    /// `HELD_OUT_THIRD[k]` played as `run1743 + k` (item 4487), held as
    /// the first two are.
    #[test]
    fn the_third_held_out_games_score_as_pinned() {
        games_score_as_pinned(&THIRD);
    }

    /// `HELD_OUT_FOURTH[k]` played as `run1760 + k` (item 4510), held as
    /// the first three are.
    #[test]
    fn the_fourth_held_out_games_score_as_pinned() {
        games_score_as_pinned(&FOURTH);
    }

    /// `HELD_OUT_FIFTH[k]` played as `run1773 + k` (item 4527), held as
    /// the first four are.
    #[test]
    fn the_fifth_held_out_games_score_as_pinned() {
        games_score_as_pinned(&FIFTH);
    }

    /// **Row 23's setup claims oil under the colonized limit** (item 4534,
    /// `docs/RECGAME.md` §12.3): run1781 holds every leaders range from
    /// state 0. `Setup::build_game@005ac190:881` runs `World::analyze_map`
    /// after the whole `build_empire` loop, so who=0's own `build_cities`
    /// pass reads region 4's colonized triple (base 44): gi 2 at (2, 43),
    /// raw distance 20, joins its `oil_patches` there, and gi 5 at (16, 42),
    /// raw distance 48, only at line 883's pass under the player triple's
    /// 48. Every leaders range on state 0, ours against theirs, byte for
    /// byte: red on the tree before the fix at range 58, record 0's
    /// `oil_patches`, ours `[5, 2]`.
    #[test]
    fn row_23_s_setup_claims_oil_under_the_colonized_limit_on_run1781_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 1) else {
            eprintln!("skipping: row 23's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1781/map-20"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1781's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 23's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let built = crate::checksum::playback::recording_start(&loaded, &rec);
        assert_eq!(built.sim.frame, 0, "the start, before its first frame");
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        let set = record.first_of(-1).expect("state 0's set").clone();
        let all = record.ranges(&set).expect("the set's ranges");
        let ours = built.sim.checksum_leaders();
        let theirs: Vec<_> = all.iter().filter(|r| r.line == 7).collect();
        assert_eq!(ours.ranges.len(), theirs.len(), "leaders' ranges on 0");
        for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
            assert_eq!(
                a.bytes, b.bytes,
                "leaders range {k} on 0, record {} {}",
                a.record, a.field
            );
        }
        // Record 0's list by value: gi 2, then gi 5, on both sides.
        let data: Vec<u8> = [2i32, 5].iter().flat_map(|v| v.to_le_bytes()).collect();
        let k = ours
            .ranges
            .iter()
            .position(|r| r.record == 0 && r.field == "oil_patches" && r.bytes.len() == 8)
            .expect("record 0's oil_patches data");
        assert_eq!(theirs[k].bytes, data, "theirs' list");
        assert_eq!(built.sim.ai.ai[0].oil_patches, vec![2, 5]);
        let cell = |gi: usize| built.sim.world.goods()[gi].pos.cell();
        assert_eq!(cell(2), sim::world::Cell::new(2, 43));
        assert_eq!(cell(5), sim::world::Cell::new(16, 42));
        // Region 4: the player triple now, the colonized one before.
        let lim = built
            .sim
            .world
            .territory_limits()
            .expect("Sea Power's limits");
        assert_eq!(built.sim.world.region_of(cell(5)), Some(4));
        assert_eq!(built.sim.world.region_territory_limits(4), Some(lim.player));
        assert_eq!((lim.player[0], lim.colonized[0]), (48, 44));
    }

    /// **The fifth's files carry the declared seats and team** (item
    /// 4527): each `item4527-row<RRR>-6000.rcx` the client recorded, read
    /// back, holds the declaration's three `PLAYER` rows slot for slot —
    /// `flags`, `tribe`, `who`, `team`, `handicap`, `play` — no fourth
    /// active slot, and `PLAYERS 2`, so what the original plays is the
    /// game declared before it was written.
    #[test]
    fn the_fifth_s_files_carry_the_declared_seats_and_team() {
        let field = |row: &str, key: &str| -> i64 {
            row.split("; ")
                .find_map(|kv| kv.strip_prefix(key)?.strip_prefix(' ')?.parse().ok())
                .unwrap_or_else(|| panic!("{key} in {row}"))
        };
        for (k, held) in HELD_OUT_FIFTH.iter().enumerate() {
            let (Some(rcx), _) = paths(&FIFTH, k) else {
                eprintln!("skipping: row {} has no file", held.row);
                continue;
            };
            let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("the file");
            assert_eq!(
                rec.options[crate::recgame::PLAYERS_OPTION],
                2,
                "row {}",
                held.row
            );
            let active: Vec<_> = rec.slots.iter().filter(|s| s.active()).collect();
            assert_eq!(active.len(), held.players.len(), "row {}: seats", held.row);
            for (slot, row) in active.iter().zip(held.players) {
                let got = (
                    i64::from(slot.flags),
                    i64::from(slot.tribe),
                    i64::from(slot.who),
                    i64::from(slot.team),
                    i64::from(slot.handicap),
                    i64::from(slot.play),
                );
                let want = (
                    field(row, "flags"),
                    field(row, "tribe"),
                    field(row, "who"),
                    field(row, "team"),
                    field(row, "handicap"),
                    field(row, "play"),
                );
                assert_eq!(got, want, "row {}: {row}", held.row);
            }
        }
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

    /// **Row 60's Observation Post lights `seen3`** (item 4480,
    /// `docs/RECGAME.md` §10.3, `docs/VISION.md` §4): run1722, the ranges
    /// take of run1703's file. On 3984 who=1's Observation Post `1/59`
    /// (building type 108, LOS 9, the `DETECT` objmask) lights its disc
    /// round fog cell (91, 79). `Wall::init@0063e9b0` runs
    /// `Object::init@00647750`, whose `has_objmask(0x2000000)` sets
    /// `field_0x8 | 0x40` for a building as for a unit, and
    /// `Object::update_seen@00651b80` then writes `seen3` over the whole
    /// disc. This crate passed a building's sweep no `seen3`. Every world
    /// range on 3983 and 3984, byte for byte: red on the tree before the fix
    /// at range 3627 on 3984, `seen3`, 433 bytes ours 0x00, theirs 0x02.
    #[test]
    fn row_60_s_observation_post_lights_seen3_on_run1722_s_ranges() {
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
        for (state, lit) in [(3983, 0u8), (3984, 2u8)] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_world();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 11).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "world's ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert!(
                    a.bytes == b.bytes,
                    "world range {k} on {state}, record {} {}: {} bytes differ",
                    a.record,
                    a.field,
                    a.bytes.iter().zip(&b.bytes).filter(|(x, y)| x != y).count()
                );
            }
            // Theirs' `seen3` at the post's own cell, by name: who=1's bit
            // arrives on 3984.
            let k = ours
                .ranges
                .iter()
                .position(|r| r.field == "seen3")
                .expect("seen3");
            let fw = built.sim.world.fog_xs() as usize;
            assert_eq!(fw * fw, theirs[k].bytes.len(), "seen3 is the fog grid");
            assert_eq!(
                theirs[k].bytes[79 * fw + 91] & 2,
                lit,
                "theirs' seen3 at (91, 79) on {state}"
            );
        }
    }

    /// **Row 60's Dragoon keeps its dropped target's uid** (item 4476,
    /// `docs/RECGAME.md` §10.3): run1722, the ranges take of run1703's file.
    /// On 3756 the Dragoon `1/21` (TypeIndex 189, land) holds the
    /// fire-on-the-move target `0/11`, uid 18; on 3757 both sides drop it
    /// (`cavarch_o` −1, `cavarch_who` −1), and theirs keeps `cavarch_uid`
    /// 18: every writer of `UnitData +0xa6` writes it only on a find
    /// (`Unit::fight@005fd4d0`, `Group::action_spell@006fe1a0`). This crate
    /// read the uid off the held target and wrote 0 without one. Every units
    /// range on 3756 and 3757, byte for byte: red on the tree before the fix
    /// at range 788 on 3757, record 21 `collide_frame..play`, `cavarch_uid`
    /// ours 0, theirs 18.
    #[test]
    fn row_60_s_dragoon_keeps_its_dropped_target_s_uid_on_run1722_s_ranges() {
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
        for (state, held) in [(3756, 11), (3757, -1)] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_units();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 0).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "units' ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "units range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            // Theirs' record 21, by name: the target dropped, the uid kept.
            let k = ours
                .ranges
                .iter()
                .position(|r| r.record == 21 && r.field == "collide_frame..play")
                .expect("record 21");
            let f = crate::checksum::fields::range_fields(
                "units",
                ours.ranges[k].field,
                &theirs[k].bytes,
            )
            .expect("record 21's fields");
            let get = |n: &str| f.iter().find(|(m, _)| m == n).map(|x| x.1);
            assert_eq!(get("cavarch_o"), Some(held), "theirs' cavarch_o on {state}");
            assert_eq!(
                get("cavarch_uid"),
                Some(18),
                "theirs' cavarch_uid on {state}"
            );
        }
    }

    /// **Row 60's army push zeroes the emptied last slot's origin** (item
    /// 4473, `docs/RECGAME.md` §10.3): run1722, the ranges take of run1703's
    /// file. On 3598 who=1's army 1 takes `1/169` through
    /// `Army::add_unit@006f9f40:37`'s `Groups::push_group(who, g, 1)`, into
    /// slot 67, while `last_group` names slot 68 (record 68), empty on both
    /// sides since its members left. `Group::equals_group@00708000` asks the
    /// last slot's `get_num@00714700`, whose empty arm writes `ox`/`oy` 0.
    /// This crate's army push skipped `equals_group`'s side effects. Every
    /// groups range on 3597 and 3598, byte for byte: red on the tree before
    /// the fix at range 92 on 3598, record 68 `id..march`, `ox` ours 33720,
    /// theirs 0 (`oy` 15048, 0).
    #[test]
    fn row_60_s_army_push_zeroes_the_empty_last_slot_s_origin_on_run1722_s_ranges() {
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
        for state in [3597, 3598] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_groups();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 5).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "groups' ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "groups range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            // Record 68's `ox` (`+0x18`, byte 20 of `id..march`), by name.
            let k = ours
                .ranges
                .iter()
                .position(|r| r.record == 68 && r.field == "id..march")
                .expect("record 68");
            let want: i32 = if state == 3597 { 33720 } else { 0 };
            assert_eq!(
                &theirs[k].bytes[20..24],
                &want.to_le_bytes(),
                "theirs' record 68 ox on {state}"
            );
        }
    }

    /// **Row 85's army drops the squad member its sort unpointed** (item
    /// 4493, `docs/RECGAME.md` §11.1): run1751, the ranges take of run1747's
    /// file, holds the groups line's every range on 3830 and 3831. On 3830
    /// computer 1's army 3 (record 68, who=1's slot 4) moves; its
    /// `Form::categorize` sorts `[117, 121]`, killing the follower-first
    /// squad `1/119..1/121` — only `1/121` listed, so only its `+0x80`
    /// cleared — and re-adding it whole. A follower's `GROUP_MOVE` then
    /// refreshes its order, and `Group::refresh_group_order@00713a50` opens
    /// with `normalize`, which prunes `1/121`: theirs' record 68 reads `num`
    /// 3, `[117, 119, 120]`, where this crate kept `num` 4. Every groups
    /// range on both states, ours against theirs, byte for byte.
    #[test]
    fn row_85_s_refresh_prunes_the_member_its_sort_unpointed_on_run1751_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&THIRD, 4) else {
            eprintln!("skipping: row 85's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1751/map-14"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1751's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 85's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [3830, 3831] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_groups();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 5).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "groups' ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "groups range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            // Record 68's `num` (`+0xc`, byte 8 of `id..march`), by name.
            let k = ours
                .ranges
                .iter()
                .position(|r| r.record == 68 && r.field == "id..march")
                .expect("record 68");
            let want: i32 = if state == 3830 { 2 } else { 3 };
            assert_eq!(
                &theirs[k].bytes[8..12],
                &want.to_le_bytes(),
                "theirs' record 68 num on {state}"
            );
        }
    }

    /// **Row 5's Slinger stone leaves from the original's socket** (item
    /// 4507, `docs/RECGAME.md` §11.1): run1753, the ammo take of run1743's
    /// file (`RNG_MASK=0x8u`), holds every ammo range on 4804 to 4806. On
    /// 4805 who=1's Slinger `1/22` (type 82, piece 32) looses its
    /// `CHAR_ATTACK2` stone at frame 22 from (5064, 30744) facing
    /// −1,097,334,784, and theirs leaves from (4945, 30709): range 1,
    /// record 0, `rolling..start_roll_angle` bytes 7 and 11 (`launch.x`,
    /// `launch.y`), then the aim's angle (byte 41) and `dx` (byte 84) that
    /// follow from it. Every ammo range on the three states, ours against
    /// theirs, byte for byte: red on the tree before the re-fit, at range 1
    /// on 4805, where ours left from (4944, 30710).
    #[test]
    fn row_5_s_slinger_stone_leaves_from_the_original_s_socket_on_run1753_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&THIRD, 0) else {
            eprintln!("skipping: row 5's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1753/map-14"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1753's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 5's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [4804, 4805, 4806] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_ammo().expect("ours' ammo line");
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 3).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "ammo ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "ammo range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            // Theirs' slot 0 `launch.x, launch.y`, by name, from 4805.
            if state >= 4805 {
                let b = &theirs[1].bytes;
                let word =
                    |k: usize| i32::from_le_bytes(b[3 + 4 * k..7 + 4 * k].try_into().unwrap());
                assert_eq!(
                    (word(1), word(2)),
                    (4945, 30709),
                    "theirs' launch on {state}"
                );
            }
        }
    }

    /// **Row 75's crossbow bolt leaves from the original's socket** (item
    /// 4517, `docs/RECGAME.md` §12): run1768, the ammo take of run1763's
    /// file (`RNG_MASK=0x8u`), holds every ammo range of the game. On 4004,
    /// the game's first round, who=1's Crossbowmen `1/163` (TI 172, land,
    /// piece 122) looses its `CHAR_ATTACK3` bolt at frame 6 from (3960,
    /// 34440) facing −475,070,464, and theirs leaves from (3923, 34374):
    /// range 1, record 0, `rolling..start_roll_angle` byte 7 (`launch.x`),
    /// then the aim's angle (byte 41) and `dx` (bytes 83 and 84) that follow
    /// from it. Every ammo range on 4003 to 4013, ours against theirs, byte
    /// for byte: red on the tree before the socket's sweep, at range 1 on
    /// 4004, where ours left from (3922, 34374).
    #[test]
    fn row_75_s_crossbow_bolt_leaves_from_the_original_s_socket_on_run1768_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FOURTH, 3) else {
            eprintln!("skipping: row 75's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1768/map-14"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1768's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 75's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 4003i64..=4013 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_ammo().expect("ours' ammo line");
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 3).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "ammo ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                let at = a.bytes.iter().zip(&b.bytes).position(|(x, y)| x != y);
                assert_eq!(
                    at.map(|i| (i, a.bytes[i], b.bytes[i])),
                    None,
                    "ammo range {k} on {state}, record {} {}: (byte, ours, theirs)",
                    a.record,
                    a.field
                );
                assert_eq!(a.bytes.len(), b.bytes.len(), "ammo range {k} on {state}");
            }
            if state == 4004 {
                // The bolt, by name: `1/163`, TI 172, land.
                let p = &built.sim.combat.projectiles[0];
                let sim::combat::Obj::Unit(u) = p.shooter else {
                    panic!("a unit's round")
                };
                let u = &built.sim.units[u];
                assert_eq!((u.owner, u.index, u.type_index), (1, 163, 172));
                assert_eq!(u.kind.domain, sim::territory::attrition::Domain::Land);
                // Theirs' slot 0 `launch.x, launch.y`.
                let b = &theirs[1].bytes;
                let word =
                    |k: usize| i32::from_le_bytes(b[3 + 4 * k..7 + 4 * k].try_into().unwrap());
                assert_eq!((word(1), word(2)), (3923, 34374), "theirs' launch");
            }
        }
    }

    /// **Row 105's CEO carries the CEO bit from its `set_type`** (item
    /// 4502, `docs/RECGAME.md` §11.1): run1752, the ranges take of
    /// run1748's file, holds every units range on 2982 and 2983. On 2983
    /// who=1's `1/17` turns from `TypeIndex` 0x163 to 0x165, and
    /// `Unit::set_type@00612fa0:208`–`219` raises `unit_masks2 & 0x10000`
    /// on a type that `is(0x165, 1)`: range 655, record 17,
    /// `collide_frame..play` byte 38 reads 1 on theirs, where this crate
    /// wrote no such bit. Every units range on both states, ours against
    /// theirs, byte for byte: red on the tree before the fix, at range 655.
    #[test]
    fn row_105_s_ceo_carries_the_ceo_bit_from_its_set_type_on_run1752_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&THIRD, 5) else {
            eprintln!("skipping: row 105's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1752/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1752's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 105's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [2982, 2983] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_units();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 0).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "units' ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "units range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            // `1/17`'s type and theirs' `unit_masks2` (`+0x6c`, byte 36 of
            // `collide_frame..play`), by name.
            let u = built
                .sim
                .units
                .iter()
                .find(|x| x.alive() && x.owner == 1 && x.index == 17)
                .expect("1/17");
            let (ty, want) = if state == 2982 {
                (0x163, 0)
            } else {
                (0x165, 0x10000)
            };
            assert_eq!(u.type_index, ty, "1/17's TypeIndex on {state}");
            assert_eq!(
                &theirs[655].bytes[36..40],
                &(want as u32).to_le_bytes(),
                "theirs' 1/17 unit_masks2 on {state}"
            );
        }
    }

    /// **Row 105's Freighter chains its riders in the order they boarded**
    /// (item 4511, `docs/RECGAME.md` §11.1): run1752 holds every units
    /// range on 3358 and 3359. On 3359 who=1's Transport Freighter `1/135`
    /// (TI 322) is born and three German Infantry (TI 103) go aboard,
    /// `1/127`, `1/128`, `1/129`. `Object::insert_inside@00647e90` appends
    /// at the bottom, so theirs prints `inside_down` 127 on the host, 128 on
    /// `1/127` and 129 on `1/128` (range 3707, `myhits..launch_frames` byte
    /// 8), and `inside_up` 127 and 128 on the last two (`collide_frame..play`
    /// byte 58). This crate wrote the last rider by index on the host, −1
    /// on every rider and the host above each. Every units range on both
    /// states, ours against theirs, byte for byte: red on the tree before
    /// the fix, at range 3707.
    #[test]
    fn row_105_s_freighter_chains_its_riders_in_the_order_they_boarded_on_run1752_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&THIRD, 5) else {
            eprintln!("skipping: row 105's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1752/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1752's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 105's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [3358, 3359] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_units();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 0).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "units' ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "units range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
        }
        // On 3359, by name: the host, its riders in order, and theirs'
        // `inside_down` on `1/127`.
        let find = |o: i16| {
            (0..built.sim.units.len())
                .find(|&x| {
                    let u = &built.sim.units[x];
                    u.alive() && u.owner == 1 && u.index == o
                })
                .expect("the unit")
        };
        let boat = find(135);
        assert_eq!(
            built.sim.units[boat].type_index, 322,
            "1/135 is the Freighter"
        );
        let chain: Vec<i16> = built
            .sim
            .cargo_chain(boat)
            .iter()
            .map(|&x| built.sim.units[x].index)
            .collect();
        assert_eq!(chain, [127, 128, 129], "1/135's riders, head first");
        let theirs = record
            .ranges(&record.first_of(3358).expect("set").clone())
            .expect("ranges");
        let units: Vec<_> = theirs.iter().filter(|r| r.line == 0).collect();
        let ours = built.sim.checksum_units();
        assert_eq!(
            (ours.ranges[3707].record, ours.ranges[3707].field),
            (127, "myhits..launch_frames")
        );
        assert_eq!(
            &units[3707].bytes[8..10],
            &128i16.to_le_bytes(),
            "theirs' 1/127 inside_down"
        );
    }

    /// **Row 130's computer bomber searches round itself** (item 4508,
    /// `docs/RECGAME.md` §11): who=1's Strategic Bomber `1/158`, on its
    /// patrol to (6240, 7008) and some 10,100 units from it, runs its
    /// sixteen-frame search on 3410. `find_new_bomber_target`'s head
    /// overwrites the point with the plane's own position for a unit with
    /// `unit_masks & 0x40000`, so no building is in reach and the patrol
    /// stands; ours searched round the point and put a strafe in front of
    /// it (units range 4078, `orders.count` ours 2, theirs 1). Every
    /// units range on 3410 and 3411 against run1754's record.
    #[test]
    fn row_130_s_computer_bomber_searches_round_itself_on_run1754_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SECOND, 6) else {
            eprintln!("skipping: row 130's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1754/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1754's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 130's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [3410, 3411] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_units();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 0).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "units' ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "units range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
        }
        // On 3411, by name: the bomber, its one order, and theirs' count.
        let b = (0..built.sim.units.len())
            .find(|&x| {
                let u = &built.sim.units[x];
                u.alive() && u.owner == 1 && u.index == 158
            })
            .expect("1/158");
        assert_eq!(
            built.sim.units[b].type_index, 305,
            "1/158 is the Strategic Bomber"
        );
        assert!(
            matches!(
                built.sim.units[b]
                    .orders
                    .iter()
                    .map(|o| o.index())
                    .collect::<Vec<_>>()[..],
                [sim::orders::index::AIR_PATROL]
            ),
            "1/158 keeps its patrol alone"
        );
        let ours = built.sim.checksum_units();
        assert_eq!(
            (ours.ranges[4078].record, ours.ranges[4078].field),
            (158, "orders.count")
        );
        let theirs = record
            .ranges(&record.first_of(3410).expect("set").clone())
            .expect("ranges");
        let units: Vec<_> = theirs.iter().filter(|r| r.line == 0).collect();
        assert_eq!(
            &units[4078].bytes[..],
            &1i32.to_le_bytes(),
            "theirs' 1/158 orders.count"
        );
    }

    /// **Row 130's leaders line carries the Kremlin's spy-due bit** (item
    /// 4520, `docs/RECGAME.md` §11): who=1's Kremlin finishes on 4573 with no
    /// tagged spy alive, and `leader_flags & 0x10000000` stands until the
    /// wonder trains its spy on 4583. This crate kept the bit and its
    /// checksum never printed it (leaders range 128, record 1, `flags` byte
    /// 3, ours `0x0e`, theirs `0x1e`). Every leaders range on 4572, 4573,
    /// 4582 and 4583 against run1754's record.
    #[test]
    fn row_130_s_kremlin_spy_due_bit_is_in_the_leaders_flags_on_run1754_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SECOND, 6) else {
            eprintln!("skipping: row 130's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1754/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1754's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 130's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for (state, due) in [(4572, false), (4573, true), (4582, true), (4583, false)] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_leaders();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 7).collect();
            assert_eq!(
                ours.ranges.len(),
                theirs.len(),
                "leaders' ranges on {state}"
            );
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "leaders range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            // Record 1's `flags`, by name: byte 3 holds `0x10000000`.
            assert_eq!(
                (ours.ranges[128].record, ours.ranges[128].field),
                (1, "flags")
            );
            assert_eq!(
                theirs[128].bytes[3] & 0x10 != 0,
                due,
                "theirs' who=1 spy-due bit on {state}"
            );
            assert_eq!(built.sim.tech[1].kremlin_spy_due, due, "ours' on {state}");
        }
    }

    /// **Row 105's rider bound for the sea is a gatherer in
    /// `reg_gatherers[1]`** (item 4499, `docs/RECGAME.md` §11.1): run1752,
    /// the ranges take of run1748's file, holds every leaders range on
    /// 1975 and 1976. On 1976, who=1's first census sweep, a citizen riding
    /// a sea transport whose move ends in sea region 65 is counted where
    /// `plan_strategy@006b9620`'s `incw 0x10de(%ebx,%esi,2)` (`6baa2f`)
    /// lands with no upper bound: `0x10de + 65 * 2 = 0x1160`,
    /// `reg_gatherers[1]`, byte 4440 of the record's `who..ally_mask`.
    /// Every leaders range on both states, ours against theirs, byte for
    /// byte: red on the tree before the fix, at range 129.
    #[test]
    fn row_105_s_rider_to_sea_region_65_lands_in_reg_gatherers_on_run1752_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&THIRD, 5) else {
            eprintln!("skipping: row 105's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1752/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1752's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 105's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [1975, 1976] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_leaders();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 7).collect();
            assert_eq!(
                ours.ranges.len(),
                theirs.len(),
                "leaders' ranges on {state}"
            );
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "leaders range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            // Record 1's `reg_gatherers[1]` (`+0x1160`, byte 4440 of
            // `who..ally_mask`), by name.
            let k = ours
                .ranges
                .iter()
                .position(|r| r.record == 1 && r.field == "who..ally_mask")
                .expect("record 1's long range");
            let want: i16 = if state == 1975 { 0 } else { 1 };
            assert_eq!(
                &theirs[k].bytes[4440..4442],
                &want.to_le_bytes(),
                "theirs' record 1 reg_gatherers[1] on {state}"
            );
        }
    }

    /// **Row 55's Merchant Fleet is born through `Unit::update_hits`**
    /// (item 4518, `docs/RECGAME.md` §12): run1769 holds every units range
    /// to 6000. On 4275 who=1's Armed Caravan `1/64` casts its transport and
    /// the Merchant Fleet `1/142` (TI 318) is born; who=1 is the Dutch (tribe
    /// 22), so `Unit::init@00612100:580`'s `update_hits` adds the merchant
    /// kinds' `ages × DUTCH_HP_BONUS`: `myhits` 192, where the raw type's 172
    /// parted the line.
    #[test]
    fn row_55_s_dutch_merchant_fleet_is_born_with_its_update_hits_on_run1769_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FOURTH, 2) else {
            eprintln!("skipping: row 55's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1769/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1769's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 55's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [4274, 4275] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_units();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 0).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "units' ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "units range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            // The fleet by name: who=1's record 142, its `type` and `myhits`.
            let at = ours
                .ranges
                .iter()
                .enumerate()
                .filter(|(_, r)| r.record == 142 && r.field == "who..y_internal")
                .find(|(k, _)| theirs[*k].bytes[0] == 1)
                .map(|(k, _)| k);
            if state == 4274 {
                assert_eq!(at, None, "1/142 is not yet born on 4274");
                continue;
            }
            let k = at.expect("1/142 on 4275");
            assert_eq!(ours.ranges[k + 1].field, "type");
            assert_eq!(
                &theirs[k + 1].bytes[..],
                &318i32.to_le_bytes(),
                "1/142's TypeIndex"
            );
            assert_eq!(ours.ranges[k + 3].field, "myhits..launch_frames");
            assert_eq!(
                &theirs[k + 3].bytes[..4],
                &192i32.to_le_bytes(),
                "theirs' myhits"
            );
            let fleet = built
                .sim
                .units
                .iter()
                .position(|u| u.alive() && u.owner == 1 && u.index == 142)
                .expect("1/142");
            let ty = built.sim.units[fleet].ty.expect("its type");
            assert_eq!(built.unit_type_names[ty], "Merchant Fleet");
            assert_eq!(built.sim.tables.unit_types[ty].hits, 172, "the raw type");
        }
    }

    /// **Row 95's CEO dirties its economy as it boards** (item 4519,
    /// `docs/RECGAME.md` §12.4): on run1770's ranges, every leaders range
    /// on 5265 and 5266 byte for byte. On 5266 who=1's `leader_flags`
    /// (range 128, record 1) gains `0x2000000`: the CEO `1/19` (TI 357)
    /// is inside its newborn Transport Freighter, and `Unit::go_inside
    /// @0061a2e0`'s CEO arm left its city.
    #[test]
    fn row_95_s_ceo_dirties_its_economy_as_it_boards_its_freighter_on_run1770_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FOURTH, 4) else {
            eprintln!("skipping: row 95's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1770/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1770's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 95's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [5265, 5266] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_leaders();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 7).collect();
            assert_eq!(
                ours.ranges.len(),
                theirs.len(),
                "leaders' ranges on {state}"
            );
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "leaders range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            // Who=1's flags: the dirty bit is byte 3's `0x02`.
            assert_eq!(ours.ranges[128].field, "flags");
            assert_eq!(ours.ranges[128].record, 1);
            let dirty = theirs[128].bytes[3] & 0x02 != 0;
            assert_eq!(dirty, state == 5266, "who=1's 0x2000000 on {state}");
            let ceo = built
                .sim
                .units
                .iter()
                .find(|u| u.alive() && u.owner == 1 && u.index == 19)
                .expect("1/19");
            assert_eq!(ceo.type_index, 357, "1/19 is the CEO");
            assert_eq!(
                ceo.inside_unit.is_some(),
                state == 5266,
                "1/19 aboard on {state}"
            );
        }
    }

    /// **Row 55's army keeps a member whose number a newborn took** (item
    /// 4531, `docs/RECGAME.md` §12.2): run1769 holds every groups range to
    /// 6000. On 5576 who=1's Elite Javelineers `1/44` (TI 84) leave their
    /// building and `push_group`'s `get_open_slot` walks `get_num` over
    /// who=1's slots. Group 83, army 5's, holds 43 members, one of them the
    /// short 44 a dead member left. `Group::get_num@00714700`'s large arm
    /// keeps a member whose object is active, and object 44 is the newborn,
    /// so theirs reads `num` 43, where this crate's list held the dead
    /// slot and dropped it (42). Every groups range on both states, ours
    /// against theirs, byte for byte.
    #[test]
    fn row_55_s_army_keeps_the_member_whose_number_a_newborn_took_on_run1769_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FOURTH, 2) else {
            eprintln!("skipping: row 55's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1769/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1769's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 55's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [5575, 5576] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_groups();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 5).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "groups' ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "groups range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            // Record 83's `num` (`+0xc`, byte 8 of `id..march`), and the
            // short 44 at list index 31, by name.
            let k = ours
                .ranges
                .iter()
                .position(|r| r.record == 83 && r.field == "id..march")
                .expect("record 83");
            assert_eq!(&theirs[k].bytes[8..12], &43i32.to_le_bytes(), "num");
            assert_eq!(ours.ranges[k + 1].field, "list");
            assert_eq!(
                &theirs[k + 1].bytes[62..64],
                &44i16.to_le_bytes(),
                "list[31]"
            );
            // Ours' member 31: dead on 5575, the newborn on 5576.
            let army = built.sim.ai.armies[1]
                .list
                .iter()
                .find(|a| a.valid && a.group.pool == Some(19))
                .expect("army 5");
            let u = &built.sim.units[army.units[31]];
            assert_eq!((u.owner, u.index), (1, 44));
            if state == 5575 {
                assert!(!u.alive(), "1/44 is dead on 5575");
                continue;
            }
            assert!(u.alive(), "the newborn on 5576");
            assert_eq!(
                built.unit_type_names[u.ty.expect("its type")],
                "Elite Javelineers"
            );
        }
    }

    /// **Row 105's pruned oil list keeps its capacity** (item 4512,
    /// `docs/RECGAME.md` §11.1): run1752 holds every leaders range on 3582
    /// and 3583. On 3583 who=1's `oil_patches` goes from twelve patches to
    /// ten, both sides (arm 4.2 of `Leader::produce_building@006e1400`
    /// prunes two land patches the leader no longer owns, by
    /// `SimpleArray<int>::remove@00462e70`), and theirs' `size` stays 20:
    /// a removal only shortens the length. Every leaders range on both
    /// states, ours against theirs, byte for byte: red on the tree before
    /// the fix, at range 191 (record 1 `oil_patches` `size`, ours 10).
    #[test]
    fn row_105_s_pruned_oil_list_keeps_its_capacity_on_run1752_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&THIRD, 5) else {
            eprintln!("skipping: row 105's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1752/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1752's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 105's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [3582, 3583] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_leaders();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 7).collect();
            assert_eq!(
                ours.ranges.len(),
                theirs.len(),
                "leaders' ranges on {state}"
            );
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "leaders range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            // Record 1's `oil_patches`: the length, then the size, by name.
            let k = ours
                .ranges
                .iter()
                .position(|r| r.record == 1 && r.field == "oil_patches")
                .expect("record 1's oil_patches");
            let (n, size): (i32, i32) = if state == 3582 { (12, 20) } else { (10, 20) };
            assert_eq!(
                theirs[k].bytes,
                n.to_le_bytes(),
                "theirs' length on {state}"
            );
            assert_eq!(
                theirs[k + 1].bytes,
                size.to_le_bytes(),
                "theirs' size on {state}"
            );
            assert_eq!(built.sim.ai.ai[1].oil_patches.len(), n as usize);
        }
    }

    /// **Row 105's pushed unit stays listed by the slot it left**
    /// (item 4512, `docs/RECGAME.md` §11.1): run1752 holds every groups
    /// range on 4354 and 4355. On 4355 `think_scout` pushes who=1's `1/94`
    /// (type 200) onto slot 22. `Groups::push_group@0070f9e0`'s second
    /// walk kills it only from the group its `+0x80` names, slot 9, so slot
    /// 20 (record 84), which still lists it, keeps it: `num` 50. Every
    /// groups range on both states, ours against theirs, byte for byte:
    /// red on the tree before the fix, at range 204 (`num` ours 49).
    #[test]
    fn row_105_s_pushed_unit_stays_listed_by_the_slot_it_left_on_run1752_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&THIRD, 5) else {
            eprintln!("skipping: row 105's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1752/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1752's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 105's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [4354, 4355] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_groups();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 5).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "groups' ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "groups range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            // Record 84's `num` (byte 8 of `id..march`) and its list, by
            // name: `1/94` is the 37th member on both states.
            assert_eq!(
                theirs[204].bytes[8..12],
                50i32.to_le_bytes(),
                "theirs' num on {state}"
            );
            assert_eq!(
                theirs[205].bytes[72..74],
                94i16.to_le_bytes(),
                "theirs' 1/94 on {state}"
            );
            let u = built
                .sim
                .units
                .iter()
                .find(|u| u.owner == 1 && u.index == 94)
                .expect("1/94");
            let want = if state == 4354 { 9 } else { 22 };
            assert_eq!(u.group_ptr, Some(want), "1/94's +0x80 on {state}");
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

    /// **Row 90's barge turns to its caster's angle with the snap**
    /// (item 4470, `docs/RECGAME.md` §11): run1738, the ranges take of
    /// run1734's file. On 4021 the computer's transport barge (type 320,
    /// record 14) is born from a land unit by `SpellType::cast_transport`,
    /// which turns it with `Unit::set_angle(boat, caster +0x50, _, 1)`
    /// (`00670db0`, the listing's `671027`–`67104f`); the snap reaches
    /// `Guy::set_angle@005d9010(guy 0, a, 1)`, which writes `angle` and
    /// `last_angle` (`+0x18`, `+0x1c`) both. This crate's view wrote the
    /// movement's facing and left guy 0's `last` angle at
    /// [`sim::movement::Angle::INITIAL`]. Every units and guys range on
    /// 4021, ours against theirs, byte for byte: red on the tree before the
    /// fix at units range 498 and guys range 150, record 14's figure,
    /// `last_angle` ours 1431655765, theirs −609026048.
    #[test]
    fn row_90_s_barge_turns_with_the_snap_on_run1738_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SECOND, 4) else {
            eprintln!("skipping: row 90's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1738/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1738's ranges record is missing");
            return;
        };
        const STATE: i64 = 4021;
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 90's file");
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
            .expect("state 4021's set")
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
                "line {line}'s ranges on 4021"
            );
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "line {line} range {k} on 4021, record {} {}",
                    a.record, a.field
                );
            }
        }
        // The barge's figure, by name: type 320, `last_angle` (`+0x1c`,
        // byte 20 of `GuyData::walk_data`'s range) `0xdbb30000`, its
        // `angle` (byte 16).
        let theirs: Vec<_> = all.iter().filter(|r| r.line == 0).collect();
        assert_eq!(&theirs[498].bytes[0..4], &320i32.to_le_bytes());
        assert_eq!(&theirs[498].bytes[16..20], &theirs[498].bytes[20..24]);
        assert_eq!(&theirs[498].bytes[20..24], &(-609_026_048i32).to_le_bytes());
    }

    /// **Row 10's SAM finishes with `job_counter` `0x40000000`** (item 4483,
    /// `docs/RECGAME.md` §11): run1739, the ranges take of run1730's file.
    /// On 537 the computer's SAM `1/2012` (`TypeIndex` `0x20d`, builds
    /// range 746 is its `WallData` 30) finishes, and `Wall::activate@0063e4b0`
    /// zeroes both counters, then asks the type `is(0x20b, 0)` (`63e54d`–
    /// `63e57f`, AIRDEFENSE's line) and sets `job_counter` to `0x40000000`,
    /// which no later frame of the take changes. Every builds range on 536
    /// and 537, ours against theirs, byte for byte: red on the tree before
    /// the fix at range 746, `job_counter` ours 0, theirs 1073741824.
    #[test]
    fn row_10_s_sam_finishes_with_the_air_defense_counter_on_run1739_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SECOND, 0) else {
            eprintln!("skipping: row 10's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1739/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1739's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 10's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [536i64, 537] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = crate::checksum::state::ranges(&built.sim, &look, &tribes)[1]
                .clone()
                .expect("the builds line");
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 1).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "builds ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "builds range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            // The SAM by name: its type local (range 741) and its
            // `job_counter`, the first word of `WallData`'s 30 (range 746).
            assert_eq!(&theirs[741].bytes[..], &0x20di32.to_le_bytes());
            let want: i32 = if state == 537 { 0x4000_0000 } else { 27_900 };
            assert_eq!(&theirs[746].bytes[0..4], &want.to_le_bytes(), "{state}");
        }
    }

    /// **Row 110's rider leaves an empty block where it boarded** (item
    /// 4485, `docs/COLLISION.md` §3.2): run1742, the ranges take of
    /// run1735's file. On 3809 who=1's Freighter `1/61` (TI 322) sinks at
    /// sea with the Citizen `1/11` (TI 50) aboard, and
    /// `Object::kill_contents@00645550` closes the rider. Its
    /// `Object::close@00647160` takes the off-map arm:
    /// `remove_from_inside`'s `add_to_world` paints its disc at its
    /// boarding point (543, 763), allocating world cell (33, 47)'s block
    /// (record 2853), and `remove_from_world` clears it. Theirs holds the
    /// block, empty. This crate allocated nothing: red on the tree before the
    /// fix at world range 6794 on 3809, record 2853 `block.present`,
    /// ours 0, theirs 1. Every world range on 3808, 3809 and 3810, byte for
    /// byte.
    #[test]
    fn row_110_s_rider_leaves_an_empty_block_where_it_boarded_on_run1742_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SECOND, 5) else {
            eprintln!("skipping: row 110's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1742/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1742's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 110's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        const CITIZEN_TI: i32 = 50;
        const FREIGHTER_TI: i32 = 322;
        let find = |sim: &sim::Sim, o: i16| {
            sim.units
                .iter()
                .rposition(|u| u.owner == 1 && u.index == o)
                .unwrap_or_else(|| panic!("1/{o}"))
        };
        for state in [3808i64, 3809, 3810] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let sim = &built.sim;
            let (rider, boat) = (find(sim, 11), find(sim, 61));
            assert_eq!(sim.units[rider].type_index, CITIZEN_TI, "1/11's TypeIndex");
            assert_eq!(sim.units[boat].type_index, FREIGHTER_TI, "1/61's TypeIndex");
            let at = sim::movement::collide::ucell(sim.units[rider].movement.body.pos);
            assert_eq!((at.x, at.y), (543, 763), "1/11's boarding point on {state}");
            assert!(!sim.units[rider].on_map, "1/11 off the map on {state}");
            if state == 3808 {
                assert_eq!(sim.units[rider].inside_unit, Some(boat), "1/11 aboard 1/61");
                assert!(sim.units[rider].alive() && sim.units[boat].alive());
            } else {
                assert!(
                    !sim.units[rider].alive() && !sim.units[boat].alive(),
                    "both dead on {state}"
                );
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = sim.checksum_world();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 11).collect();
            // The cell by name: record 2853 is world cell (33, 47) of the
            // 60-wide map, which holds unit cell (543, 763). Theirs
            // allocates its block on 3809 and leaves its body empty.
            let k = ours
                .ranges
                .iter()
                .position(|r| r.record == 2853 && r.field == "block.present")
                .expect("cell 2853's presence");
            let present: u8 = if state == 3808 { 0 } else { 1 };
            assert_eq!(
                theirs[k].bytes,
                [present, 0, 0, 0],
                "theirs' block on {state}"
            );
            if state != 3808 {
                assert!(
                    theirs[k + 2].bytes.iter().all(|&b| b == 0),
                    "theirs' body empty on {state}"
                );
            }
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                let at = a.bytes.iter().zip(&b.bytes).position(|(x, y)| x != y);
                assert_eq!(
                    at.map(|i| (i, a.bytes[i], b.bytes[i])),
                    None,
                    "world range {k} on {state}, record {} {}: (byte, ours, theirs)",
                    a.record,
                    a.field
                );
            }
            assert_eq!(ours.ranges.len(), theirs.len(), "world ranges on {state}");
        }
    }

    /// **Row 110's Citizen takes the civilian heal on a whole unit**
    /// (item 4539, `docs/COMBAT.md` §72.2): run1742, the ranges take of
    /// run1735's file. who=1's French Citizen `1/9` (TI 50), one point
    /// down, on 4491, where `9 + 4491` is both the supply heal's 20 and
    /// the civilian heal's 45. `Unit::process_healing@005e0670` reads
    /// `has_damage(1)` once at its head, so the civilian heal lands after
    /// the supply heal made it whole and writes `healing` 45. Red on the
    /// tree before the fix at units range 487 on 4492, record 9
    /// `myhits..launch_frames` byte 24, ours 20, theirs 45. Every units
    /// range on 4490..=4494, byte for byte.
    #[test]
    fn row_110_s_citizen_takes_the_civilian_heal_on_a_whole_unit_on_run1742_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SECOND, 5) else {
            eprintln!("skipping: row 110's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1742/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1742's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 110's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 4490i64..=4494 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 0).collect();
            let ours = built.sim.checksum_units();
            assert_eq!(ours.ranges.len(), theirs.len(), "units ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                let at = a.bytes.iter().zip(&b.bytes).position(|(x, y)| x != y);
                assert_eq!(
                    at.map(|i| (i, a.bytes[i], b.bytes[i])),
                    None,
                    "units range {k} on {state}, record {} {}: (byte, ours, theirs)",
                    a.record,
                    a.field
                );
                assert_eq!(a.bytes.len(), b.bytes.len(), "units range {k} on {state}");
            }
            if state == 4492 {
                // The Citizen by name, and theirs' `healing` (`+0x38`).
                assert_eq!(ours.ranges[487].record, 9);
                assert_eq!(ours.ranges[487].field, "myhits..launch_frames");
                assert_eq!(theirs[487].bytes[24..26], 45i16.to_le_bytes(), "healing");
                let sim = &built.sim;
                let c = sim
                    .units
                    .iter()
                    .position(|u| u.alive() && u.owner == 1 && u.index == 9)
                    .expect("1/9");
                let u = &sim.units[c];
                assert_eq!(u.type_index, 50);
                assert_eq!(u.kind.domain, sim::territory::attrition::Domain::Land);
                assert_eq!(u.healing, 45);
                assert_eq!(u.health, u.max_health, "made whole on 4491");
            }
        }
    }

    /// **Row 110's guided rounds re-plan to the Citizen's figure height**
    /// (item 4537, `docs/CHECKSUM.md` §18.1): run1742, the ranges take of
    /// run1735's file. who=2's `2/35` (TI 330, sea) has two guided rounds
    /// in slots 2 and 3 on who=1's fleeing Citizen `1/2` (TI 50), and on
    /// 4104 `Ammo::inc_time@0067d380`'s spline arm re-plans both to the
    /// Citizen's point. Their `ez` is `guys[0] +0x14`, the ground under the
    /// figure (36), where the tile's object height reads 34. Red on the
    /// tree before the fix at ammo range 1 on 4104, record 2
    /// `rolling..start_roll_angle` byte 27 (`ez`), ours 0x22, theirs 0x24.
    /// Every ammo range on 4100..=4140, byte for byte.
    #[test]
    fn row_110_s_guided_rounds_re_plan_to_the_citizen_s_figure_height_on_run1742_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SECOND, 5) else {
            eprintln!("skipping: row 110's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1742/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1742's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 110's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 4100i64..=4140 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 3).collect();
            let ours = built
                .sim
                .checksum_ammo()
                .unwrap_or_else(|| panic!("ours emits the ammo line on {state}"));
            assert_eq!(ours.ranges.len(), theirs.len(), "ammo ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                let at = a.bytes.iter().zip(&b.bytes).position(|(x, y)| x != y);
                assert_eq!(
                    at.map(|i| (i, a.bytes[i], b.bytes[i])),
                    None,
                    "ammo range {k} on {state}, record {} {}: (byte, ours, theirs)",
                    a.record,
                    a.field
                );
                assert_eq!(a.bytes.len(), b.bytes.len(), "ammo range {k} on {state}");
            }
            if state == 4104 {
                // The shooter and the target by name.
                let sim = &built.sim;
                let p = sim
                    .combat
                    .projectiles
                    .iter()
                    .find(|p| p.slot == 2)
                    .expect("slot 2's round");
                let (sim::combat::Obj::Unit(s), Some(sim::combat::Obj::Unit(t))) =
                    (p.shooter, p.target)
                else {
                    panic!("a unit's round on a unit")
                };
                let (s, t) = (&sim.units[s], &sim.units[t]);
                assert_eq!((s.owner, s.index, s.type_index), (2, 35, 330));
                assert_eq!(s.kind.domain, sim::territory::attrition::Domain::Sea);
                assert_eq!((t.owner, t.index, t.type_index), (1, 2, 50));
                assert_eq!(t.kind.domain, sim::territory::attrition::Domain::Land);
                // Record 2's `ez` (word 6, bytes 27..31): the figure's 36,
                // where the tile's object height under it is 34.
                assert_eq!(theirs[1].bytes[27..31], 36i32.to_le_bytes(), "ez");
                assert_eq!(
                    (p.landing.x, p.landing.y),
                    (21558, 42096),
                    "the Citizen's point"
                );
                assert_eq!(p.ez, 36);
                assert_eq!(
                    sim.world.object_z(p.landing.tile()),
                    34,
                    "the tile's height"
                );
            }
        }
    }

    /// **Row 10's walker steps inside a freed cell and marks nothing**
    /// (item 4504, `docs/COLLISION.md` §2.4): run1739, the ranges take of
    /// run1730's file. who=1's `1/43` (TI 104) stands at unit cell
    /// (632, 813) with its disc holed out and cell (39, 50)'s block
    /// (record 3039) freed by the sweep; on 2365 it steps to (633, 814),
    /// both discs inside that one cell, and `CollCheck::move_unit@00682ad0`'s
    /// one-block form finds the block null (`WorldData::get_coll_block@006b5350`
    /// only reads) and returns before either pass. This crate ran the set
    /// pass and allocated the block. Every world range on 2364, 2365 and
    /// 2366, byte for byte: red on the tree before the fix at range 6920 on
    /// 2365, record 3039 `block.present`, ours 1, theirs 0.
    #[test]
    fn row_10_s_walker_steps_inside_a_freed_cell_and_marks_nothing_on_run1739_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SECOND, 0) else {
            eprintln!("skipping: row 10's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1739/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1739's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 10's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [2364i64, 2365, 2366] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_world();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 11).collect();
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "world range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            assert_eq!(ours.ranges.len(), theirs.len(), "world's ranges on {state}");
            // The cell and the walker by name: record 3039 is world cell
            // (39, 50) on the 60-wide map, and `1/43` steps into it on 2365.
            let k = ours
                .ranges
                .iter()
                .position(|r| r.record == 3039 && r.field == "block.present")
                .expect("cell 3039's presence");
            let present: u8 = if state == 2366 { 1 } else { 0 };
            assert_eq!(
                theirs[k].bytes,
                [present, 0, 0, 0],
                "theirs' block on {state}"
            );
            let u = built
                .sim
                .units
                .iter()
                .find(|u| u.owner == 1 && u.index == 43)
                .expect("1/43");
            assert_eq!(u.type_index, 104);
            let at = sim::movement::collide::ucell(u.pos);
            let want = match state {
                2364 => (632, 813),
                2365 => (633, 814),
                _ => (634, 815),
            };
            assert_eq!((at.x, at.y), want, "1/43's unit cell on {state}");
        }
    }

    /// **Row 10's Citizen over a peaceful border carries the attrition
    /// bits** (item 4514, `docs/CHECKSUM.md` §18.3): run1739, the ranges take
    /// of run1730's file. who=1's `1/6` (TI 50) stands on who=0's ground,
    /// the two at peace, and its refresh on 3771 takes the peace period, 8.
    /// `Unit::process_attrition@005e11a0` clears `unit_masks` `0x400080` on
    /// entry and sets both bits on the peace and assassin arms (`0x80`
    /// alone on the war arm's period), which this crate kept as
    /// `Unit::attrition` and `Unit::ignores_supply` and never emitted. Every
    /// units range on 3770 and 3771, byte for byte: red on the tree before
    /// the fix at range 467, record 6, `collide_frame..play` byte 32,
    /// `unit_masks` ours `0x840008`, theirs `0xc40088`.
    #[test]
    fn row_10_s_citizen_over_a_peaceful_border_carries_the_attrition_bits_on_run1739_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SECOND, 0) else {
            eprintln!("skipping: row 10's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1739/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1739's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 10's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [3770i64, 3771] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_units();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 0).collect();
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "units range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            assert_eq!(ours.ranges.len(), theirs.len(), "units ranges on {state}");
            // The walker by name: `1/6` on who=0's ground at peace, and
            // theirs' `unit_masks` (bytes 32..36 of range 467).
            let u = built
                .sim
                .units
                .iter()
                .find(|u| u.owner == 1 && u.index == 6)
                .expect("1/6");
            assert_eq!(u.type_index, 50);
            assert_eq!(built.sim.world.owner_at(u.pos), sim::Owner::Player(0));
            assert!(!built.sim.leaders.at_war[1][0], "1 and 0 at peace");
            assert_eq!(u.attrition, if state == 3771 { 8 } else { 0 }, "{state}");
            if state == 3771 {
                assert_eq!(ours.ranges[467].record, 6);
                assert_eq!(&theirs[467].bytes[32..36], &0x00c4_0088u32.to_le_bytes());
            }
        }
    }

    /// **Row 10's Citizen casting the transport is an `xport` peasant**
    /// (item 4516, `docs/AI.md` §58): run1739, the ranges take of run1730's
    /// file. who=1's Citizen `1/97` (TI 50) holds `[CAST_SPELL, MOVE_TO]` on
    /// 3975, and the census of the tick into 3976 counts it:
    /// `Leader::plan_strategy@006b9620`'s on-map arm for kinds 8 and 0xe
    /// bumps `xport_peasants` (`incl 0x9c0(%ebx)` at `6bae19`) whatever its
    /// front order, where this crate's census returned on the kind. Every
    /// leaders range on 3975 to 3977, byte for byte: red on the tree before
    /// the fix at range 129, record 1, `who..ally_mask` byte 2488,
    /// `xport_peasants` ours 2, theirs 3.
    #[test]
    fn row_10_s_citizen_casting_the_transport_is_an_xport_peasant_on_run1739_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SECOND, 0) else {
            eprintln!("skipping: row 10's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1739/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1739's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 10's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [3975i64, 3976, 3977] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            if state == 3975 {
                // The caster by name: `1/97`, a Citizen, the cast in front.
                let u = built
                    .sim
                    .units
                    .iter()
                    .find(|u| u.alive() && u.owner == 1 && u.index == 97)
                    .expect("1/97");
                assert_eq!(u.type_index, 50);
                let kinds: Vec<u8> = u.orders.iter().map(|o| o.index()).collect();
                assert_eq!(kinds, [14, 1], "CAST_SPELL, then MOVE_TO");
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_leaders();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 7).collect();
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                let at = a.bytes.iter().zip(&b.bytes).position(|(x, y)| x != y);
                assert_eq!(
                    at.map(|i| (i, a.bytes[i], b.bytes[i])),
                    None,
                    "leaders range {k} on {state}, record {} {}: (byte, ours, theirs)",
                    a.record,
                    a.field
                );
                assert_eq!(a.bytes.len(), b.bytes.len(), "leaders range {k} on {state}");
            }
            assert_eq!(ours.ranges.len(), theirs.len(), "leaders ranges on {state}");
            if state == 3976 {
                // Theirs' `xport_peasants` (`+0x9c0`, byte 2488 of who=1's
                // `who..ally_mask`).
                assert_eq!(ours.ranges[129].record, 1);
                assert_eq!(&theirs[129].bytes[2488..2492], &3i32.to_le_bytes());
            }
        }
    }

    /// **Row 10's Attack Helicopters set apart keep their `dest_angle`**
    /// (item 4522, `docs/RECGAME.md` §11): run1739, the ranges take of
    /// run1730's file. who=1's Attack Helicopter `1/140` (TI 311, air) on
    /// `GROUP_ATTACK_TO` is set apart from its pair on 4880 by
    /// `Unit::work@0060d180`'s tail, each through
    /// `Unit::set_new_location(x, y, 1, 1)`: `Guy::set_angle(guy 0, +0x50,
    /// 1)` writes the figure's `angle` and `last_angle` and leaves the
    /// unit's `dest_angle` (`+0x58`) as it was. Every units and guys range
    /// on 4879 and 4880, byte for byte: red on the tree before the fix at
    /// units range 7447, record 140, `collide_frame..play` byte 18,
    /// `dest_angle` ours 555024384, theirs −1956511744.
    #[test]
    fn row_10_s_helicopters_set_apart_keep_their_dest_angle_on_run1739_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SECOND, 0) else {
            eprintln!("skipping: row 10's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1739/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1739's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 10's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [4879i64, 4880] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            if state == 4879 {
                // The helicopter by name: `1/140`, TI 311, on its group's
                // attack-move.
                let u = built
                    .sim
                    .units
                    .iter()
                    .find(|u| u.alive() && u.owner == 1 && u.index == 140)
                    .expect("1/140");
                assert_eq!(u.type_index, 311);
                assert_eq!(u.kind.domain, sim::territory::attrition::Domain::Air);
                let kinds: Vec<u8> = u.orders.iter().map(|o| o.index()).collect();
                assert_eq!(kinds, [21], "GROUP_ATTACK_TO");
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            for (line, ours) in [
                (0u8, built.sim.checksum_units()),
                (6u8, built.sim.checksum_guys()),
            ] {
                let theirs: Vec<_> = all.iter().filter(|r| r.line == line).collect();
                for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                    let at = a.bytes.iter().zip(&b.bytes).position(|(x, y)| x != y);
                    assert_eq!(
                        at.map(|i| (i, a.bytes[i], b.bytes[i])),
                        None,
                        "{} range {k} on {state}, record {} {}: (byte, ours, theirs)",
                        ours.name,
                        a.record,
                        a.field
                    );
                    assert_eq!(a.bytes.len(), b.bytes.len(), "{} range {k}", ours.name);
                }
                assert_eq!(
                    ours.ranges.len(),
                    theirs.len(),
                    "{} ranges on {state}",
                    ours.name
                );
                if state == 4880 {
                    let want: &[(usize, usize, i32)] = if line == 0 {
                        // `dest_angle` (`+0x58`, bytes 16..20).
                        &[(7447, 16, -1_956_511_744)]
                    } else {
                        // Guy 0's `last_angle` (`+0x1c`, bytes 20..24).
                        &[(1030, 20, 555_024_384)]
                    };
                    for &(k, at, v) in want {
                        assert_eq!(ours.ranges[k].record, 140);
                        assert_eq!(&theirs[k].bytes[at..at + 4], &v.to_le_bytes());
                    }
                }
            }
        }
    }

    /// **Row 10's Jet Fighter shot down falls as the ammo line's record**
    /// (item 4523, `docs/RECGAME.md` §11): run1739, the ranges take of
    /// run1730's file. who=1's Jet Fighter `1/114` (TI 295, air) dies on
    /// `AIR_PATROL` on 5177, and `Ammo::init_crash@0067b800` lays its
    /// round in slot 0. Theirs prints it from 5178, where ours emitted no
    /// line. Every ammo range on 5176..=5200, byte for byte, and on 5178
    /// theirs' `rolling` −2 (`+0x5`), `dx` 93.18 (`+0x58`, the landing's
    /// first drift), `bank_dx`/`bank_dy` −8.0 and −17.0 and
    /// `start_roll_angle` 0 (`+0x64`, guy 0's bank truncated).
    #[test]
    fn row_10_s_jet_fighter_shot_down_falls_as_the_ammo_line_s_record_on_run1739_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SECOND, 0) else {
            eprintln!("skipping: row 10's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1739/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1739's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 10's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 5176i64..=5200 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 3).collect();
            let ours = built
                .sim
                .checksum_ammo()
                .unwrap_or_else(|| panic!("ours emits the ammo line on {state}"));
            assert_eq!(ours.ranges.len(), theirs.len(), "ammo ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                let at = a.bytes.iter().zip(&b.bytes).position(|(x, y)| x != y);
                assert_eq!(
                    at.map(|i| (i, a.bytes[i], b.bytes[i])),
                    None,
                    "ammo range {k} on {state}, record {} {}: (byte, ours, theirs)",
                    a.record,
                    a.field
                );
                assert_eq!(a.bytes.len(), b.bytes.len(), "ammo range {k} on {state}");
            }
            if state == 5178 {
                // The crash, by name: `1/114`, TI 295.
                let p = built.sim.combat.projectiles[0];
                let sim::combat::Obj::Unit(u) = p.shooter else {
                    panic!("a unit's round")
                };
                let u = &built.sim.units[u];
                assert_eq!((u.owner, u.index, u.type_index), (1, 114, 295));
                assert_eq!(u.kind.domain, sim::territory::attrition::Domain::Air);
                assert!(p.crash.is_some());
                // Record 0's body: `rolling` at byte 0, then word k at
                // 3 + 4k: `dx` (20) at 83..87, the bank pair at 87..95,
                // `start_roll_angle` at 95..99.
                let body = &theirs[1].bytes;
                assert_eq!(body[0], (-2i8) as u8, "rolling");
                assert_eq!(&body[83..87], &0x42ba_5e0du32.to_le_bytes(), "dx");
                assert_eq!(&body[87..91], &0xc100_0000u32.to_le_bytes(), "bank_dx");
                assert_eq!(&body[91..95], &0xc188_0000u32.to_le_bytes(), "bank_dy");
                assert_eq!(&body[95..99], &[0; 4], "start_roll_angle");
            }
        }
    }

    /// **Row 130's Helicopters set apart keep their `dest_angle` too**
    /// (item 4522, `docs/RECGAME.md` §11): run1754, the ranges take of
    /// run1736's file, where 4508 left the row's units parting on 3515 at
    /// range 5857, record 184, `dest_angle` ours −406650880, theirs
    /// −424345600, who=1's Attack Helicopter `1/184` (TI 311) — the
    /// separation of row 10's 4880. Every units range on 3515, byte for
    /// byte.
    #[test]
    fn row_130_s_helicopters_set_apart_keep_their_dest_angle_on_run1754_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SECOND, 6) else {
            eprintln!("skipping: row 130's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1754/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1754's ranges record is missing");
            return;
        };
        const STATE: i64 = 3515;
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 130's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        while built.sim.frame < STATE {
            stream.apply(built.sim.frame as i32, &mut built);
            built.sim.command().tick();
        }
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        let set = record.first_of(STATE - 1).expect("the state's set").clone();
        let all = record.ranges(&set).expect("the set's ranges");
        let ours = built.sim.checksum_units();
        let theirs: Vec<_> = all.iter().filter(|r| r.line == 0).collect();
        // The helicopter by name: who=1's Attack Helicopter `1/184`, TI 311.
        let u = built
            .sim
            .units
            .iter()
            .find(|u| u.alive() && u.owner == 1 && u.index == 184)
            .expect("1/184");
        assert_eq!(u.type_index, 311);
        assert_eq!(u.kind.domain, sim::territory::attrition::Domain::Air);
        for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
            let at = a.bytes.iter().zip(&b.bytes).position(|(x, y)| x != y);
            assert_eq!(
                at.map(|i| (i, a.bytes[i], b.bytes[i])),
                None,
                "units range {k} on {STATE}, record {} {}: (byte, ours, theirs)",
                a.record,
                a.field
            );
        }
        assert_eq!(ours.ranges.len(), theirs.len(), "units ranges on {STATE}");
        assert_eq!(ours.ranges[5857].record, 184);
        assert_eq!(
            &theirs[5857].bytes[16..20],
            &(-424_345_600i32).to_le_bytes()
        );
    }

    /// **Row 130's Kremlin spy carries `unit_masks2 & 0x400`** (item 4524,
    /// `docs/RECGAME.md` §11): who=1's Kremlin trains its spy `1/169` on
    /// 4583, and `Build::process@0061edf0:320` ORs the tag into the unit's
    /// `+0x6c`. This crate held it in a side bool the units line never
    /// printed (units range 4183, record 169, `unit_masks2` ours 0, theirs
    /// 1024). Every units range on 4583 and 4584 against run1754's record,
    /// byte for byte.
    #[test]
    fn row_130_s_kremlin_spy_carries_unit_masks2_0x400_on_run1754_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SECOND, 6) else {
            eprintln!("skipping: row 130's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1754/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1754's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 130's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        const SPY_TI: i32 = 58;
        for state in [4583, 4584] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_units();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 0).collect();
            // The spy by name: who=1's Spy `1/169`.
            let u = built
                .sim
                .units
                .iter()
                .find(|u| u.alive() && u.owner == 1 && u.index == 169)
                .expect("1/169");
            assert_eq!(
                (u.type_index, u.kind.domain),
                (SPY_TI, sim::territory::attrition::Domain::Land),
                "1/169's TypeIndex and domain"
            );
            assert_eq!(u.unit_masks2 & sim::combat::umask2::KREMLIN_SPY, 0x400);
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                let at = a.bytes.iter().zip(&b.bytes).position(|(x, y)| x != y);
                assert_eq!(
                    at.map(|i| (i, a.bytes[i], b.bytes[i])),
                    None,
                    "units range {k} on {state}, record {} {}: (byte, ours, theirs)",
                    a.record,
                    a.field
                );
            }
            assert_eq!(ours.ranges.len(), theirs.len(), "units ranges on {state}");
        }
    }

    /// **Row 30's decoy member lights its disc where it is seated** (item
    /// 4509, `docs/GOLDEN.md` §72): on 1662 who=1's General casts a decoy
    /// squad of three TI 100 figures, and `Objects::init_unit` seats the
    /// member `1/37` 144 east of its captain `1/36`, across a fog cell
    /// (x 90 -> 91) but inside one `p >> 8` column.
    /// `Unit::set_new_location@005f8d20` calls `Object::update_seen(0)`
    /// (vslot `+0x174`) on the fog crossing, lighting `seen` on (94, 87) to
    /// (97, 91) for who=1. Every world range on 1661 to 1663 agrees.
    #[test]
    fn row_30_s_decoy_member_lights_its_disc_on_run1755_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SECOND, 1) else {
            eprintln!("skipping: row 30's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1755/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1755's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 30's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        const DECOY_TI: i32 = 100;
        for state in [1661, 1662, 1663] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_world();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 11).collect();
            if state >= 1662 {
                // The member by name: who=1's decoy `1/37`, east of `1/36`.
                let find = |o: i16| {
                    built
                        .sim
                        .units
                        .iter()
                        .find(|u| u.alive() && u.owner == 1 && u.index == o)
                        .unwrap_or_else(|| panic!("1/{o}"))
                };
                let (m, c) = (find(37), find(36));
                assert_eq!(
                    (m.type_index, m.kind.domain, m.decoy),
                    (DECOY_TI, sim::territory::attrition::Domain::Land, true),
                    "1/37's TypeIndex, domain and decoy bit"
                );
                assert_eq!(
                    (m.pos.x - c.pos.x, m.pos.y - c.pos.y),
                    (144, 0),
                    "seated east"
                );
                assert_eq!(
                    (m.pos.x / 0x180, c.pos.x / 0x180),
                    (91, 90),
                    "across a fog cell"
                );
                assert_eq!(m.pos.x >> 8, c.pos.x >> 8, "inside one p >> 8 column");
            }
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                let at = a.bytes.iter().zip(&b.bytes).position(|(x, y)| x != y);
                assert_eq!(
                    at.map(|i| (i, a.bytes[i], b.bytes[i])),
                    None,
                    "world range {k} on {state}, record {} {}: (byte, ours, theirs)",
                    a.record,
                    a.field
                );
            }
            assert_eq!(ours.ranges.len(), theirs.len(), "world ranges on {state}");
        }
    }

    /// **Row 30's Freighter heads its cell once more as it dies into its
    /// oil platform** (item 4530, `docs/COLLISION.md` §3.2): on 3184
    /// who=1's Transport Freighter `1/90` (TI 322, sea), carrying the
    /// Citizen `1/45` (TI 50), reaches the platform it built and dies into
    /// it. `Object::close@00647160` finds it off the map and calls
    /// `Object::remove_from_inside@006480f0`, which ends in
    /// `Object::add_to_world`: the boat heads its own cell, over the
    /// Freighter `1/121` standing there, and `remove_from_world` takes it
    /// off again, leaving `1/121`'s `up_who` 1 (units range 4108, record
    /// 121, ours -1 before). Every units range on 3183 to 3185 agrees.
    #[test]
    fn row_30_s_freighter_heads_its_cell_as_it_dies_into_its_platform_on_run1755_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SECOND, 1) else {
            eprintln!("skipping: row 30's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1755/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1755's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 30's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        const FREIGHTER_TI: i32 = 322;
        const CITIZEN_TI: i32 = 50;
        let find = |sim: &sim::Sim, o: i16| {
            sim.units
                .iter()
                .rposition(|u| u.owner == 1 && u.index == o)
                .unwrap_or_else(|| panic!("1/{o}"))
        };
        for state in [3183, 3184, 3185] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let (boat, other, rider) = (
                find(&built.sim, 90),
                find(&built.sim, 121),
                find(&built.sim, 45),
            );
            let sim = &built.sim;
            assert_eq!(
                (sim.units[boat].type_index, sim.units[boat].kind.domain),
                (FREIGHTER_TI, sim::territory::attrition::Domain::Sea),
                "1/90's TypeIndex and domain"
            );
            assert_eq!(
                sim.units[other].type_index, FREIGHTER_TI,
                "1/121's TypeIndex"
            );
            assert_eq!(sim.units[rider].type_index, CITIZEN_TI, "1/45's TypeIndex");
            if state == 3183 {
                assert!(sim.units[boat].alive(), "1/90 alive on 3183");
                assert_eq!(sim.units[rider].inside_unit, Some(boat), "1/45 aboard 1/90");
                assert_eq!(
                    (sim.units[boat].pos.cell(), sim.unit_chain_words(boat)),
                    (sim.units[other].pos.cell(), (121, -1, 0, 1)),
                    "1/90 under 1/121 on one cell"
                );
            } else {
                assert!(!sim.units[boat].alive(), "1/90 gone on {state}");
                assert_eq!(
                    sim.unit_chain_words(other),
                    (-1, -1, 0, 1),
                    "1/121's four words on {state}"
                );
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_units();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 0).collect();
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                let at = a.bytes.iter().zip(&b.bytes).position(|(x, y)| x != y);
                assert_eq!(
                    at.map(|i| (i, a.bytes[i], b.bytes[i])),
                    None,
                    "units range {k} on {state}, record {} {}: (byte, ours, theirs)",
                    a.record,
                    a.field
                );
            }
            assert_eq!(ours.ranges.len(), theirs.len(), "units ranges on {state}");
        }
    }

    /// **Row 30's Anti-Aircraft Gun repaints its disc as it casts its
    /// transport** (item 4532, `docs/COLLISION.md` §2.2): on 3790 who=1's
    /// `1/114` (TI 281, `BLOCK_RADIUS` 2), standing at unit cell (653, 871),
    /// casts the Transport Freighter `1/152` (TI 322) and boards it in its
    /// own work. `Unit::process@00610bc0` still runs `Guy::process` over
    /// its figure after the think, and `Guy::process@005e0230`'s tail asks
    /// nothing of the unit: on `(frame + o) % 64 == 0` (3790 + 114 = 64 ×
    /// 61) a standing figure re-marks its whole disc, the one
    /// `remove_from_world` has just cleared. World cell (40, 54) (record
    /// 3280) holds the full 5×5 at x 11..15, y 5..9 in theirs from 3791;
    /// this crate cleared it (`block.body` byte 22 ours 0x00, theirs 0xe0).
    /// Every world range on 3790 to 3792 agrees.
    #[test]
    fn row_30_s_aa_gun_repaints_its_disc_as_it_boards_its_transport_on_run1755_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SECOND, 1) else {
            eprintln!("skipping: row 30's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1755/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1755's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 30's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        const AAGUN_TI: i32 = 281;
        const FREIGHTER_TI: i32 = 322;
        let find = |sim: &sim::Sim, o: i16| {
            sim.units
                .iter()
                .rposition(|u| u.owner == 1 && u.index == o)
                .unwrap_or_else(|| panic!("1/{o}"))
        };
        for state in [3790i64, 3791, 3792] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let gun = find(&built.sim, 114);
            let sim = &built.sim;
            assert_eq!(sim.units[gun].type_index, AAGUN_TI, "1/114's TypeIndex");
            let at = sim::movement::collide::ucell(sim.units[gun].pos);
            assert_eq!((at.x, at.y), (653, 871), "1/114's unit cell on {state}");
            if state == 3790 {
                assert!(sim.units[gun].on_map, "1/114 on the map on 3790");
                assert_eq!((3790 + 114) % 64, 0, "3790 is 1/114's repaint frame");
            } else {
                let boat = find(&built.sim, 152);
                assert_eq!(
                    sim.units[boat].type_index, FREIGHTER_TI,
                    "1/152's TypeIndex"
                );
                assert_eq!(sim.units[gun].inside_unit, Some(boat), "1/114 aboard 1/152");
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_world();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 11).collect();
            // The cell by name: record 3280 is world cell (40, 54) of the
            // 60-wide map; words 11..15 of its body are unit columns
            // x 11..15, bits y. Theirs holds the gun's disc (y 5..9) whole
            // from 3791, over the two columns' y 11..14 another unit keeps.
            let k = ours
                .ranges
                .iter()
                .position(|r| r.record == 3280 && r.field == "block.body")
                .expect("cell 3280's body");
            let words: Vec<u16> = theirs[k].bytes[22..32]
                .chunks(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            let want: [u16; 5] = if state == 3790 {
                [0x7be0, 0x7800, 0, 0, 0]
            } else {
                [0x7be0, 0x7be0, 0x03e0, 0x03e0, 0x03e0]
            };
            assert_eq!(words, want, "theirs' columns x 11..15 on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                let at = a.bytes.iter().zip(&b.bytes).position(|(x, y)| x != y);
                assert_eq!(
                    at.map(|i| (i, a.bytes[i], b.bytes[i])),
                    None,
                    "world range {k} on {state}, record {} {}: (byte, ours, theirs)",
                    a.record,
                    a.field
                );
            }
            assert_eq!(ours.ranges.len(), theirs.len(), "world ranges on {state}");
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

    /// **Row 100's Jet Fighter walks its patrol** (item 4474,
    /// `docs/CHECKSUM.md` §18): on tick 2742 the computer hands the Jet
    /// Fighter `1/91` (TI 295) in its airbase an `AIR_PATROL`, and
    /// `AirPatrolOrder::walk_data@00483ed0` hands over fourteen ranges after
    /// the node byte. Every units range on 2743: red without the walker at
    /// range 2939, record 91, where ours walked `guys.length` (`01 00 00
    /// 00`) and theirs the flags byte (`00`); theirs' `x_pos` element is
    /// range 2945.
    #[test]
    fn row_100_s_jet_fighter_walks_its_air_patrol_order_on_run1721_s_ranges() {
        let Some(theirs) = units_ranges_agree(5, "run1721", 2743, 2945) else {
            return;
        };
        assert_eq!(theirs, 3168i32.to_le_bytes(), "theirs' x_pos[0]");
    }

    /// **Row 100's Jet Fighter leaves its base through `Unit::set_angle`**
    /// (item 4474, `docs/CHECKSUM.md` §18): on tick 2743 `Unit::do_spec_anim`'s
    /// EXIT turns the plane from its stored 0x55555555 to 0, a turn past 90°,
    /// so `unit_masks & 2` flips, and `Guy::set_angle(guy 0, 0, 1)` snaps the
    /// figure's `last_angle`. Every units range on 2744: red without the
    /// flip at range 2930, record 91, `unit_masks` ours `00 00 04 00`, theirs
    /// `02 00 04 00`; every guys range: red without the snap at range 760,
    /// `last_angle` ours `55 55 55 55`, theirs 0.
    #[test]
    fn row_100_s_jet_fighter_turns_through_set_angle_at_its_airbase_on_run1721_s_ranges() {
        let Some(theirs) = units_ranges_agree(5, "run1721", 2744, 2930) else {
            return;
        };
        assert_eq!(
            &theirs[0x68 - 0x48..0x6c - 0x48],
            &0x40002i32.to_le_bytes(),
            "theirs' unit_masks"
        );
        let Some(theirs) = line_ranges_agree(5, "run1721", 2744, 6, 760) else {
            return;
        };
        assert_eq!(
            &theirs[0x1c - 0x8..0x20 - 0x8],
            &0i32.to_le_bytes(),
            "theirs' last_angle"
        );
    }

    /// **Row 100's nuke strikes its units in `Objects::find_units`' order**
    /// (item 4478, `docs/RECGAME.md` §10.3): on tick 2755 the ring reaches
    /// two Citizens (TI 50) of player 0, and the original's circle walk —
    /// the ring's cells outward, each cell's chain — takes `0/11` at
    /// (2712, 31896) before `0/3` at (3384, 31992), where the unit slots
    /// hold `0/3` first. Every deaths range on 2756: red with the slot
    /// order at range 1, record 0, `x` ours `38 0d`, theirs `98 0a`.
    #[test]
    fn row_100_s_nuke_strikes_its_citizens_in_find_units_order_on_run1721_s_ranges() {
        let Some(theirs) = line_ranges_agree(5, "run1721", 2756, 4, 1) else {
            return;
        };
        assert_eq!(&theirs[8..12], &0xa98i32.to_le_bytes(), "theirs' x");
        assert_eq!(&theirs[24..28], &11i32.to_le_bytes(), "theirs' o");
    }

    /// **Row 100's Jet Fighter walks its strafe** (item 4491,
    /// `docs/RECGAME.md` §10.3): on tick 2757 the computer hands the Jet
    /// Fighter `1/91` (TI 295), on its patrol, a `STRAFE` on player 0's
    /// building 2000 at (3168, 30816), first in its list, and
    /// `StrafeOrder::walk_data@0047f310` hands over six ranges after the
    /// node byte. Every units range on 2758: red without the walker, 3,952
    /// ranges against 3,958, the first unequal at range 2907, record 91,
    /// where ours walked the patrol's `order.type` (`11 00 00 00`) and
    /// theirs the strafe's flags byte (`00`); theirs' `AttackOrder` row is
    /// range 2909: `def` (−1, −1), `mandatory` 1, `new_ord` 1.
    #[test]
    fn row_100_s_jet_fighter_walks_its_strafe_order_on_run1721_s_ranges() {
        let Some(theirs) = units_ranges_agree(5, "run1721", 2758, 2909) else {
            return;
        };
        let mut want = [-1i32, -1].map(i32::to_le_bytes).concat();
        want.extend([1, 0, 0, 0, 1]);
        assert_eq!(theirs, want, "theirs' AttackOrder row");
    }

    /// Game `k`'s units line on `state`, ours against the ranges take
    /// `run`'s, every range byte for byte; returns theirs' range `range`.
    fn units_ranges_agree(k: usize, run: &str, state: i64, range: usize) -> Option<Vec<u8>> {
        line_ranges_agree(k, run, state, 0, range)
    }

    /// **Row 100's Jet Fighter fires a missile off the recycler** (item
    /// 4495, `docs/CHECKSUM.md` §18.1): on tick 3209 the strafing Jet
    /// Fighter `1/91` (TI 295) fires a guided round on `0/2000`, and
    /// `Ammo::init` lays `calc_from_dir`'s spline into the `Spline`
    /// `Recycler<Spline>::pop` hands it: the nuke's of 2617, which
    /// `Ammo::close` cleared and pushed, its arrays' sizes kept. Every
    /// ammo range on 3210: red without the walk (ours refused the line),
    /// and red at range 8, `control_verts.size`, ours 4 and theirs 16,
    /// with a fresh `Spline` for every round.
    #[test]
    fn row_100_s_jet_fighter_missile_walks_its_recycled_spline_on_run1721_s_ranges() {
        let Some(theirs) = line_ranges_agree(5, "run1721", 3210, 3, 8) else {
            return;
        };
        assert_eq!(theirs, 16i32.to_le_bytes(), "theirs' control_verts.size");
    }

    /// **Row 100's Strategic Bomber drops its slot's last accuracy** (item
    /// 4497, `docs/CHECKSUM.md` §18.1): on tick 3311 the Strategic Bomber
    /// `1/83` (TI 305, air) drops two bombs on `0/2000`. `Ammo::init`'s
    /// bomb arm writes no `accuracy` (`+0x6`), and `Objects::add_ammo`
    /// keeps one `Ammo` per pool slot whose `close` clears only `flags`,
    /// so the round in slot 2 carries the 260 its slot's last round left.
    /// Every ammo range on 3312: red at range 53, record 2, ours `00 00`
    /// and theirs `04 01` at bytes 1 and 2 of the body, with a bomb's
    /// accuracy 0.
    #[test]
    fn row_100_s_strategic_bomber_drops_its_slot_s_last_accuracy_on_run1721_s_ranges() {
        let Some(theirs) = line_ranges_agree(5, "run1721", 3312, 3, 53) else {
            return;
        };
        assert_eq!(theirs[1..3], 260i16.to_le_bytes(), "theirs' accuracy");
    }

    /// The same for line `line` (0 units, 3 ammo, 4 deaths, 6 guys).
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
            3 => built.sim.checksum_ammo().expect("the ammo line"),
            4 => built.sim.checksum_deaths().expect("the deaths line"),
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

    /// The third battery's line, from its table alone (item 4487).
    #[test]
    fn the_third_held_out_line_reads_as_pinned() {
        line_reads_as_pinned(&THIRD);
    }

    /// The fourth battery's line, from its table alone (item 4510).
    #[test]
    fn the_fourth_held_out_line_reads_as_pinned() {
        line_reads_as_pinned(&FOURTH);
    }

    /// The fifth battery's line, from its table alone (item 4527).
    #[test]
    fn the_fifth_held_out_line_reads_as_pinned() {
        line_reads_as_pinned(&FIFTH);
    }

    fn line_reads_as_pinned(b: &Battery) {
        let taken = b.scores.iter().flatten().count();
        assert_eq!(
            (b.whole_now(), taken),
            b.line,
            "{}: N of 8, T taken",
            b.label
        );
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
        let col = |now: bool| -> (usize, String, u64) {
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
            // The reach: the states a game runs before its first parting,
            // summed over the battery, of all its states (DECISIONS 147 §2).
            let (ran, of) = scores.clone().flatten().fold((0u64, 0u64), |(ran, of), s| {
                let first = s.first.iter().map(|f| f.1.0).min();
                let before = first.map_or(s.states, |f| f.max(0).cast_unsigned());
                (ran + u64::from(before), of + u64::from(s.states))
            });
            let firsts: Vec<String> = scores
                .map(|s| s.map_or("-".to_string(), |s| lowest(&s)))
                .collect();
            (n, firsts.join(" "), 100 * ran / of.max(1))
        };
        let (found, found_firsts, found_reach) = col(false);
        let (now, now_firsts, now_reach) = col(true);
        format!(
            "{}: {} of {} whole \u{b7} {} taken \u{b7} lines {found} of {} as found, \
             {now} now \u{b7} first partings {found_firsts} as found, {now_firsts} now \u{b7} \
             reach {found_reach} % of states as found, {now_reach} % now",
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

    /// **The handoff's third parity-held-out line** (item 4487): the third
    /// battery's [`queue_line`], `Parity held out, third:`, in the first's
    /// grammar. Its found half is the estimate (DECISIONS 147 §2).
    #[test]
    fn the_handoff_s_parity_held_out_third_line_is_the_pinned_scores() {
        handoff_line_is_the_pinned_scores(&THIRD);
    }

    /// **The handoff's fourth parity-held-out line** (item 4510): the
    /// fourth battery's [`queue_line`], `Parity held out, fourth:`, in the
    /// first's grammar. Its found half is the estimate (DECISIONS 148 §2).
    #[test]
    fn the_handoff_s_parity_held_out_fourth_line_is_the_pinned_scores() {
        handoff_line_is_the_pinned_scores(&FOURTH);
    }

    /// **The handoff's fifth parity-held-out line** (item 4527): the fifth
    /// battery's [`queue_line`], `Parity held out, fifth:`, in the first's
    /// grammar. Its found half is the estimate (DECISIONS 149 §3, 150 §2).
    #[test]
    fn the_handoff_s_parity_held_out_fifth_line_is_the_pinned_scores() {
        handoff_line_is_the_pinned_scores(&FIFTH);
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

/// **The third eight, as declared** (item 4487, before any file is
/// recorded; DECISIONS 147 §2): rows 5, 25, …, 145 of the same order, each
/// the human (tribe 4, slot 0) against the one computer its lobby seats,
/// `MAP_SIZE 2`, played as the lobby stands (adding seats or a team is
/// Ramon's):
///
/// | row | first capture | `MAP_STYLE` | seed | computer tribe | `DIFFICULTY` | ages |
/// |---|---|---|---|---|---|---|
/// | 5 | run1004 Great Lakes | 14 | 12345 | 20 | 4 | 0..7 |
/// | 25 | run1128 Himalayas | 9 | 12345 | 11 | 0 | 0..7 |
/// | 45 | run1231 Great Sahara | 7 | 12345 | 9 | 5 | 1..1 |
/// | 65 | run1309 Great Sahara | 7 | 12345 | 2 | 5 | 4..4 |
/// | 85 | run1371 Great Lakes | 14 | 12345 | 2 | 5 | 5..5 |
/// | 105 | run1443 East Indies | 18 | 12345 | 6 | 4 | 6..6 |
/// | 125 | run595 Islands | 18 | 12345 | 10 | 5 | 0..7 |
/// | 145 | run964 Great Sahara | 7 | 12345 | 16 | 3 | 0..7 |
///
/// Each is played to 6,000 frames with the human idle, as `run1743 + k`
/// for `HELD_OUT_THIRD[k]`.
#[cfg(test)]
pub(crate) const HELD_OUT_THIRD: [HeldOut; 8] = [
    HeldOut {
        row: 5,
        capture: "gamelog-run1004-greatlakes-american-diff4-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 14; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 4; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 20; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Abraham Lincoln",
        ],
        digest: 0x9f4f508514de4337,
    },
    HeldOut {
        row: 25,
        capture: "gamelog-run1128-himalayas-cover1-clickfree-diagonal.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 9; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 0; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 11; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; William the Conqueror",
        ],
        digest: 0x7858a61bf0e12f6e,
    },
    HeldOut {
        row: 45,
        capture: "gamelog-run1231-greatsahara-spanish-classical-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 7; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 1; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 1; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 9; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Queen Isabella",
        ],
        digest: 0x690180601cb6e020,
    },
    HeldOut {
        row: 65,
        capture: "gamelog-run1309-greatsahara-inca-enlightenment-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 7; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 4; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 4; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 2; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Emperor Huayna Capac",
        ],
        digest: 0xa2409dbb3a7fa2c6,
    },
    HeldOut {
        row: 85,
        capture: "gamelog-run1371-greatlakes-inca-industrial-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 14; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 5; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 5; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 2; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Emperor Huayna Capac",
        ],
        digest: 0xb6f310c48daebaa2,
    },
    HeldOut {
        row: 105,
        capture: "gamelog-run1443-eastindies-roman-modern-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 18; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 4; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 6; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 6; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 6; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Emperor Marcus Aurelius",
        ],
        digest: 0x0e4f4fbb1c93c256,
    },
    HeldOut {
        row: 125,
        capture: "gamelog-run595-islands-french-toughest-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 18; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 10; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Emperor Napoleon",
        ],
        digest: 0xe6130c5d377c62f6,
    },
    HeldOut {
        row: 145,
        capture: "gamelog-run964-greatsahara-korean-diff3-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 7; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 3; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 16; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; King Jinheung",
        ],
        digest: 0x3bee6b1c485bb51d,
    },
];

/// **The fourth eight, as declared** (item 4510, before any file is
/// recorded; DECISIONS 148 §2): rows 15, 35, …, 155 of the same order, each
/// the human (tribe 4, slot 0) against the one computer its lobby seats,
/// `MAP_SIZE 2`, played as the lobby stands (adding seats or a team is
/// Ramon's):
///
/// | row | first capture | `MAP_STYLE` | seed | computer tribe | `DIFFICULTY` | ages |
/// |---|---|---|---|---|---|---|
/// | 15 | run1118 Southwest Mesa | 10 | 12345 | 11 | 0 | 0..7 |
/// | 35 | run1193 Great Sahara | 7 | 12345 | 6 | 5 | 1..1 |
/// | 55 | run1272 East Indies | 18 | 12345 | 22 | 5 | 2..2 |
/// | 75 | run1339 Great Lakes | 14 | 12345 | 13 | 5 | 3..3 |
/// | 95 | run1400 East Indies | 18 | 12345 | 13 | 4 | 7..7 |
/// | 115 | run1525b Southwest Mesa | 10 | 12345 | 19 | 2 | 0..7 |
/// | 135 | run895 Great Sahara | 7 | 12345 | 0 | 2 | 0..7 |
/// | 155 | run988 Great Sahara | 7 | 12345 | 19 | 2 | 0..7 |
///
/// Each is played to 6,000 frames with the human idle, as `run1760 + k`
/// for `HELD_OUT_FOURTH[k]`.
#[cfg(test)]
pub(crate) const HELD_OUT_FOURTH: [HeldOut; 8] = [
    HeldOut {
        row: 15,
        capture: "gamelog-run1118-southwestmesa-cover1-clickfree-diagonal.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 10; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 0; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 11; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; William the Conqueror",
        ],
        digest: 0xadc93c529561b286,
    },
    HeldOut {
        row: 35,
        capture: "gamelog-run1193-greatsahara-roman-classical-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 7; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 1; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 1; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 6; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Emperor Marcus Aurelius",
        ],
        digest: 0xfc723da88040b447,
    },
    HeldOut {
        row: 55,
        capture: "gamelog-run1272-eastindies-dutch-medieval-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 18; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 2; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 2; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 22; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Beatrix Wilhelmina Armgard",
        ],
        digest: 0x495da748ca8eca83,
    },
    HeldOut {
        row: 75,
        capture: "gamelog-run1339-greatlakes-russian-gunpowder-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 14; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 3; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 3; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 13; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Tsar Ivan the Terrible",
        ],
        digest: 0x818b52a4edfd1702,
    },
    HeldOut {
        row: 95,
        capture: "gamelog-run1400-eastindies-russian-information-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 18; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 4; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 7; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 13; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Tsar Ivan the Terrible",
        ],
        digest: 0xc31dd4fa7fcdbeaf,
    },
    HeldOut {
        row: 115,
        capture: "gamelog-run1525b-southwestmesa-lakota-diff2-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 10; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 2; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 19; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Sitting Bull",
        ],
        digest: 0xd56ba04094546c0f,
    },
    HeldOut {
        row: 135,
        capture: "gamelog-run895-greatsahara-aztec-diff2-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 7; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 2; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 0; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Emperor Moctezuma",
        ],
        digest: 0x898d6b50a323e659,
    },
    HeldOut {
        row: 155,
        capture: "gamelog-run988-greatsahara-lakota-diff2-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 7; MAP_SIZE 2; PLAYERS 1; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 2; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 19; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Sitting Bull",
        ],
        digest: 0x8cc04d2712f00d75,
    },
];

/// **The fifth declaration** (item 4527; DECISIONS 149 §3, 150 §2): rows
/// 3, 23, 43, 63, 83, 103, 123 and 143, each the page's block of its row's
/// lobby with [`HELD_OUT_FIFTH_SEATS`]'s seat and team. Played to 6,000
/// frames with the human idle, as `run1773 + k` for `HELD_OUT_FIFTH[k]`.
#[cfg(test)]
pub(crate) const HELD_OUT_FIFTH: [HeldOut; 8] = [
    HeldOut {
        row: 3,
        capture: "gamelog-run1000-greatsahara-german-diff3-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 7; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 3; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 0; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 12; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Alaric the Visigoth",
            "flags 1; tribe 0; who 2; team 0; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; Emperor Acamapichtli",
        ],
        digest: 0x588ac47a0bfc1732,
    },
    HeldOut {
        row: 23,
        capture: "gamelog-run1126-atlanticseapower-cover1-clickfree-diagonal.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 20; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 0; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 11; who 1; team 1; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; William the Conqueror",
            "flags 1; tribe 1; who 2; team 1; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; Pacal",
        ],
        digest: 0xf7a10a7a8581a213,
    },
    HeldOut {
        row: 43,
        capture: "gamelog-run1227-greatsahara-iroquois-ancient-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 7; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 1; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 0; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 18; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Seneca",
            "flags 1; tribe 2; who 2; team 0; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; Emperor Manco Capac",
        ],
        digest: 0x1475a20a1109de13,
    },
    HeldOut {
        row: 63,
        capture: "gamelog-run1290-eastindies-iroquois-enlightenment-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 18; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 4; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 4; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 18; who 1; team 1; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Seneca",
            "flags 1; tribe 3; who 2; team 1; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; King Shaka",
        ],
        digest: 0xcb9cd01b62261c3a,
    },
    HeldOut {
        row: 83,
        capture: "gamelog-run1367-greatlakes-spanish-industrial-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 14; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 5; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 5; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 0; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 9; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Queen Isabella",
            "flags 1; tribe 5; who 2; team 0; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; Pericles",
        ],
        digest: 0x95b9b6cbf08e8842,
    },
    HeldOut {
        row: 103,
        capture: "gamelog-run1430-eastindies-chinese-modern-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 18; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 4; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 6; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 6; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 14; who 1; team 1; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Emperor Wu Di",
            "flags 1; tribe 6; who 2; team 1; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; Julius Caesar",
        ],
        digest: 0x442cbecb1d8bf212,
    },
    HeldOut {
        row: 123,
        capture: "gamelog-run4-gunpowder-nubian-leaders9.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 14; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 0; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 3; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 289",
        players: &[
            "flags 7; tribe 4; who 0; team 0; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 11; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; William the Conqueror",
            "flags 1; tribe 7; who 2; team 0; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; Queen Cleopatra",
        ],
        digest: 0x5ed9c8b29ad4b6c0,
    },
    HeldOut {
        row: 143,
        capture: "gamelog-run960-greatsahara-japanese-diff1-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 7; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 1; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 15; who 1; team 1; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Shogun Takauji Ashikaga",
            "flags 1; tribe 8; who 2; team 1; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; Suleiman the Magnificent",
        ],
        digest: 0xc5fb64925d2ada9f,
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
        // Moved by item 4473 (`Army::add_unit`'s push runs `equals_group`'s
        // normalize and `get_num` on the last slot): groups 4,964 -> 4,974,
        // whole. No ranges take of this row; the leaf is row 60's.
        // Moved by item 4480 (a `DETECT` building lights `seen3` over its
        // whole disc, `Object::init`'s `has_objmask(0x2000000)` taken for a
        // building too): world 4,859 -> 4,974, whole (no ranges
        // take of this row; measured, not read).
        // Moved by item 4470 (`cast_transport` turns the boat with
        // `Unit::set_angle(…, 1)` whole, so its figure's `last_angle` is the
        // caster's): guys
        // 4,965 -> 4,974, whole. No ranges take of this row.
        // Moved by item 4511 (a unit host's `inside_down` chain is the order
        // of entry, `Object::insert_inside` appending at the bottom:
        // run1752, state 3359, units range 3707, record 127, who=1's
        // Freighter `1/135` holding `1/127`, `1/128`, `1/129`, `1/127`'s
        // `inside_down` ours -1, theirs 128): units 2,487 -> 4,974, whole.
        moved: Some(Score {
            states: 4974,
            agreed: [
                4974, 4974, 4974, 4974, 4974, 4974, 4974, 4974, 4974, 4974, 4974, 4974,
            ],
            first: &[],
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
        // Moved by item 4473 (`Army::add_unit`'s push runs `equals_group`'s
        // normalize and `get_num` on the last slot): groups 3,863 -> 4,075,
        // whole. On 3598 who=1's army 1 pushes `1/169` into slot 67 while
        // slot 68 is the last and empty: run1722's groups range 92, record 68,
        // `ox`/`oy` ours (33720, 15048), theirs (0, 0).
        // Moved by item 4476 (`cavarch_uid` is carried, written only on a
        // find and kept past a dropped target): units 3,788 -> 4,075, whole.
        // On 3757 the Dragoon `1/21` (TI 189, land) drops its target `0/11`
        // (`cavarch_o` 11 -> -1, `cavarch_who` 0 -> -1 on both sides):
        // run1722's units range 788, record 21, `cavarch_uid` ours 0,
        // theirs 18.
        // Moved by item 4480 (a `DETECT` building lights `seen3` over its
        // whole disc, `Object::init`'s `has_objmask(0x2000000)` taken for a
        // building too): world 3,984 -> 4,075, whole; the game is
        // whole. On 3984 who=1's Observation Post `1/59` (type 108, LOS 9) at
        // fog cell (91, 79) lights its disc: run1722's world range 3627,
        // `seen3`, 433 bytes ours 0x00, theirs 0x02, x 80..102, y 68..90.
        moved: Some(Score {
            states: 4075,
            agreed: [4075; 12],
            first: &[],
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
        // Moved by item 4474 (the units line walks `AirPatrolOrder`'s own
        // ranges; an airbase exit turns through `Unit::set_angle`): units
        // 2,743 -> 2,977, first 2743 -> 2758; guys 3,323 -> 3,331, whole. On
        // 2743, run1721's units range 2939, record 91 (the Jet Fighter 1/91,
        // TI 295, handed a patrol on 2742): ours walked `guys.length`
        // 01 00 00 00 after the order's node, theirs the order's flags 00,
        // `waypoint` 0, `x_pos` (1, 1, -1, 0, [3168]), `y_pos` (1, 1, -1, 0,
        // [30816]), flags again and the `AirOrder` row. On 2744, units range
        // 2930, record 91, `unit_masks` read ours 0x40000, theirs 0x40002
        // (`mirror`), and guys range 760, record 91, `last_angle` ours
        // 0x55555555, theirs 0.
        // Moved by item 4478 (the nuke's ring strikes its units in
        // `Objects::find_units`' order): deaths 2,756 -> 3,331, whole. On
        // 2756, run1721's deaths ranges 1 and 3, records 0 and 1: the two
        // Citizens (TI 50) the ring killed on 2755, theirs `0/11` at
        // (2712, 31896) then `0/3` at (3384, 31992), the circle walk's
        // order; ours held them swapped, `x` ours 0x0d38, theirs 0x0a98 at
        // byte 8 of record 0, `cur_anim` 0x13 and 0x14 in slot order on both.
        // Moved by item 4491 (the units line walks `StrafeOrder`'s own
        // ranges): units 2,977 -> 3,331, whole. On 2758, run1721's units
        // range 2907, record 91 (the Jet Fighter 1/91, TI 295, handed a
        // strafe on 2757 over its patrol): ours walked the patrol's
        // `order.type` 11 00 00 00 after the strafe's node, theirs the
        // strafe's flags 00, its target (2000, 0, uid 0), `def` (-1, -1)
        // with `mandatory` 1 and `new_ord` 1, flags again, the `AirOrder`
        // row (2026, 1, 0x640) and `xx`, `yy` (3168, 30816).
        // Moved by item 4495 (a round's `Spline` comes off
        // `Recycler<Spline>` with its arrays' sizes, and the ammo line walks
        // `calc_from_dir`'s spline): ammo 3,291 -> 3,312, first 3210 ->
        // 3312. On 3210, run1721's ammo ranges 3 to 54, record 1: the Jet
        // Fighter 1/91's missile on 0/2000 (slot 1, `traj` 2), ours refused
        // the line, theirs walked `has_path` 01 and its `SplineData`:
        // degree 2, depth 15, ratio 4.0, three control points in 16, six
        // knots `0 0 0 1 1 1` in 16, thirteen points and normals in 128 --
        // the sizes of the nuke's `Spline` of 2617, popped again.
        // Moved by item 4497 (a bomb's round carries its pool slot's last
        // `accuracy`): ammo 3,312 -> 3,331, whole; the row is whole. On
        // 3312, run1721's ammo range 53, record 2: the Strategic Bomber
        // 1/83 (TI 305, air)'s bomb on 0/2000 dropped on 3311 into slot 2,
        // `accuracy` ours 0 (`00 00`), theirs 260 (`04 01`), what the
        // slot's last round left; on 3313 record 1's, ours 0, theirs 300.
        moved: Some(Score {
            states: 3331,
            agreed: [
                3331, 3331, 3331, 3331, 3331, 3331, 3331, 3331, 3331, 3331, 3331, 3331,
            ],
            first: &[],
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
/// whole (a boarder's `last_z` on 4026). Item 4480: 5 -> 6, row 60 whole
/// (a `DETECT` building's `seen3`, the Observation Post `1/59` on 3984).
/// Item 4497: 6 -> 7, row 100 whole (a bomb's round carries its pool
/// slot's last `accuracy`, the Strategic Bomber `1/83` on 3312). Item
/// 4511: 7 -> 8, row 40 whole (units 2,487 -> 4,974: a unit host's
/// `inside_down` chain is the order of entry; no ranges take of row 40, so
/// its value diff is run1752's Freighter `1/135` on 3359).
#[cfg(test)]
pub(crate) const HELD_OUT_LINE: (usize, usize) = (8, 8);

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
        // Moved by item 4474 (`AirPatrolOrder`'s ranges walked; an airbase
        // exit turns through `Unit::set_angle`): units 2,496 -> 2,712, first
        // on 1747; guys 5,963 -> 5,990, first on 2449.
        // Moved by item 4478 (the nuke's ring strikes its units in
        // `Objects::find_units`' order): deaths 5,350 -> 6,001, whole. No
        // ranges are on disk for run1730: on 4686 ours' deaths sum was
        // 4192616794 and is now theirs' 999637338.
        // Moved by item 4477 (the farmer's turn east is `Unit::set_angle`
        // whole, so a leader's reversal toggles its group's `facing`):
        // groups 3,066 -> 6,001, whole; units 2,712 -> 3,783, first on 1747.
        // Moved by item 4480 (a `DETECT` building lights `seen3`): world
        // 537 -> 5,995, its first parting 537 -> 2365.
        // Moved by item 4470 (`cast_transport` turns the boat with
        // `Unit::set_angle(…, 1)` whole, so its figure's `last_angle` is the
        // caster's): units
        // 3,783 -> 3,785, first on 1747; guys 5,990 -> 5,997, first
        // 2449 -> 4880. No ranges take of this row.
        // Moved by item 4491 (`StrafeOrder`'s ranges walked): units
        // 3,785 -> 3,914, first on 1747. No ranges take of this row.
        // Moved by item 4495 (`calc_from_dir`'s spline walked, its sizes
        // the recycled `Spline`'s): ammo 5,907 -> 5,924, first 5135 ->
        // 5148, ours refused there. No ranges take of this row.
        // Moved by item 4497 (a bomb's round carries its pool slot's last
        // `accuracy`): ammo 5,924 -> 5,962, first 5148 -> 5178, ours
        // refused there. No ranges take of this row.
        // Moved by item 4499 (`reg_xport_peasants` is indexed without a
        // bound, so a rider bound for a sea region counts in
        // `reg_gatherers`): leaders
        // 3,976 -> 5,801, first 1576 -> 3976. No ranges take of this row.
        // Moved by item 4483 (an AIRDEFENSE-line building finishes with
        // `job_counter` `0x40000000`, `Wall::activate@0063e4b0:54`): builds
        // 537 -> 6,001, whole. run1739, the ranges take: on 537 builds range
        // 746, the SAM `1/2012`'s `job_counter`, ours 0, theirs 1073741824.
        // Moved by item 4511 (a unit host's `inside_down` chain is the order
        // of entry, `Object::insert_inside` appending at the bottom:
        // run1752, state 3359, units range 3707, record 127, who=1's
        // Freighter `1/135` holding `1/127`, `1/128`, `1/129`, `1/127`'s
        // `inside_down` ours -1, theirs 128): units 3,914 -> 5,664, first
        // 1747 -> 3771.
        // Moved by item 4504 (`CollCheck::move_unit@00682ad0`'s one-block
        // form returns on a null block: run1739, state 2365, world range
        // 6920, record 3039 `block.present`, ours 1, theirs 0, who=1's
        // `1/43` stepping (632, 813) -> (633, 814) inside the freed cell
        // (39, 50)): world 5,995 -> 6,001, whole.
        // Moved by item 4514 (`unit_masks` carries the attrition period's
        // `0x80` and the peace arm's `0x400000`,
        // `Unit::process_attrition@005e11a0`: run1739, state 3771, units
        // range 467, record 6, who=1's `1/6` (TI 50) on who=0's ground at
        // peace, ours 0x840008, theirs 0xc40088): units 5,664 -> 5,792,
        // first 3771 -> 4880.
        // Moved by item 4508 (a computer's bomber searches round itself,
        // `find_new_bomber_target`'s `unit_masks & 0x40000` head), measured
        // on the tree merged with 4514: units 5,792 -> 5,979, first on 4880,
        // its sums unchanged. No ranges take of this row names which states
        // went whole.
        // Moved by item 4516 (a citizen whose action is `CAST_SPELL` or
        // `BOARD_SHIP` is an `xport` peasant, `Leader::plan_strategy@006b9620`
        // `6bae19`: run1739, state 3976, leaders range 129, record 1,
        // `xport_peasants`, ours 2, theirs 3, who=1's Citizen `1/97` (TI 50)
        // holding `[CAST_SPELL, MOVE_TO]` on 3975): leaders 5,801 -> 6,001,
        // whole.
        // Moved by item 4522 (two Helicopters set apart turn through
        // `Unit::set_new_location(x, y, 1, 1)`'s `Guy::set_angle(guy 0,
        // +0x50, 1)`, which leaves `dest_angle` and snaps `last_angle`:
        // run1739, state 4880, who=1's Attack Helicopter `1/140` (TI 311),
        // units range 7447 `dest_angle` ours 555024384, theirs -1956511744,
        // guys range 1030 `last_angle` ours 610064529, theirs 555024384):
        // units 5,979 -> 6,001 and guys 5,997 -> 6,001, whole.
        // Moved by item 4523 (a falling aircraft's round is the ammo line's
        // record: `Ammo::init_crash@0067b800`'s `rolling`, bank pair and
        // `start_roll_angle`, and `+0x58` rewritten on every drift by
        // `Ammo::inc_time`'s crash arm: run1739, state 5178, ammo record 0,
        // who=1's Jet Fighter `1/114` (TI 295) shot down on `AIR_PATROL`,
        // ours no line, theirs 3 ranges, `dx` 0x42ba5e0d where ours' birth
        // value 0x42b8749b stood): ammo 5,962 -> 6,001, whole; the row is
        // whole.
        moved: Some(Score {
            states: 6001,
            agreed: [
                6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001,
            ],
            first: &[],
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
        // Moved by item 4470 (`cast_transport` turns the boat with
        // `Unit::set_angle(…, 1)` whole, so its figure's `last_angle` is the
        // caster's): guys
        // 4,542 -> 4,548, whole; units stands on 2334, its sum there moved.
        // Moved by item 4499 (`reg_xport_peasants` is indexed without a
        // bound, so a rider bound for a sea region counts in
        // `reg_gatherers`): leaders
        // 3,748 -> 4,548, whole. No ranges take of this row.
        // Moved by item 4511 (a unit host's `inside_down` chain is the order
        // of entry, `Object::insert_inside` appending at the bottom:
        // run1752, state 3359, units range 3707, record 127, who=1's
        // Freighter `1/135` holding `1/127`, `1/128`, `1/129`, `1/127`'s
        // `inside_down` ours -1, theirs 128): units 2,334 -> 4,114, first
        // 2334 -> 3184.
        // Moved by item 4509 (a seated squad member lights its disc on a
        // fog-cell crossing, `Unit::set_new_location@005f8d20`'s `+0x174`
        // arm: run1755, state 1662, world range 3625 `seen`, fog cells
        // (94, 87) to (97, 91), ours 0x00, theirs 0x02, who=1's decoy
        // member `1/37` (TI 100) seated 144 east of its captain `1/36`):
        // world 4,423 -> 4,495, first 1662 -> 3791.
        // Moved by item 4530 (a boat that dies into its oil platform heads
        // its cell once more, `Object::remove_from_inside@006480f0`'s
        // `add_to_world` before `Object::close`'s `remove_from_world`:
        // run1755, state 3184, units range 4108, record 121, who=1's
        // Freighter `1/121` (TI 322) over `1/90`, `up_who` ours -1,
        // theirs 1): units 4,114 -> 4,530, first 3184 -> 3813.
        // Moved by item 4532 (a unit that boards in its own think still
        // takes `Guy::process`'s sixty-fourth-frame repaint, `Unit::process`
        // testing `inside_up` before the think: run1755, state 3791, world
        // range 7413, record 3280 `block.body`, who=1's Anti-Aircraft Gun
        // `1/114` (TI 281) casting its Freighter `1/152` on its phase frame,
        // byte 22 ours 0x00, theirs 0xe0): world 4,495 -> 4,548, whole;
        // units 4,530 -> 4,548, whole. The row is whole.
        moved: Some(Score {
            states: 4548,
            agreed: [
                4548, 4548, 4548, 4548, 4548, 4548, 4548, 4548, 4548, 4548, 4548, 4548,
            ],
            first: &[],
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
        // Moved by item 4473 (`Army::add_unit`'s push runs `equals_group`'s
        // normalize and `get_num` on the last slot): groups
        // 4,415 -> 4,455; the row is whole. No ranges take of this row.
        moved: Some(Score {
            states: 4455,
            agreed: [
                4455, 4455, 4455, 4455, 4455, 4455, 4455, 4455, 4455, 4455, 4455, 4455,
            ],
            first: &[],
            over: None,
        }),
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
        // Moved by item 4480 (a `DETECT` building lights `seen3`): world
        // 5,054 -> 5,141, whole; the game is whole (no ranges take of this
        // row; measured, not read).
        moved: Some(Score {
            states: 5141,
            agreed: [5141; 12],
            first: &[],
            over: None,
        }),
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
        // Moved by item 4470 (`cast_transport` turns the boat with
        // `Unit::set_angle(…, 1)` whole, so its figure's `last_angle` is the
        // caster's): units
        // 6,000 -> 6,001 and guys 6,000 -> 6,001; the row is whole. run1738,
        // the ranges take of this file, on 4021: units range 498 and guys
        // range 150, the barge's figure (record 14, type 320), `last_angle`
        // ours 1431655765 (`Angle::INITIAL`), theirs -609026048.
        moved: Some(Score {
            states: 6001,
            agreed: [
                6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001,
            ],
            first: &[],
            over: None,
        }),
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
        // Moved by item 4473 (`Army::add_unit`'s push runs `equals_group`'s
        // normalize and `get_num` on the last slot): groups
        // 3,544 -> 3,553, first 1608 -> 3553.
        // Moved by item 4477 (the farmer's turn east is `Unit::set_angle`
        // whole, so a leader's reversal toggles its group's `facing`):
        // groups 3,553 -> 6,001, whole.
        // Moved by item 4480 (a `DETECT` building lights `seen3`): world
        // 3,999 -> 5,379, its first parting on 3809.
        // Moved by item 4470 (`cast_transport` turns the boat with
        // `Unit::set_angle(…, 1)` whole, so its figure's `last_angle` is the
        // caster's): units
        // 2,702 -> 2,703, first on 2184; guys 6,000 -> 6,001, whole.
        // Moved by item 4495 (`calc_from_dir`'s spline walked, its sizes
        // the recycled `Spline`'s): ammo 4,472 -> 5,826, first 3303 ->
        // 4104, ours refused there. No ranges take of this row.
        // Moved by item 4499 (`reg_xport_peasants` is indexed without a
        // bound, so a rider bound for a sea region counts in
        // `reg_gatherers`): leaders
        // 4,995 -> 5,995, first 3176 -> 5344. No ranges take of this row.
        // Moved by item 4483 (an AIRDEFENSE-line building finishes with
        // `job_counter` `0x40000000`): builds 4,221 -> 5,577, first 4221 ->
        // 5577. No ranges take of this row.
        // Moved by item 4502 (the CEO bit, `unit_masks2 & 0x10000`, raised
        // at birth and by `set_type` on a type that `is(0x165, 1)`): units
        // 2,703 -> 5,017, first on 2184. No ranges take of this row.
        // Moved by item 4511 (a unit host's `inside_down` chain is the order
        // of entry, `Object::insert_inside` appending at the bottom:
        // run1752, state 3359, units range 3707, record 127, who=1's
        // Freighter `1/135` holding `1/127`, `1/128`, `1/129`, `1/127`'s
        // `inside_down` ours -1, theirs 128): units 5,017 -> 5,297, first
        // 2184 -> 4492.
        // Moved by item 4505 (a gatherer chain is one link a unit,
        // `Build::add_gatherer@0062f640`): units 5,297 -> 5,304, first on
        // 4492, measured on the merged tree; the seven states not read one
        // by one. No ranges take of this row.
        // Moved by item 4520 (the leaders line's `flags` carry the Kremlin's
        // spy-due bit, `leader_flags & 0x10000000`; run1754's leaf, row
        // 130): leaders 5,995 -> 6,000, first 5344 -> 5409. No ranges take
        // of this row names the five states or the one left.
        // Moved by item 4524 (the Kremlin's spy tag is `unit_masks2 &
        // 0x400`; run1754's leaf, row 130): units 5,304 -> 5,532, first on
        // 4492, its sums unmoved. No ranges take of this row.
        // Moved by item 4485 (a rider closed aboard its sunk boat takes
        // `Object::close@00647160`'s off-map arm, `remove_from_inside`'s
        // `add_to_world` painting its boarding disc and `remove_from_world`
        // clearing it: run1742, state 3809, world range 6794, record 2853
        // (cell (33, 47)) `block.present`, who=1's Citizen `1/11` (TI 50)
        // dying aboard the Freighter `1/61` (TI 322), ours 0, theirs 1):
        // world 5,379 -> 5,601, first 3809 -> 5601.
        // Moved by item 4537 (a guided round's re-plan and close test read a
        // unit target's height as `guys[0] +0x14`, the ground under its
        // figure, `Ammo::inc_time@0067d380`'s spline arm: run1742, state
        // 4104, ammo range 1, record 2 `ez`, who=2's `2/35` (TI 330)
        // re-planning on who=1's Citizen `1/2` (TI 50), ours 34, theirs 36):
        // ammo 5,826 -> 6,001, whole.
        // Moved by item 4539 (`Unit::process_healing@005e0670` reads
        // `has_damage(1)` once at its head, so the civilian heal lands on a
        // unit the supply heal just made whole: run1742, state 4492, units
        // range 487, record 9 `myhits..launch_frames` byte 24 `healing`,
        // who=1's French Citizen `1/9` (TI 50) 84 -> 85 of 85 on 4491,
        // ours 20, theirs 45): units 5,532 -> 5,577, first 4492 -> 5577.
        moved: Some(Score {
            states: 6001,
            agreed: [
                5577, 5577, 6001, 6001, 6001, 6001, 6001, 6000, 6001, 6001, 6001, 5601,
            ],
            first: &[
                ("units", (5577, Some(918696023), 1119301727)),
                ("builds", (5577, Some(3250328791), 1226052951)),
                ("leaders", (5409, Some(2436184419), 4237637983)),
                ("world", (5601, Some(3728895985), 3060364311)),
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
        // Moved by item 4473 (`Army::add_unit`'s push runs `equals_group`'s
        // normalize and `get_num` on the last slot): groups
        // 4,553 -> 4,731, whole.
        // Moved by item 4474 (`AirPatrolOrder`'s ranges walked; an airbase
        // exit turns through `Unit::set_angle`): units 1,735 -> 2,100, first
        // 1735 -> 1742; guys 4,695 -> 4,719, first 1736 -> 2066.
        // Moved by item 4480 (a `DETECT` building lights `seen3`): world
        // 553 -> 4,731, whole.
        // Moved by item 4470 (`cast_transport` turns the boat with
        // `Unit::set_angle(…, 1)` whole, so its figure's `last_angle` is the
        // caster's): guys
        // 4,719 -> 4,731, whole. No ranges take of this row.
        // Moved by item 4478 (the nuke's ring strikes its units in
        // `Objects::find_units`' order): deaths 4,084 -> 4,731, whole. No
        // ranges are on disk for run1736: on 1746 ours' deaths sum was
        // 1419733817 and is now theirs' 1134652217.
        // Moved by item 4491 (`StrafeOrder`'s ranges walked): units
        // 2,100 -> 3,034, first 1742 -> 2354. No ranges are on disk for
        // run1736: on 1742 ours' units sum was 1100206902 and is now
        // theirs' 2410470946.
        // Moved by item 4495 (`calc_from_dir`'s spline walked, its sizes
        // the recycled `Spline`'s): ammo 4,511 -> 4,647, first on 2367. No
        // ranges are on disk for run1736.
        // Moved by item 4497 (a bomb's round carries its pool slot's last
        // `accuracy`): ammo 4,647 -> 4,731, whole. No ranges are on disk
        // for run1736: on 2367 ours' ammo sum was 1123882197 and is now
        // theirs' 1189418202.
        // Moved by item 4499 (`reg_xport_peasants` is indexed without a
        // bound, so a rider bound for a sea region counts in
        // `reg_gatherers`): leaders
        // 1,976 -> 4,721, first 1376 -> 4573.
        // Moved by item 4483 (an AIRDEFENSE-line building finishes with
        // `job_counter` `0x40000000`): builds 553 -> 4,731, whole. No
        // ranges take of this row; row 10's run1739 names the leaf.
        // Moved by item 4511 (a unit host's `inside_down` chain is the order
        // of entry, `Object::insert_inside` appending at the bottom:
        // run1752, state 3359, units range 3707, record 127, who=1's
        // Freighter `1/135` holding `1/127`, `1/128`, `1/129`, `1/127`'s
        // `inside_down` ours -1, theirs 128): units 3,034 -> 4,422, first
        // 2354 -> 3411.
        // Moved by item 4514 (`unit_masks` carries the attrition period's
        // `0x80` and the peace arm's `0x400000`; run1739's leaf, row 10):
        // units 4,422 -> 4,459, first on 3411, its sums unmoved. No ranges
        // take of this row.
        // Moved by item 4508 (a computer's bomber searches round itself:
        // `find_new_bomber_target@005eb960`'s head puts the point on the
        // plane for `unit_masks & 0x40000`; run1754, state 3411, units range
        // 4078, record 158, who=1's Strategic Bomber `1/158`,
        // `orders.count` ours 2, theirs 1), measured on the tree merged with
        // 4514: units 4,459 -> 4,574, first 3411 -> 3515.
        // Moved by item 4522 (two Helicopters set apart keep `dest_angle`,
        // `Unit::set_new_location(x, y, 1, 1)`: run1754, state 3515, units
        // range 5857, record 184, who=1's Attack Helicopter `1/184` (TI 311),
        // `dest_angle` ours -406650880, theirs -424345600): units 4,574 ->
        // 4,583, first 3515 -> 4583 (range 4183, record 169, `unit_masks2`
        // ours 0, theirs 1024).
        // Moved by item 4520 (the leaders line's `flags` carry the Kremlin's
        // spy-due bit, `leader_flags & 0x10000000`; run1754, state 4573,
        // leaders range 128, record 1, `flags` byte 3, ours 0x0e, theirs
        // 0x1e): leaders 4,721 -> 4,731, whole.
        // Moved by item 4524 (the Kremlin's spy tag is `unit_masks2 &
        // 0x400`, `Build::process@0061edf0:320`; run1754, state 4583, units
        // range 4183, record 169, who=1's Spy `1/169` (TI 58), `unit_masks2`
        // ours 0, theirs 1024): units 4,583 -> 4,731, whole; row 130 whole.
        moved: Some(Score {
            states: 4731,
            agreed: [
                4731, 4731, 4731, 4731, 4731, 4731, 4731, 4731, 4731, 4731, 4731, 4731,
            ],
            first: &[],
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
/// 2026-11-20: 1 of 8, row 150 (run1737). Item 4473: 1 -> 2, row 50
/// (run1732) whole with its groups line. Item 4480: 2 -> 3, row 70
/// (run1733) whole with its world line (a `DETECT` building's `seen3`).
/// Item 4470: 3 -> 4, row 90 (run1734) whole with its units and guys
/// lines. Item 4524: 4 -> 5, row 130 (run1736) whole with its units line
/// (the Kremlin's spy, `unit_masks2 & 0x400`). Item 4523: 5 -> 6, row 10
/// (run1730) whole with its ammo line (a falling aircraft's round, who=1's
/// Jet Fighter `1/114` on 5178). Item 4532: 6 -> 7, row 30 (run1731) whole
/// with its world and units lines (a unit that boards in its own think
/// repaints its disc, who=1's Anti-Aircraft Gun `1/114` on 3790).
#[cfg(test)]
pub(crate) const HELD_OUT_SECOND_LINE: (usize, usize) = (7, 8);

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

/// **A held-out row whole now**: its `now` column ([`FirstScore::moved`]
/// where a fix has moved it, else as found) parts on no line and agrees on
/// every state of all twelve. The one predicate: each battery's line
/// ([`Battery::line`], checked by `line_reads_as_pinned`) and the
/// headline's fold (`sample::parity_line`, DECISIONS 150 §1) both count
/// with it, so the two cannot disagree on a row.
#[cfg(test)]
pub(crate) fn whole_now(p: &FirstScore) -> bool {
    let s = p.moved.unwrap_or(p.found);
    s.first.is_empty() && s.agreed == [s.states; 12]
}

#[cfg(test)]
impl Battery {
    /// The battery's rows whole now, by [`whole_now`].
    pub(crate) fn whole_now(&self) -> usize {
        self.scores
            .iter()
            .flatten()
            .filter(|p| whole_now(p))
            .count()
    }
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

/// **The third battery's scores**, `HELD_OUT_THIRD[k]` played as
/// `run1743 + k` (item 4487): `None` until the original has played it. The
/// found column is pinned once and never re-pinned; it is the estimate
/// (DECISIONS 147 §2).
#[cfg(test)]
pub(crate) const SCORES_THIRD: [Option<FirstScore>; 8] = [
    // run1743
    // row 5, as found 2026-11-21: 11 lines whole; ammo 5,934 of 6,001, first 4805.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 6001,
            agreed: [
                6001, 6001, 6001, 5934, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001,
            ],
            first: &[("ammo", (4805, Some(2964392822), 2975730574))],
            over: None,
        },
        // Moved by item 4507 (piece 32's three bays re-fitted on the
        // original's own 361-heading sweep: run1753, state 4805, ammo
        // range 1, record 0, `launch.x, launch.y` ours 4944, 30710,
        // theirs 4945, 30709, who=1's Slinger `1/22`): ammo 5,934 ->
        // 6,001, whole; the game is whole.
        moved: Some(Score {
            states: 6001,
            agreed: [6001; 12],
            first: &[],
            over: None,
        }),
    }),
    // run1744
    // row 25, as found 2026-11-21: 12 lines whole.
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
    // run1745
    // row 45, as found 2026-11-21: 12 lines whole.
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
    // run1746
    // row 65, as found 2026-11-21, ended by the original on 4045: 12 lines whole.
    Some(FirstScore {
        last: Some(4045),
        found: Score {
            states: 4046,
            agreed: [
                4046, 4046, 4046, 4046, 4046, 4046, 4046, 4046, 4046, 4046, 4046, 4046,
            ],
            first: &[],
            over: None,
        },
        moved: None,
    }),
    // run1747
    // row 85, as found 2026-11-21, ended by the original on 4471: 11 lines whole; groups 4,458 of
    // 4,472, first 3831.
    Some(FirstScore {
        last: Some(4471),
        found: Score {
            states: 4472,
            agreed: [
                4472, 4472, 4472, 4472, 4472, 4458, 4472, 4472, 4472, 4472, 4472, 4472,
            ],
            first: &[("groups", (3831, Some(2097888135), 2964599741))],
            over: None,
        },
        // Moved by item 4493 (`Group::refresh_group_order`'s `normalize`
        // prunes the seated group: run1751, state 3831, groups range 86,
        // record 68 `num` ours 4, theirs 3): groups 4,458 -> 4,472, whole;
        // the game is whole.
        moved: Some(Score {
            states: 4472,
            agreed: [4472; 12],
            first: &[],
            over: None,
        }),
    }),
    // run1748
    // row 105, as found 2026-11-21, ended by the original on 5234: 8 lines whole; units 2,983 of
    // 5,235, first 2983; builds 5,162 of 5,235, first 5162; groups 5,217 of 5,235, first 4355;
    // leaders 4,656 of 5,235, first 1976.
    Some(FirstScore {
        last: Some(5234),
        found: Score {
            states: 5235,
            agreed: [
                2983, 5162, 5235, 5235, 5235, 5217, 5235, 4656, 5235, 5235, 5235, 5235,
            ],
            first: &[
                ("units", (2983, Some(794628535), 1924338104)),
                ("builds", (5162, Some(2385371548), 2688344540)),
                ("groups", (4355, Some(2731263697), 520406832)),
                ("leaders", (1976, Some(227834778), 1853848475)),
            ],
            over: None,
        },
        // Moved by item 4512, second (`Groups::push_group`'s second walk
        // asks only the group a member's `+0x80` names: run1752, state
        // 4355, groups range 204, record 84, who=1's slot 20 keeps `1/94`
        // when `think_scout` pushes it onto slot 22, `num` ours 49, theirs
        // 50): groups 5,217 -> 5,235, whole; the game is whole.
        // Moved by item 4512 (a leader's `SimpleArray<int>` keeps its
        // capacity across a removal: run1752, state 3583, leaders range
        // 191, record 1, who=1's `oil_patches` pruned from 12 to 10 by arm
        // 4.2, `size` ours 10, theirs 20): leaders 5,056 -> 5,235, whole;
        // the row's first parting 3583 -> 4355 (groups).
        // Moved by item 4511 (a unit host's `inside_down` chain is the order
        // of entry, `Object::insert_inside` appending at the bottom:
        // run1752, state 3359, units range 3707, record 127, who=1's
        // Freighter `1/135` holding `1/127`, `1/128`, `1/129`, `1/127`'s
        // `inside_down` ours -1, theirs 128): units 3,359 -> 5,235, whole;
        // the row's first parting 3359 -> 3583 (leaders).
        // Moved by item 4502 (the CEO bit, `unit_masks2 & 0x10000`, raised
        // by `set_type` on a type that `is(0x165, 1)`: run1752, state 2983,
        // units range 655, record 17 (`1/17`, 0x163 -> 0x165)
        // `unit_masks2` ours 0, theirs 65536): units 2,983 -> 3,359, first
        // 2983 -> 3359; the row's first parting 2983 -> 3359 (units).
        // Moved by item 4499 (`reg_xport_peasants` is indexed without a
        // bound: run1752, state 1976, leaders range 129, record 1
        // `reg_gatherers[1]` ours 0, theirs 1, who=1's rider bound for sea
        // region 65): leaders 4,656 -> 5,056, first 1976 -> 3583; the
        // row's first parting 1976 -> 2983 (units).
        // Moved by item 4483 (an AIRDEFENSE-line building finishes with
        // `job_counter` `0x40000000`): builds 5,162 -> 5,235, whole. No
        // ranges take of this row.
        moved: Some(Score {
            states: 5235,
            agreed: [5235; 12],
            first: &[],
            over: None,
        }),
    }),
    // run1749
    // row 125, as found 2026-11-21: 12 lines whole.
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
    // run1750
    // row 145, as found 2026-11-21: 12 lines whole.
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

/// **Parity held out, third, N of 8** (item 4487): the third battery's
/// line, games whole of the eight and games taken. As found 2026-11-21:
/// 5 of 8, rows 25, 45, 65, 125 and 145 whole. Item 4493: 6 of 8, row 85
/// whole (`Group::refresh_group_order`'s prune). Item 4507: 7 of 8, row 5
/// whole (piece 32's bays re-fitted on the original's sweep). Item 4512:
/// 8 of 8, row 105 whole (`push_group` asks only the group `+0x80` names).
#[cfg(test)]
pub(crate) const HELD_OUT_THIRD_LINE: (usize, usize) = (8, 8);

/// The third battery (item 4487).
#[cfg(test)]
pub(crate) const THIRD: Battery = Battery {
    label: "Parity held out, third",
    item: 4487,
    run: 1743,
    table: &HELD_OUT_THIRD,
    scores: &SCORES_THIRD,
    scores_name: "SCORES_THIRD",
    line: HELD_OUT_THIRD_LINE,
};

/// **The fourth battery's scores**, `HELD_OUT_FOURTH[k]` played as
/// `run1760 + k` (item 4510): `None` until the original has played it. The
/// found column is pinned once and never re-pinned; it is the estimate
/// (DECISIONS 148 §2).
#[cfg(test)]
pub(crate) const SCORES_FOURTH: [Option<FirstScore>; 8] = [
    // run1760
    // row 15, as found 2026-11-21: 12 lines whole.
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
    // run1761
    // row 35, as found 2026-11-21: 12 lines whole.
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
    // run1762
    // row 55, as found 2026-11-21: 10 lines whole; units 5,833 of 6,001, first 4275; groups 5,989
    // of 6,001, first 5576.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 6001,
            agreed: [
                5833, 6001, 6001, 6001, 6001, 5989, 6001, 6001, 6001, 6001, 6001, 6001,
            ],
            first: &[
                ("units", (4275, Some(1914015254), 708677162)),
                ("groups", (5576, Some(4003049513), 3939479638)),
            ],
            over: None,
        },
        // Moved by item 4518 (a boat is born through `Unit::update_hits`: the
        // Dutch Merchant Fleet 1/142's myhits 172 -> 192 on 4275): units
        // 5,833 -> 6,001, whole. Moved by item 4531 (a seated group's list
        // holds object numbers: group 83's num 42 -> 43 on 5576, the short
        // 44 now the newborn Elite Javelineers): groups 5,989 -> 6,001,
        // whole; twelve lines whole.
        moved: Some(Score {
            states: 6001,
            agreed: [6001; 12],
            first: &[],
            over: None,
        }),
    }),
    // run1763
    // row 75, as found 2026-11-21, ended by the original on 5341: 11 lines whole; ammo 4,682 of
    // 5,342, first 4004.
    Some(FirstScore {
        last: Some(5341),
        found: Score {
            states: 5342,
            agreed: [
                5342, 5342, 5342, 4682, 5342, 5342, 5342, 5342, 5342, 5342, 5342, 5342,
            ],
            first: &[("ammo", (4004, Some(3143699460), 3217755213))],
            over: None,
        },
        // Moved by item 4517 (piece 122's sockets, the original's sweep):
        // ammo 4,682 -> 5,342, first 4004 -> none; twelve lines whole to the
        // original's end. run1768's ammo range 1, record 0 on 4004: `launch.x`
        // ours 3922, theirs 3923.
        moved: Some(Score {
            states: 5342,
            agreed: [
                5342, 5342, 5342, 5342, 5342, 5342, 5342, 5342, 5342, 5342, 5342, 5342,
            ],
            first: &[],
            over: None,
        }),
    }),
    // run1764
    // row 95, as found 2026-11-21: 9 lines whole; units 5,303 of 6,001, first 5303; guys 5,909 of
    // 6,001, first 5909; leaders 5,266 of 6,001, first 5266.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 6001,
            agreed: [
                5303, 6001, 6001, 6001, 6001, 6001, 5909, 5266, 6001, 6001, 6001, 6001,
            ],
            first: &[
                ("units", (5303, Some(1676718193), 3912806517)),
                ("guys", (5909, Some(1310729884), 912139918)),
                ("leaders", (5266, Some(871988666), 426802620)),
            ],
            over: None,
        },
        // Moved by item 4524 (the Kremlin's spy tag is `unit_masks2 &
        // 0x400`; run1754's leaf, row 130): units 5,303 -> 5,909, first
        // 5303 -> 5909, beside guys. Moved by item 4519 (a CEO going in
        // asks `update_ceo_position`; run1770's leaders range 128):
        // leaders 5,266 -> 6,001, whole; ten lines whole.
        moved: Some(Score {
            states: 6001,
            agreed: [
                5909, 6001, 6001, 6001, 6001, 6001, 5909, 6001, 6001, 6001, 6001, 6001,
            ],
            first: &[
                ("units", (5909, Some(3899268178), 2012421188)),
                ("guys", (5909, Some(1310729884), 912139918)),
            ],
            over: None,
        }),
    }),
    // run1765
    // row 115, as found 2026-11-21: 12 lines whole.
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
    // run1766
    // row 135, as found 2026-11-21: 12 lines whole.
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
    // run1767
    // row 155, as found 2026-11-21: 12 lines whole.
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

/// The fourth battery's line: games whole of the eight, and games taken
/// (6 -> 7, item 4531: row 55 whole).
#[cfg(test)]
pub(crate) const HELD_OUT_FOURTH_LINE: (usize, usize) = (7, 8);

/// The fourth battery (item 4510).
#[cfg(test)]
pub(crate) const FOURTH: Battery = Battery {
    label: "Parity held out, fourth",
    item: 4510,
    run: 1760,
    table: &HELD_OUT_FOURTH,
    scores: &SCORES_FOURTH,
    scores_name: "SCORES_FOURTH",
    line: HELD_OUT_FOURTH_LINE,
};

/// **The fifth battery's scores**, `HELD_OUT_FIFTH[k]` played as
/// `run1773 + k` (item 4527): `None` until the original has played it. The
/// found column is pinned once and never re-pinned; it is the estimate
/// (DECISIONS 149 §3, 150 §2).
#[cfg(test)]
pub(crate) const SCORES_FIFTH: [Option<FirstScore>; 8] = [
    // run1773
    // row 3, as found 2026-11-21: 1 line whole; units 3,872 of 6,001, first 3857; builds 4,065 of
    // 6,001, first 3857; ammo 4,957 of 6,001, first 3857; deaths 4,376 of 6,001, first 4376; groups
    // 4,378 of 6,001, first 4378; guys 4,376 of 6,001, first 4376; leaders 3,879 of 6,001, first
    // 3879; cities 4,725 of 6,001, first 4725; items 5,930 of 6,001, first 5105; goods 5,800 of
    // 6,001, first 5384; world 4,376 of 6,001, first 4376.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 6001,
            agreed: [
                3872, 4065, 6001, 4957, 4376, 4378, 4376, 3879, 4725, 5930, 5800, 4376,
            ],
            first: &[
                ("units", (3857, Some(1076448530), 1110134034)),
                ("builds", (3857, Some(314592769), 74403328)),
                ("ammo", (3857, Some(4289990178), 3011644317)),
                ("deaths", (4376, Some(1), 412945055)),
                ("groups", (4378, Some(3456385596), 3748018508)),
                ("guys", (4376, Some(3417311684), 1086390141)),
                ("leaders", (3879, Some(2652012725), 3916529900)),
                ("cities", (4725, Some(2712083841), 921771259)),
                ("items", (5105, Some(3013827624), 2892717092)),
                ("goods", (5384, Some(3119871758), 3073734412)),
                ("world", (4376, Some(2578809896), 1356956670)),
            ],
            over: None,
        },
        moved: None,
    }),
    // run1774
    // row 23, as found 2026-11-21: 10 lines whole; units 5,695 of 6,001, first 5695; leaders 0 of
    // 6,001, first 0.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 6001,
            agreed: [
                5695, 6001, 6001, 6001, 6001, 6001, 6001, 0, 6001, 6001, 6001, 6001,
            ],
            first: &[
                ("units", (5695, Some(1829564320), 64286658)),
                ("leaders", (0, Some(569079079), 568292647)),
            ],
            over: None,
        },
        // Moved by item 4534 (the walked setup leaves the player-region bit to
        // `analyze_map`; run1781's leaf, who=0's `oil_patches` on state 0, ours
        // [5, 2], theirs [2, 5]): leaders 0 -> 6,001, whole; eleven lines whole.
        moved: Some(Score {
            states: 6001,
            agreed: [
                5695, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001,
            ],
            first: &[("units", (5695, Some(1829564320), 64286658))],
            over: None,
        }),
    }),
    // run1775
    // row 43, as found 2026-11-21: 1 line whole; units 3,290 of 6,001, first 3196; builds 3,345 of
    // 6,001, first 3299; ammo 3,355 of 6,001, first 3192; deaths 3,247 of 6,001, first 3247; groups
    // 3,395 of 6,001, first 3198; guys 3,374 of 6,001, first 3374; leaders 3,218 of 6,001, first
    // 3196; cities 3,373 of 6,001, first 3223; items 4,262 of 6,001, first 3670; goods 4,002 of
    // 6,001, first 3710; world 3,374 of 6,001, first 3374.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 6001,
            agreed: [
                3290, 3345, 6001, 3355, 3247, 3395, 3374, 3218, 3373, 4262, 4002, 3374,
            ],
            first: &[
                ("units", (3196, Some(3542717112), 1535152825)),
                ("builds", (3299, Some(3013899671), 1891857812)),
                ("ammo", (3192, Some(3057849010), 3663794903)),
                ("deaths", (3247, Some(2200713404), 3391044026)),
                ("groups", (3198, Some(2769144758), 3563965351)),
                ("guys", (3374, Some(641651763), 4151560086)),
                ("leaders", (3196, Some(1537863292), 2642538110)),
                ("cities", (3223, Some(3058186551), 2580035877)),
                ("items", (3670, Some(3687073795), 3707258885)),
                ("goods", (3710, Some(3114104596), 3160241942)),
                ("world", (3374, Some(2564315079), 3645396767)),
            ],
            over: Some(5977),
        },
        moved: None,
    }),
    // run1776
    // row 63, as found 2026-11-21, ended by the original on 4222: 1 line whole; units 1,789 of
    // 4,223, first 1789; builds 1,810 of 4,223, first 1810; ammo 3,780 of 4,223, first 3321; deaths
    // 3,481 of 4,223, first 3481; groups 1,903 of 4,223, first 1903; guys 1,789 of 4,223, first
    // 1789; leaders 1,582 of 4,223, first 1582; cities 1,958 of 4,223, first 1958; items 2,179 of
    // 4,223, first 2027; goods 1,965 of 4,223, first 1965; world 1,806 of 4,223, first 1806.
    Some(FirstScore {
        last: Some(4222),
        found: Score {
            states: 4223,
            agreed: [
                1789, 1810, 4223, 3780, 3481, 1903, 1789, 1582, 1958, 2179, 1965, 1806,
            ],
            first: &[
                ("units", (1789, Some(2509609850), 1943706443)),
                ("builds", (1810, Some(3537404403), 1462992888)),
                ("ammo", (3321, Some(1), 3987474659)),
                ("deaths", (3481, Some(1), 2307460364)),
                ("groups", (1903, Some(3874615139), 167898819)),
                ("guys", (1789, Some(3120588302), 1384277471)),
                ("leaders", (1582, Some(205841799), 2417223212)),
                ("cities", (1958, Some(4197331207), 4255592714)),
                ("items", (2027, Some(1138496331), 1141379917)),
                ("goods", (1965, Some(1208545731), 1344074179)),
                ("world", (1806, Some(1643701640), 1687741832)),
            ],
            over: None,
        },
        moved: None,
    }),
    // run1777
    // row 83, as found 2026-11-21: 1 line whole; units 2,338 of 6,001, first 2338; builds 1,958 of
    // 6,001, first 1958; ammo 3,426 of 6,001, first 2404; deaths 2,941 of 6,001, first 2452; groups
    // 2,400 of 6,001, first 2400; guys 2,338 of 6,001, first 2338; leaders 1,956 of 6,001, first
    // 1956; cities 2,340 of 6,001, first 2340; items 3,563 of 6,001, first 3026; goods 3,952 of
    // 6,001, first 2829; world 2,338 of 6,001, first 2338.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 6001,
            agreed: [
                2338, 1958, 6001, 3426, 2941, 2400, 2338, 1956, 2340, 3563, 3952, 2338,
            ],
            first: &[
                ("units", (2338, Some(1361964866), 3925824949)),
                ("builds", (1958, Some(977573852), 2524157908)),
                ("ammo", (2404, Some(1723730597), 1867713323)),
                ("deaths", (2452, Some(1982270641), 1)),
                ("groups", (2400, Some(4286445278), 2313221958)),
                ("guys", (2338, Some(3548161827), 1424743536)),
                ("leaders", (1956, Some(3921751680), 3148426205)),
                ("cities", (2340, Some(2157784556), 130625532)),
                ("items", (3026, Some(1796424164), 835927342)),
                ("goods", (2829, Some(3059258644), 3039073554)),
                ("world", (2338, Some(858147796), 2819705606)),
            ],
            over: None,
        },
        // Moved by item 4535 (a garrisoned worker whose owner has the
        // militia counts a third of the militia line's attack in a
        // building's arrows, `count_inside@0064dda0`): ammo 3,426 -> 3,330
        // and items 3,563 -> 3,491, named falls past the first partings,
        // which stand; the states not read one by one.
        moved: Some(Score {
            states: 6001,
            agreed: [
                2338, 1958, 6001, 3330, 2941, 2400, 2338, 1956, 2340, 3491, 3952, 2338,
            ],
            first: &[
                ("units", (2338, Some(1361964866), 3925824949)),
                ("builds", (1958, Some(977573852), 2524157908)),
                ("ammo", (2404, Some(1723730597), 1867713323)),
                ("deaths", (2452, Some(1982270641), 1)),
                ("groups", (2400, Some(4286445278), 2313221958)),
                ("guys", (2338, Some(3548161827), 1424743536)),
                ("leaders", (1956, Some(3921751680), 3148426205)),
                ("cities", (2340, Some(2157784556), 130625532)),
                ("items", (3026, Some(1796424164), 835927342)),
                ("goods", (2829, Some(3059258644), 3039073554)),
                ("world", (2338, Some(858147796), 2819705606)),
            ],
            over: None,
        }),
    }),
    // run1778
    // row 103, as found 2026-11-21, ended by the original on 3294: 1 line whole; units 1,790 of
    // 3,295, first 1790; builds 1,854 of 3,295, first 1824; ammo 2,882 of 3,295, first 2860; deaths
    // 2,508 of 3,295, first 2508; groups 1,963 of 3,295, first 1963; guys 1,790 of 3,295, first
    // 1790; leaders 1,757 of 3,295, first 1757; cities 1,951 of 3,295, first 1951; items 2,728 of
    // 3,295, first 2728; goods 1,987 of 3,295, first 1987; world 1,791 of 3,295, first 1750.
    Some(FirstScore {
        last: Some(3294),
        found: Score {
            states: 3295,
            agreed: [
                1790, 1854, 3295, 2882, 2508, 1963, 1790, 1757, 1951, 2728, 1987, 1791,
            ],
            first: &[
                ("units", (1790, Some(4038452573), 1054592778)),
                ("builds", (1824, Some(512225234), 3008295372)),
                ("ammo", (2860, Some(1), 460261739)),
                ("deaths", (2508, Some(2009468066), 1)),
                ("groups", (1963, Some(321777859), 660866967)),
                ("guys", (1790, Some(2094189105), 2360396383)),
                ("leaders", (1757, Some(2828401504), 1487010739)),
                ("cities", (1951, Some(1722361204), 1721574772)),
                ("items", (2728, Some(3164999615), 3159232445)),
                ("goods", (1987, Some(1178399185), 1567683031)),
                ("world", (1750, Some(1825769550), 1731133486)),
            ],
            over: None,
        },
        moved: None,
    }),
    // run1779
    // row 123, as found 2026-11-21: 3 lines whole; units 3,020 of 6,001, first 2986; builds 2,977
    // of 6,001, first 2977; groups 2,977 of 6,001, first 2977; guys 3,294 of 6,001, first 3294;
    // leaders 3,294 of 6,001, first 3294; cities 3,376 of 6,001, first 3376; items 4,876 of 6,001,
    // first 4280; goods 5,449 of 6,001, first 5107; world 3,294 of 6,001, first 3294.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 6001,
            agreed: [
                3020, 2977, 6001, 6001, 6001, 2977, 3294, 3294, 3376, 4876, 5449, 3294,
            ],
            first: &[
                ("units", (2986, Some(2924321191), 783587750)),
                ("builds", (2977, Some(1963123823), 450488504)),
                ("groups", (2977, Some(4066703869), 4068538878)),
                ("guys", (3294, Some(3387158684), 8782212)),
                ("leaders", (3294, Some(66965636), 4273262857)),
                ("cities", (3376, Some(4268236955), 4286980252)),
                ("items", (4280, Some(3142337459), 3096200111)),
                ("goods", (5107, Some(562599133), 412652761)),
                ("world", (3294, Some(221010258), 2748209768)),
            ],
            over: None,
        },
        moved: None,
    }),
    // run1780
    // row 143, as found 2026-11-21: 12 lines whole.
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

/// The fifth battery's line: games whole of the eight, and games taken.
#[cfg(test)]
pub(crate) const HELD_OUT_FIFTH_LINE: (usize, usize) = (1, 8);

/// The fifth battery (item 4527): three seats and one team of two a row.
#[cfg(test)]
pub(crate) const FIFTH: Battery = Battery {
    label: "Parity held out, fifth",
    item: 4527,
    run: 1773,
    table: &HELD_OUT_FIFTH,
    scores: &SCORES_FIFTH,
    scores_name: "SCORES_FIFTH",
    line: HELD_OUT_FIFTH_LINE,
};

/// **The spent batteries, folded into the headline** (DECISIONS 150 §1).
/// A battery is spent once a lane has read a parting off one of its rows
/// (entry 145 §2); its `now` column then counts in `Parity:` row by row,
/// by [`whole_now`], beside the six and the sample, and its found column
/// never folds. The first, second and third were spent on `43ee59434`;
/// the fourth since item 4517 read row 75's ammo (`b52f075dd`). A battery
/// declared and not yet spent stays off this list; the fifth (item 4527)
/// joins by one entry here once a lane reads a parting off one of its rows.
#[cfg(test)]
pub(crate) const SPENT: &[&Battery] = &[&FIRST, &SECOND, &THIRD, &FOURTH];
