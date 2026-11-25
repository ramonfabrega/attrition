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

/// **The sixth battery's rows** (DECISIONS 151 §6, 152 §4, item 4566): the
/// same order, the rows ten past the fifth's, by the fifth's recipe, each
/// recorded to [`SIXTH_FRAMES`] with the human scripted.
#[cfg(test)]
pub(crate) const HELD_OUT_SIXTH_ROWS: [usize; 8] = [13, 33, 53, 73, 93, 113, 133, 153];

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

/// **No seat added** (item 4600): a row whose own lobby already holds three
/// seats, the most its `MAP_SIZE 2` holds, takes the team alone.
#[cfg(test)]
pub(crate) const NO_SEAT: u8 = u8::MAX;

#[cfg(test)]
impl Seated {
    /// The page's events, after a lobby of `seats` seats: no seat added
    /// when `tribe` is [`NO_SEAT`].
    pub(crate) fn events(&self, seats: usize) -> Vec<crate::lobby::Event> {
        use crate::lobby::Event;
        let mut e = Vec::new();
        if self.tribe != NO_SEAT {
            e.push(Event::AddSeat);
            e.push(Event::Nation {
                seat: seats,
                tribe: self.tribe,
            });
        }
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

/// **The sixth's additions** (DECISIONS 151 §6, 152 §4, item 4566), row
/// for row with [`HELD_OUT_SIXTH_ROWS`], by the fifth's recipe. Every row's
/// lobby is the human (seat 0, tribe 4) against one computer (seat 1) of
/// tribe 11, 8, 2, 11, 12, 11, 17 and 6. The team alternates as the
/// fifth's: on rows 13, 53, 93 and 133 the human and the added computer are
/// team 0; on rows 33, 73, 113 and 153 the two computers are team 1. The
/// added nations are 9, 10, 13, 14, 15, 16, 18 and 19: none already seated
/// on its row, and none the fifth added.
#[cfg(test)]
pub(crate) const HELD_OUT_SIXTH_SEATS: [Seated; 8] = [
    Seated {
        tribe: 9,
        mates: [0, 2],
        team: 0,
    },
    Seated {
        tribe: 10,
        mates: [1, 2],
        team: 1,
    },
    Seated {
        tribe: 13,
        mates: [0, 2],
        team: 0,
    },
    Seated {
        tribe: 14,
        mates: [1, 2],
        team: 1,
    },
    Seated {
        tribe: 15,
        mates: [0, 2],
        team: 0,
    },
    Seated {
        tribe: 16,
        mates: [1, 2],
        team: 1,
    },
    Seated {
        tribe: 18,
        mates: [0, 2],
        team: 0,
    },
    Seated {
        tribe: 19,
        mates: [1, 2],
        team: 1,
    },
];

/// **The sixth's recordings close on frame 12,000** (DECISIONS 152 §4),
/// twice the five before: the sample parts on state 7055, past the end of
/// every earlier battery.
#[cfg(test)]
pub(crate) const SIXTH_FRAMES: i32 = 12_000;

/// **A row's scripted human** (DECISIONS 152 §4): `RECGAME.md` §9's three
/// orders, a unit moved, a Citizen trained and a Farm placed, on the
/// minting's frames ([`crate::script::MOVE_FRAME`], `QUEUE_FRAME`,
/// `BUILD_FRAME`), with the objects and points read off the row's own built
/// start by [`script_rule`].
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Scripted {
    /// Slot 0's unit `0`, moved.
    pub scout: i16,
    /// Where it is sent.
    pub scout_to: sim::Pos,
    /// Slot 0's building `2000`, which trains one Citizen.
    pub capital: i16,
    /// Slot 0's Citizen nearest the Farm's site, which builds it.
    pub builder: i16,
    /// The Farm's drop point.
    pub farm_at: sim::Pos,
}

/// The packages a [`Scripted`] human issues, in `RECGAME.md` §9.1's
/// fields: the move a right-click's (`orders` 1, `form`/`width` −1) but
/// `queued` 2, a plain click's; the `queue_up` one Citizen; the `build` a
/// Farm dropped at one point, `queued` 2.
#[cfg(test)]
pub(crate) fn scripted_orders(h: &Scripted) -> Vec<crate::recgame::Order> {
    use crate::commands::Command;
    use crate::script::{BUILD_FRAME, CITIZEN, FARM_TYPE, MOVE_FRAME, QUEUE_FRAME};
    let group = |o: i16| Command::Group {
        who: 0,
        objects: vec![o],
    };
    vec![
        (
            MOVE_FRAME,
            0,
            vec![
                group(h.scout),
                Command::MoveTo {
                    to_x: h.scout_to.x,
                    to_y: h.scout_to.y,
                    set_angle: 0,
                    angle: 0,
                    orders: 1,
                    queued: 2,
                    form: -1,
                    width: -1,
                    disembark: 0,
                },
            ],
        ),
        (
            QUEUE_FRAME,
            0,
            vec![
                group(h.capital),
                Command::QueueUp {
                    unit: CITIZEN,
                    num: 1,
                },
            ],
        ),
        (
            BUILD_FRAME,
            0,
            vec![
                group(h.builder),
                Command::Build {
                    x: h.farm_at.x,
                    y: h.farm_at.y,
                    x2: h.farm_at.x,
                    y2: h.farm_at.y,
                    build: FARM_TYPE,
                    queued: 2,
                },
            ],
        ),
    ]
}

/// The rule's reach for the Farm's site: tiles from the capital's.
#[cfg(test)]
const FARM_REACH: i32 = 12;

/// **The rule that reads a row's script off its built start** (item 4566):
/// `RECGAME.md` §9.1's choices made general, so a held-out lobby gets the
/// orders §9 gave run53's. Held against §9's own script on run53's lobby
/// (`the_script_rule_gives_run53_s_lobby_the_minted_script`).
///
/// - **The move**: slot 0's unit `0`, sent 2,400 units (12.5 tiles) along
///   the first of four headings — toward the map's centre on x, on y, then
///   away on x, on y — whose point is on the map and in the unit's own
///   region (`tregion_alt`), so the original and ours both walk it on land.
/// - **The train**: slot 0's building `2000`, one Citizen, the Citizen
///   available to slot 0.
/// - **The Farm**: the Farm available to slot 0, the Citizen's and the
///   Farm's prices together inside slot 0's stock; dropped at the tile
///   centre nearest the capital, within [`FARM_REACH`] tiles, that our
///   placement check calls clear (ties by y, then x); built by slot 0's
///   Citizen nearest the drop point (ties by object number).
///
/// Refuses, with the reason, a start where one of them is wanting: that
/// row's human is recorded idle.
#[cfg(test)]
pub(crate) fn script_rule(built: &crate::diff::Built) -> Result<Scripted, String> {
    use sim::ids::scale::{HALF_TILE, UNITS_PER_TILE};
    let s = &built.sim;
    let (from, headings, first) = scout_headings(built)?;
    let scout_to = step(from, headings[first]);
    let capital_b = s
        .building_by_o(0, crate::script::CAPITAL)
        .ok_or("slot 0 has no building 2000")?;
    // The unit type whose tree entry's `TypeIndex` is the Citizen's.
    let citizen = built
        .unit_tree
        .iter()
        .position(|&t| built.type_index[t] == crate::script::CITIZEN)
        .ok_or("no Citizen type")?;
    if s.type_avail(0, crate::script::CITIZEN as usize) != sim::tech::AVAILABLE {
        return Err("the Citizen is not available to slot 0".into());
    }
    let farm_ty = (crate::script::FARM_TYPE - crate::load::BASE_BUILDTYPES) as usize;
    if s.type_avail(0, crate::script::FARM_TYPE as usize) != sim::tech::AVAILABLE {
        return Err("the Farm is not available to slot 0".into());
    }
    let price: Vec<i32> = s
        .price_of(0, citizen)
        .iter()
        .zip(s.building_price(0, farm_ty))
        .map(|(a, b)| a + b)
        .collect();
    let stock = s.economy.ledgers[0].bucket;
    if !price.iter().zip(stock).all(|(p, b)| *p <= b) {
        return Err(format!(
            "a Citizen and a Farm, {price:?}, pass slot 0's stock {stock:?}"
        ));
    }
    let c = s.buildings[capital_b].pos;
    let centre = |v: i32| v.div_euclid(UNITS_PER_TILE) * UNITS_PER_TILE + HALF_TILE;
    let (cx, cy) = (centre(c.x), centre(c.y));
    let mut sites: Vec<(i64, i32, i32)> = Vec::new();
    for j in -FARM_REACH..=FARM_REACH {
        for i in -FARM_REACH..=FARM_REACH {
            let (x, y) = (cx + i * UNITS_PER_TILE, cy + j * UNITS_PER_TILE);
            let d = i64::from(x - c.x).pow(2) + i64::from(y - c.y).pow(2);
            sites.push((d, y, x));
        }
    }
    sites.sort_unstable();
    let farm_at = sites
        .iter()
        .map(|&(_, y, x)| sim::Pos { x, y })
        .find(|&p| s.blocked_site(Some(0), farm_ty, p, None) == sim::cities::place::Blocked::Clear)
        .ok_or("no clear Farm site near the capital")?;
    let builder = s
        .units
        .iter()
        .filter(|u| u.owner == 0 && u.alive() && u.ty == Some(citizen))
        .min_by_key(|u| {
            (
                i64::from(u.pos.x - farm_at.x).pow(2) + i64::from(u.pos.y - farm_at.y).pow(2),
                u.index,
            )
        })
        .ok_or("slot 0 has no Citizen")?
        .index;
    Ok(Scripted {
        scout: 0,
        scout_to,
        capital: crate::script::CAPITAL,
        builder,
        farm_at,
    })
}

/// The move's length: 2,400 units, 12.5 tiles (`RECGAME.md` §9.1).
#[cfg(test)]
const STEP: i32 = 2400;

/// `at` moved [`STEP`] units along `heading`.
#[cfg(test)]
fn step(at: sim::Pos, (dx, dy): (i32, i32)) -> sim::Pos {
    sim::Pos {
        x: at.x + dx * STEP,
        y: at.y + dy * STEP,
    }
}

/// Where unit `0` stands, its four headings, and the first one taken.
#[cfg(test)]
type Headings = (sim::Pos, [(i32, i32); 4], usize);

/// **Unit `0`'s four headings off the built start** (item 4566's, made a
/// function by item 4600): where slot 0's unit `0` stands, its four
/// headings in [`script_rule`]'s order — toward the map's centre on x, on
/// y, then away on x, on y — and the first of them whose point [`STEP`]
/// units off is on the map and in the unit's own region (`tregion_alt`).
#[cfg(test)]
fn scout_headings(built: &crate::diff::Built) -> Result<Headings, String> {
    use sim::ids::scale::UNITS_PER_CELL;
    let s = &built.sim;
    let scout_u = s.unit_by_o(0, 0).ok_or("slot 0 has no unit 0")?;
    let from = s.units[scout_u].pos;
    let region = s
        .world
        .tregion_alt(from.tile())
        .ok_or("unit 0 stands in no region")?;
    let (w, h) = (
        s.world.width() * UNITS_PER_CELL,
        s.world.height() * UNITS_PER_CELL,
    );
    let toward = |at: i32, extent: i32| if at < extent / 2 { 1 } else { -1 };
    let (sx, sy) = (toward(from.x, w), toward(from.y, h));
    let headings = [(sx, 0), (0, sy), (-sx, 0), (0, -sy)];
    let first = headings
        .iter()
        .position(|&d| {
            let p = step(from, d);
            (0..w).contains(&p.x)
                && (0..h).contains(&p.y)
                && s.world.tregion_alt(p.tile()) == Some(region)
        })
        .ok_or("no heading 2,400 units from unit 0 stays in its region")?;
    Ok((from, headings, first))
}

/// **The seventh's rows** (DECISIONS 154 §7, item 4600): the same order,
/// the rows five past the third's, by the sixth's recipe, each recorded to
/// [`SEVENTH_FRAMES`] with the human ordering through the game.
#[cfg(test)]
pub(crate) const HELD_OUT_SEVENTH_ROWS: [usize; 8] = [8, 28, 48, 68, 88, 108, 128, 148];

/// **The seventh's recordings close on frame 12,000**, as the sixth's.
#[cfg(test)]
pub(crate) const SEVENTH_FRAMES: i32 = SIXTH_FRAMES;

/// **The seventh's rounds** (DECISIONS 154 §7): after `RECGAME.md` §9's
/// three orders, one round every [`ROUND_EVERY`] frames, the last on
/// [`ROUND_LAST`]: nineteen.
#[cfg(test)]
pub(crate) const ROUND_EVERY: i32 = 600;

/// The seventh's last round's frame.
#[cfg(test)]
pub(crate) const ROUND_LAST: i32 = 11_400;

/// **A row's ordering human** (DECISIONS 154 §7, item 4600): the sixth's
/// [`Scripted`] three orders, then a round every [`ROUND_EVERY`] frames to
/// [`ROUND_LAST`], each the capital queueing one Citizen and unit `0`
/// moving [`STEP`] units along the next of its four headings
/// ([`scout_headings`]' order, after the one the first move took). The
/// moves are named from the built start, not walked: four of them close a
/// square, so round `r`'s point is `square[(r - 1) % 4]`, and the square's
/// last corner is the first move's point.
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ordering {
    /// The sixth's three orders.
    pub first: Scripted,
    /// The rounds' four points, in order.
    pub square: [sim::Pos; 4],
}

/// **The rule that reads a row's ordering human off its built start**
/// (item 4600): [`script_rule`], then the square. Unit `0` moved along
/// heading `i` first (§9's move); round `r` moves it along heading
/// `(i + r) % 4` from where round `r - 1` sent it. Nothing is filtered:
/// a point off the unit's region, a unit or a capital gone by the
/// round's frame, is part of the test (DECISIONS 154 §7).
#[cfg(test)]
pub(crate) fn ordering_rule(built: &crate::diff::Built) -> Result<Ordering, String> {
    let first = script_rule(built)?;
    let (_, headings, i) = scout_headings(built)?;
    let mut at = first.scout_to;
    let mut square = [at; 4];
    for (r, corner) in square.iter_mut().enumerate() {
        at = step(at, headings[(i + r + 1) % 4]);
        *corner = at;
    }
    Ok(Ordering { first, square })
}

/// The packages an [`Ordering`] human issues: [`scripted_orders`], then
/// each round one package, as one turn of a person's input carries
/// both: the capital grouped and one Citizen queued (`queue_up`, num 1),
/// then unit `0` grouped and moved in §9.1's fields (`orders` 1,
/// `queued` 2, `form`/`width` −1).
#[cfg(test)]
pub(crate) fn ordering_orders(o: &Ordering) -> Vec<crate::recgame::Order> {
    use crate::commands::Command;
    let group = |obj: i16| Command::Group {
        who: 0,
        objects: vec![obj],
    };
    let mut out = scripted_orders(&o.first);
    for (r, frame) in (ROUND_EVERY..=ROUND_LAST)
        .step_by(ROUND_EVERY as usize)
        .enumerate()
    {
        let to = o.square[r % 4];
        out.push((
            frame,
            0,
            vec![
                group(o.first.capital),
                Command::QueueUp {
                    unit: crate::script::CITIZEN,
                    num: 1,
                },
                group(o.first.scout),
                Command::MoveTo {
                    to_x: to.x,
                    to_y: to.y,
                    set_angle: 0,
                    angle: 0,
                    orders: 1,
                    queued: 2,
                    form: -1,
                    width: -1,
                    disembark: 0,
                },
            ],
        ));
    }
    out
}

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
pub(crate) mod tests {
    use super::*;

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
        seated_rows_are_the_table(
            &HELD_OUT_FIFTH_ROWS,
            &HELD_OUT_FIFTH_SEATS,
            &HELD_OUT_FIFTH,
            "5",
        );
    }

    /// **The sixth declaration** (item 4566's first act; DECISIONS 151 §6,
    /// 152 §4): rows 13, 33, …, 153, each with the seat and team
    /// [`HELD_OUT_SIXTH_SEATS`] adds, and each row's human as
    /// [`script_rule`] reads it off the row's start built from the page's
    /// block, written out before any file is recorded.
    /// `RON_HELD_OUT_PRINT=6` prints both tables as Rust.
    #[test]
    fn the_sixth_held_out_eight_are_the_rows_the_rule_names_seated_and_scripted() {
        seated_rows_are_the_table(
            &HELD_OUT_SIXTH_ROWS,
            &HELD_OUT_SIXTH_SEATS,
            &HELD_OUT_SIXTH,
            "6",
        );
        let Some(dir) = crate::testenv::gamelog_dir() else {
            return;
        };
        let Some(inst) = crate::testenv::install() else {
            eprintln!("skipping: no install (set RON_INSTALL)");
            return;
        };
        let roster = crate::lobby::Roster::read(inst.root()).expect("the roster");
        let lobbies = crate::lobby::lobbies_on_disk(&dir, crate::corpus::Ledger::of_tree())
            .expect("the logs");
        let loaded = crate::load::load(&inst).unwrap();
        let print = std::env::var("RON_HELD_OUT_PRINT").is_ok_and(|v| v == "6");
        for (k, &row) in HELD_OUT_SIXTH_ROWS.iter().enumerate() {
            let lobby = crate::lobby::Lobby::parse(&lobbies[row].block).expect("the row's block");
            let mut c = crate::lobby::Choices::blank();
            for e in crate::lobby::events_for(&lobby, &roster) {
                c.apply(&e, &roster).expect("the row's own lobby");
            }
            for e in HELD_OUT_SIXTH_SEATS[k].events(c.seats.len()) {
                c.apply(&e, &roster).expect("the additions");
            }
            let page = c.lobby();
            let (info, players) = page.borrowed();
            let start = crate::diff::from_lobby(&loaded, &info, &players, sim::Tuning::RON);
            let built = crate::diff::built_from_lobby(&loaded, start).0;
            let got = script_rule(&built);
            if let Some(u) = built.sim.unit_by_o(0, 0) {
                let ty = built.sim.units[u].ty.unwrap_or(0);
                eprintln!(
                    "row {row}: unit 0/0 {} TI {}",
                    built.unit_type_names[ty], built.type_index[built.unit_tree[ty]]
                );
            }
            if print {
                println!("    // row {row}");
                match &got {
                    Ok(h) => println!("    {h:?},"),
                    Err(why) => println!("    // idle: {why}"),
                }
                continue;
            }
            // A row the rule refuses fails here with its reason: it is
            // recorded idle, and the declaration says so.
            assert_eq!(
                got,
                Ok(HELD_OUT_SIXTH_HUMANS[k]),
                "row {row}: the human the rule names"
            );
        }
    }

    /// **The seventh declaration** (item 4600's first act; DECISIONS 154
    /// §7): rows 8, 28, …, 148, each with the seat and team
    /// [`HELD_OUT_SEVENTH_SEATS`] adds, and each row's human as
    /// [`ordering_rule`] reads it off the row's start built from the page's
    /// block, written out before any file is recorded.
    /// `RON_HELD_OUT_PRINT=7` prints both tables as Rust.
    #[test]
    fn the_seventh_held_out_eight_are_the_rows_the_rule_names_seated_and_ordering() {
        seated_rows_are_the_table(
            &HELD_OUT_SEVENTH_ROWS,
            &HELD_OUT_SEVENTH_SEATS,
            &HELD_OUT_SEVENTH,
            "7",
        );
        let Some(dir) = crate::testenv::gamelog_dir() else {
            return;
        };
        let Some(inst) = crate::testenv::install() else {
            eprintln!("skipping: no install (set RON_INSTALL)");
            return;
        };
        let roster = crate::lobby::Roster::read(inst.root()).expect("the roster");
        let lobbies = crate::lobby::lobbies_on_disk(&dir, crate::corpus::Ledger::of_tree())
            .expect("the logs");
        let loaded = crate::load::load(&inst).unwrap();
        let print = std::env::var("RON_HELD_OUT_PRINT").is_ok_and(|v| v == "7");
        for (k, &row) in HELD_OUT_SEVENTH_ROWS.iter().enumerate() {
            let lobby = crate::lobby::Lobby::parse(&lobbies[row].block).expect("the row's block");
            let mut c = crate::lobby::Choices::blank();
            for e in crate::lobby::events_for(&lobby, &roster) {
                c.apply(&e, &roster).expect("the row's own lobby");
            }
            for e in HELD_OUT_SEVENTH_SEATS[k].events(c.seats.len()) {
                c.apply(&e, &roster).expect("the additions");
            }
            let page = c.lobby();
            let (info, players) = page.borrowed();
            let start = crate::diff::from_lobby(&loaded, &info, &players, sim::Tuning::RON);
            let built = crate::diff::built_from_lobby(&loaded, start).0;
            let got = ordering_rule(&built);
            if let Some(u) = built.sim.unit_by_o(0, 0) {
                let ty = built.sim.units[u].ty.unwrap_or(0);
                let (w, h) = (
                    built.sim.world.width() * sim::ids::scale::UNITS_PER_CELL,
                    built.sim.world.height() * sim::ids::scale::UNITS_PER_CELL,
                );
                let region = built.sim.world.tregion_alt(built.sim.units[u].pos.tile());
                let corners: Vec<String> = got
                    .iter()
                    .flat_map(|o| o.square)
                    .map(|p| {
                        let on = (0..w).contains(&p.x) && (0..h).contains(&p.y);
                        let same = on && built.sim.world.tregion_alt(p.tile()) == region;
                        format!(
                            "({}, {}) {}",
                            p.x,
                            p.y,
                            if same {
                                "in"
                            } else if on {
                                "out"
                            } else {
                                "off"
                            }
                        )
                    })
                    .collect();
                eprintln!(
                    "row {row}: unit 0/0 {} TI {} at {:?}; corners {corners:?}",
                    built.unit_type_names[ty],
                    built.type_index[built.unit_tree[ty]],
                    built.sim.units[u].pos
                );
            }
            if print {
                println!("    // row {row}");
                match &got {
                    Ok(h) => println!("    {h:?},"),
                    Err(why) => println!("    // idle: {why}"),
                }
                continue;
            }
            assert_eq!(
                got,
                Ok(HELD_OUT_SEVENTH_HUMANS[k]),
                "row {row}: the human the rule names"
            );
        }
    }

    /// **The ordering rule is the script rule and a square** (item 4600):
    /// on run53's start, [`ordering_rule`]'s first three orders are
    /// [`crate::script::script`], its four moves close on the first move's
    /// point, each one [`STEP`] along one axis, and [`ordering_orders`]
    /// issues nineteen rounds after them, from 600 to 11,400, each the
    /// capital's Citizen then unit `0`'s move.
    #[test]
    fn the_ordering_rule_is_the_script_and_a_closed_square_on_run53_s_start() {
        let Some((_, built, _)) = crate::diff::minted::built_start() else {
            eprintln!("skipping: no install or no run53 lobby dump");
            return;
        };
        let o = ordering_rule(&built).expect("run53's start serves the rule");
        assert_eq!(Ok(o.first), script_rule(&built));
        let orders = ordering_orders(&o);
        assert_eq!(orders[..3], crate::script::script()[..]);
        assert_eq!(o.square[3], o.first.scout_to, "four moves close the square");
        let mut at = o.first.scout_to;
        for p in o.square {
            let (dx, dy) = ((p.x - at.x).abs(), (p.y - at.y).abs());
            assert!(
                (dx, dy) == (STEP, 0) || (dx, dy) == (0, STEP),
                "{at:?} -> {p:?}"
            );
            at = p;
        }
        let rounds: Vec<i32> = orders[3..].iter().map(|o| o.0).collect();
        assert_eq!(rounds, (1..=19).map(|r| 600 * r).collect::<Vec<_>>());
        for (r, (_, play, cmds)) in orders[3..].iter().enumerate() {
            assert_eq!(*play, 0);
            let to = o.square[r % 4];
            assert!(matches!(
                cmds.as_slice(),
                [
                    crate::commands::Command::Group { who: 0, objects: a },
                    crate::commands::Command::QueueUp { unit: 50, num: 1 },
                    crate::commands::Command::Group { who: 0, objects: b },
                    crate::commands::Command::MoveTo { to_x, to_y, queued: 2, orders: 1, .. },
                ] if a == &[2000] && b == &[0] && (*to_x, *to_y) == (to.x, to.y)
            ));
        }
        crate::recgame::minted_packages(
            crate::commands::Command::Pause { state: 0 },
            &orders,
            SEVENTH_FRAMES,
        )
        .expect("the rounds mint");
    }

    /// **The seventh's files are minted** (item 4600): opt-in by
    /// `RON_SEVENTH_DRAFTS=<dir>` and `RON_SEVENTH_OUT=<dir>`, as the
    /// sixth's ([`the_sixth_s_files_are_minted`]): each row whose draft
    /// `row<RRR>.rcx` is in the directory, its start checked against the
    /// declared human, minted with [`ordering_orders`] and closing on
    /// [`SEVENTH_FRAMES`], written to `item4600-row<RRR>-12000.rcx`, never
    /// over a file. A row with no draft is skipped and named, so row 8 is
    /// minted alone first.
    #[test]
    fn the_seventh_s_files_are_minted() {
        let (Some(drafts), Some(out)) = (
            std::env::var_os("RON_SEVENTH_DRAFTS"),
            std::env::var_os("RON_SEVENTH_OUT"),
        ) else {
            eprintln!("skipping: RON_SEVENTH_DRAFTS and RON_SEVENTH_OUT name no directories");
            return;
        };
        let inst = crate::testenv::install().expect("an install");
        let loaded = crate::load::load(&inst).unwrap();
        for (k, held) in HELD_OUT_SEVENTH.iter().enumerate() {
            let draft = std::path::Path::new(&drafts).join(format!("row{:03}.rcx", held.row));
            if !draft.exists() {
                eprintln!("row {}: no draft, skipped", held.row);
                continue;
            }
            let mut rec = crate::recgame::read(&draft.to_string_lossy()).expect("the draft");
            let built = crate::checksum::playback::recording_start(&loaded, &rec);
            let h = HELD_OUT_SEVENTH_HUMANS[k];
            assert_eq!(
                ordering_rule(&built),
                Ok(h),
                "row {}: the file's start",
                held.row
            );
            let leader =
                crate::commands::decode(&rec.packages[0].data).expect("frame 0")[0].clone();
            rec.packages =
                crate::recgame::minted_packages(leader, &ordering_orders(&h), SEVENTH_FRAMES)
                    .expect("the rounds mint");
            let path = std::path::Path::new(&out)
                .join(format!("item4600-row{:03}-{SEVENTH_FRAMES}.rcx", held.row));
            let mut f = std::fs::File::create_new(&path).expect("an unused path");
            std::io::Write::write_all(&mut f, &crate::recgame::write_gz(&rec)).unwrap();
            eprintln!("wrote {}", path.display());
        }
    }

    /// **The seventh's files carry the declared seats, team and orders**
    /// (item 4600): each file's active slots are the declared rows', and
    /// its packages are the frame-0 order, [`ordering_orders`] of the
    /// row's human, and the closing package on [`SEVENTH_FRAMES`].
    #[test]
    fn the_seventh_s_files_carry_the_declared_seats_team_and_orders() {
        files_carry_the_declared_seats_and_team(&SEVENTH);
        for (k, held) in HELD_OUT_SEVENTH.iter().enumerate() {
            let (Some(rcx), _) = paths(&SEVENTH, k) else {
                continue;
            };
            let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("the file");
            let h = HELD_OUT_SEVENTH_HUMANS[k];
            let orders: Vec<_> = rec.packages[1..rec.packages.len() - 1]
                .iter()
                .map(|p| (p.frame, p.play, crate::commands::decode(&p.data).unwrap()))
                .collect();
            assert_eq!(orders, ordering_orders(&h), "row {}: the orders", held.row);
            let last = rec.packages.last().unwrap();
            assert_eq!(
                (last.frame, last.data.len()),
                (SEVENTH_FRAMES, 0),
                "row {}: the closing package",
                held.row
            );
        }
    }

    /// **The rule gives run53's lobby the minted script** (item 4566):
    /// [`script_rule`] on `RECGAME.md` §9's built start names §9's objects
    /// and points, and [`scripted_orders`] of it is
    /// [`crate::script::script`], package for package.
    #[test]
    fn the_script_rule_gives_run53_s_lobby_the_minted_script() {
        let Some((_, built, _)) = crate::diff::minted::built_start() else {
            eprintln!("skipping: no install or no run53 lobby dump");
            return;
        };
        let h = script_rule(&built).expect("run53's start serves the rule");
        assert_eq!(scripted_orders(&h), crate::script::script());
    }

    /// **The sixth's files are minted** (item 4566): opt-in by
    /// `RON_SIXTH_DRAFTS=<dir>`, where `row<RRR>.rcx` is the client's idle
    /// recording of the row (`client --record --headless --page-of
    /// <capture> --seat <tribe> --team <a>=<t> --team <b>=<t> --frame 1`):
    /// its header, donor and frame-0 package are the client's. The start
    /// built from that header must give [`script_rule`] the declared
    /// human; the packages are then minted again with
    /// [`scripted_orders`], closing on [`SIXTH_FRAMES`], and written to
    /// `RON_SIXTH_OUT/item4566-row<RRR>-12000.rcx`, never over a file.
    #[test]
    fn the_sixth_s_files_are_minted() {
        let (Some(drafts), Some(out)) = (
            std::env::var_os("RON_SIXTH_DRAFTS"),
            std::env::var_os("RON_SIXTH_OUT"),
        ) else {
            eprintln!("skipping: RON_SIXTH_DRAFTS and RON_SIXTH_OUT name no directories");
            return;
        };
        let inst = crate::testenv::install().expect("an install");
        let loaded = crate::load::load(&inst).unwrap();
        for (k, held) in HELD_OUT_SIXTH.iter().enumerate() {
            let draft = std::path::Path::new(&drafts).join(format!("row{:03}.rcx", held.row));
            let mut rec = crate::recgame::read(&draft.to_string_lossy()).expect("the draft");
            let built = crate::checksum::playback::recording_start(&loaded, &rec);
            let h = HELD_OUT_SIXTH_HUMANS[k];
            assert_eq!(
                script_rule(&built),
                Ok(h),
                "row {}: the file's start",
                held.row
            );
            let leader =
                crate::commands::decode(&rec.packages[0].data).expect("frame 0")[0].clone();
            rec.packages =
                crate::recgame::minted_packages(leader, &scripted_orders(&h), SIXTH_FRAMES)
                    .expect("the script mints");
            let path = std::path::Path::new(&out)
                .join(format!("item4566-row{:03}-{SIXTH_FRAMES}.rcx", held.row));
            let mut f = std::fs::File::create_new(&path).expect("an unused path");
            std::io::Write::write_all(&mut f, &crate::recgame::write_gz(&rec)).unwrap();
            eprintln!("wrote {}", path.display());
        }
    }

    /// **The sixth's files carry the declared seats, team and orders**
    /// (item 4566): each file's active slots are the declared rows', and
    /// its packages are the frame-0 order, [`scripted_orders`] of the
    /// row's human, and the closing package on [`SIXTH_FRAMES`].
    #[test]
    fn the_sixth_s_files_carry_the_declared_seats_team_and_orders() {
        files_carry_the_declared_seats_and_team(&SIXTH);
        for (k, held) in HELD_OUT_SIXTH.iter().enumerate() {
            let (Some(rcx), _) = paths(&SIXTH, k) else {
                continue;
            };
            let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("the file");
            let h = HELD_OUT_SIXTH_HUMANS[k];
            let orders: Vec<_> = rec.packages[1..rec.packages.len() - 1]
                .iter()
                .map(|p| (p.frame, p.play, crate::commands::decode(&p.data).unwrap()))
                .collect();
            assert_eq!(orders, scripted_orders(&h), "row {}: the orders", held.row);
            let last = rec.packages.last().unwrap();
            assert_eq!(
                (last.frame, last.data.len()),
                (SIXTH_FRAMES, 0),
                "row {}: the closing package",
                held.row
            );
        }
    }

    /// `rows` of `lobbies_on_disk`, each with `seats[k]` added on the page,
    /// against `table`; `RON_HELD_OUT_PRINT=which` prints the table, each
    /// row's own seats first as a comment.
    fn seated_rows_are_the_table(
        rows: &[usize; 8],
        seats: &[Seated; 8],
        table: &[HeldOut; 8],
        which: &str,
    ) {
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
        let print = std::env::var("RON_HELD_OUT_PRINT").is_ok_and(|v| v == which);
        for (k, &row) in rows.iter().enumerate() {
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
            if print {
                println!("    // row {row} as it stands: {:?}", columns(&c.lines()).1);
            }
            let add = &seats[k];
            let own = if add.tribe == NO_SEAT { 3 } else { 2 };
            assert_eq!(c.seats.len(), own, "row {row}: {own} seats as it stands");
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
    pub(crate) fn paths(
        b: &Battery,
        k: usize,
    ) -> (Option<std::path::PathBuf>, Option<std::path::PathBuf>) {
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let rcx = crate::testenv::experiment(
            &home.join("ron-golden/recordings"),
            &format!("item{}-row{:03}-{}.rcx", b.item, b.table[k].row, b.frames),
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

    /// Probe (item 4664): every differing range on run1834's ranges over
    /// `RON_PROBE_4664`'s frames (`lo-hi`), with the unit behind a units range.
    #[test]
    fn probe_4664() {
        let Ok(win) = std::env::var("RON_PROBE_4664") else {
            return;
        };
        let (lo, hi) = win
            .split_once('-')
            .map(|(a, b)| (a.parse::<i64>().unwrap(), b.parse::<i64>().unwrap()))
            .unwrap();
        let inst = crate::testenv::install().expect("install");
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SEVENTH, 4) else {
            panic!("rcx")
        };
        let ranges =
            crate::testenv::experiment(&home.join("ron-golden/run1834/map-18"), "rontrace.ranges")
                .expect("ranges");
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 88's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        let names = [
            "units", "builds", "walls", "ammo", "deaths", "groups", "guys", "leaders", "cities",
            "items", "goods", "world",
        ];
        for state in lo..=hi {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                let f0 = built.sim.frame;
                let sites = std::env::var("RON_PROBE_SITES")
                    .ok()
                    .and_then(|v| v.parse::<i64>().ok())
                    == Some(f0);
                built.sim.trace.trace_phases = sites;
                let seed0 = built.sim.rng.seed;
                built.sim.command().tick();
                if sites {
                    eprintln!("S4664 f{f0} seed {seed0:#010x}");
                    for (k, (label, who)) in crate::diff::harness::attributed_sites(&built)
                        .iter()
                        .enumerate()
                    {
                        eprintln!("S4664 f{f0} {k} {who}: {label}");
                    }
                }
                crate::diff::harness::debug_watch(&built, built.sim.frame);
                crate::diff::harness::debug_ammo(&built, built.sim.frame);
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            for (li, name) in names.iter().enumerate() {
                let Some(ours) = lines[li].as_ref() else {
                    continue;
                };
                let theirs: Vec<_> = all.iter().filter(|r| r.line as usize == li).collect();
                if ours.ranges.len() != theirs.len() {
                    eprintln!(
                        "P4664 {state} {name} count ours {} theirs {}",
                        ours.ranges.len(),
                        theirs.len()
                    );
                }
                let mut who = 0u8;
                let (mut i, mut j) = (0usize, 0usize);
                let mut pairs = Vec::new();
                while i < ours.ranges.len() && j < theirs.len() {
                    if ours.ranges[i].bytes == theirs[j].bytes
                        || ours.ranges[i].bytes.len() == theirs[j].bytes.len()
                            && ours.ranges[i].bytes.len() > 1
                    {
                        pairs.push((i, j));
                        i += 1;
                        j += 1;
                        continue;
                    }
                    // resync: skip up to 40 on either side where the next 4 agree
                    let ok = |a: usize, b: usize| {
                        (0..4).all(|t| {
                            a + t < ours.ranges.len()
                                && b + t < theirs.len()
                                && ours.ranges[a + t].bytes == theirs[b + t].bytes
                        })
                    };
                    let mut found = None;
                    'r: for d in 1..40 {
                        for (da, db) in [(d, 0), (0, d)] {
                            if ok(i + da, j + db) {
                                found = Some((da, db));
                                break 'r;
                            }
                        }
                    }
                    if let Some((da, db)) = found {
                        for t in 0..da {
                            eprintln!(
                                "P4664 {state} {name} ours-only range {} record {} {} len {}",
                                i + t,
                                ours.ranges[i + t].record,
                                ours.ranges[i + t].field,
                                ours.ranges[i + t].bytes.len()
                            );
                        }
                        for t in 0..db {
                            eprintln!(
                                "P4664 {state} {name} theirs-only range {} len {} {:?}",
                                j + t,
                                theirs[j + t].bytes.len(),
                                &theirs[j + t].bytes[..theirs[j + t].bytes.len().min(24)]
                            );
                        }
                        i += da;
                        j += db;
                    } else {
                        pairs.push((i, j));
                        i += 1;
                        j += 1;
                    }
                }
                for (k, (a, b)) in pairs
                    .iter()
                    .map(|&(i, j)| (&ours.ranges[i], theirs[j]))
                    .enumerate()
                {
                    if a.field == "who..y_internal" {
                        who = a.bytes[0];
                    }
                    if a.bytes != b.bytes {
                        let d = crate::checksum::fields::differing_fields(
                            name, a.field, &a.bytes, &b.bytes,
                        );
                        eprintln!(
                            "P4664 {state} {name} range {k} who {who} record {} {} len {}/{} {:?}",
                            a.record,
                            a.field,
                            a.bytes.len(),
                            b.bytes.len(),
                            d
                        );
                        if d.is_empty() && a.bytes.len() <= 128 {
                            eprintln!("P4664   ours {:?}\nP4664   theirs {:?}", a.bytes, b.bytes);
                        }
                        if li == 0 {
                            if let Some(u) = built.sim.unit_by_o(who, a.record) {
                                let un = &built.sim.units[u];
                                eprintln!(
                                    "P4664   unit slot {u} TI {} pos ({}, {})",
                                    un.type_index, un.pos.x, un.pos.y
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// **The held-out scores** (item 4413, `docs/RECGAME.md` §10.3): each of
    /// the eight games the original has played, scored by
    /// `checksum::playback` on every state to the take's last, against its
    /// pinned score: [`FirstScore::moved`] where a later fix has moved it,
    /// else the score as found. A game with no take skips, and says so; a
    /// take with no pin fails and prints the score to pin, unless it has no
    /// receipt either (`in_flight`). **What fails is
    /// what the queue books** (item 4598, [`held_verdict`]): the whole
    /// lines and the first parting; the rest of the pin prints as `HELD OUT
    /// DRIFT` and is re-pinned at a booking.
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

    /// `HELD_OUT_SIXTH[k]` played as `run1788 + k` (item 4566), held as
    /// the first five are, on every state to 12,000.
    #[test]
    fn the_sixth_held_out_games_score_as_pinned() {
        games_score_as_pinned(&SIXTH);
    }

    /// `HELD_OUT_SEVENTH[k]` played as `run1804 + k` (item 4600), held as
    /// the first six are, on every state to 12,000.
    #[test]
    fn the_seventh_held_out_games_score_as_pinned() {
        games_score_as_pinned(&SEVENTH);
    }

    /// The seventh battery's line, from its table alone (item 4600).
    #[test]
    fn the_seventh_held_out_line_reads_as_pinned() {
        line_reads_as_pinned(&SEVENTH);
    }

    /// **The verdict holds the booked numbers and lets the rest drift**
    /// (item 4598, DECISIONS 154 §5), on row 0's pin as found (units first
    /// on 2990, guys on 3157, ten lines whole). Each bend is one a lane
    /// made: a count or a parting past the first parting drifts (item
    /// 4594's row 93 ammo and deaths); a bend on the first parting, a
    /// whole line parting anywhere, or a parted line coming whole fails.
    #[test]
    fn a_held_pin_fails_on_its_whole_lines_and_first_parting_alone() {
        let pin = SCORES[0].as_ref().expect("row 0's pin").found;
        let got = |f: &dyn Fn(&mut Got)| -> (Vec<String>, Vec<String>) {
            let mut g: Got = (pin.states, pin.agreed, pin.first.to_vec(), pin.over);
            f(&mut g);
            held_verdict(&pin, &g)
        };
        let units = 0;
        let ammo = 3;
        let guys = 6;
        assert_eq!(got(&|_| ()), (vec![], vec![]), "the pin against itself");
        // Past the first parting: drift, printed, not failed.
        let (fails, drift) = got(&|g| {
            g.1[guys] -= 7;
            g.2[1].1.0 = 3200;
            g.1[units] += 3;
            g.2[0].1.1 = Some(1);
            g.3 = Some(5000);
        });
        assert!(fails.is_empty(), "past the parting: {fails:?}");
        assert_eq!(drift.len(), 5, "{drift:?}");
        // On the first parting: its state.
        let (fails, _) = got(&|g| g.2[0].1.0 = 2991);
        assert_eq!(fails.len(), 1, "{fails:?}");
        // On the first parting: a second line joins it.
        let (fails, _) = got(&|g| g.2[1].1.0 = 2990);
        assert_eq!(fails.len(), 1, "{fails:?}");
        // A whole line parts past the first parting.
        let (fails, _) = got(&|g| {
            g.1[ammo] -= 1;
            g.2.push(("ammo", (6000, Some(1), 2)));
        });
        assert_eq!(fails.len(), 1, "{fails:?}");
        // A parted line comes whole: guys.
        let (fails, _) = got(&|g| {
            g.1[guys] = g.0;
            g.2.truncate(1);
        });
        assert_eq!(fails.len(), 1, "{fails:?}");
        // The states compared.
        let (fails, _) = got(&|g| g.0 += 1);
        assert!(fails.iter().any(|f| f.starts_with("states")), "{fails:?}");
        // The row comes whole: the whole lines and the parting both move.
        let (fails, _) = got(&|g| {
            g.1 = [g.0; 12];
            g.2.clear();
        });
        assert_eq!(fails.len(), 2, "{fails:?}");
    }

    /// The sixth battery's line, from its table alone (item 4566).
    #[test]
    fn the_sixth_held_out_line_reads_as_pinned() {
        line_reads_as_pinned(&SIXTH);
    }

    /// **A battery a lane has read a parting off is on [`SPENT`]** (the
    /// pass of 2026-11-22, DECISIONS 151 §2). Entry 150 §1 folds a battery
    /// into the headline once it is spent, and the list was the driver's
    /// to extend by hand: items 4534 and 4538 read row 23's leaders and
    /// units off run1781's ranges and the fifth stayed off it for seven
    /// landings, so `Parity:` read 37 of 39 where the rule reads 39 of 47.
    /// A leaf read off a row lands as a test named `row_<N>_s_…` in this
    /// file, so that name is what spends the row's battery. Read off this
    /// file's own source; a row in no battery is a battery this list has
    /// not been told of.
    #[test]
    fn a_battery_with_a_row_s_leaf_test_is_spent() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/diff/held_out.rs");
        let src = std::fs::read_to_string(path).expect("held_out.rs");
        let all: [&Battery; 7] = [&FIRST, &SECOND, &THIRD, &FOURTH, &FIFTH, &SIXTH, &SEVENTH];
        let mut rows = std::collections::BTreeSet::new();
        for line in src.lines() {
            let Some(rest) = line.trim_start().strip_prefix("fn row_") else {
                continue;
            };
            let Some((n, _)) = rest.split_once("_s_") else {
                continue;
            };
            if let Ok(row) = n.parse::<usize>() {
                rows.insert(row);
            }
        }
        assert!(
            rows.len() >= 17,
            "the reader found {} rows with a leaf test; seventeen stood on 2026-11-22",
            rows.len()
        );
        for row in rows {
            let b = all
                .iter()
                .find(|b| b.table.iter().any(|h| h.row == row))
                .unwrap_or_else(|| panic!("row {row} has a leaf test and is in no battery"));
            assert!(
                SPENT.iter().any(|s| s.label == b.label),
                "row {row} has a leaf test, so `{}` is spent (entry 145 §2) and folds \
                 (entry 150 §1): add it to SPENT and write the `Parity:` line",
                b.label
            );
        }
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

    /// **Row 23's Fishermen takes Copper's hit points** (item 4538,
    /// `docs/RECGAME.md` §12.6): run1781 holds every units range to 6000.
    /// On 5695 who=2's leader holds Copper (`TypeIndex` 24, `rare` byte 2
    /// bit 2), the mask's change raises `0x4000000`, and `Leader::process`'s
    /// `calc_unit_stats@006cf970` re-runs `Unit::update_hits@0060e930` on
    /// every unit: the Fishermen `2/15` (TI 317, a sea type) takes
    /// `(COPPER_FACTORY_HP + 100) × 170 / 100` = 204. Every units range on
    /// 5694 and 5695, ours against theirs, byte for byte: red on the tree
    /// before the arm at range 957, record 15's `myhits`, ours 170.
    #[test]
    fn row_23_s_fishermen_takes_copper_s_hit_points_on_run1781_s_ranges() {
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
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for (state, hits, copper) in [(5694, 170i32, false), (5695, 204, true)] {
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
            // The Fishermen by name: who=2's record 15, its `type` and `myhits`.
            let k = ours
                .ranges
                .iter()
                .enumerate()
                .filter(|(_, r)| r.record == 15 && r.field == "who..y_internal")
                .find(|(k, _)| theirs[*k].bytes[0] == 2)
                .map(|(k, _)| k)
                .expect("2/15");
            assert_eq!(ours.ranges[k + 1].field, "type");
            assert_eq!(
                &theirs[k + 1].bytes[..],
                &317i32.to_le_bytes(),
                "2/15's TypeIndex"
            );
            assert_eq!(ours.ranges[k + 3].field, "myhits..launch_frames");
            assert_eq!(
                &theirs[k + 3].bytes[..4],
                &hits.to_le_bytes(),
                "theirs' myhits on {state}"
            );
            assert_eq!(
                built.sim.has_rare(2, sim::economy::COPPER),
                copper,
                "who=2's Copper on {state}"
            );
            let boat = built
                .sim
                .units
                .iter()
                .position(|u| u.alive() && u.owner == 2 && u.index == 15)
                .expect("2/15");
            let ty = built.sim.units[boat].ty.expect("its type");
            assert_eq!(built.unit_type_names[ty], "Fishermen");
            assert_eq!(built.sim.tables.unit_types[ty].hits, 170, "the raw type");
        }
    }

    /// **Row 133's computer offers no nuke at an enemy researching the
    /// shield** (item 4575, `docs/AI.md` §158): run1797, the ranges take
    /// of row 133's file. On 956 who=2 (the added computer, team 0 with
    /// the human) rebuilds its make list, and theirs' fourth slot is a Jet
    /// Fighter (TI 295) ×10 at `val` 986,484, no escrow. `create_units@006c40a0`'s
    /// nuke arm counts an enemy only while it neither holds
    /// `MISSILE_DEFENSE_BONUS` nor is `researching(MISSILE_SHIELD)`, and
    /// who=2's one enemy, who=1, has the Missile Shield (`0x243`) queued
    /// at its `1/2005`. Ours counted who=1 and listed an ICBM (TI 316) at
    /// 2,000,000 with an escrow in that slot, and on 958 queued it at
    /// `2/2017` where theirs queued four Jet Fighters at `2/2011`. Every
    /// leaders range on 955 to 958, ours against theirs, byte for byte:
    /// red on the tree before the fix at range 309, record 2,
    /// `make_list[k]`, `t` ours 316, theirs 295.
    #[test]
    fn row_133_s_computer_offers_no_nuke_at_an_enemy_researching_the_shield_on_run1797_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SIXTH, 6) else {
            eprintln!("skipping: row 133's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1797/map-7"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1797's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 133's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        let shield = built
            .sim
            .missile_shield()
            .expect("the tree's Missile Shield");
        for state in 955..=958 {
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
            // Who=2's fourth make-list slot: range 309, `t`, `val`, `escrow`, `num`.
            assert_eq!(ours.ranges[309].field, "make_list[k]");
            assert_eq!(ours.ranges[309].record, 2);
            let word = |n: usize| {
                i32::from_le_bytes(theirs[309].bytes[4 * n..4 * n + 4].try_into().unwrap())
            };
            let slot = (word(0), word(1), word(2), word(6));
            if (956..=957).contains(&state) {
                assert_eq!(slot, (295, 986_484, 0, 10), "theirs' Jet Fighter entry");
            }
            // Who=2's one enemy researches the shield and holds none.
            assert!(built.sim.is_ally(2, 0) && !built.sim.is_ally(2, 1));
            assert!(
                built.sim.researching(1, shield),
                "who=1's shield on {state}"
            );
            assert!(
                !built.sim.tech[1].tech[shield],
                "who=1 holds no shield on {state}"
            );
        }
    }

    /// **Row 133's computer lists no Bunker in its attacked capital**
    /// (item 4585, `docs/AI.md` §167): run1797, the ranges take of row
    /// 133's file. On 1757 who=2 rebuilds its make list. Its capital's
    /// `city_flags` are `0x405b` on both sides, and bit `0x8`
    /// (`ever_attacked`) spares a tower's value the arm's halving
    /// (`0x6c390f`). The Bunker (TI 442) is worth 13,500,000 there, and
    /// `check_income`'s 256 × that wraps negative, so theirs lists none:
    /// slot 2 is TI 273 at 6,478,056 and slot 8 the Market (TI 436) at
    /// 5,610,000 in its second city. Ours halved the Bunker to 6,750,000
    /// and listed it in slots 2 and 8. Every leaders range on 1756 and 1757,
    /// ours against theirs, byte for byte: red on the tree before the fix
    /// at range 308, record 2, `make_list[k]`, `t` ours 442, theirs 273.
    #[test]
    fn row_133_s_computer_lists_no_bunker_in_its_attacked_capital_on_run1797_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SIXTH, 6) else {
            eprintln!("skipping: row 133's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1797/map-7"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1797's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 133's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 1756..=1757 {
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
            // Who=2's capital: attacked, its flags the same on both sides.
            let cities = built.sim.checksum_cities();
            let flags = cities
                .ranges
                .iter()
                .find(|r| r.record == 2 && r.field == "city_flags")
                .expect("who=2's capital");
            assert_eq!(
                u16::from_le_bytes([flags.bytes[0], flags.bytes[1]]),
                0x405b,
                "the capital's flags on {state}"
            );
            if state == 1757 {
                let word = |k: usize, n: usize| {
                    i32::from_le_bytes(theirs[k].bytes[4 * n..4 * n + 4].try_into().unwrap())
                };
                assert_eq!(ours.ranges[308].field, "make_list[k]");
                assert_eq!(ours.ranges[308].record, 2);
                assert_eq!(
                    (word(308, 0), word(308, 1)),
                    (273, 6_478_056),
                    "theirs' slot 2"
                );
                assert_eq!(
                    (word(314, 0), word(314, 1), word(314, 3)),
                    (436, 5_610_000, 1),
                    "theirs' slot 8, the Market in the second city"
                );
            }
        }
    }

    /// **Row 133's computer calls a citizen to repair its capital**
    /// (item 4594, `docs/AI.md` §80.2 H12): run1797, the ranges take of
    /// row 133's file. who=2's capital `2/2000` (a Major City) carries
    /// damage 5,949, past half its hits, with its city's `no_heal` clear
    /// and no enemy within `0xc00`. On frame 1904, `(1904 + 2000) & 63 ==
    /// 0`, `Build::process@0061edf0`'s repair recruit (`61fe00`..`61feed`)
    /// gives the nearest not-busy citizen, `2/72` (TI 0x32) walking to its
    /// Oil Well `2/2008`, a `QUEUE_NEW` repair of the capital. Ours had no
    /// such arm: on 1905 units range 3966 (record 72) held ours' `MOVE_TO`
    /// and `GATHER` against theirs' one `REPAIR` (13) of `2/2000`, builds
    /// range 2061 held `2/2008`'s `gather_down` 72 against −1, and leaders
    /// range 265 who=2's `flags` byte 3 0x00 against 0x02 (the economy's
    /// dirty bit the killed gather raises). Every units, builds and leaders
    /// range on 1904 and 1905, ours against theirs, byte for byte.
    #[test]
    fn row_133_s_computer_calls_a_citizen_to_repair_its_capital_on_run1797_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SIXTH, 6) else {
            eprintln!("skipping: row 133's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1797/map-7"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1797's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 133's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 1904..=1905 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            for (line, name) in [(0usize, "units"), (1, "builds"), (7, "leaders")] {
                let ours = lines[line].as_ref().expect("the line");
                let theirs: Vec<_> = all.iter().filter(|r| r.line as usize == line).collect();
                assert_eq!(ours.ranges.len(), theirs.len(), "{name}' ranges on {state}");
                for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                    assert_eq!(
                        a.bytes, b.bytes,
                        "{name} range {k} on {state}, record {} {}",
                        a.record, a.field
                    );
                }
            }
            let u = (0..built.sim.units.len())
                .find(|&u| built.sim.units[u].owner == 2 && built.sim.units[u].index == 72)
                .expect("2/72");
            let capital = built.sim.building_by_o(2, 2000).expect("2/2000");
            let bd = &built.sim.buildings[capital];
            assert!(
                bd.damage > bd.hits_now() / 2,
                "past half its hits on {state}"
            );
            let repairs = built.sim.units[u]
                .orders
                .iter()
                .any(|o| matches!(o.body, sim::orders::Body::Repair(b) if b == capital));
            assert_eq!(
                repairs,
                state == 1905,
                "2/72 repairs the capital on {state}"
            );
        }
    }

    /// **Row 133's repair recruit walks under a move order** (item 4597):
    /// run1797 again. On 1905 `2/72` holds the capital's `REPAIR`; on 1906
    /// `Unit::do_repair` swarms it to the site through
    /// `Group::action_swarm_around@0070fbe0`, whose approach class
    /// `local_40` is 1 (`MOVE_TO`) unless the order is `BUILD_AT` for a
    /// computer (`:115`). Ours walked it under `EXPLORE_TO`: units range
    /// 3967, record 72, `order.type` ours 3, theirs 1. Every units range on
    /// 1905 and 1906, ours against theirs, byte for byte, and `2/72`'s
    /// head on 1906 a `MOVE_TO` ahead of the repair.
    #[test]
    fn row_133_s_repair_recruit_walks_under_a_move_order_on_run1797_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SIXTH, 6) else {
            eprintln!("skipping: row 133's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1797/map-7"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1797's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 133's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 1905..=1906 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            let ours = lines[0].as_ref().expect("the units line");
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
        let u = (0..built.sim.units.len())
            .find(|&u| built.sim.units[u].owner == 2 && built.sim.units[u].index == 72)
            .expect("2/72");
        let capital = built.sim.building_by_o(2, 2000).expect("2/2000");
        let orders = &built.sim.units[u].orders;
        assert!(
            matches!(
                orders.front().map(|o| &o.body),
                Some(sim::orders::Body::Move(m)) if m.kind == sim::orders::MoveKind::MoveTo
            ),
            "2/72's head on 1906 is a MOVE_TO: {orders:?}"
        );
        assert!(
            orders
                .iter()
                .any(|o| matches!(o.body, sim::orders::Body::Repair(b) if b == capital)),
            "2/72 still repairs the capital on 1906"
        );
    }

    /// **Row 133's orphan check walks the sites by object number** (item
    /// 4610, `docs/RECGAME.md` §13.5): run1797's ranges. On 2350 who=2 has
    /// two unbuilt sites with no builder, `2/2003` (uid 112, started) and
    /// `2/2033` (uid 69, which sits in an earlier slot of ours), and one idle
    /// citizen, `2/11`. `check_orphaned_buildings@006c9f20` walks
    /// `objects +0x1ec`'s list from `obj_base`, so 2003 takes the citizen and
    /// 2033 takes `build_masks | 0x2000`. Ours walked slots: units range
    /// 3699, record 11, `x` ours 10632, theirs 10608 on 2351; builds range
    /// 2085, `build_masks` ours 8584, theirs 392; group 132's `facing` ours
    /// 1, theirs 0. Every units, builds, groups and guys range on 2350 and
    /// 2351, byte for byte, and `2/11` building 2003 on 2351.
    #[test]
    fn row_133_s_orphan_check_walks_its_sites_by_object_number_on_run1797_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SIXTH, 6) else {
            eprintln!("skipping: row 133's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1797/map-7"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1797's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 133's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 2350..=2351 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            if state == 2350 {
                let started = built.sim.building_by_o(2, 2003).expect("2/2003");
                let fresh = built.sim.building_by_o(2, 2033).expect("2/2033");
                assert!(
                    fresh < started,
                    "2/2033 sits before 2/2003 in our slots: {fresh} against {started}"
                );
                assert!(!built.sim.buildings[started].active && !built.sim.buildings[fresh].active);
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            for (li, name) in [(0usize, "units"), (1, "builds"), (5, "groups"), (6, "guys")] {
                let ours = lines[li].as_ref().expect("the line");
                let theirs: Vec<_> = all.iter().filter(|r| r.line as usize == li).collect();
                assert_eq!(ours.ranges.len(), theirs.len(), "{name}' ranges on {state}");
                for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                    assert_eq!(
                        a.bytes, b.bytes,
                        "{name} range {k} on {state}, record {} {}",
                        a.record, a.field
                    );
                }
            }
        }
        let u = built.sim.unit_by_o(2, 11).expect("2/11");
        let started = built.sim.building_by_o(2, 2003).expect("2/2003");
        let fresh = built.sim.building_by_o(2, 2033).expect("2/2033");
        assert!(
            built.sim.units[u]
                .orders
                .iter()
                .any(|o| matches!(o.body, sim::orders::Body::Build(b) if b == started)),
            "2/11 builds 2/2003 on 2351: {:?}",
            built.sim.units[u].orders
        );
        assert!(built.sim.buildings[fresh].no_builder && !built.sim.buildings[started].no_builder);
    }

    /// **Row 133's new city sweeps its buildings by object number** (item
    /// 4620, `docs/RECGAME.md` §13.5): run1797's ranges. who=2's Village
    /// `2/2026` finishes on 2370, and `City::find_buildings@007384c0` walks
    /// each leader's `+0x1ec` list from `obj_base`, so the site `2/2007`
    /// (slot 92 of ours, a reused number) joins before the Oil Well
    /// `2/2008` (slot 28). Ours walked slots: builds range 2661, record
    /// 2026, `city_down` ours 2008, theirs 2007. Every builds range on 2369
    /// and 2370, byte for byte, and the chain 2026, 2007, 2008 on 2370.
    #[test]
    fn row_133_s_new_city_sweeps_its_buildings_by_object_number_on_run1797_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SIXTH, 6) else {
            eprintln!("skipping: row 133's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1797/map-7"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1797's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 133's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 2369..=2370 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let village = built.sim.building_by_o(2, 2026).expect("2/2026");
            let well = built.sim.building_by_o(2, 2008).expect("2/2008");
            let silo = built.sim.building_by_o(2, 2007).expect("2/2007");
            if state == 2369 {
                assert!(
                    well < silo,
                    "2/2008 sits before 2/2007 in our slots: {well} against {silo}"
                );
                assert!(!built.sim.buildings[village].active);
                assert!(built.sim.buildings[well].city.is_none());
                assert!(built.sim.buildings[silo].city.is_none());
            } else {
                let c = built.sim.buildings[village].city.expect("2/2026's city");
                assert_eq!(
                    built.sim.city_chain(c),
                    vec![village, silo, well],
                    "2/2026's chain on 2370"
                );
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            let ours = lines[1].as_ref().expect("the builds line");
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 1).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "builds' ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "builds range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
        }
    }

    /// **Row 133's SAM flies its rockets on their splines** (item 4604,
    /// `docs/RECGAME.md` §13.5): run1797's ranges. On 2198 who=2's SAM
    /// `2/2027` launches its first SAM Rocket (`missile="1"`) at who=1's
    /// `71`: theirs from node 0's (9079, 27270, 688) on a `calc_from_dir`
    /// spline (traj 2, `depth` 20, the polygon over `unit_move_speed ×
    /// 0x96`), ours from the building's (9024, 27264, 479) on a straight
    /// line. Every ammo range on 2197 to 2227, ours against theirs, byte
    /// for byte: two swings of three rockets, from nodes 0, 1 and 2, and
    /// their re-plans.
    #[test]
    fn row_133_s_sam_flies_its_rockets_on_their_splines_on_run1797_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SIXTH, 6) else {
            eprintln!("skipping: row 133's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1797/map-7"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1797's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 133's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        let mut launches = std::collections::BTreeSet::new();
        for state in 2197..=2227 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            let ours = lines[3].as_ref().expect("the ammo line");
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 3).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "ammo's ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "ammo range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            let sam = built.sim.building_by_o(2, 2027).expect("2/2027");
            for p in &built.sim.combat.projectiles {
                if p.shooter == sim::combat::Obj::Building(sam) {
                    assert!(p.spline.is_some(), "a SAM Rocket flies a spline");
                    launches.insert((p.launch.x, p.launch.y, p.sz));
                }
            }
        }
        assert_eq!(
            launches.into_iter().collect::<Vec<_>>(),
            vec![(9017, 27209, 688), (9040, 27248, 702), (9079, 27270, 688)],
            "its three nodes"
        );
    }

    /// **Row 133's SAM rocket strikes its own capital with the recapture
    /// bonus** (item 4655, `docs/RECGAME.md` §13.5): run1797's ranges, the
    /// row's builds parting on 2451 (range 2180, record 2000, `damage` ours
    /// 5,963, theirs 5,985). On 2450 two rounds land, as the ammo line on
    /// both sides says: aircraft `1/70`'s (dead before 2446) for 46, and
    /// the SAM `2/2027`'s rocket, its target `1/70` gone, which comes down
    /// at (6277, 29986) on who=2's own capital `2/2000`. Step 20 makes it
    /// 22 (attack 290, armour 7, pct 100), and `get_damage`'s last step
    /// doubles it: the capital's city's race is the attacker's player
    /// (`RECAPTURE_CITY_MODIFIER` 512). Every builds range on 2450 to 2452,
    /// ours against theirs, byte for byte, and the capital's `damage`.
    #[test]
    fn row_133_s_sam_rocket_strikes_its_own_capital_with_the_recapture_bonus_on_run1797_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SIXTH, 6) else {
            eprintln!("skipping: row 133's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1797/map-7"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1797's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 133's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        let mut damage = Vec::new();
        for state in 2450..=2452 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            for line in [1usize, 3] {
                let ours = lines[line].as_ref().expect("the line");
                let theirs: Vec<_> = all.iter().filter(|r| r.line == line as u8).collect();
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
            }
            let cap = built.sim.building_by_o(2, 2000).expect("2/2000");
            damage.push(built.sim.buildings[cap].damage);
        }
        assert_eq!(
            damage,
            vec![5895, 5985, 6031],
            "46 + 44 on 2451, 46 on 2452"
        );
    }

    /// **Row 133's Redoubt kills with `dtype` 3** (item 4659,
    /// `docs/RECGAME.md` §13.5): run1797's ranges, the row's deaths parting
    /// on 2457 (death slot 7, who=2's Elite Special Forces `2/0`, TI 27,
    /// land). who=1's Redoubt `1/2025` lands an 18 on it on 2456 and kills
    /// it. `get_shot` is 4 for a Redoubt, so `get_damage`'s building arm
    /// writes `dtype` 3, not 2: `cur_anim` ours 17, theirs 19 (the same
    /// even roll), and `take_damage`'s turn of a foot casualty toward the
    /// blow, `place.angle` ours `0xbd69188d`, theirs `0xe9b90000`, impact
    /// angle ours 100.0, theirs 38.0. Every deaths range on 2456 to 2458,
    /// ours against theirs, byte for byte, and the record's `cur_anim`.
    #[test]
    fn row_133_s_redoubt_kills_with_the_explosive_death_on_run1797_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SIXTH, 6) else {
            eprintln!("skipping: row 133's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1797/map-7"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1797's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 133's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 2456..=2458 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            let ours = lines[4].as_ref().expect("the deaths line");
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 4).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "deaths' ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "deaths range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
        }
        let d = built
            .sim
            .combat
            .deaths
            .iter()
            .find(|d| (d.who, d.o, d.first_frame) == (2, 0, 2456))
            .expect("2/0's death on 2456");
        assert_eq!(d.cur_anim, 19, "dtype 3: 2 × 3 + 0xd, an even roll");
    }

    /// **Row 28's two Bazookas drop their chase of a Citizen on one tick**
    /// (item 4613, `docs/RECGAME.md` §14.2): run1813's ranges, the
    /// seventh's first leaf. who=1's Bazookas `1/35` and `1/52` (TI 139,
    /// land, army slot 1, whose target is who=0's capital `0/2000`) each
    /// hold `[MOVE, ATTACK 0/7]` on both sides through state 3955, the
    /// Citizen `0/7` (TI 50) being the chase. On frame 3955 the original
    /// ends both walks and retargets both: `1/35` holds one order, an
    /// ATTACK on `0/2000`, with no path; `1/52` heads an ATTACK on `0/0`
    /// (the Special Forces, TI 75). Ours keeps both walks. `0/7`'s own
    /// record agrees on 3956, and the two `targeted` bumps theirs takes
    /// (`0/0` 5 → 6, `0/2000` 0 → 1) are the two searches' winners.
    /// Every units, builds, guys and world range on 3955, ours against
    /// theirs, byte for byte; on 3956, theirs' two records as read, and
    /// since item 4619 every range of the four lines on 3956 too: the
    /// actor is a third member of their group, `1/25`, whose find of `0/0`
    /// goes to `Group::target_opportunity` (run1816's packet at 3955).
    #[test]
    fn row_28_s_bazookas_drop_their_chase_of_a_citizen_on_run1813_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SEVENTH, 1) else {
            eprintln!("skipping: row 28's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1813/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1813's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 28's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        while built.sim.frame < 3955 {
            crate::diff::harness::debug_watch(&built, built.sim.frame);
            stream.apply(built.sim.frame as i32, &mut built);
            built.sim.command().tick();
        }
        crate::diff::harness::debug_watch(&built, built.sim.frame);
        if std::env::var("RON_DEBUG_UNIT").is_ok() {
            // The parting frame itself, on ours, for `RON_DEBUG_UNIT`'s
            // `@…-3956`: a copy ticked once, the assertions below untouched.
            let mut next = built.clone();
            stream.clone().apply(next.sim.frame as i32, &mut next);
            next.sim.command().tick();
            crate::diff::harness::debug_watch(&next, next.sim.frame);
        }
        let set = record.first_of(3954).expect("state 3955's set").clone();
        let all = record.ranges(&set).expect("the set's ranges");
        let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
        for (line, name) in [(0, "units"), (1, "builds"), (6, "guys"), (11, "world")] {
            let ours = lines[line].as_ref().expect("the line");
            let theirs: Vec<_> = all.iter().filter(|r| r.line as usize == line).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "{name}' ranges on 3955");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "{name} range {k} on 3955, record {} {}",
                    a.record, a.field
                );
            }
        }
        // Both chase `0/7` on 3955, under a walk.
        for o in [35, 52] {
            let u = built.sim.unit_by_o(1, o).expect("the Bazooka");
            let t = built.sim.units[u].combat.target.expect("a target");
            assert!(
                matches!(t, sim::combat::Obj::Unit(v)
                    if built.sim.units[v].owner == 0 && built.sim.units[v].index == 7),
                "1/{o} chases 0/7"
            );
            assert!(!built.sim.units[u].path.is_empty(), "1/{o} walks");
        }
        // State 3956, theirs: each Bazooka's record, cut at `flags`
        // (`006621fe`) and named by `who..y_internal` (`0066221c`).
        let set = record.first_of(3955).expect("state 3956's set").clone();
        let all = record.ranges(&set).expect("the set's ranges");
        let mut records: Vec<Vec<&crate::trace::ranges::Range>> = Vec::new();
        for r in all.iter().filter(|r| r.line == 0) {
            if r.site == 0x0066_21fe {
                records.push(Vec::new());
            }
            if let Some(l) = records.last_mut() {
                l.push(r);
            }
        }
        let of = |o: i16| {
            records
                .iter()
                .find(|rs| {
                    rs.iter().any(|r| {
                        r.site == 0x0066_221c
                            && r.bytes[0] == 1
                            && i16::from_le_bytes([r.bytes[1], r.bytes[2]]) == o
                    })
                })
                .unwrap_or_else(|| panic!("theirs' 1/{o} on 3956"))
        };
        let word =
            |b: &[u8], at: usize| i32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]]);
        // `path.size..increment` (`0046d8df`): the count at byte 4.
        let path = |rs: &Vec<&crate::trace::ranges::Range>| {
            word(
                &rs.iter()
                    .find(|r| r.site == 0x0046_d8df)
                    .expect("the path")
                    .bytes,
                4,
            )
        };
        // The head order: `orders.count` (`00730291`), its type
        // (`007303ac`), its target's `o` and `who` (`0047f291`).
        let head = |rs: &Vec<&crate::trace::ranges::Range>| {
            let count = word(
                &rs.iter()
                    .find(|r| r.site == 0x0073_0291)
                    .expect("the count")
                    .bytes,
                0,
            );
            let ty = word(
                &rs.iter()
                    .find(|r| r.site == 0x0073_03ac)
                    .expect("the type")
                    .bytes,
                0,
            );
            let t = &rs
                .iter()
                .find(|r| r.site == 0x0047_f291)
                .expect("the target")
                .bytes;
            (count, ty, word(t, 0), word(t, 4))
        };
        let (a35, a52) = (of(35), of(52));
        assert_eq!(path(a35), 0, "theirs' 1/35 walks no more");
        assert_eq!(
            head(a35),
            (1, 0x0a, 2000, 0),
            "theirs' 1/35: one ATTACK on 0/2000"
        );
        assert_eq!(
            head(a52),
            (2, 0x0a, 0, 0),
            "theirs' 1/52: ATTACK on 0/0 first"
        );
        // Ours on 3956 (item 4619): the frame ticked once, every units,
        // builds, guys and world range byte for byte. `1/25`'s
        // `think_attack` hands group 66 the Special Forces `0/0`, and each
        // member's `Unit::target_opportunity` passes the anti-siege shield,
        // whose held Citizen is not siege.
        stream.apply(built.sim.frame as i32, &mut built);
        built.sim.command().tick();
        let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
        for (line, name) in [(0, "units"), (1, "builds"), (6, "guys"), (11, "world")] {
            let ours = lines[line].as_ref().expect("the line");
            let theirs: Vec<_> = all.iter().filter(|r| r.line as usize == line).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "{name}' ranges on 3956");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "{name} range {k} on 3956, record {} {}",
                    a.record, a.field
                );
            }
        }
    }

    /// **Row 28's ammo on 4027** (item 4626, `docs/RECGAME.md` §14.2):
    /// run1820's ranges (units, builds, ammo, guys and world, to 4028).
    /// who=2's `2/34` fires at who=0's capital `0/2000` on frame 4026; its
    /// round lands in slot 1 on state 4027. Every kept line on states 4026
    /// and 4027, ours against theirs, byte for byte.
    #[test]
    fn row_28_s_round_at_the_capital_on_run1820_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SEVENTH, 1) else {
            eprintln!("skipping: row 28's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1820/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1820's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 28's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        let mut check = |built: &crate::diff::setup::Built, state: i64| {
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            for (line, name) in [
                (0, "units"),
                (1, "builds"),
                (3, "ammo"),
                (6, "guys"),
                (11, "world"),
            ] {
                let ours = lines[line].as_ref().expect("the line");
                let theirs: Vec<_> = all.iter().filter(|r| r.line as usize == line).collect();
                assert_eq!(ours.ranges.len(), theirs.len(), "{name}' ranges on {state}");
                for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                    assert_eq!(
                        a.bytes, b.bytes,
                        "{name} range {k} on {state}, record {} {}",
                        a.record, a.field
                    );
                }
            }
        };
        while built.sim.frame < 4026 {
            crate::diff::harness::debug_watch(&built, built.sim.frame);
            crate::diff::harness::debug_ammo(&built, built.sim.frame);
            stream.apply(built.sim.frame as i32, &mut built);
            built.sim.command().tick();
        }
        check(&built, 4026);
        stream.apply(built.sim.frame as i32, &mut built);
        built.sim.command().tick();
        crate::diff::harness::debug_watch(&built, built.sim.frame);
        crate::diff::harness::debug_ammo(&built, built.sim.frame);
        check(&built, 4027);
    }

    /// **Row 28's India Infantry looses from its bay** (item 4630,
    /// `docs/RECGAME.md` §14.2): run1821, row 28's file to 4168 with the
    /// ammo line's ranges. who=2's `2/50` (Infantry, TI 102, piece 1812)
    /// fires at the capital `0/2000` on 4166; theirs' round leaves (12973,
    /// 9025, 144), ours left the figure's square (13032, 9000, 24). Every
    /// units, builds, ammo, guys and world range on 4166 and 4167.
    #[test]
    fn row_28_s_round_on_run1821_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SEVENTH, 1) else {
            eprintln!("skipping: row 28's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1821/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1821's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 28's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        let mut check = |built: &crate::diff::setup::Built, state: i64| {
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            if state == 4167 {
                // Theirs' launch by value: ammo range 1, record 0, `launch.x`,
                // `launch.y` and `sz` at bytes 7, 11 and 15.
                let word =
                    |b: &[u8], at: usize| i32::from_le_bytes(b[at..at + 4].try_into().unwrap());
                let ammo = lines[3].as_ref().expect("ammo");
                assert_eq!(ammo.ranges[1].record, 0);
                assert_eq!(ammo.ranges[1].field, "rolling..start_roll_angle");
                let theirs: Vec<_> = all.iter().filter(|r| r.line == 3).collect();
                let b = &theirs[1].bytes;
                assert_eq!(
                    (word(b, 7), word(b, 11), word(b, 15)),
                    (12973, 9025, 144),
                    "theirs' launch"
                );
            }
            for (line, name) in [
                (0, "units"),
                (1, "builds"),
                (3, "ammo"),
                (6, "guys"),
                (11, "world"),
            ] {
                let ours = lines[line].as_ref().expect("the line");
                let theirs: Vec<_> = all.iter().filter(|r| r.line as usize == line).collect();
                assert_eq!(ours.ranges.len(), theirs.len(), "{name}' ranges on {state}");
                for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                    assert_eq!(
                        a.bytes, b.bytes,
                        "{name} range {k} on {state}, record {} {}",
                        a.record, a.field
                    );
                }
            }
        };
        while built.sim.frame < 4166 {
            stream.apply(built.sim.frame as i32, &mut built);
            built.sim.command().tick();
        }
        // The shooter by name: who=2's Infantry on piece 1812.
        let u = built.sim.unit_by_o(2, 50).expect("who=2's Infantry");
        let ty = built.sim.units[u].ty.expect("a type");
        assert_eq!(built.sim.tables.unit_types[ty].type_index, 102, "2/50's TI");
        assert_eq!(built.sim.units[u].guys[0].gpiece, 1812, "2/50's piece");
        assert_eq!(
            built.sim.units[u].kind.domain,
            sim::territory::attrition::Domain::Land
        );
        check(&built, 4166);
        stream.apply(built.sim.frame as i32, &mut built);
        built.sim.command().tick();
        check(&built, 4167);
    }

    /// **Row 128's Russians enter the Classical age and the borders are
    /// fixed** (item 4635, `docs/RECGAME.md` §14.6): run1828's leaders
    /// ranges to 12000. who=1 (tribe 13, tribe bonus 0xd) gains the age on
    /// 3336; `Leader::gain_tech@006dcb60:1322` calls
    /// `Regions::fix_all_borders` for an age under that bonus whether the
    /// border table moves or not, so on 3338 every leader's `reg_terr[1]`
    /// recounts from 0 (who=1's 104, the others' 0) and `check_borders`
    /// zeroes `reg_known_rares`. Ours held 198, 495 and 447. The leaders
    /// line on states 3336 to 3340, byte for byte.
    #[test]
    fn row_128_s_russian_age_fixes_the_borders_on_run1828_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SEVENTH, 6) else {
            eprintln!("skipping: row 128's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1828/map-14"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1828's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 128's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        while built.sim.frame < 3336 {
            stream.apply(built.sim.frame as i32, &mut built);
            built.sim.command().tick();
        }
        let ages = |b: &crate::diff::setup::Built| b.sim.tech[1].ages;
        let before = ages(&built);
        loop {
            let state = built.sim.frame;
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
            if state == 3338 {
                // who=1's `reg_terr[1]` by value: byte 5208 of its long range.
                let k = ours
                    .ranges
                    .iter()
                    .position(|r| r.record == 1 && r.field == "who..ally_mask")
                    .expect("who=1's long range");
                let v = u16::from_le_bytes(theirs[k].bytes[5208..5210].try_into().unwrap());
                assert_eq!(v, 104, "theirs' recount, one frame in");
            }
            if state == 3340 {
                break;
            }
            stream.apply(built.sim.frame as i32, &mut built);
            built.sim.command().tick();
        }
        assert_eq!(
            ages(&built),
            before + 1,
            "who=1 enters an age in the window"
        );
    }

    /// **Row 133's Temple site's close fixes the borders** (item 4607,
    /// `docs/RECGAME.md` §13.5): run1797's ranges. who=1 razes who=2's
    /// unfinished Temple `2/2009` on frame 2295 (state 2296).
    /// `Build::close@00628980` tests `is(0x1b5)` at `0x628d2c` and zeroes
    /// every region's resume index inline, built or not, so theirs' pass
    /// starts again on 2296. On 2297 every leader's `reg_terr[1]` is
    /// recounted (who=0's 0, who=1's 177, who=2's 0) and every city's
    /// `bordering` is cleared. Ours held who=0's 741 and city 1's 3. The
    /// leaders and cities lines on states 2295 to 2300, byte for byte.
    #[test]
    fn row_133_s_temple_site_close_fixes_the_borders_on_run1797_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SIXTH, 6) else {
            eprintln!("skipping: row 133's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1797/map-7"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1797's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 133's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 2295..=2300 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            for (line, name) in [(7, "leaders"), (8, "cities")] {
                let ours = lines[line].as_ref().expect("the line");
                let theirs: Vec<_> = all.iter().filter(|r| r.line as usize == line).collect();
                assert_eq!(ours.ranges.len(), theirs.len(), "{name}' ranges on {state}");
                for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                    assert_eq!(
                        a.bytes, b.bytes,
                        "{name} range {k} on {state}, record {} {}",
                        a.record, a.field
                    );
                }
            }
            let temple = built.sim.building_by_o(2, 2009);
            if state == 2295 {
                let t = temple.expect("2/2009 on 2295");
                assert!(!built.sim.buildings[t].active, "the Temple is a site");
            }
        }
        // who=1's `reg_terr` on 2300, the restarted pass's fourth frame: its
        // one land region (slot 1 in theirs) holds 755, not the 1,353 the
        // finished pass held.
        assert!(
            built.sim.territory.reg_terr[1].contains(&755),
            "who=1 recounts"
        );
        assert!(
            !built.sim.territory.reg_terr[1].contains(&1353),
            "not the old count"
        );
    }

    /// **Row 133's SAM winds on its sighting and turns its turret at it**
    /// (item 4602, `docs/RECGAME.md` §13.5): run1797's ranges. On 2166
    /// who=2's SAM `2/2027` holds `near_o` 71 (who 1) with no target
    /// (`attack_ox` −1); `Wall::inc_time@0063fb60:204` winds `recharging`
    /// on `near_o` alone (ours held 0 on the target's sign), and from 2167
    /// `Build::process@0061edf0`'s `61ef10`..`61ef91` turns `job_counter`
    /// (`+0x48`) at the sighting out of range (ours held 0x40000000).
    /// Every builds range on 2165 to 2172, ours against theirs, byte for
    /// byte, and the SAM's sighting is a unit of who=1 with no target.
    #[test]
    fn row_133_s_sam_winds_and_turns_at_its_sighting_on_run1797_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SIXTH, 6) else {
            eprintln!("skipping: row 133's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1797/map-7"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1797's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 133's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 2165..=2172 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            let ours = lines[1].as_ref().expect("the builds line");
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 1).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "builds' ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "builds range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
        }
        let sam = built.sim.building_by_o(2, 2027).expect("2/2027");
        let bd = &built.sim.buildings[sam];
        assert_eq!(bd.target, None, "the SAM takes no target");
        assert_eq!(bd.recharging, 7, "wound from 0 on 2165");
        let Some(sim::combat::Obj::Unit(u)) = bd.near else {
            panic!("the SAM's sighting is a unit: {:?}", bd.near)
        };
        assert_eq!(
            (built.sim.units[u].owner, built.sim.units[u].index),
            (1, 71)
        );
    }

    /// **Row 83's computer lists a caravan by its ally's city** (item
    /// 4545, `docs/CARAVAN.md` §4): run1784, the ranges take of row 83's
    /// file. On 1956 who=2 (the added computer, team 0 with the human)
    /// rebuilds its make list, and theirs' fifth slot is a Caravan (TI 59,
    /// `CARA`) at `val` 980,392, city 0. `create_units@006c40a0`'s caravan
    /// arm offers one only while `get_caravan_limit(1)` exceeds the
    /// caravans queued and standing, and that limit's `n` is who=2's two
    /// cities plus its mutual ally who=0's one under
    /// `has_preq(BASE_BONUSTYPES)` (`006dca50`): three pairings. Ours
    /// counted who=2's alone, one pairing, so the arm offered nothing and
    /// the slot kept its stale entry. Every leaders range on 1955 and 1956,
    /// ours against theirs, byte for byte: red on the tree before the fix
    /// at range 310, record 2, `make_list[k]`, `t` ours 561, theirs 59.
    #[test]
    fn row_83_s_computer_lists_a_caravan_by_its_ally_s_city_on_run1784_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 4) else {
            eprintln!("skipping: row 83's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1784/map-14"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1784's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 83's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        assert_eq!(built.sim.ai.lobby.team_style, 1, "row 83 locks its teams");
        for state in [1955, 1956] {
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
            // Who=2's fifth make-list slot: range 310, `t`, `val`, `escrow`, `city`.
            assert_eq!(ours.ranges[310].field, "make_list[k]");
            assert_eq!(ours.ranges[310].record, 2);
            let word = |n: usize| {
                i32::from_le_bytes(theirs[310].bytes[4 * n..4 * n + 4].try_into().unwrap())
            };
            let slot = (word(0), word(1), word(2), word(3));
            if state == 1956 {
                assert_eq!(slot, (59, 980_392, 1, 0), "theirs' Caravan entry");
            } else {
                assert_ne!(slot.0, 59, "no Caravan entry before 1956");
            }
            // Who=2's two cities and its mutual ally who=0's one make three
            // pairings; its own two made one.
            assert_eq!(
                (built.sim.cities_of(0).len(), built.sim.cities_of(2).len()),
                (1, 2),
                "the allies' cities on {state}"
            );
            assert!(built.sim.is_ally(2, 0) && !built.sim.is_ally(2, 1));
        }
        // After the ranges, so a limit without the ally term parts on range
        // 310 first.
        assert_eq!(built.sim.caravan_limit(2), 3, "who=2's limit on 1956");
    }

    /// **Row 83's Artillery carries the supply veto's mark** (item 4554,
    /// `docs/SUPPLY.md`, "Behavioural check (run16)"): run1784, the ranges
    /// take of row 83's file. who=1's Artillery `1/70` (TI 269, land)
    /// bleeds on a period of 48 on enemy ground, and on its tick due on
    /// 2810 `process_supply` answers supplied: `Unit::process@00610bc0:535`
    /// ORs `0x40000` into `unit_masks2` (`+0x6c`), and `:453` clears it on
    /// the next 32-frame refresh, frame 2842 (its phase `frame + 70`). Ours
    /// took the veto and kept it in a separate bool the walk never printed.
    /// Every units range on 2810 to 2812, 2842 and 2843, ours against
    /// theirs, byte for byte: red on the tree before the fix at range 2319,
    /// record 70, `unit_masks2` ours 0, theirs 0x40000.
    #[test]
    fn row_83_s_artillery_carries_the_supply_veto_s_mark_on_run1784_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 4) else {
            eprintln!("skipping: row 83's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1784/map-14"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1784's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 83's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for (state, marked) in [
            (2810, false),
            (2811, true),
            (2812, true),
            (2842, true),
            (2843, false),
        ] {
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
            // The Artillery by name: who=1's record 70, its `unit_masks2`
            // (`+0x6c`, byte 0x24 of the `collide_frame..play` range).
            let k = ours
                .ranges
                .iter()
                .enumerate()
                .filter(|(_, r)| r.record == 70 && r.field == "who..y_internal")
                .find(|(k, _)| theirs[*k].bytes[0] == 1)
                .map(|(k, _)| k)
                .expect("1/70");
            assert_eq!(
                &theirs[k + 1].bytes[..],
                &269i32.to_le_bytes(),
                "1/70's TypeIndex"
            );
            let body = (k..ours.ranges.len())
                .find(|&j| ours.ranges[j].field == "collide_frame..play")
                .expect("1/70's body");
            let masks2 = u32::from_le_bytes(theirs[body].bytes[0x24..0x28].try_into().unwrap());
            assert_eq!(
                masks2,
                if marked { 0x4_0000 } else { 0 },
                "theirs' unit_masks2 on {state}"
            );
            let u = built
                .sim
                .units
                .iter()
                .position(|u| u.alive() && u.owner == 1 && u.index == 70)
                .expect("1/70");
            let unit = &built.sim.units[u];
            assert_eq!(
                built.unit_type_names[unit.ty.expect("its type")],
                "Artillery"
            );
            assert_eq!(unit.attrition, 48, "1/70's period on {state}");
            assert_eq!(unit.sheltered(), marked, "ours' veto on {state}");
        }
    }

    /// **Row 83's Taj Mahal site keeps its hit points through a blow until
    /// the next walk** (item 4556, `docs/CITIES.md` §3.4): run1784, the
    /// ranges take of row 83's file. who=2's unfinished Taj Mahal `2/2037`
    /// (`TypeIndex` 0x21b) takes a 19-point round on 3041, after its
    /// builders' progress put `job_counter` at 76,214: the blow takes
    /// `19 × 50` off it, to 75,264. `Wall::inc_time@0063fb60:48`..`56`
    /// refreshes a site's `construct_hits` in `Objects::inc_time`'s leader
    /// walk, ahead of the ammo list, so theirs prints 6023 (`site_hits` at
    /// 76,214) on 3041 and 6017 on 3042; ours refreshed after the ammo list
    /// and printed 6017 on 3041. Every builds range on 3040 to 3042, ours
    /// against theirs, byte for byte: red on the tree before the fix at
    /// range 3157, record 2037, `construct_hits` ours 6017, theirs 6023.
    #[test]
    fn row_83_s_taj_mahal_site_keeps_its_hits_through_a_blow_on_run1784_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 4) else {
            eprintln!("skipping: row 83's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1784/map-14"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1784's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 83's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        // (state, theirs' `damage`, theirs' `construct_hits`, ours'
        // `job_counter`).
        for (state, damage, hits, jc) in [
            (3040, 0, 6022, None),
            (3041, 19, 6023, Some(75_264)),
            (3042, 19, 6017, Some(75_264)),
        ] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            let ours = lines[1].as_ref().expect("builds");
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 1).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "builds' ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "builds range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            // The site by name: who=2's record 2037, its type, its
            // object's `damage` (`+0x24`, byte 4 of `myhits..launch_frames`)
            // and its `construct_hits` (`+0x54`, byte 12 of
            // `job_counter..demolition`).
            let k = ours
                .ranges
                .iter()
                .enumerate()
                .filter(|(_, r)| r.record == 2037 && r.field == "who..y_internal")
                .find(|(k, _)| theirs[*k].bytes[0] == 2)
                .map(|(k, _)| k)
                .expect("2/2037");
            assert_eq!(ours.ranges[k + 1].field, "type");
            assert_eq!(
                &theirs[k + 1].bytes[..],
                &0x21bi32.to_le_bytes(),
                "2/2037's TypeIndex"
            );
            let word = |field: &str, at: usize| {
                let j = (k..ours.ranges.len())
                    .find(|&j| ours.ranges[j].field == field)
                    .expect("2/2037's range");
                i32::from_le_bytes(theirs[j].bytes[at..at + 4].try_into().unwrap())
            };
            assert_eq!(
                word("myhits..launch_frames", 4),
                damage,
                "theirs' damage on {state}"
            );
            assert_eq!(
                word("job_counter..demolition", 12),
                hits,
                "theirs' construct_hits on {state}"
            );
            let b = built.sim.building_by_o(2, 2037).expect("2/2037");
            let bd = &built.sim.buildings[b];
            assert_eq!(
                loaded.build_type_names[bd.ty.expect("its type")],
                "Taj Mahal"
            );
            assert!(!bd.active, "2/2037 is a site on {state}");
            if let Some(jc) = jc {
                assert_eq!(bd.job_counter, jc, "ours' job_counter on {state}");
            }
            if state == 3041 {
                // theirs' 6023 is the site's figure at the pre-blow 76,214.
                assert_eq!(
                    sim::cities::build::site_hits(
                        bd.hits,
                        75_264 + 19 * 50,
                        built.sim.construct_time_of(b),
                        true
                    ),
                    6023
                );
            }
        }
    }

    /// **Row 83's Barracks re-homes when its city is taken** (item 4568,
    /// `docs/RECGAME.md` §12.3): run1784, the ranges take of row 83's file.
    /// who=2's city `2/2020` falls on frame 3644; its chain was 2020, 2027,
    /// the Barracks 2036, the wonder 2037 and 2039, and every member but the
    /// Barracks closes with it. `Cities::capture_city@00733380:1157` closes
    /// the old record on the chain the member closes left, and
    /// `City::close@00737550` sends each live non-centre building in it to
    /// `Build::find_city`: theirs' state 3645 has `2/2036`'s `city` 1 at the
    /// tail of who=2's city 1, `2/2029`'s `city_down` 2036. Ours cleared the
    /// list first and left it on the dead slot 2. Every units, builds, guys
    /// and cities range on 3644 to 3646, ours against theirs, byte for byte:
    /// red on the tree before the fix at builds record 2029, `city_down` ours
    /// −1, theirs 2036, on 3645.
    #[test]
    fn row_83_s_barracks_re_homes_when_its_city_is_taken_on_run1784_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 4) else {
            eprintln!("skipping: row 83's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1784/map-14"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1784's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 83's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 3644..=3646 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            for (li, name) in [(0u8, "units"), (1, "builds"), (6, "guys"), (8, "cities")] {
                let ours = lines[usize::from(li)].as_ref().expect(name);
                let theirs: Vec<_> = all.iter().filter(|r| r.line == li).collect();
                assert_eq!(ours.ranges.len(), theirs.len(), "{name}' ranges on {state}");
                for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                    assert_eq!(
                        a.bytes, b.bytes,
                        "{name} range {k} on {state}, record {} {}",
                        a.record, a.field
                    );
                }
            }
            // The Barracks and its new chain by name, on theirs' side:
            // who=2's `2/2036` (`city`, `+0x72`) and `2/2029` (`city_down`,
            // `+0x74`), each the bytes of its record's `BuildData` range.
            let builds = lines[1].as_ref().expect("builds");
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 1).collect();
            let field = |o: i32, at: usize| -> i16 {
                let mut who = -1i16;
                for (k, a) in builds.ranges.iter().enumerate() {
                    if a.field == "who..y_internal" {
                        who = i16::from(theirs[k].bytes[0]);
                    }
                    if who == 2
                        && i64::from(a.record) == i64::from(o)
                        && a.field == "gather_down..infiltrate2"
                    {
                        return i16::from_le_bytes([theirs[k].bytes[at], theirs[k].bytes[at + 1]]);
                    }
                }
                panic!("2/{o}'s BuildData on {state}");
            };
            let (city, down) = if state == 3644 { (2, 2037) } else { (1, -1) };
            assert_eq!(field(2036, 2), city, "2/2036's city on {state}");
            assert_eq!(field(2036, 4), down, "2/2036's city_down on {state}");
            let after = if state == 3644 { -1 } else { 2036 };
            assert_eq!(field(2029, 4), after, "2/2029's city_down on {state}");
            let b = built.sim.building_by_o(2, 2036).expect("2/2036");
            assert_eq!(
                built.sim.tables.build_types[built.sim.buildings[b].orig_ty.expect("its type")]
                    .ident,
                sim::cities::build::Ident::Barracks
            );
            assert_eq!(
                built.sim.building_by_o(2, 2020).is_some(),
                state == 3644,
                "2/2020, the city, stands on 3644 alone"
            );
        }
    }

    /// **Row 83's taken city assimilates three frames a frame faster with
    /// its owner's Citizen in reach** (item 4570, `docs/RECGAME.md` §12.3):
    /// run1784, the ranges take of row 83's file. who=1 took who=2's city
    /// `1/2043` on frame 3644, so its `assimilation_timer` is 3644 and it
    /// is unassimilated (`race` 2). `Build::process@0061edf0:698` asks
    /// `ObjectData::has_general(this, 0, 0x164)` of the city building, and
    /// while one of the owner's Citizens is in `find_hero`'s reach the stamp
    /// walks back `THECITIZEN_ASSIMILATION_SPEED − 1`, three a frame:
    /// who=1's Citizen `1/15` (`TypeIndex` 356) walks in, and theirs' stamp
    /// reads 3644 on 3655, then 3641 and 3638. Ours read a flag nothing
    /// wrote. Every cities range on 3655 to 3658, ours against theirs, byte
    /// for byte: red on the tree before the fix at cities record 8,
    /// `assimilation_timer` ours 3644, theirs 3641, on 3656.
    #[test]
    fn row_83_s_taken_city_assimilates_faster_by_its_owner_s_citizen_on_run1784_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 4) else {
            eprintln!("skipping: row 83's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1784/map-14"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1784's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 83's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for (state, stamp) in [(3655, 3644), (3656, 3641), (3657, 3638), (3658, 3635)] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            let ours = lines[8].as_ref().expect("cities");
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 8).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "cities' ranges on {state}");
            let mut timer = None;
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "cities range {k} on {state}, record {} {}",
                    a.record, a.field
                );
                if a.record == 8 {
                    let f = crate::checksum::fields::range_fields("cities", a.field, &b.bytes);
                    if let Some(v) = f
                        .unwrap_or_default()
                        .iter()
                        .find(|(n, _)| n == "assimilation_timer")
                    {
                        timer = Some(v.1);
                    }
                }
            }
            // Theirs' stamp by name, then the city and the Citizen on ours.
            assert_eq!(
                timer,
                Some(stamp),
                "record 8's assimilation_timer on {state}"
            );
            let b = built.sim.building_by_o(1, 2043).expect("1/2043");
            let c = built.sim.buildings[b].city.expect("its city");
            let city = &built.sim.cities.cities[c];
            assert_eq!((city.owner, city.race, city.building), (1, Some(2), b));
            let u = built.sim.unit_by_o(1, 15).expect("1/15");
            assert_eq!(built.sim.units[u].type_index, 356, "1/15 is a Citizen");
        }
    }

    /// **Row 83's Airbase keeps its Biplane in when an enemy wonder out of
    /// reach outscores the city** (item 4562, `docs/RECGAME.md` §12.3):
    /// run1784, the ranges take of row 83's file. who=2's computer sortie
    /// at its Airbase `2/2018` on frame 3358 (`(3358 + 2018) % 32 == 0`)
    /// scores who=1's wonder `1/2026` at 217, `(0 / 1000 + 1000) × 10 /
    /// (d / 768 + 1)`, over its own city's: `Object::do_launch@0064f3b0`'s
    /// wonders loop (`64fb6a`..`64fda4`). The wonder is past the Biplane
    /// `2/117`'s reach, so its orders are cleared and it takes none, and
    /// it stays in the base; ours scored the cities alone and sent it on
    /// patrol. Every units and builds range on 3358 to 3360, ours against
    /// theirs, byte for byte: red on the tree before the fix at units range
    /// 7702, record 117, `orders.count` ours 1, theirs 0 on 3359.
    #[test]
    fn row_83_s_airbase_keeps_its_biplane_when_a_wonder_outscores_the_city_on_run1784_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 4) else {
            eprintln!("skipping: row 83's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1784/map-14"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1784's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 83's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 3358..=3360 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            for (li, name) in [(0u8, "units"), (1, "builds")] {
                let ours = lines[usize::from(li)].as_ref().expect(name);
                let theirs: Vec<_> = all.iter().filter(|r| r.line == li).collect();
                assert_eq!(ours.ranges.len(), theirs.len(), "{name}' ranges on {state}");
                for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                    assert_eq!(
                        a.bytes, b.bytes,
                        "{name} range {k} on {state}, record {} {}",
                        a.record, a.field
                    );
                }
            }
            // The Biplane by name: who=2's unit record 117, theirs'
            // `TypeIndex` 0x11f, no order on its stack.
            let units = lines[0].as_ref().expect("units");
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 0).collect();
            let k = units
                .ranges
                .iter()
                .enumerate()
                .filter(|(_, r)| r.record == 117 && r.field == "who..y_internal")
                .find(|(k, _)| theirs[*k].bytes[0] == 2)
                .map(|(k, _)| k)
                .expect("2/117");
            assert_eq!(
                &theirs[k + 1].bytes[..],
                &0x11fi32.to_le_bytes(),
                "2/117's TypeIndex"
            );
            let j = (k..units.ranges.len())
                .find(|&j| units.ranges[j].field == "orders.count")
                .expect("2/117's orders");
            assert_eq!(
                &theirs[j].bytes[..],
                &[0, 0, 0, 0],
                "2/117's orders on {state}"
            );
            let u = built.sim.unit_by_o(2, 117).expect("2/117");
            assert_eq!(
                loaded.unit_type_names[built.sim.units[u].ty.expect("its type")],
                "Biplane"
            );
            assert!(
                !built.sim.units[u].on_map,
                "2/117 is in its base on {state}"
            );
            // The base holds it: who=2's builds record 2018's
            // `inside_down` (`+0x28`, byte 8 of `myhits..launch_frames`) is 117
            // on both sides.
            let builds = lines[1].as_ref().expect("builds");
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 1).collect();
            let k = builds
                .ranges
                .iter()
                .enumerate()
                .filter(|(_, r)| r.record == 2018 && r.field == "who..y_internal")
                .find(|(k, _)| theirs[*k].bytes[0] == 2)
                .map(|(k, _)| k)
                .expect("2/2018");
            let j = (k..builds.ranges.len())
                .find(|&j| builds.ranges[j].field == "myhits..launch_frames")
                .expect("2/2018's inside_down");
            assert_eq!(
                &theirs[j].bytes[8..10],
                &117i16.to_le_bytes(),
                "2/2018's inside_down on {state}"
            );
        }
        // The two buildings the sortie weighed, by type, and the wonder's
        // figure: undamaged, ×10 for who=2's team trailing on wonder
        // points, over 46 steps of 768 from the base.
        let base = built.sim.building_by_o(2, 2018).expect("2/2018");
        let wonder = built.sim.building_by_o(1, 2026).expect("1/2026");
        let (bb, wb) = (&built.sim.buildings[base], &built.sim.buildings[wonder]);
        assert_eq!(loaded.build_type_names[bb.ty.expect("its type")], "Airbase");
        assert!(
            built.sim.is_wonder_building(wb.ty.expect("its type")),
            "1/2026 is a wonder"
        );
        assert!(wb.active && wb.damage == 0, "1/2026 stands whole");
        let d = sim::world::vector_dist(wb.pos.x - bb.pos.x, wb.pos.y - bb.pos.y);
        assert_eq!(d / 768 + 1, 46);
        assert_eq!((wb.damage / 1000 + 1000) * 10 / (d / 768 + 1), 217);
    }

    /// **Row 103's caravan road is laid on the frame after the budget**
    /// (item 4544, `docs/ROADS.md` §8.3): run1783, the ranges take of
    /// row 103's file. A caravan's road search costs 3,207, 3,202 and
    /// 3,203 nodes on trace frames 1747 to 1749, node for node on both
    /// sides, and pops its goal on 1749 with 0xc80 spent. Theirs parks it
    /// (`astar_caravan_road@00685990`, `6860af`) and lays the road on
    /// 1750 having costed nothing; ours laid it on 1749, thirteen cells
    /// from (44, 18) to (49, 24) a state early. Every world range on states
    /// 1749 to 1751, ours against theirs, byte for byte: red on the tree
    /// before the fix at range 1148, record 1124, `cell` byte 0, `ROAD`
    /// ours 0x80, theirs 0.
    #[test]
    fn row_103_s_caravan_road_is_laid_the_frame_after_the_budget_on_run1783_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 5) else {
            eprintln!("skipping: row 103's recording is missing");
            return;
        };
        let map = home.join("ron-golden/run1783/map-18");
        let (Some(ranges), Some(log)) = (
            crate::testenv::experiment(&map, "rontrace.ranges"),
            crate::testenv::experiment(&map, "rontrace.log"),
        ) else {
            eprintln!("skipping: run1783's ranges record or trace is missing");
            return;
        };
        let tr = crate::trace::Trace::read(&log)
            .expect("run1783's trace reads")
            .expect("a trace");
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 103's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        built.sim.trace.trace_costs = true;
        let road = sim::world::Cell::new(44, 18);
        for (state, nodes, laid) in [(1749, 3202, false), (1750, 3203, false), (1751, 0, true)] {
            while built.sim.frame < state {
                built.sim.trace.road_marks.clear();
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            // The frame before the state: its road nodes, both sides.
            let ours_nodes = std::mem::take(&mut built.sim.trace.road_marks);
            let theirs_nodes = tr.road_nodes(state - 1);
            assert_eq!(
                theirs_nodes.len(),
                nodes,
                "theirs' road nodes on {}",
                state - 1
            );
            assert_eq!(ours_nodes, theirs_nodes, "the road nodes on {}", state - 1);
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_world();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 11).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "world's ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "world range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            // The road's first cell by name: record 1124, `flags`' low byte.
            let k = ours
                .ranges
                .iter()
                .position(|r| r.record == 1124 && r.field == "cell")
                .expect("cell 1124");
            let want = if laid { 0x80 } else { 0 };
            assert_eq!(theirs[k].bytes[0] & 0x80, want, "theirs' ROAD on {state}");
            assert_eq!(
                built.sim.world.cell_data(road).flags & sim::ids::cell::ROAD,
                u16::from(want),
                "ours' ROAD on {state}"
            );
        }
    }

    /// **Row 103's army record keeps its role when a squad is pushed out**
    /// (item 4551, `docs/GROUPS.md` §3.4): run1783, the ranges take of row
    /// 103's file. On 2556 who=2's Infantry `2/69` disembarks and
    /// `disembark_squad` pushes it; `push_group`'s second walk asks army
    /// 1's group (pool slot 3, record 131) to `Group::kill@00714110` it,
    /// which stamps and runs the speed tail and calls no `find_role`. So
    /// theirs' record 131 lists `2/65`, `2/66`, `2/58` on 2557 and still
    /// reads `role` 0x150d00, the Infantry's `FOOT | INFANTRY` bits in it;
    /// ours re-roled it 0x050500 in the army's recount. Every groups range
    /// on states 2556 to 2558, ours against theirs, byte for byte: red on
    /// the tree before the fix at range 215 on 2557, record 131, `id..march`
    /// bytes 49 and 50, ours 0x05 0x05, theirs 0x0d 0x15.
    #[test]
    fn row_103_s_army_record_keeps_its_role_when_a_squad_leaves_on_run1783_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 5) else {
            eprintln!("skipping: row 103's recording is missing");
            return;
        };
        let map = home.join("ron-golden/run1783/map-18");
        let Some(ranges) = crate::testenv::experiment(&map, "rontrace.ranges") else {
            eprintln!("skipping: run1783's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 103's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [2556i64, 2557, 2558] {
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
            // Record 131 by name: `num` (byte 8) and `role` (bytes 48..52).
            let k = ours
                .ranges
                .iter()
                .position(|r| r.record == 131 && r.field == "id..march")
                .expect("record 131");
            let word = |n: usize| u32::from_le_bytes(theirs[k].bytes[n..n + 4].try_into().unwrap());
            let (num, role) = (word(8), word(48));
            let want = match state {
                2556 => 4,
                2557 => 3,
                _ => 6,
            };
            assert_eq!(num, want, "theirs' record 131 `num` on {state}");
            assert_eq!(role, 0x150d00, "theirs' record 131 `role` on {state}");
        }
    }

    /// **Row 103's Armored Scout Car fires from its turret's muzzle** (item
    /// 4559, `docs/RECGAME.md` §12.3): run1783, the ranges take of row 103's
    /// file. On 2859 who=1's Armored Scout Car `1/35` (unit 223 of our
    /// array; TI 218, the Asian art style's piece 1224) looses its
    /// `CHAR_ATTACK2` round; with no row for the piece it left from its own
    /// square, and the ammo line parted on 2860 at `launch.x`, ours 8952,
    /// theirs 8994. The piece is 169's model, anims and scale
    /// (`sim::data::pivot`'s `SAME_AS`), so the round leaves through the
    /// pivot arm, from (8994, 15803, 236): every ammo range on 2858..=2870
    /// byte for byte.
    #[test]
    fn row_103_s_armored_scout_car_fires_from_its_turret_on_run1783_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 5) else {
            eprintln!("skipping: row 103's recording is missing");
            return;
        };
        let map = home.join("ron-golden/run1783/map-18");
        let Some(ranges) = crate::testenv::experiment(&map, "rontrace.ranges") else {
            eprintln!("skipping: run1783's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 103's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 2858..=2870 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            if state == 2860 {
                let p = built
                    .sim
                    .combat
                    .projectiles
                    .iter()
                    .find(|p| p.slot == 0)
                    .expect("the round in slot 0");
                let sim::combat::Obj::Unit(u) = p.shooter else {
                    panic!("a unit's round");
                };
                let x = &built.sim.units[u];
                assert_eq!((x.owner, x.index), (1, 35), "the shooter is 1/35");
                assert_eq!(x.type_index, 218, "1/35 is an Armored Scout Car");
                assert_eq!(x.guys[0].gpiece, 1224, "1/35's piece");
                assert_eq!((p.launch.x, p.launch.y, p.sz), (8994, 15803, 236));
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let theirs: Vec<_> = record
                .ranges(&set)
                .expect("the set's ranges")
                .into_iter()
                .filter(|r| r.line == 3)
                .collect();
            let ours = built.sim.checksum_ammo().expect("ours' ammo line");
            assert_eq!(ours.ranges.len(), theirs.len(), "ammo's ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "ammo range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
        }
    }

    /// **Row 103's President clears his crew's discs by their own cells'
    /// region** (item 4567, `docs/COLLISION.md` §2): run1783, the ranges
    /// take of row 103's file. On 2983 who=1's President `1/14` (TI 355),
    /// aboard Transport Freighter `1/120` since 2490, becomes the CEO (TI
    /// 357). `Unit::set_type@00612fa0`'s second kill loop hands crew
    /// figures 1 and 2 to `Objects::kill_guy@00659410`, which clears
    /// `radius[coll_size]` around each figure's unit cell, gated by the
    /// figure's **world cell's plain `region`** (`WData +0x4`). Figure 2
    /// stands at unit cell (299, 640), on the ocean half of coastal cell
    /// (18, 40), where `get_tregion` answers the sea's `region2`; ours
    /// gated by that and cleared nothing there. Every world range on states
    /// 2982 to 2984, ours against theirs, byte for byte: red on the tree
    /// before the fix at range 6479 on 2983, record 2358 (cell (18, 39)),
    /// `block.body` byte 21, ours 0xc0, theirs 0x40.
    #[test]
    fn row_103_s_president_clears_his_crew_by_their_cells_region_on_run1783_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 5) else {
            eprintln!("skipping: row 103's recording is missing");
            return;
        };
        let map = home.join("ron-golden/run1783/map-18");
        let Some(ranges) = crate::testenv::experiment(&map, "rontrace.ranges") else {
            eprintln!("skipping: run1783's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 103's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [2982i64, 2983, 2984] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let president = built
                .sim
                .units
                .iter()
                .find(|x| x.owner == 1 && x.index == 14)
                .expect("1/14");
            assert!(!president.on_map, "1/14 is aboard on {state}");
            let want = if state == 2982 { 355 } else { 357 };
            assert_eq!(president.type_index, want, "1/14's TypeIndex on {state}");
            if state == 2982 {
                // Figure 2's own cell is coastal: its tile is the sea's.
                let g = president.guys[2].follow.expect("figure 2 tracks").body.pos;
                assert_eq!((g.x / 48, g.y / 48), (299, 640), "figure 2's unit cell");
                let cell = sim::world::Cell::new(18, 40);
                assert_ne!(
                    built.sim.world.tregion_alt(g.tile()),
                    built.sim.world.region_of(cell),
                    "figure 2's tile region is not its cell's"
                );
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_world();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 11).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "world's ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "world range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
        }
    }

    /// **Row 123's producers lap from 2001 when the cursor is another
    /// player's** (item 4546, `docs/RECGAME.md` §12.3): run1785, the ranges
    /// take of row 123's file. `find_counters[0x1c]` is one array for every
    /// player; who=2's `train_unit_with_cost` on 2951 left it at 2011, past
    /// who=1's count (its mark is 2011 too). On 2976 who=1's script trains
    /// three of type 50 at its two producers, 2000 and 2007.
    /// `ScenarioFuncSet::find_build@009e2970` resets a cursor at or past the
    /// player's count to 2000 and starts at 2001, so theirs queue at 2007,
    /// 2000, 2007; ours started at 2000 and went 2000, 2007, 2000. Every
    /// builds, groups and units range on states 2976 to 2978, ours against
    /// theirs, byte for byte: red on the tree before the fix at builds range
    /// 261, record 2000, `queued` ours 3, theirs 2.
    #[test]
    fn row_123_s_producer_lap_starts_past_the_base_on_run1785_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 6) else {
            eprintln!("skipping: row 123's recording is missing");
            return;
        };
        let map = home.join("ron-golden/run1785/map-14");
        let Some(ranges) = crate::testenv::experiment(&map, "rontrace.ranges") else {
            eprintln!("skipping: run1785's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 123's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [2976i64, 2977, 2978] {
            while built.sim.frame < state {
                if built.sim.frame == 2976 {
                    // The cursor who=2 left, at who=1's count.
                    let train = sim::ai::ai_host::counter::TRAIN_PRODUCER;
                    assert_eq!(built.sim.ai.script_env.find_counters[train], 2011);
                    assert_eq!(built.sim.roster.marks[1].build, 2011, "who=1's count");
                }
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            for li in [0usize, 1, 5] {
                let ours = lines[li].as_ref().expect("the line");
                let theirs: Vec<_> = all.iter().filter(|r| usize::from(r.line) == li).collect();
                assert_eq!(
                    ours.ranges.len(),
                    theirs.len(),
                    "line {li}'s ranges on {state}"
                );
                for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                    assert_eq!(
                        a.bytes, b.bytes,
                        "line {li} range {k} on {state}, record {} {}",
                        a.record, a.field
                    );
                }
            }
            // The two producers by name: who=1's 2000 and 2007 (builds
            // ranges 261 and 479 on 2977, `queued` at byte 18): empty on
            // 2976, two queued apiece on 2977 and 2978, theirs and ours (one
            // at 2000 queued earlier on frame 2976, before the three calls).
            let want = if state == 2976 { [0, 0] } else { [2, 2] };
            for (o, n) in [2000, 2007].into_iter().zip(want) {
                let b = built.sim.building_by_o(1, o).expect("who=1's producer");
                assert!(built.sim.buildings[b].active, "{o} active");
                assert_eq!(
                    built.sim.buildings[b].queue.items.len(),
                    n,
                    "ours' {o} on {state}"
                );
            }
            if state == 2977 {
                let builds = lines[1].as_ref().expect("builds");
                let theirs: Vec<_> = all.iter().filter(|r| r.line == 1).collect();
                for (k, o) in [(261, 2000), (479, 2007)] {
                    assert_eq!(builds.ranges[k].record, o);
                    assert_eq!(builds.ranges[k].field, "gather_down..infiltrate2");
                    assert_eq!(theirs[k].bytes[0x82 - 0x70], 2, "theirs' {o} `queued`");
                }
                // who=1's `last_group` after the three pushes: slot 1
                // (65) on both sides; before the fix ours stood at 64.
                let groups = lines[5].as_ref().expect("groups");
                let k = groups.ranges.len() - 8 + 1;
                assert_eq!(groups.ranges[k].field, "last_group");
                assert_eq!(
                    groups.ranges[k].bytes,
                    65i32.to_le_bytes(),
                    "who=1's last_group"
                );
            }
        }
    }

    /// **Row 43's Village holds the wounded Spearmen** (item 4565,
    /// `docs/RECGAME.md` §12.3): run1787, the ranges take of row 43's file.
    /// who=2's Village `2/2010` has held who=1's Spearmen `1/46` (TI 166,
    /// land) since its last arrow, and its count reaches 0 on frame 3298;
    /// `compare_target` re-weighs every candidate there. Theirs keeps `1/46`
    /// (`attack_ox` 46 on 3299): its `damage` is 4 (122 of its `myhits`
    /// 126, over the type's 120), so `×3/2`, and `1/49`, unhurt, has a move
    /// at the head of its orders, so `/4`. Ours read the type's hits and the
    /// destination, which `1/49` no longer held when the Village weighed it
    /// on 3298, and shot `1/49`. The units,
    /// builds and ammo lines on states 3297 to 3301, byte for byte.
    #[test]
    fn row_43_s_village_holds_the_wounded_spearmen_on_run1787_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 2) else {
            eprintln!("skipping: row 43's recording is missing");
            return;
        };
        let map = home.join("ron-golden/run1787/map-7");
        let Some(ranges) = crate::testenv::experiment(&map, "rontrace.ranges") else {
            eprintln!("skipping: run1787's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 43's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 3297i64..=3301 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            if state == 3298 {
                // The shooter and the two candidates by name.
                let sim = &built.sim;
                let village = sim
                    .buildings
                    .iter()
                    .position(|b| {
                        b.owner == 2
                            && b.index == 2010
                            && b.ty.is_some_and(|t| {
                                sim.tables.build_types[t].ident
                                    == sim::cities::build::Ident::Village
                            })
                    })
                    .expect("who=2's Village");
                assert_eq!(sim.buildings[village].recharging, 0, "the count is out");
                for o in [46, 49] {
                    let u = sim.unit_by_o(1, o).expect("who=1's Spearmen");
                    let ty = sim.units[u].ty.expect("a type");
                    assert_eq!(sim.tables.unit_types[ty].type_index, 166, "1/{o}'s TI");
                    assert_eq!(sim.tables.unit_types[ty].hits, 120, "the type's hits");
                    assert_eq!(
                        sim.units[u].kind.domain,
                        sim::territory::attrition::Domain::Land
                    );
                }
                let held = sim.unit_by_o(1, 46).expect("1/46");
                assert_eq!(
                    sim.buildings[village].target,
                    Some(sim::combat::Obj::Unit(held)),
                    "the Village holds 1/46"
                );
                assert_eq!(
                    (sim.units[held].max_health, sim.units[held].health),
                    (126, 122),
                    "1/46's damage, 4"
                );
                let walker = sim.unit_by_o(1, 49).expect("1/49");
                assert!(
                    sim.units[walker]
                        .orders
                        .front()
                        .is_some_and(|o| o.is_move()),
                    "1/49: a move at the head"
                );
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            for li in [0usize, 1, 3] {
                let ours = lines[li].as_ref().expect("the line");
                let theirs: Vec<_> = all.iter().filter(|r| usize::from(r.line) == li).collect();
                assert_eq!(
                    ours.ranges.len(),
                    theirs.len(),
                    "line {li}'s ranges on {state}"
                );
                for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                    let at = a.bytes.iter().zip(&b.bytes).position(|(x, y)| x != y);
                    assert!(
                        at.is_none() && a.bytes.len() == b.bytes.len(),
                        "line {li} range {k} on {state}, record {} {}: byte {at:?} ours {:?}, theirs {:?}",
                        a.record,
                        a.field,
                        at.map(|i| a.bytes[i]),
                        at.map(|i| b.bytes[i])
                    );
                }
            }
            if state == 3299 {
                // Theirs by value: the Village's `attack_ox` (`+0x7c`, byte
                // 12 of `gather_down..infiltrate2`) is 46, `attack_whom` 1.
                let builds = lines[1].as_ref().expect("builds");
                // Record 2010 of who=2: its head range names the owner.
                let who_of = |k: usize| {
                    builds.ranges[..=k]
                        .iter()
                        .rev()
                        .find(|r| r.record == 2010 && r.field.starts_with("who"))
                        .and_then(|r| {
                            crate::checksum::fields::range_fields("builds", r.field, &r.bytes)
                        })
                        .and_then(|f| f.iter().find(|(n, _)| n == "who").map(|x| x.1))
                };
                let k = (0..builds.ranges.len())
                    .find(|&k| {
                        let r = &builds.ranges[k];
                        r.record == 2010
                            && r.field == "gather_down..infiltrate2"
                            && who_of(k) == Some(2)
                    })
                    .expect("the Village's range");
                let theirs: Vec<_> = all.iter().filter(|r| r.line == 1).collect();
                let b = &theirs[k].bytes;
                assert_eq!(
                    (i16::from_le_bytes([b[12], b[13]]), b[17]),
                    (46, 1),
                    "theirs' attack_ox and attack_whom"
                );
            }
        }
    }

    /// **Row 43's follower keeps its captain's target past the leash**
    /// (item 4564, `docs/COMBAT.md` §43.2): run1787, the ranges take of
    /// row 43's file. who=2's Hoplites `2/63` (TI 132, land, `DEFENSIVE`)
    /// follow `2/62`, whose action is an attack on who=1's Slingers `1/23`.
    /// `2/63` enters `fight` on 3441 under an attack on `1/46` with a post
    /// (9816, 28680) it is past; the mirror takes `1/23`, the leash's
    /// `find_new_target` re-adds `1/23` with the post kept, and the
    /// original's "same target again" asks `fight`'s entry `(46, 1)`, so
    /// the attack stands with `unit_masks2` 16. Ours asked the mirrored
    /// target and killed it. The units, builds, ammo and guys lines on
    /// states 3440 to 3444, byte for byte.
    #[test]
    fn row_43_s_follower_keeps_its_captain_s_target_past_the_leash_on_run1787_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 2) else {
            eprintln!("skipping: row 43's recording is missing");
            return;
        };
        let map = home.join("ron-golden/run1787/map-7");
        let Some(ranges) = crate::testenv::experiment(&map, "rontrace.ranges") else {
            eprintln!("skipping: run1787's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 43's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 3440i64..=3444 {
            while built.sim.frame < state {
                if built.sim.frame == 3441 {
                    let sim = &built.sim;
                    let u = sim.unit_by_o(2, 63).expect("who=2's Hoplites");
                    let ty = sim.units[u].ty.expect("a type");
                    assert_eq!(sim.tables.unit_types[ty].type_index, 132, "2/63's TI");
                    assert_eq!(
                        sim.units[u].kind.domain,
                        sim::territory::attrition::Domain::Land
                    );
                    assert_eq!(sim.units[u].combat.stance, sim::combat::Stance::Defensive);
                    let cap = sim.unit_by_o(2, 62).expect("2/62");
                    assert_eq!(sim.units[u].o_up, Some(cap), "2/63 follows 2/62");
                    let s23 = sim.unit_by_o(1, 23).expect("1/23");
                    let s46 = sim.unit_by_o(1, 46).expect("1/46");
                    assert_eq!(
                        sim.units[cap].combat.target,
                        Some(sim::combat::Obj::Unit(s23))
                    );
                    assert_eq!(
                        sim.units[u].combat.target,
                        Some(sim::combat::Obj::Unit(s46))
                    );
                    let attack = sim.units[u]
                        .orders
                        .iter()
                        .find_map(|o| match o.body {
                            sim::orders::Body::Attack(a) => Some(a),
                            _ => None,
                        })
                        .expect("2/63's attack");
                    assert!(attack.defensive, "a post");
                    assert_eq!(attack.def, Some(sim::Pos::new(9816, 28680)), "the post");
                }
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            for li in [0usize, 1, 3, 6] {
                let ours = lines[li].as_ref().expect("the line");
                let theirs: Vec<_> = all.iter().filter(|r| usize::from(r.line) == li).collect();
                assert_eq!(
                    ours.ranges.len(),
                    theirs.len(),
                    "line {li}'s ranges on {state}"
                );
                for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                    let at = a.bytes.iter().zip(&b.bytes).position(|(x, y)| x != y);
                    assert!(
                        at.is_none() && a.bytes.len() == b.bytes.len(),
                        "line {li} range {k} on {state}, record {} {}: byte {at:?} ours {:?}, theirs {:?}",
                        a.record,
                        a.field,
                        at.map(|i| a.bytes[i]),
                        at.map(|i| b.bytes[i])
                    );
                }
            }
            if state == 3442 {
                // Theirs by value: 2/63's head order is the attack on
                // (23, 1), and its `unit_masks2` carries 16.
                let units = lines[0].as_ref().expect("units");
                let theirs: Vec<_> = all.iter().filter(|r| r.line == 0).collect();
                let head = |k: usize| {
                    units.ranges[..=k]
                        .iter()
                        .rev()
                        .find(|r| r.record == 63 && r.field.starts_with("who"))
                        .and_then(|r| {
                            crate::checksum::fields::range_fields("units", r.field, &r.bytes)
                        })
                        .and_then(|f| f.iter().find(|(n, _)| n == "who").map(|x| x.1))
                };
                let field = |name: &str| {
                    (0..units.ranges.len())
                        .find(|&k| {
                            let r = &units.ranges[k];
                            r.record == 63 && r.field == name && head(k) == Some(2)
                        })
                        .expect("2/63's range")
                };
                let k = field("order.target");
                let t = crate::checksum::fields::range_fields(
                    "units",
                    "order.target",
                    &theirs[k].bytes,
                )
                .expect("theirs' target");
                let get = |n: &str| t.iter().find(|(m, _)| m == n).map(|x| x.1);
                assert_eq!(
                    (get("ox"), get("whom")),
                    (Some(23), Some(1)),
                    "theirs' target"
                );
                let k = field("collide_frame..play");
                let m = crate::checksum::fields::range_fields(
                    "units",
                    "collide_frame..play",
                    &theirs[k].bytes,
                )
                .expect("theirs' masks");
                assert_eq!(
                    m.iter().find(|(n, _)| n == "unit_masks2").map(|x| x.1),
                    Some(16),
                    "theirs' frozen mark"
                );
            }
        }
    }

    /// **Row 93's Volksgrenadiers heal beside their President** (item 4577,
    /// `docs/RECGAME.md` §13): run1799, the ranges take of row 93's file.
    /// who=1's Volksgrenadiers `1/39` (TI 109, land, a captain) walk under
    /// an `AttackTo` guarding who=1's President `1/17` (TI 355); on frame
    /// 2341, `(39 + 2341) % 20 == 0`, `process_healing`'s President arm
    /// takes the figure `1/40`'s one point and sets the captain's `healing`
    /// to 20. Every units range on states 2341 to 2343, byte for byte.
    #[test]
    fn row_93_s_volksgrenadiers_heal_beside_their_president_on_run1799_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SIXTH, 4) else {
            eprintln!("skipping: row 93's recording is missing");
            return;
        };
        let map = home.join("ron-golden/run1799/map-14");
        let Some(ranges) = crate::testenv::experiment(&map, "rontrace.ranges") else {
            eprintln!("skipping: run1799's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 93's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 2341i64..=2343 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let captain = built.sim.unit_by_o(1, 39).expect("who=1's Volksgrenadiers");
            let figure = built.sim.unit_by_o(1, 40).expect("the squad's figure");
            let president = built.sim.unit_by_o(1, 17).expect("who=1's President");
            if state == 2341 {
                let ti = |u: usize| {
                    built.sim.tables.unit_types[built.sim.units[u].ty.expect("a type")].type_index
                };
                assert_eq!(ti(captain), 109, "1/39's TI");
                assert!(built.sim.units[captain].captain);
                assert_eq!(
                    built.sim.units[captain].kind.domain,
                    sim::territory::attrition::Domain::Land
                );
                assert_eq!(ti(president), 0x163, "1/17 is THEPRESIDENT");
                let f = &built.sim.units[figure];
                assert_eq!(
                    (f.max_health - f.health, f.damage_frac),
                    (1, 2),
                    "1/40 down a point"
                );
            }
            if state == 2342 {
                let f = &built.sim.units[figure];
                assert_eq!(built.sim.units[captain].healing, 20, "the arm's period");
                assert_eq!(
                    (f.max_health - f.health, f.damage_frac),
                    (0, 6),
                    "theirs' 0 and 6"
                );
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            let ours = lines[0].as_ref().expect("units");
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
    }

    /// **Row 93's siege order walks its point** (item 4606, `docs/RECGAME.md`
    /// §13.3): run1799, the ranges take of row 93's file. On 2962 who=2's
    /// Howitzer `2/77` (TI 271, land) holds a fired `ATTACK_GROUND` (`flags` 0x80) ahead of its
    /// `ATTACK` and `ATTACK_TO`, and `AttackGroundOrder::walk_data@00487a60`
    /// walks the flags byte and `att_x..attack_unit` after the node: units
    /// range 6364, which ours walked as the next order's type. Every units
    /// range on states 2962 to 2964, byte for byte.
    #[test]
    fn row_93_s_siege_order_walks_its_point_on_run1799_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SIXTH, 4) else {
            eprintln!("skipping: row 93's recording is missing");
            return;
        };
        let map = home.join("ron-golden/run1799/map-14");
        let Some(ranges) = crate::testenv::experiment(&map, "rontrace.ranges") else {
            eprintln!("skipping: run1799's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 93's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 2962i64..=2964 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            if state == 2962 {
                let sieges: Vec<_> = built
                    .sim
                    .units
                    .iter()
                    .filter(|u| u.alive() && u.index == 77)
                    .filter(|u| {
                        matches!(
                            u.orders.front().map(|o| o.body),
                            Some(sim::orders::Body::AttackGround(_))
                        )
                    })
                    .collect();
                assert_eq!(sieges.len(), 1, "one 77 with a ground order");
                let u = sieges[0];
                let ty = u.ty.expect("a type");
                assert_eq!(loaded.unit_names[ty], "Howitzer", "2/77's name");
                assert_eq!(u.owner, 2, "who=2's 77");
                assert_eq!(built.sim.tables.unit_types[ty].type_index, 271, "2/77's TI");
                assert_eq!(u.kind.domain, sim::territory::attrition::Domain::Land);
                let head = u.orders.front().expect("a head order");
                assert_eq!(head.flags, sim::orders::flag::FIRED, "fired");
                let sim::orders::Body::AttackGround(g) = head.body else {
                    unreachable!("filtered above")
                };
                assert_eq!(
                    (g.at.x, g.at.y, g.sea, g.attack_unit),
                    (12984, 22872, false, 1),
                    "theirs' range 6365: b8320000 58590000 0 1"
                );
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            let ours = lines[0].as_ref().expect("units");
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
    }

    /// **Row 43's Slinger stones leave from piece 736's sockets** (item
    /// 4547, `docs/RECGAME.md` §12.3): run1787, the ranges take of row 43's
    /// file. who=2's Slingers `2/30` and `2/29` (TI 82, land, piece 736,
    /// `32 + 2 × 0x160`) loose at who=1's `1/14` on 3191 and 3192 on
    /// `CHAR_ATTACK1` frame 21; theirs' stones leave from (8571, 23834) and
    /// (8714, 24012), 174 over the ground, where ours left the figure's own
    /// square. Every ammo and units range on states 3191 to 3197, byte for
    /// byte.
    #[test]
    fn row_43_s_slinger_stones_leave_from_piece_736_s_sockets_on_run1787_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 2) else {
            eprintln!("skipping: row 43's recording is missing");
            return;
        };
        let map = home.join("ron-golden/run1787/map-7");
        let Some(ranges) = crate::testenv::experiment(&map, "rontrace.ranges") else {
            eprintln!("skipping: run1787's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 43's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 3191i64..=3197 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            if state == 3191 {
                // The two shooters by name: who=2's Slingers on piece 736.
                for o in [29, 30] {
                    let u = built.sim.unit_by_o(2, o).expect("who=2's Slinger");
                    let ty = built.sim.units[u].ty.expect("a type");
                    assert_eq!(built.sim.tables.unit_types[ty].type_index, 82, "2/{o}'s TI");
                    assert_eq!(built.sim.units[u].guys[0].gpiece, 736, "2/{o}'s piece");
                    assert_eq!(
                        built.sim.units[u].kind.domain,
                        sim::territory::attrition::Domain::Land
                    );
                }
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            for li in [0usize, 3] {
                let ours = lines[li].as_ref().expect("the line");
                let theirs: Vec<_> = all.iter().filter(|r| usize::from(r.line) == li).collect();
                assert_eq!(
                    ours.ranges.len(),
                    theirs.len(),
                    "line {li}'s ranges on {state}"
                );
                for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                    assert_eq!(
                        a.bytes, b.bytes,
                        "line {li} range {k} on {state}, record {} {}",
                        a.record, a.field
                    );
                }
            }
            // Theirs' launch points by value: record 0's on 3192 and record
            // 2's on 3193, `launch.x`/`launch.y`/`sz` at bytes 7, 11, 15.
            let word = |b: &[u8], at: usize| i32::from_le_bytes(b[at..at + 4].try_into().unwrap());
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 3).collect();
            let named = match state {
                3192 => Some((1, 0, (8571, 23834, 690))),
                3193 => Some((7, 2, (8714, 24012, 668))),
                _ => None,
            };
            if let Some((k, slot, (x, y, z))) = named {
                let ammo = lines[3].as_ref().expect("ammo");
                assert_eq!(ammo.ranges[k].record, slot);
                assert_eq!(ammo.ranges[k].field, "rolling..start_roll_angle");
                let b = &theirs[k].bytes;
                assert_eq!(
                    (word(b, 7), word(b, 11), word(b, 15)),
                    (x, y, z),
                    "theirs' launch"
                );
            }
        }
    }

    /// **Row 43's unbuilt Village closes and the borders are fixed** (item
    /// 4561, `docs/RECGAME.md` §12.3): run1787, the ranges take of row 43's
    /// file. who=2's Village site (building 2027 at cell (27, 25), placed,
    /// never started) closes on frame 3221 (state 3222); theirs'
    /// `Build::close` takes its `is a city` arm (`SubObjectData::flags &
    /// 0x20`, the whole `VILLAGE` lineage, built or not) and calls
    /// `Region::fix_borders`, so the next frame's `check_borders` starts the
    /// pass again from every region's first cell: on 3223 every leader's
    /// `reg_terr[1]` is zeroed and recounted (who=1's 130, theirs) and every
    /// city's `bordering` is cleared. Ours carried the Temple's pass of 3210
    /// on to its end (who=0's 368, cities 3, 4 and 6 at 6). The leaders and
    /// cities lines on states 3221 to 3226, byte for byte.
    #[test]
    fn row_43_s_unbuilt_village_close_fixes_the_borders_on_run1787_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 2) else {
            eprintln!("skipping: row 43's recording is missing");
            return;
        };
        let map = home.join("ron-golden/run1787/map-7");
        let Some(ranges) = crate::testenv::experiment(&map, "rontrace.ranges") else {
            eprintln!("skipping: run1787's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 43's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        let village = |sim: &sim::Sim| {
            sim.buildings.iter().position(|b| {
                b.owner == 2
                    && b.index == 2027
                    && b.ty.is_some_and(|t| {
                        sim.tables.build_types[t].ident == sim::cities::build::Ident::Village
                    })
            })
        };
        for state in 3221i64..=3226 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            if state == 3221 {
                // The site by name: who=2's Village, placed and never started.
                let b = village(&built.sim).expect("who=2's Village site");
                let site = &built.sim.buildings[b];
                assert!(site.alive && !site.started && !site.active, "the site");
                assert_eq!(site.pos.cell(), sim::world::Cell { x: 27, y: 25 });
                assert_eq!(site.city, None, "an unbuilt site has no city");
            }
            if state == 3222 {
                let b = village(&built.sim).expect("who=2's Village site");
                assert!(!built.sim.buildings[b].alive, "the site closed");
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            for li in [7usize, 8] {
                let ours = lines[li].as_ref().expect("the line");
                let theirs: Vec<_> = all.iter().filter(|r| usize::from(r.line) == li).collect();
                assert_eq!(
                    ours.ranges.len(),
                    theirs.len(),
                    "line {li}'s ranges on {state}"
                );
                for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                    let at = a.bytes.iter().zip(&b.bytes).position(|(x, y)| x != y);
                    assert!(
                        at.is_none() && a.bytes.len() == b.bytes.len(),
                        "line {li} range {k} on {state}, record {} {}: byte {at:?} ours {:?}, theirs {:?}",
                        a.record,
                        a.field,
                        at.map(|i| a.bytes[i]),
                        at.map(|i| b.bytes[i])
                    );
                }
            }
            if state == 3223 {
                // Theirs by value: who=1's `reg_terr[1]` (`+0x145e + 2`,
                // byte 5208 of `who..ally_mask`) is the restarted pass's
                // first frame, 130; who=0's and who=2's are 0.
                let leaders = lines[7].as_ref().expect("leaders");
                let theirs: Vec<_> = all.iter().filter(|r| r.line == 7).collect();
                for (who, want) in [(0i16, 0u16), (1, 130), (2, 0)] {
                    let k = leaders
                        .ranges
                        .iter()
                        .position(|r| r.record == who && r.field == "who..ally_mask")
                        .expect("the leader's range");
                    let b = &theirs[k].bytes;
                    assert_eq!(
                        u16::from_le_bytes([b[5208], b[5209]]),
                        want,
                        "theirs' who={who} reg_terr[1]"
                    );
                }
            }
        }
    }

    /// Item 4642 also changes Great Sahara's group destination. Run1787
    /// keeps every field around the first step of the recorded formation.
    #[test]
    fn row_43_s_muster_and_first_steps_match_run1787() {
        use crate::diff::checksum_lines::packets::{Pieces, compare, our_lines};
        let (Some(inst), Some(pieces), (Some(rcx), _)) =
            (crate::testenv::install(), Pieces::read(), paths(&FIFTH, 2))
        else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1787/map-7"), "rontrace.ranges")
        else {
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).unwrap();
        let loaded = crate::load::load(&inst).unwrap();
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).unwrap();
        assert_eq!(record.mask, 0xffb);
        for state in 3831..=3836 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let theirs = crate::diff::checksum_frames::record::stream_at(&mut record, state)
                .expect("run1787 covers the muster");
            let ours = our_lines(&built.sim, &pieces);
            for (line, outcome) in compare(&ours, &theirs).iter().enumerate() {
                if record.mask & (1 << line) != 0 {
                    assert_eq!(
                        outcome.parted, 0,
                        "state {state}, line {line}: {:?}",
                        outcome.first
                    );
                }
            }
        }
    }

    /// **The fifth's files carry the declared seats and team** (item
    /// 4527): each `item4527-row<RRR>-6000.rcx` the client recorded, read
    /// back, holds the declaration's three `PLAYER` rows slot for slot —
    /// `flags`, `tribe`, `who`, `team`, `handicap`, `play` — no fourth
    /// active slot, and `PLAYERS 2`, so what the original plays is the
    /// game declared before it was written.
    #[test]
    fn the_fifth_s_files_carry_the_declared_seats_and_team() {
        files_carry_the_declared_seats_and_team(&FIFTH);
    }

    fn files_carry_the_declared_seats_and_team(b: &Battery) {
        let field = |row: &str, key: &str| -> i64 {
            row.split("; ")
                .find_map(|kv| kv.strip_prefix(key)?.strip_prefix(' ')?.parse().ok())
                .unwrap_or_else(|| panic!("{key} in {row}"))
        };
        for (k, held) in b.table.iter().enumerate() {
            let (Some(rcx), _) = paths(b, k) else {
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

    /// **A take in flight** (DECISIONS 156 §5): the runner writes
    /// `<run>/receipt.json` when the game has exited, so a trace with no
    /// receipt beside its map directory is still being written, or died.
    /// `gate-4629` went red on run1810's trace, twenty minutes into its
    /// take, on a row no pin yet named.
    fn in_flight(take: &std::path::Path) -> bool {
        take.parent()
            .and_then(std::path::Path::parent)
            .is_none_or(|run| !run.join("receipt.json").is_file())
    }

    #[test]
    fn a_trace_with_no_receipt_is_a_take_in_flight() {
        let run = std::env::temp_dir().join(format!("attrition-in-flight-{}", std::process::id()));
        let map = run.join("map-7");
        std::fs::create_dir_all(&map).expect("the run directory");
        let take = map.join("rontrace.log");
        std::fs::write(&take, b"RONT").expect("the trace");
        assert!(in_flight(&take), "no receipt yet");
        std::fs::write(run.join("receipt.json"), b"[]").expect("the receipt");
        assert!(!in_flight(&take), "the receipt is written");
        std::fs::remove_dir_all(&run).expect("removed");
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
            if pin.is_none() && in_flight(&take) {
                eprintln!(
                    "skipping: row {} run{} has no receipt and no pin, a take in flight",
                    held.row,
                    b.run + k
                );
                continue;
            }
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
            let (fails, drift) = held_verdict(&want, &got);
            for d in &drift {
                eprintln!(
                    "HELD OUT DRIFT row {} run{}: {d} (printed, not held)",
                    held.row,
                    b.run + k
                );
            }
            if !fails.is_empty() {
                wrong.push(format!(
                    "row {} run{}: {}; got {got:?}",
                    held.row,
                    b.run + k,
                    fails.join("; ")
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

    /// **Row 110's Elite Special Forces jam who=1's Tower** (item 4541,
    /// `docs/COMBAT.md` §8.6): run1742, the ranges take of run1735's file.
    /// who=2's Elite Special Forces `2/0` (TI 77, land, specials slot 0)
    /// stands beside who=1's Tower `1/2040` (type 111, `ANTI_AIR`) when the
    /// Tower's 32-frame `Build::do_attack@006228f0` runs on 5576.
    /// `ObjectData::is_jammed@00653660` finds it inside `GENERAL_RADIUS ×
    /// 0x180`, so the Tower takes `build_masks |= 0x8000` and holds its
    /// shot, and the unit takes `unit_masks2 |= 8`; on 5600, the unit's own
    /// phase, `Unit::process@00610bc0`'s caster arm trades the 8 for
    /// `unit_masks |= 0x80000000`. Red on the tree before the fix at units
    /// range 1673 on 5577, record 0 `collide_frame..play` byte 36
    /// (`unit_masks2`), ours 0, theirs 8, and builds range 1554, record
    /// 2040 `job_counter..demolition` byte 25 (`build_masks`' high byte),
    /// ours 0x10, theirs 0x90. Every units and builds range on 5575 to
    /// 5610, byte for byte.
    #[test]
    fn row_110_s_special_forces_jam_the_tower_on_run1742_s_ranges() {
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
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let tribes = inst.tribe_defs(&inst.rules().unwrap()).unwrap();
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 5575i64..=5610 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            for (line, name) in [(0u8, "units"), (1, "builds")] {
                let theirs: Vec<_> = all.iter().filter(|r| r.line == line).collect();
                let ours = lines[line as usize].as_ref().expect("the line");
                for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                    let at = a.bytes.iter().zip(&b.bytes).position(|(x, y)| x != y);
                    assert_eq!(
                        at.map(|i| (i, a.bytes[i], b.bytes[i])),
                        None,
                        "{name} range {k} on {state}, record {} {}: (byte, ours, theirs)",
                        a.record,
                        a.field
                    );
                    assert_eq!(a.bytes.len(), b.bytes.len(), "{name} range {k} on {state}");
                }
                assert_eq!(ours.ranges.len(), theirs.len(), "{name} ranges on {state}");
            }
            let sim = &built.sim;
            let sf = sim
                .units
                .iter()
                .position(|u| u.alive() && u.owner == 2 && u.index == 0)
                .expect("2/0");
            let tower = sim
                .buildings
                .iter()
                .position(|b| b.alive && b.owner == 1 && b.index == 2040)
                .expect("1/2040");
            let jammer = sim.units[sf].unit_masks2 & sim::combat::umask2::JAMMER != 0;
            if state == 5577 {
                // The two by name, and theirs' two bits.
                let u = &sim.units[sf];
                assert_eq!(u.type_index, 77, "Elite Special Forces");
                assert_eq!(u.kind.domain, sim::territory::attrition::Domain::Land);
                assert_eq!(u.special, Some(0), "specials slot 0");
                assert_eq!(sim.buildings[tower].ty, Some(111), "the Tower");
                let units: Vec<_> = all.iter().filter(|r| r.line == 0).collect();
                let builds: Vec<_> = all.iter().filter(|r| r.line == 1).collect();
                assert_eq!(units[1673].bytes[36], 8, "theirs' unit_masks2");
                assert_eq!(builds[1554].bytes[25], 0x90, "theirs' build_masks");
                assert!(jammer, "ours' unit_masks2 & 8");
                assert!(sim.buildings[tower].jammed, "ours' build_masks & 0x8000");
                assert!(!sim.units[sf].jam_shown);
            }
            if state == 5601 {
                // The caster arm on 5600: the 8 traded for the top bit.
                assert!(!jammer, "the 8 dropped on 5600");
                assert!(sim.units[sf].jam_shown, "unit_masks & 0x80000000");
                assert!(sim.buildings[tower].jammed, "the Tower stays jammed");
            }
        }
    }

    /// **Row 110's Space Program sees every enemy building in the danger
    /// map** (item 4550, `docs/DANGER.md` §5): run1742, the ranges take of
    /// run1735's file. who=2 holds `leader_flags & 0x800` from its Space
    /// Program between the rebuilds of 5400 and 5600, and
    /// `WallData::is_seen@00642bd0`'s second clause takes that bit (beside
    /// `ever_seen & ally_mask`, `num_units[0x141]` and `reveal_map == 3`)
    /// for a started building. So `GameDaemon::do_danger@00732390`'s enemy
    /// arm stops halving who=1's buildings in who=2's row on the rebuild of
    /// 5600, among them the Barracks-line `1/2030` (type 13, no city) that
    /// no ally of who=2 has ever seen. This crate's `build_is_seen` read
    /// `ever_seen` alone: red on the tree before the fix at world range 3631
    /// on 5601, record 2 `danger`, byte 2328 (half-cell (12, 19)), ours
    /// 0x25, theirs 0x32. Every world range on 5600 and 5601, byte for
    /// byte.
    #[test]
    fn row_110_s_space_program_sees_every_enemy_building_in_the_danger_map_on_run1742_s_ranges() {
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
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let tribes = inst.tribe_defs(&inst.rules().unwrap()).unwrap();
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [5401i64, 5600, 5601] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let sim = &built.sim;
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            // who=2's `leader_flags` byte 1, theirs: `0x800` is off on 5401
            // and on by 5600.
            let lines = crate::checksum::state::ranges(sim, &look, &tribes);
            let leaders = lines[7].as_ref().expect("the leaders line");
            let theirs_leaders: Vec<_> = all.iter().filter(|r| r.line == 7).collect();
            let k = leaders
                .ranges
                .iter()
                .position(|r| r.record == 2 && r.field == "flags")
                .expect("who=2's flags");
            let reveal = theirs_leaders[k].bytes[1] & 0x08 != 0;
            assert_eq!(
                reveal,
                state != 5401,
                "theirs' who=2 leader_flags & 0x800 on {state}"
            );
            assert_eq!(
                leaders.ranges[k].bytes, theirs_leaders[k].bytes,
                "who=2's flags"
            );
            if state == 5401 {
                continue;
            }
            // The Barracks-line building by name: started, never seen by
            // who=2's side, and seen by the leader bit alone.
            let b = sim
                .buildings
                .iter()
                .position(|b| b.alive && b.owner == 1 && b.index == 2030)
                .expect("1/2030");
            let bd = &sim.buildings[b];
            assert_eq!(bd.ty, Some(13), "1/2030's type");
            assert!(
                bd.active && bd.started && bd.city.is_none(),
                "1/2030 finished, no city"
            );
            assert_eq!(bd.ever_seen & 0x4, 0, "who=2 never saw 1/2030");
            let ours = sim.checksum_world();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 11).collect();
            if state == 5601 {
                // Half-cell (12, 19) of who=2's row, 1/2030's north-west
                // neighbour: its 50, halved once for the ring and not
                // again for being unseen.
                let k = ours
                    .ranges
                    .iter()
                    .position(|r| r.record == 2 && r.field == "danger")
                    .expect("who=2's danger row");
                assert_eq!(k, 3631, "the booking's range");
                assert_eq!(
                    theirs[k].bytes[2328..2332],
                    [0x32, 0, 0, 0],
                    "theirs' (12, 19)"
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
            // Asked after the comparison, so the red names the record.
            if state == 5601 {
                assert!(sim.build_is_seen(b, 2), "who=2 sees 1/2030 by its 0x800");
            }
        }
    }

    /// **Row 110's losing wonder site closes without the unit-stats bit**
    /// (item 4540, `docs/RECGAME.md` §11): run1742, the ranges take of
    /// run1735's file. On 5408 who=2's wonder `2/2030` finishes and
    /// who=1's site of the same type, unfinished, closes with it.
    /// `Build::close@00628980`'s wonder arm (`close_wonder`, `leader_flags
    /// |= 0x4000000`) is gated on `BuildData::wonder >= 0` (`+0x76`), the
    /// slot only an activation's `Wonders::init_wonder` gives, so the site
    /// raises the close's wall bit alone. Every leaders range on 5408 to
    /// 5410, ours against theirs, byte for byte: red on the tree before the
    /// fix at range 128 on 5409, record 1 `flags`, byte 3 ours `0e`,
    /// theirs `0a`.
    #[test]
    fn row_110_s_losing_wonder_site_closes_without_the_unit_stats_bit_on_run1742_s_ranges() {
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
        for state in 5408..=5410 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 7).collect();
            let ours = built.sim.checksum_leaders();
            assert_eq!(ours.ranges.len(), theirs.len(), "leaders ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "leaders range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            if state == 5409 {
                // who=1's `leader_flags`, by name: the economy and wall
                // bits, not the unit-stats one.
                assert_eq!(
                    (ours.ranges[128].record, ours.ranges[128].field),
                    (1, "flags")
                );
                assert_eq!(theirs[128].bytes[3], 0x0a, "theirs' leader_flags byte 3");
                // The two buildings: who=2's wonder up, who=1's site gone.
                let sim = &built.sim;
                let won = sim
                    .buildings
                    .iter()
                    .position(|b| b.owner == 2 && b.index == 2030)
                    .expect("2/2030");
                let ty = sim.buildings[won].ty.expect("its type");
                assert!(sim.is_wonder_building(ty) && sim.buildings[won].active);
                assert!(
                    sim.buildings
                        .iter()
                        .any(|b| b.owner == 1 && b.ty == Some(ty) && !b.alive && !b.active),
                    "who=1's unfinished site of the type, closed"
                );
            }
        }
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

    /// **Row 33's musket ball leaves from the original's socket** (item
    /// 4591, `docs/RECGAME.md` §13): run1801, the ammo take of run1789's
    /// file, `RNG_MASK=0x8u`. State 3386's ammo range 4, record 4, parted on
    /// `launch.x` (ours 6906, theirs 6907) and the aim it sets: who=2's
    /// Musketeers `2/25` (TI 99, land, piece 49) loosing `CHAR_ATTACK3` at
    /// frame 4. Every ammo range on 3385 to 3400, byte for byte.
    #[test]
    fn row_33_s_musket_ball_leaves_from_the_original_s_socket_on_run1801_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SIXTH, 1) else {
            eprintln!("skipping: row 33's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1801/map-7"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1801's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 33's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 3385i64..=3400 {
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
            if state == 3386 {
                // Theirs' range 4 `launch.x, launch.y`.
                let b = &theirs[4].bytes;
                let word =
                    |k: usize| i32::from_le_bytes(b[3 + 4 * k..7 + 4 * k].try_into().unwrap());
                assert_eq!((word(1), word(2)), (6907, 32948), "theirs' launch");
                // The ball, by name: `2/25`, TI 99, land.
                let p = built
                    .sim
                    .combat
                    .projectiles
                    .iter()
                    .find(|p| (p.launch.x, p.launch.y) == (6907, 32948))
                    .expect("ours' ball");
                let sim::combat::Obj::Unit(u) = p.shooter else {
                    panic!("a unit's round")
                };
                let u = &built.sim.units[u];
                assert_eq!((u.owner, u.index, u.type_index), (2, 25, 99));
                assert_eq!(u.kind.domain, sim::territory::attrition::Domain::Land);
            }
        }
    }

    /// **Row 113's city walks its caravan links in the order they were
    /// made** (item 4592, `docs/RECGAME.md` §13.4, `docs/CHECKSUM.md` §12):
    /// run1802, the cities take of run1793's file, `RNG_MASK=0x100u`. On
    /// 3995 who=1's Caravan `1/3` links who=1's city 0 to who=2's city 1,
    /// and city 0's `vans` already holds who=2's `2/3`: theirs appends, so
    /// range 11, record 0, `vans[k]` byte 4 (`who`) reads 2, where a walk by
    /// owner wrote 1. Every cities range on 3994 to 3996, byte for byte.
    #[test]
    fn row_113_s_city_walks_its_caravan_links_in_the_order_they_were_made_on_run1802_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&SIXTH, 5) else {
            eprintln!("skipping: row 113's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1802/map-14"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1802's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 113's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 3994i64..=3996 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let ours = built.sim.checksum_cities();
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 8).collect();
            assert_eq!(ours.ranges.len(), theirs.len(), "cities ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                let at = a.bytes.iter().zip(&b.bytes).position(|(x, y)| x != y);
                assert_eq!(
                    at.map(|i| (i, a.bytes[i], b.bytes[i])),
                    None,
                    "cities range {k} on {state}, record {} {}: (byte, ours, theirs)",
                    a.record,
                    a.field
                );
                assert_eq!(a.bytes.len(), b.bytes.len(), "cities range {k} on {state}");
            }
            if state == 3995 {
                // Theirs' city 0 of who=1: `(cara, who)` at ranges 9 to 12.
                let link = |k: usize| {
                    let b = &theirs[k].bytes;
                    (
                        i32::from_le_bytes(b[0..4].try_into().unwrap()),
                        i32::from_le_bytes(b[4..8].try_into().unwrap()),
                    )
                };
                assert_eq!(
                    [link(9), link(10), link(11), link(12)],
                    [(0, 1), (1, 1), (3, 2), (3, 1)],
                    "theirs' who=1 city 0"
                );
                // The two caravans, by name: slot 3 of each, TI 59, land;
                // who=2's `2/131` linked before who=1's `1/34`.
                let at = |who: usize, o: i16| {
                    let v = &built.sim.economy.caravans[who].slots[3];
                    let u = &built.sim.units[v.unit.expect("slot 3's caravan")];
                    assert_eq!((u.owner as usize, u.index, u.type_index), (who, o, 59));
                    assert_eq!(u.kind.domain, sim::territory::attrition::Domain::Land);
                    v.linked_at
                };
                assert!(at(2, 131) < at(1, 34), "who=2's link made first");
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

    /// **Row 63's army skips an ally its leader does not call strong**
    /// (item 4553, `docs/ARMY.md` §12): on run1782, frame 1788's draws site
    /// by site against the trace, and every units and guys range on 1788
    /// and 1789 byte for byte. `Army::find_target@006f69b0` scores a mutual
    /// ally's cities only when `weak[L] == 0` and `strong[L] != 0`; who=1's
    /// `strong[2]` is 0 there, so theirs spends three `find_target` draws
    /// (who=0's city, who=1's two) where the stand-in drew six, and the
    /// idle rolls after them gave who=1's Fusiliers `1/35` and `1/36` (TI
    /// 136) anims 0 and 3 on 1789 against theirs' `IDLE1`.
    #[test]
    fn row_63_s_army_skips_an_ally_it_does_not_call_strong_on_run1782() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 3) else {
            eprintln!("skipping: row 63's recording is missing");
            return;
        };
        let dir = home.join("ron-golden/run1782/map-18");
        let (Some(ranges), Some(log)) = (
            crate::testenv::experiment(&dir, "rontrace.ranges"),
            crate::testenv::experiment(&dir, "rontrace.log"),
        ) else {
            eprintln!("skipping: run1782's ranges or trace is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 63's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        let trace = crate::trace::Trace::read(&log)
            .expect("the trace reads")
            .expect("the trace parses");
        while built.sim.frame < 1788 {
            stream.apply(built.sim.frame as i32, &mut built);
            built.sim.command().tick();
        }
        // Frame 1788's draws, named, both sides.
        let word = built.sim.rng.seed;
        assert_eq!(
            trace.frame_draws(1788).first().map(|d| d.seed),
            Some(word),
            "the word entering 1788"
        );
        assert!(built.sim.is_ally(1, 2), "who=2 is who=1's ally");
        assert_eq!(built.sim.ai.ai[1].strong[2], 0, "who=1's strong[2]");
        stream.apply(1788, &mut built);
        built.sim.trace.trace_phases = true;
        built.sim.command().tick();
        built.sim.trace.trace_phases = false;
        let ours =
            crate::diff::report::mark_sites(&built.sim.trace.phase_marks, built.sim.rng.seed)
                .expect("ours' sites");
        let theirs = trace.labels(1788);
        let scores = |v: &[String]| {
            v.iter()
                .filter(|s| s.as_str() == sim::ai::army::SITE_FIND_TARGET_SCORE)
                .count()
        };
        assert_eq!(
            (scores(&ours), scores(&theirs)),
            (3, 3),
            "find_target's scores"
        );
        assert_eq!(ours, theirs, "1788's draws, site by site");
        for state in [1789, 1790] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            for (line, ours) in [
                (0, built.sim.checksum_units()),
                (6, built.sim.checksum_guys()),
            ] {
                let theirs: Vec<_> = all.iter().filter(|r| r.line == line).collect();
                assert_eq!(
                    ours.ranges.len(),
                    theirs.len(),
                    "{}' ranges on {state}",
                    ours.name
                );
                for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                    assert_eq!(
                        a.bytes, b.bytes,
                        "{} range {k} on {state}, record {} {}",
                        ours.name, a.record, a.field
                    );
                }
            }
        }
        // The Fusiliers' figure 0 plays IDLE1 from 1789 (`cur_anim` 1,
        // `end_time` 30).
        for o in [35, 36] {
            let u = built
                .sim
                .units
                .iter()
                .position(|u| u.alive() && u.owner == 1 && u.index == o)
                .expect("1/35, 1/36");
            assert_eq!(built.sim.units[u].type_index, 136, "1/{o} is a Fusilier");
            let g = built.sim.units[u].guys[0];
            assert_eq!((g.anim, g.end_time), (1, 30), "1/{o}'s figure 0 on 1790");
        }
    }

    /// **Row 63's Fusilier round leaves from the original's socket** (item
    /// 4558, `docs/RECGAME.md` §12.3): on run1782's ranges, every ammo range
    /// on 3682..=3692 byte for byte. who=1's Fusiliers `1/45` (TI 136, piece
    /// 1494) loose a `CHAR_ATTACK1` round at `0/1` on 3683; with no
    /// [`sim::data::launch`] row for the piece it left from its own square
    /// (8568, 12312, `sz` 468), where theirs leaves from (8633, 12404, 578).
    #[test]
    fn row_63_s_fusilier_round_leaves_from_the_original_s_socket_on_run1782() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 3) else {
            eprintln!("skipping: row 63's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1782/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1782's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 63's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 3682..=3692 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let theirs: Vec<_> = record
                .ranges(&set)
                .expect("the set's ranges")
                .into_iter()
                .filter(|r| r.line == 3)
                .collect();
            let ours = built.sim.checksum_ammo().expect("ours' ammo line");
            assert_eq!(ours.ranges.len(), theirs.len(), "ammo's ranges on {state}");
            for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                assert_eq!(
                    a.bytes, b.bytes,
                    "ammo range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            if state == 3683 {
                let u = built
                    .sim
                    .units
                    .iter()
                    .position(|u| u.alive() && u.owner == 1 && u.index == 45)
                    .expect("1/45");
                assert_eq!(built.sim.units[u].type_index, 136, "1/45 is a Fusilier");
                assert_eq!(built.sim.units[u].guys[0].gpiece, 1494, "1/45's piece");
                let p = built
                    .sim
                    .combat
                    .projectiles
                    .iter()
                    .find(|p| p.slot == 1)
                    .expect("the round in slot 1");
                assert_eq!((p.launch.x, p.launch.y, p.sz), (8633, 12404, 578));
            }
        }
    }

    /// **Row 63's Fort pick walks the fort records** (item 4563,
    /// `docs/AI.md` §29.5, `docs/RECGAME.md` §12.3): on run1782's ranges,
    /// every units, builds and groups range on 3984..=3987 byte for byte.
    /// On 3985 who=1's `produce_unit` sends one TI 58 (unit type 8) down the
    /// fort arm; its two Forts, 2032 and 2035, tie at three queued, and the
    /// original's walk over `forts[1]` reaches 2035's record first. Ours
    /// walked object order and queued at 2032: builds range 1659, record
    /// 2032, `queued` ours 4, theirs 3, on 3986.
    #[test]
    fn row_63_s_fort_pick_walks_the_fort_records_on_run1782() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 3) else {
            eprintln!("skipping: row 63's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1782/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1782's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 63's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
        let rules = inst.rules().expect("rules");
        let tribes = inst.tribe_defs(&rules).expect("tribes");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in 3984i64..=3987 {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            let lines = crate::checksum::state::ranges(&built.sim, &look, &tribes);
            for li in [0usize, 1, 5] {
                let ours = lines[li].as_ref().expect("the line");
                let theirs: Vec<_> = all.iter().filter(|r| usize::from(r.line) == li).collect();
                assert_eq!(
                    ours.ranges.len(),
                    theirs.len(),
                    "line {li}'s ranges on {state}"
                );
                for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                    assert_eq!(
                        a.bytes, b.bytes,
                        "line {li} range {k} on {state}, record {} {}",
                        a.record, a.field
                    );
                }
            }
            // The two Forts by name, and their records: 2035's below 2032's.
            let fort = |o: i16| {
                let b = built.sim.building_by_o(1, o).expect("who=1's Fort");
                let bd = &built.sim.buildings[b];
                let ty = bd.ty.expect("a type");
                assert!(
                    sim::cities::build::is_fort(&built.sim.tables.build_types, ty),
                    "{o} is a fort"
                );
                (bd.fort_slot.expect("a fort record"), bd.queue.items.len())
            };
            let (s32, q32) = fort(2032);
            let (s35, q35) = fort(2035);
            assert!(
                s35 < s32,
                "2035's record ({s35}) stands below 2032's ({s32})"
            );
            let want = if state <= 3985 { (3, 3) } else { (3, 4) };
            assert_eq!((q32, q35), want, "2032 and 2035 queued on {state}");
            if state == 3986 {
                let builds = lines[1].as_ref().expect("builds");
                let theirs: Vec<_> = all.iter().filter(|r| r.line == 1).collect();
                for (k, o, n) in [(1659, 2032, 3), (1753, 2035, 4)] {
                    assert_eq!(builds.ranges[k].record, o);
                    assert_eq!(builds.ranges[k].field, "gather_down..infiltrate2");
                    assert_eq!(theirs[k].bytes[0x82 - 0x70], n, "theirs' {o} `queued`");
                }
            }
        }
    }

    /// **Row 63's Colosseum is priced by its ally's wonder site** (item
    /// 4542, `docs/COSTS.md`, the team term): on run1782's ranges, every leaders
    /// range on 1581 and 1582 byte for byte. Row 63's `TEAM_STYLE 1` locks
    /// the teams, so `TypeData::get_cost@00664090`'s wonder count starts
    /// from `get_team_wonders − held + get_team_unbuilt_wonders`
    /// (`00665886`..`0066589f`): who=2's one unbuilt wonder ramps who=1's
    /// Colosseum (TI 530) to 345 Timber, `check_income`'s Timber quotient
    /// goes 15 → 17, and the make list's entry reads 53,400 on 1582, where
    /// the unlocked count gave 300 Timber and 66,750.
    #[test]
    fn row_63_s_colosseum_is_priced_by_its_ally_s_wonder_site_on_run1782_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 3) else {
            eprintln!("skipping: row 63's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1782/map-18"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1782's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 63's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        assert_eq!(built.sim.ai.lobby.team_style, 1, "row 63 locks its teams");
        let colosseum = built
            .sim
            .tables
            .build_types
            .iter()
            .position(|b| b.tree == Some(530))
            .expect("the Colosseum's record");
        for state in [1581, 1582] {
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
            // Who=1's third make-list slot: range 171, `t` then `val`.
            assert_eq!(ours.ranges[171].field, "make_list[k]");
            assert_eq!(ours.ranges[171].record, 1);
            let word = |n: usize| {
                i32::from_le_bytes(theirs[171].bytes[4 * n..4 * n + 4].try_into().unwrap())
            };
            if state == 1582 {
                assert_eq!((word(0), word(1)), (530, 53_400), "theirs' Colosseum entry");
            } else {
                assert_ne!(word(0), 530, "no Colosseum entry before 1582");
            }
            // Who=2, a mutual ally, holds the one unbuilt wonder; who=1 none.
            assert!(built.sim.is_ally(1, 2));
            let unbuilt = |p: u8| {
                built
                    .sim
                    .buildings
                    .iter()
                    .filter(|b| {
                        b.alive
                            && !b.active
                            && b.owner == p
                            && b.ty.is_some_and(|t| built.sim.tables.build_types[t].wonder)
                    })
                    .count()
            };
            assert_eq!(
                (unbuilt(1), unbuilt(2)),
                (0, 1),
                "unbuilt wonders on {state}"
            );
        }
        // The team term is 1, so the count is 1 and the halved step adds 45
        // to each of the two goods' 300.
        let price = built.sim.building_price(1, colosseum);
        assert_eq!(price, [0, 345, 0, 0, 345, 0], "who=1's Colosseum on 1582");
    }

    /// **Row 3's computer blocks on its captured city's Library** (item
    /// 4548, `docs/AI.md` §3.1, `docs/RECGAME.md` §12.3): on run1796's
    /// ranges, every leaders and cities range on 4776 and 4777 byte for
    /// byte. Who=1 (`defensive`, script step 15) holds three cities on
    /// 4776, the third captured on 4756 and stamped 4756, the first never
    /// struck. `get_city_index@009e2ba0` finds `""` in the first live
    /// city's empty `id`, so the head's `was_city_attacked/raided(who, "",
    /// -1)` answer 0 (the walk over every city answered 1 and the script
    /// returned `SCRIPT_DONE`). Step 15's `research_tech_with_cost(Barter)`
    /// finds the Library 2016 in the captured city first, and
    /// `BuildData::can_make@0062db10` refuses an unassimilated building, so
    /// it returns 0 and the script blocks: theirs' `production_step` 0,
    /// `prod_script_run` 1 and `script_step` 15 on 4777.
    #[test]
    fn row_3_s_computer_blocks_on_its_captured_city_s_library_on_run1796_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 0) else {
            eprintln!("skipping: row 3's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1796/map-7"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1796's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 3's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [4776, 4777] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            for (line, ours) in [
                (7, built.sim.checksum_leaders()),
                (8, built.sim.checksum_cities()),
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
            }
            // Who=1's `who..ally_mask`, range 129: `production_step`
            // (`+0x788`), `prod_script_run` (`+0x78c`), `script_step`
            // (`+0x790`), the range starting at `+0x8`.
            let theirs: Vec<_> = all.iter().filter(|r| r.line == 7).collect();
            assert_eq!(theirs[129].bytes.len(), 0x6922, "who=1's whole range");
            let word = |off: usize| {
                i32::from_le_bytes(theirs[129].bytes[off - 8..off - 4].try_into().unwrap())
            };
            let want = if state == 4776 {
                (1, 1, 15)
            } else {
                (0, 1, 15)
            };
            assert_eq!(
                (word(0x788), word(0x78c), word(0x790)),
                want,
                "theirs' who=1 step machine on {state}"
            );
        }
        // The inputs both arms read, on the walked 4777.
        let sim = &built.sim;
        let cities: Vec<usize> = sim.cities_of(1);
        assert_eq!(cities.len(), 3, "who=1's three cities");
        assert_eq!(sim.cities.cities[cities[0]].attack_stamp, 0);
        assert_eq!(sim.cities.cities[cities[0]].raid_stamp, 0);
        let captured = cities[2];
        assert_eq!(sim.cities.cities[captured].attack_stamp, 4756);
        assert!(sim.city_unassimilated(captured), "the captured city");
        let library = sim.building_by_o(1, 2016).expect("who=1's 2016");
        assert_eq!(sim.buildings[library].city, Some(captured));
        assert!(sim.building_unassimilated(library), "2016 makes nothing");
        assert_eq!(sim.ai.ai[1].script.as_deref(), Some("defensive"));
    }

    /// **Row 3's foundation disband refreshes no neighbour** (item 4572,
    /// `docs/RECGAME.md` §12.3): on run1796's ranges, every world range on
    /// 4989 and 4990 byte for byte. On 4989 who=2's Barracks 2014 (TI 427),
    /// a foundation (`SubObject::flags` 1, never started), closes; who=1's
    /// 2016 (TI 435, active) stands in its band's cell (8, 40), offset (2,
    /// 0) at d = −3 from the Barracks' (9, 43). The close's reason is 0
    /// (`disband_steps`' `close(0, -1, 0)`, or the scripts'
    /// `destroy_building`), and `Wall::close@0063f6e0:156` refreshes no
    /// neighbour on it: theirs' 2016 band (tiles x 29..=35, y 158..=164,
    /// 27 tiles) carries no `BEHIND` bit on 4990, where ours set it on
    /// every one.
    #[test]
    fn row_3_s_foundation_disband_refreshes_no_neighbour_on_run1796_s_ranges() {
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let (Some(rcx), _) = paths(&FIFTH, 0) else {
            eprintln!("skipping: row 3's recording is missing");
            return;
        };
        let Some(ranges) =
            crate::testenv::experiment(&home.join("ron-golden/run1796/map-7"), "rontrace.ranges")
        else {
            eprintln!("skipping: run1796's ranges record is missing");
            return;
        };
        let rec = crate::recgame::read(&rcx.to_string_lossy()).expect("row 3's file");
        let loaded = crate::load::load(&inst).expect("the install loads");
        let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut record = crate::trace::ranges::Record::open(&ranges).expect("the record opens");
        for state in [4989, 4990] {
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
                assert_eq!(
                    a.bytes, b.bytes,
                    "world range {k} on {state}, record {} {}",
                    a.record, a.field
                );
            }
            // Range 3624, the tile masks, 240 tiles a row: 2016's band.
            assert_eq!(ours.ranges[3624].field, "mask");
            let mask = |x: usize, y: usize| {
                let i = 2 * (y * 240 + x);
                u16::from_le_bytes([theirs[3624].bytes[i], theirs[3624].bytes[i + 1]])
            };
            let mut band = 0;
            for k in 1..=3 {
                for x in 32 - k..32 - k + 5 {
                    assert_eq!(mask(x, 161 - k) & 4, 0, "({x}, {}) on {state}", 161 - k);
                    band += 1;
                }
                for y in 161 - k + 1..161 - k + 5 {
                    assert_eq!(mask(32 - k, y) & 4, 0, "({}, {y}) on {state}", 32 - k);
                    band += 1;
                }
            }
            assert_eq!(band, 27, "2016's band, three rings of a 5x5");
            let sim = &built.sim;
            let library = sim.building_by_o(1, 2016).expect("who=1's 2016");
            assert!(sim.buildings[library].active, "2016 is active on {state}");
            let barracks = sim.building_by_o(2, 2014);
            if state == 4989 {
                let b = barracks.expect("who=2's 2014 on 4989");
                assert!(!sim.buildings[b].started, "2014 is a foundation");
            } else {
                assert!(barracks.is_none(), "2014 closed on 4989");
            }
        }
    }

    /// **Row 95's MLRS bears from its pivot node** (item 4536,
    /// `docs/RECGAME.md` §12.5): on run1770's ranges, every units and guys
    /// range on 5908 and 5909 byte for byte. On 5908 who=1's MLRS `1/99`
    /// (TI 273, figure piece 223) takes its attack on who=0's 2000, and
    /// `Guy::set_all_pivots@005d8bc0` bears from the unit's point plus
    /// node 4's vector: theirs' `turret_angles[0]` and `des_turret_angles
    /// [0]` read 680 << 16 on 5909, where the launcher's own square gave
    /// 687 << 16.
    #[test]
    fn row_95_s_mlrs_bears_from_its_pivot_node_on_run1770_s_ranges() {
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
        for state in [5908, 5909] {
            while built.sim.frame < state {
                stream.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = record.first_of(state - 1).expect("the state's set").clone();
            let all = record.ranges(&set).expect("the set's ranges");
            for (line, ours) in [
                (0, built.sim.checksum_units()),
                (6, built.sim.checksum_guys()),
            ] {
                let theirs: Vec<_> = all.iter().filter(|r| r.line == line).collect();
                assert_eq!(
                    ours.ranges.len(),
                    theirs.len(),
                    "{}' ranges on {state}",
                    ours.name
                );
                for (k, (a, b)) in ours.ranges.iter().zip(&theirs).enumerate() {
                    assert_eq!(
                        a.bytes, b.bytes,
                        "{} range {k} on {state}, record {} {}",
                        ours.name, a.record, a.field
                    );
                }
                // Who=1's record 99, its figure: `type` at byte 0,
                // `turret_angles[0]` at byte 24 (`+0x20`), `des_turret_
                // angles[0]` at byte 40 (`+0x30`).
                let k = ours
                    .ranges
                    .iter()
                    .position(|r| r.record == 99 && r.field == "guy")
                    .expect("record 99's figure");
                assert_eq!(&theirs[k].bytes[..4], &273i32.to_le_bytes(), "TI");
                let turret = if state == 5909 { 680i32 << 16 } else { 0 };
                assert_eq!(&theirs[k].bytes[24..28], &turret.to_le_bytes(), "turret");
                assert_eq!(&theirs[k].bytes[40..44], &turret.to_le_bytes(), "des");
            }
            let mlrs = built
                .sim
                .units
                .iter()
                .find(|u| u.alive() && u.owner == 1 && u.index == 99)
                .expect("1/99");
            assert_eq!(mlrs.type_index, 273, "1/99 is the MLRS");
            assert_eq!(mlrs.guys[0].gpiece, 223, "its figure's piece");
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
        let col = |now: bool| -> (usize, String, u64, u64) {
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
            // The reach inside the first 6,001 states, the length of the
            // five batteries before the sixth (DECISIONS 152 §4).
            let (ran6, of6) = scores.clone().flatten().fold((0u64, 0u64), |(ran, of), s| {
                let first = s.first.iter().map(|f| f.1.0).min();
                let states = s.states.min(SHORT_STATES);
                let before = first.map_or(states, |f| f.max(0).cast_unsigned().min(states));
                (ran + u64::from(before), of + u64::from(states))
            });
            let firsts: Vec<String> = scores
                .map(|s| s.map_or("-".to_string(), |s| lowest(&s)))
                .collect();
            (
                n,
                firsts.join(" "),
                100 * ran / of.max(1),
                100 * ran6 / of6.max(1),
            )
        };
        let (found, found_firsts, found_reach, found_short) = col(false);
        let (now, now_firsts, now_reach, now_short) = col(true);
        let short = if b.frames > 6000 {
            format!(
                " \u{b7} reach in the first 6,001 states {found_short} % as found, {now_short} % now"
            )
        } else {
            String::new()
        };
        format!(
            "{}: {} of {} whole \u{b7} {} taken \u{b7} lines {found} of {} as found, \
             {now} now \u{b7} first partings {found_firsts} as found, {now_firsts} now \u{b7} \
             reach {found_reach} % of states as found, {now_reach} % now{short}",
            b.label,
            b.line.0,
            b.scores.len(),
            b.line.1,
            12 * b.line.1,
        )
    }

    /// **The handoff's closed line is the whole batteries** (item 4566;
    /// DECISIONS 151 §6, 152 §4): every battery whose eight rows are taken
    /// and whole now ([`Battery::closed`]) gives up its own line to one,
    /// `Parity held out, closed:`, [`closed_line`], so the sixth's own line
    /// leaves the handoff's bound where it stands. Seen red on a queue still
    /// carrying the four whole batteries' lines.
    #[test]
    fn the_handoff_s_closed_parity_held_out_line_is_the_whole_batteries() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/QUEUE.md");
        let q = std::fs::read_to_string(path).expect("docs/QUEUE.md");
        let want = closed_line(BATTERIES);
        let line = q.lines().find(|l| l.starts_with(CLOSED_HEAD));
        assert_eq!(
            line.map(str::trim_end),
            want.as_deref(),
            "docs/QUEUE.md's handoff carries the closed line (`diff::held_out::closed_line`)"
        );
        for b in BATTERIES.iter().filter(|b| b.closed()) {
            let head = format!("{}:", b.label);
            assert!(
                !q.lines().any(|l| l.starts_with(&head)),
                "`{head}` is closed: its row is the closed line's, not a line of its own"
            );
        }
    }

    /// **The handoff carries each open battery's line** (the pass of
    /// 2026-11-19, second, DECISIONS 145 §1; item 4566): every battery not
    /// closed has its [`queue_line`] in the handoff, in the first's grammar,
    /// its found half the estimate. The commander writes it at the booking.
    #[test]
    fn the_handoff_s_open_parity_held_out_lines_are_the_pinned_scores() {
        for b in BATTERIES.iter().filter(|b| !b.closed()) {
            handoff_line_is_the_pinned_scores(b);
        }
    }

    /// The states of a 6,000-frame game, the five batteries before the
    /// sixth's length: the sixth's line reads its reach inside them too.
    const SHORT_STATES: u32 = 6001;

    /// The closed line's head.
    const CLOSED_HEAD: &str = "Parity held out, closed:";

    /// **The closed line** (item 4566): the closed batteries by name, their
    /// rows whole of their rows, their lines as found of twelve a game and
    /// now, and their reach as found and now, as [`queue_line`] sums them.
    /// `None` when no battery is closed.
    fn closed_line(all: &[&Battery]) -> Option<String> {
        let closed: Vec<&&Battery> = all.iter().filter(|b| b.closed()).collect();
        if closed.is_empty() {
            return None;
        }
        let names: Vec<&str> = closed
            .iter()
            .map(|b| b.label.strip_prefix("Parity held out, ").unwrap_or("first"))
            .collect();
        let found = || {
            closed
                .iter()
                .flat_map(|b| b.scores.iter().flatten().map(|p| p.found))
        };
        let now = || {
            closed.iter().flat_map(|b| {
                b.scores
                    .iter()
                    .flatten()
                    .map(|p| p.moved.unwrap_or(p.found))
            })
        };
        let lines = |s: Score| s.agreed.iter().filter(|&&a| a == s.states).count();
        let reach = |it: &mut dyn Iterator<Item = Score>| {
            let (ran, of) = it.fold((0u64, 0u64), |(ran, of), s| {
                let first = s.first.iter().map(|f| f.1.0).min();
                let before = first.map_or(s.states, |f| f.max(0).cast_unsigned());
                (ran + u64::from(before), of + u64::from(s.states))
            });
            100 * ran / of.max(1)
        };
        let rows = 8 * closed.len();
        Some(format!(
            "{CLOSED_HEAD} {} \u{b7} {rows} of {rows} whole \u{b7} lines {} of {} as found, {} now \u{b7} \
             reach {} % of states as found, {} % now",
            names.join(", "),
            found().map(lines).sum::<usize>(),
            12 * rows,
            now().map(lines).sum::<usize>(),
            reach(&mut found()),
            reach(&mut now()),
        ))
    }

    /// **The closed line goes red when a closed battery's row parts**:
    /// the line counts every closed battery's lines, so one row of the
    /// first battery bent off whole leaves it out of the closed set.
    #[test]
    fn a_battery_with_a_row_not_whole_is_not_closed() {
        assert!(FIRST.closed(), "the first battery reads 8 of 8");
        let mut scores = SCORES;
        let p = SCORES[0].as_ref().unwrap();
        scores[0] = Some(FirstScore {
            last: p.last,
            found: p.found,
            moved: Some(p.found),
        });
        let bent = Battery {
            scores: Box::leak(Box::new(scores)),
            ..FIRST
        };
        assert!(!bent.closed(), "row 0 as found is not whole");
        assert!(!SIXTH.closed() || SIXTH.whole_now() == 8);
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
        // Moved by item 4540 (`Build::close@00628980`'s wonder arm, the
        // `0x4000000` beside `close_wonder`, is gated on `BuildData::wonder
        // >= 0`, so an unfinished wonder site closed by a rival's finish
        // raises the wall bit alone: run1742, state 5409, leaders range 128,
        // record 1 `flags` byte 3, who=1's site of who=2's wonder `2/2030`,
        // ours 0x0e, theirs 0x0a): leaders 6,000 -> 6,001, whole.
        // Moved by item 4541 (`ObjectData::is_jammed@00653660`: an enemy
        // special forces unit within `GENERAL_RADIUS × 0x180` of an
        // anti-air building jams it, `Build::do_attack@006228f0` marking
        // `build_masks |= 0x8000` and the jammer's `unit_masks2 |= 8`, and
        // `Unit::process@00610bc0`'s caster arm trading the 8 for
        // `unit_masks |= 0x80000000` on the unit's phase: run1742, state
        // 5577, units range 1673, record 0 `unit_masks2`, who=2's Elite
        // Special Forces `2/0` (TI 77), ours 0, theirs 8; builds range
        // 1554, record 2040 `build_masks`, who=1's Tower `1/2040` (type
        // 111), ours 4096, theirs -28672): units 5,577 -> 6,001 and builds
        // 5,577 -> 6,001, whole.
        // Moved by item 4550 (`WallData::is_seen@00642bd0`'s second clause
        // takes the viewer's `leader_flags & 0x800`, its Fouché and the
        // lobby's reveal beside `ever_seen`, so who=2's Space Program sees
        // every started building of who=1's in the danger map's rebuild:
        // run1742, state 5601, world range 3631, record 2 `danger` byte
        // 2328, half-cell (12, 19), the ring of who=1's `1/2030` (type 13),
        // ours 37, theirs 50): world 5,601 -> 6,001, whole.
        moved: Some(Score {
            states: 6001,
            agreed: [
                6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001,
            ],
            first: &[],
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
/// repaints its disc, who=1's Anti-Aircraft Gun `1/114` on 3790). Item
/// 4550: 7 -> 8, row 110 (run1735) whole with its world line (who=2's
/// Space Program sees every started building of who=1's in the danger
/// map's rebuild of 5600).
#[cfg(test)]
pub(crate) const HELD_OUT_SECOND_LINE: (usize, usize) = (8, 8);

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
    /// The recordings' closing frame (`item<N>-row<RRR>-<frames>.rcx`):
    /// 6,000 for the first five, [`SIXTH_FRAMES`] for the sixth.
    pub frames: i32,
}

/// A scored game in a pin's terms: states, each line's agreeing states,
/// the lines' first partings, ours' end.
#[cfg(test)]
pub(crate) type Got = (u32, [u32; 12], Vec<First>, Option<i32>);

/// **What a held-out pin holds** (item 4598, DECISIONS 154 §5): the
/// queue books a row's whole lines and its first parting, so those are
/// the assertion, and every other difference from the pin is drift the
/// test prints. Returns `(fails, drift)`, each a line a person reads.
///
/// - **Fails**: the states compared; the set of whole lines (a line
///   agreeing on every state and parting nowhere), either way; the first
///   parting — its state and the lines that part on it — either way.
/// - **Drift**: a parted line's agreeing count, a line's first parting
///   past the row's, the Adlers on the first parting, ours' end.
#[cfg(test)]
pub(crate) fn held_verdict(want: &Score, got: &Got) -> (Vec<String>, Vec<String>) {
    let names: Vec<&str> = crate::checksum::CHECKSUM_LINES[..12]
        .iter()
        .map(|l| l.0)
        .collect();
    let whole = |states: u32, agreed: &[u32; 12], first: &[First]| -> Vec<&str> {
        names
            .iter()
            .zip(agreed)
            .filter(|(n, a)| **a == states && !first.iter().any(|f| f.0 == **n))
            .map(|(n, _)| *n)
            .collect()
    };
    let parting = |first: &[First]| -> Option<(i32, Vec<&str>)> {
        let at = first.iter().map(|f| f.1.0).min()?;
        let mut lines: Vec<&str> = first.iter().filter(|f| f.1.0 == at).map(|f| f.0).collect();
        lines.sort_unstable();
        Some((at, lines))
    };
    let (states, agreed, first, over) = got;
    let mut fails = Vec::new();
    let mut drift = Vec::new();
    if *states != want.states {
        fails.push(format!("states {} -> {states}", want.states));
    }
    let (was, now) = (
        whole(want.states, &want.agreed, want.first),
        whole(*states, agreed, first),
    );
    if was != now {
        fails.push(format!("whole lines {was:?} -> {now:?}"));
    }
    let (pw, pg) = (parting(want.first), parting(first));
    if pw != pg {
        fails.push(format!("first parting {pw:?} -> {pg:?}"));
    }
    for (i, n) in names.iter().enumerate() {
        if want.agreed[i] != agreed[i] && !was.contains(n) && !now.contains(n) {
            drift.push(format!("{n} agreeing {} -> {}", want.agreed[i], agreed[i]));
        }
        let (fw, fg) = (
            want.first.iter().find(|f| f.0 == *n).map(|f| f.1),
            first.iter().find(|f| f.0 == *n).map(|f| f.1),
        );
        if fw != fg && was.contains(n) == now.contains(n) {
            drift.push(format!("{n} first {fw:?} -> {fg:?}"));
        }
    }
    if want.over != *over {
        drift.push(format!("ours' end {:?} -> {over:?}", want.over));
    }
    (fails, drift)
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
    /// **Closed**: all eight rows taken and whole now, so the battery's
    /// row in the handoff is the closed line's (item 4566).
    pub(crate) fn closed(&self) -> bool {
        self.scores.iter().all(Option::is_some) && self.whole_now() == self.scores.len()
    }

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
    frames: 6000,
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
    frames: 6000,
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
    frames: 6000,
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
        // leaders 5,266 -> 6,001, whole; ten lines whole. Moved by item
        // 4536 (the MLRS bears from piece 223's pivot node; run1770's
        // units range 2689 and guys range 720, record 99, on 5909:
        // `turret_angles[0]` ours 687 << 16, theirs 680 << 16): units and
        // guys 5,909 -> 6,001, whole; twelve lines whole.
        moved: Some(Score {
            states: 6001,
            agreed: [
                6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001,
            ],
            first: &[],
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
/// (6 -> 7, item 4531: row 55 whole; 7 -> 8, item 4536: row 95 whole).
#[cfg(test)]
pub(crate) const HELD_OUT_FOURTH_LINE: (usize, usize) = (8, 8);

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
    frames: 6000,
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
        // Moved by item 4565 (`compare_target`'s building arms read the
        // target's `damage` and `UnitData::is_moving`, `64eb6e`..`64ebc6`;
        // the leaf read on row 43's run1787 and the sample's run1786, sums
        // only here): units 3,872 -> 4,784 (first 3857 -> 4784), builds
        // 4,065 -> 4,784, ammo 4,957 -> 5,456 (first 3857 -> 4801), leaders
        // 3,879 -> 4,777, deaths 4,376 -> 4,867, groups 4,378 -> 4,814,
        // guys and world 4,376 -> 4,784, cities 4,725 -> 4,784, goods 5,800
        // -> 5,709 (first 5384 -> 5415), items 5,930 -> 5,679 (first 5105
        // -> 5110): the row's first parting 3857 -> 4777 (leaders).
        // Moved by item 4548 (run1796's ranges: `get_city_index` finds the
        // empty `id` every random-map city holds, so the scripts'
        // `was_city_attacked/raided(who, "", -1)` read the first live city
        // alone; and `research_tech_with_cost` asks `can_make` of the one
        // producer it finds, an unassimilated one refusing): on 4777
        // leaders range 129, record 1 (who=1), `production_step` ours 2 ->
        // 0, theirs 0, `prod_script_run` ours 0 -> 1, theirs 1. Eleven
        // lines whole, 6,001 of 6,001 (units, builds, guys, cities and
        // world 4,784, leaders 4,777, ammo 5,456, deaths 4,867, groups
        // 4,814, items 5,679, goods 5,709 before); world 4,784 -> 4,990
        // (first 4784 -> 4990): the row's first parting 4777 -> 4990.
        // Moved by item 4572 (run1796's ranges: who=2's Barracks
        // foundation 2014 closes on 4989 with reason 0, the disband's
        // `close(0, -1, 0)`, and `Wall::close@0063f6e0:156` refreshes no
        // neighbour on it): on 4990 world range 3624, `mask`, 27 bytes of
        // who=1's 2016 band (x 29..=35, y 158..=164), ours 0x07/0x17/
        // 0x04/0x14 -> 0x03/0x13/0x00/0x10, theirs the same, bit `0x04`
        // off. World 4,990 -> 6,001: twelve lines whole, 6,001 of 6,001,
        // the row whole.
        moved: Some(Score {
            states: 6001,
            agreed: [
                6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001,
            ],
            first: &[],
            over: None,
        }),
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
        // Moved by item 4538 (`Unit::update_hits`' Copper arm; run1781's leaf,
        // units range 957, record 15, who=2's Fishermen `2/15` (TI 317) on
        // 5695, `myhits` ours 170, theirs 204): units 5,695 -> 6,001, whole;
        // twelve lines whole.
        moved: Some(Score {
            states: 6001,
            agreed: [
                6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001,
            ],
            first: &[],
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
        // Moved by item 4547 (piece 736's three bays, piece 1440's centroids
        // on run1579's sweep; run1787's leaf, ammo range 1, record 0, who=2's
        // Slinger `2/30` (TI 82) stone on 3192, `launch` ours (8472, 23832,
        // 516), theirs (8571, 23834, 690)): ammo 3,355 -> 3,401 (first 3192 ->
        // 3299), units 3,290 -> 3,299 (first 3196 -> 3299), leaders 3,218 ->
        // 3,223 (first 3196 -> 3223), groups 3,395 -> 3,423 (first 3198 ->
        // 3376), deaths 3,247 -> 3,374 (first 3247 -> 3374). Items 4,262 ->
        // 4,042 and goods 4,002 -> 3,710 are named falls past their unmoved
        // first partings, 3670 and 3710. Our walk no longer holds
        // `game_over` from 5977: `over` 5977 -> none, as the original.
        // Moved by item 4561 (`Build::close`'s `is a city` arm fixes the
        // borders for an unbuilt site, `0x628bd3`; run1787's leaf on 3223,
        // leaders range 1, record 0, who=0's `reg_terr[1]` ours 368, theirs
        // 0, and cities range 12, record 4, `bordering` ours 6, theirs 0:
        // who=2's Village site at (27, 25) closes on frame 3221 and theirs'
        // pass starts again): leaders 3,223 -> 3,372 (first 3223 -> 3302),
        // cities 3,373 -> 3,379 (first 3223 -> 3376).
        // Moved by item 4559 (the eight pieces with no bay; row 43's is the
        // Native American Bowmen, piece 1528, whose 788 rounds left from the
        // figure's square from 3360, past this row's first parting at 3299,
        // which stands): builds 3,345 -> 3,452, deaths 3,374 -> 3,459 (first
        // 3374 -> 3459), groups 3,423 -> 3,484 (first 3376 -> 3463), guys
        // 3,374 -> 3,442 (first 3374 -> 3442), leaders 3,372 -> 3,460,
        // cities 3,379 -> 3,561 (first 3376 -> 3561), goods 3,710 -> 4,325
        // (first 3710 -> 4102), world 3,374 -> 3,444 (first 3374 -> 3443).
        // Items 4,042 -> 4,034 (first 3670 -> 3658) is a named fall, and our
        // walk holds `game_over` from 5972 again where the original plays to
        // 6000 (`over` none -> 5972), both past the first parting.
        // Moved by item 4565 (`compare_target`'s building arms read the
        // target's `damage` and `UnitData::is_moving`, `64eb6e`..`64ebc6`;
        // run1787's leaf on 3299, builds range 1549, record 2010, who=2's
        // Village `attack_ox` ours 49, theirs 46, and units range 1866,
        // record 46, `targeted` ours 0, theirs 1: the Village keeps the
        // wounded Spearmen `1/46` over `1/49`, whose head order is a move
        // with no destination held): units 3,299 -> 3,374 (first 3299 ->
        // 3374), builds 3,345 -> 3,376 (first 3299 -> 3376), ammo first
        // 3299 -> 3361 (3,401 -> 3,373, a fall past the new first),
        // leaders 3,372 -> 3,374 (first 3302 -> 3374), goods 3,710 ->
        // 4,099; items 4,042 -> 4,036 (first 3670 -> 3660) and goods' first
        // 3710 -> 3665 are named falls past the row's first parting, 3361.
        // On the tree merged with 4559 (the two items' numbers above are
        // each against the tree before the other), measured: units 3,442
        // (first 3442), builds 3,483, ammo 3,492 (first 3444), leaders
        // 3,489 (first 3459), groups 3,485, cities 3,583, items 4,042
        // (first 3670), goods 4,100 (first 4054), world 3,444 (first 3443);
        // the row's first parting 3442; our walk holds `game_over` from
        // 5784 (`over` 5972 -> 5784), past it.
        // Moved by item 4564 (`fight`'s defensive leash asks for its entry
        // target, `-0x1c`/`-0x20(%ebp)` at `5fe292`..`5fe2b5`; run1787's
        // leaf on 3442, units range 5905, record 63, who=2's Hoplites
        // `2/63` `orders.count` ours 1, theirs 2: the follower keeps the
        // re-added attack on its captain's `1/23`): units 3,442 -> 3,833
        // (first 3442 -> 3833), builds 3,483 -> 3,860 (first -> 3836), ammo
        // 3,492 -> 3,874 (first 3444 -> 3833), deaths 3,459 -> 3,834,
        // groups 3,485 -> 3,833, guys 3,442 -> 3,833, leaders 3,489 ->
        // 3,841 (first 3459 -> 3834), cities 3,583 -> 4,149 (first -> 4040),
        // items 4,042 -> 4,060 (first 3670 -> 4060), goods 4,100 -> 4,215
        // (first 4054 -> 4111), world 3,444 -> 3,833: the row's first
        // parting 3442 -> 3833; our walk plays to 6000 (`over` 5784 ->
        // none).
        // Item 4642: muster uses the building's current alert. The original
        // group destination now agrees on 3833; builds first part on 3899.
        // Then item 4655 (`get_damage`'s recapture step reads the target
        // city's race, whoever owns the building): builds 4,201 -> 4,353
        // (first 3899 -> 4350); the row's first parting is groups alone on
        // 3995.
        moved: Some(Score {
            states: 6001,
            agreed: [
                4341, 4353, 6001, 4356, 4416, 4239, 4341, 4341, 4351, 5173, 4413, 4341,
            ],
            first: &[
                ("units", (4341, Some(630316088), 571078055)),
                ("builds", (4350, Some(4014773854), 1772852329)),
                ("ammo", (4356, Some(1671400503), 4115106818)),
                ("deaths", (4416, Some(1680720123), 2417413765)),
                ("groups", (3995, Some(866943998), 1455327226)),
                ("guys", (4341, Some(3154895219), 3388006762)),
                ("leaders", (4341, Some(3434555180), 2510759726)),
                ("cities", (4341, Some(2806731487), 1671058207)),
                ("items", (4887, Some(2284009543), 2246522949)),
                ("goods", (4413, Some(3243865884), 3324606240)),
                ("world", (4341, Some(234423208), 3308913912)),
            ],
            over: None,
        }),
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
        // Moved by item 4542 (`get_cost`'s locked-team wonder term; run1782's
        // leaf, leaders range 171, record 1, who=1's Colosseum (TI 530)
        // make-list entry on 1582, `val` ours 66,750, theirs 53,400): leaders
        // 1,582 -> 1,828; cities 1,958 -> 2,342, items 2,179 -> 3,435, goods
        // 1,965 -> 2,046; ammo 3,780 -> 3,758, a named fall past its first
        // parting (3321, unmoved).
        // Moved by item 4553 (`Army::find_target`'s ally arm reads the
        // leader's `strong[8]`/`weak[8]`: run1782's 1788, who=1's `strong[2]`
        // is 0, so its ally's three cities are not scored, theirs' three
        // `find_target` draws where ours took six; the idle rolls after
        // them land on theirs' values, 1/35 and 1/36's IDLE1 on 1789):
        // units 1,789 -> 4,219, first 3691; builds 1,810 -> 3,978, first
        // 3789; ammo 3,758 -> 3,992, first 3321 -> 3683; deaths 3,481 ->
        // 3,718; groups, guys, items, goods and world whole (1,903, 1,789,
        // 3,435, 2,046, 1,806 -> 4,223); leaders 1,828 -> 4,187, first 3691;
        // cities 2,342 -> 4,194, first 3691.
        // Moved by item 4558 (piece 1494's release bays, the Fusiliers':
        // run1782's 3683, ammo range 4, record 1, who=1's `1/45` (TI 136)
        // left from its own square (8568, 12312, `sz` 468) where theirs
        // leaves from the `CHAR_ATTACK1` socket (8633, 12404, 578)): units
        // 4,219, ammo 3,992, deaths 3,718, leaders 4,187 and cities 4,194
        // -> 4,223, whole; builds 3,978 -> 3,986, first 3789 -> 3986.
        // Eleven lines whole.
        // Moved by item 4563 (`produce_unit`'s fort arm walks the leader's
        // fort records: run1782's 3986, builds range 1659, record 2032,
        // `queued` ours 4, theirs 3, and range 1753, record 2035, ours 3,
        // theirs 4: who=1's TI 58 on 3985 went to the Fort 2032, first in
        // object order, where theirs' record walk reaches 2035 first, the
        // two tied at three queued): builds 3,986 -> 4,223. Row 63 whole,
        // twelve lines.
        moved: Some(Score {
            states: 4223,
            agreed: [
                4223, 4223, 4223, 4223, 4223, 4223, 4223, 4223, 4223, 4223, 4223, 4223,
            ],
            first: &[],
            over: None,
        }),
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
        // and items 3,563 -> 3,491, named falls past the first partings.
        // Item 4545: leaders 1,956 -> 3,376, first 1956 -> 2811 (units).
        // run1784, the ranges take: on 1956 leaders range 310, record 2
        // (who=2), `make_list[k]` `t` ours 561, theirs 59, a Caravan at
        // `val` 980,392: `get_caravan_limit(1)` counts the mutual ally
        // who=0's city beside who=2's two (`006dca50`, `docs/CARAVAN.md`
        // §4). Builds 1,958 -> 3,356, units and guys 2,338 -> 2,811 and
        // 3,360, world 2,338 -> 3,360, the rest later with it.
        // Item 4554: units 2,811 -> 3,359, first 2811 -> 3359. run1784, the
        // ranges take: on 2811 units range 2319, record 70, who=1's
        // Artillery (TI 269) `unit_masks2` ours 0, theirs 0x40000, the
        // supply veto's mark (`Unit::process@00610bc0:535`,
        // `docs/SUPPLY.md`). The row's first parting is builds 3041.
        // Item 4556: builds 3,356 -> 3,360, first 3041 -> 3360. run1784, the
        // ranges take: on 3041 builds range 3157, record 2037, who=2's Taj
        // Mahal site (`TypeIndex` 0x21b) `construct_hits` ours 6017, theirs
        // 6023: `Wall::inc_time@0063fb60:48`..`56` refreshes a site in the
        // leader walk, ahead of the ammo list (`docs/CITIES.md` §3.4). The
        // row's first parting is units 3359.
        // Item 4560 (`Build::process@0061edf0`'s all-clear, a computer's
        // city emptying on its 60-frame phase with no stranger in sight):
        // ammo 3,659 -> 3,651, a named fall past the first partings; every
        // first parting unchanged.
        // Item 4559 (the eight pieces with no bay; row 83's are the
        // Anti-Aircraft Gun, 231, from 3997 and the Machine Gun, 73, from
        // 4252, both past the row's first parting, units 3359): goods 4,678
        // -> 4,572 (first 4618 -> 4572), a named fall; every other column
        // unmoved.
        // Item 4562: units 3,359 -> 3,645, first 3359 -> 3645; builds 3,360
        // -> 3,645; ammo 3,651 -> 3,684; groups 3,385 -> 3,662; guys 3,360
        // -> 3,645; leaders 3,376 -> 3,647; cities 3,576 -> 3,645; items
        // 4,069 -> 5,165; world 3,360 -> 3,650; goods 4,572 -> 4,272, a
        // named fall past the first partings. run1784, the ranges take: on
        // 3359 units range 7702, record 117, who=2's Biplane (`TypeIndex`
        // 0x11f) `orders.count` ours 1, theirs 0: the computer's sortie at
        // the Airbase `2/2018` on 3358 scores who=1's wonder `1/2026` (217)
        // over the city, out of the plane's reach, and adds no patrol
        // (`Object::do_launch@0064f3b0`, `64fa71`..`64fda4`). The row's
        // first parting is units 3645.
        // Item 4568: units 3,645 -> 3,662, first 3645 -> 3662; builds 3,645
        // -> 3,695; ammo 3,684 -> 3,710; guys 3,645 -> 3,663; leaders 3,647
        // -> 3,674; cities 3,645 -> 3,656; world 3,650 -> 3,668; items 5,165
        // -> 4,129 and goods 4,272 -> 4,155, named falls past the first
        // partings. run1784, the ranges take: on 3645 builds record 2029
        // (who=2) `city_down` ours -1, theirs 2036, and 2036's `city` ours 2,
        // theirs 1: who=2's city `2/2020` falls on 3644 and its Barracks
        // `2/2036` stays its owner's; `City::close@00737550` sends it to
        // `Build::find_city`, where ours had cleared the chain first. The
        // row's first parting is cities 3656.
        // Item 4570: cities 3,656 -> 3,693, first 3656 -> 3693; ammo 3,710
        // -> 3,711; goods 4,155 -> 4,148, a named fall past the first
        // partings (on the merged tree; world 3,668 unmoved). run1784, the ranges take: on 3656
        // cities record 8, who=1's city `1/2043` (taken from who=2 on 3644),
        // `assimilation_timer` ours 3644, theirs 3641, then 3638:
        // `Build::process@0061edf0:698` asks `has_general(this, 0, 0x164)`
        // and who=1's Citizen `1/15` (TI 356) walks into reach; ours read a
        // flag nothing wrote. The row's first parting is units, builds and
        // groups 3662.
        // Then item 4585 (a tower is halved only while its city has never
        // been attacked): goods 4,148 -> 4,087 agreeing, its first parting
        // 4087 standing; the row's first parting stands.
        // Then item 4608 (`Guy::set_all_pivots` bears an `ATTACK_GROUND`
        // order's figures on the order's point, aimed or not): items 4,129 -> 4,069 (first 4129 ->
        // 4053) and goods 4,087 -> 4,117 (first 4087 -> 4117), past the row's first parting 3662,
        // which stands.
        // Then item 4655 (`get_damage`'s recapture step reads the target
        // city's race, whoever owns the building): units 3,662 -> 3,706,
        // builds 3,695 -> 3,730 (first 3662 -> 3718), ammo 3,711 -> 3,747
        // (first 3665 -> 3706), deaths 3,672 -> 3,715, groups 3,662 ->
        // 3,746 (first -> 3733), guys 3,663 -> 3,706, leaders 3,674 ->
        // 3,746 (first -> 3714), cities 3,693 -> 3,751, items 4,069 ->
        // 4,893, goods 4,117 -> 4,166 (first -> 4155), world 3,668 ->
        // 3,706; the row's first parting is units, ammo, guys and world on
        // 3706.
        moved: Some(Score {
            states: 6001,
            agreed: [
                3706, 3730, 6001, 3747, 3715, 3746, 3706, 3746, 3751, 4893, 4166, 3706,
            ],
            first: &[
                ("units", (3706, Some(1442083163), 2455439998)),
                ("builds", (3718, Some(3072307388), 431271696)),
                ("ammo", (3706, Some(1338324735), 459028267)),
                ("deaths", (3715, Some(2424789091), 2467256425)),
                ("groups", (3733, Some(716527825), 2283821262)),
                ("guys", (3706, Some(4002952463), 3588110382)),
                ("leaders", (3714, Some(2035969356), 3149425998)),
                ("cities", (3751, Some(3595451697), 3592174896)),
                ("items", (4893, Some(947073420), 912470408)),
                ("goods", (4155, Some(3194787109), 3189019939)),
                ("world", (3706, Some(2648689735), 889900131)),
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
        // Item 4544: world 1,791 -> 1,792, first 1750 -> 1792. run1783, the
        // ranges take: on 1750 world range 1148, record 1124 (cell (44, 18)),
        // `flags` ROAD ours 0x80, theirs 0: a caravan's road whose goal was
        // popped past 0xc80 on 1749, laid on 1750 in theirs
        // (`astar_caravan_road`'s `6860af`, `docs/ROADS.md` §8.3).
        // Moved by item 4542 (`get_cost`'s locked-team wonder term, built on
        // row 63's leaf; row 103's two computers are allied too, no leaf read
        // here): leaders 1,757 -> 1,911; ammo 2,882 -> 2,900, deaths 2,508 ->
        // 2,510, items 2,728 -> 3,045; world first at 1792 as 4544 left it.
        // Item 4545 (the caravan limit's ally term), on the tree merged
        // with 4542: ammo 2,900 -> 2,899, a named fall past its first
        // parting at 2860, which stands.
        // Moved by item 4553 (built on row 63's leaf, `find_target`'s ally
        // arm reading `strong[8]`/`weak[8]`; row 103's two computers are
        // allied too, no leaf read here): units 1,790 -> 2,951, first 2910;
        // builds 1,854 -> 3,049, first 1824 -> 3049; ammo 2,900 -> 2,942;
        // deaths 2,510 -> 3,112; groups 1,963 -> 3,062, first 2557; guys
        // 1,790 -> 3,049; leaders 1,911 -> 3,093, first 2910; cities 1,951
        // -> 3,013, first 2910; items and goods whole (3,045, 1,987 ->
        // 3,295); world 1,792 -> 2,983. On the tree merged with 4545 (the
        // caravan limit's ally term): ammo 2,943, groups 3,058, cities
        // 3,017, every first parting as above.
        // Moved by item 4551 (`Group::kill` leaves the army record's `role`;
        // run1783's leaf, groups range 215, record 131 (who=2's army 1,
        // pool slot 3) on 2557, `role` ours 0x050500, theirs 0x150d00: the
        // Infantry `2/69` pushed out by `disembark_squad` on 2556 leaves its
        // `FOOT | INFANTRY` bits): groups 3,058 -> 3,059, first 2557 -> 3057.
        // Moved by item 4559 (the eight pieces with no bay; row 103's are the
        // Asian Armored Scout Car, piece 1224, `SAME_AS` 169 in
        // `sim::data::pivot`, and the Tank, 198. run1783's leaf on 2860, ammo
        // range 1, who=1's `1/223` (TI 218) `CHAR_ATTACK2` round left from
        // its own square, `launch.x` ours 0x..22f8, theirs 0x..2322): units
        // 2,951, builds 3,049, ammo 2,943, deaths 3,112, groups 3,059, guys
        // 3,049, leaders 3,093 and cities 3,017 -> 3,295, all whole to the
        // original's end; world 2,983 (first 2983) unmoved.
        // Moved by item 4567 (`Objects::kill_guy` clears by the figure's
        // cell's plain region, `docs/COLLISION.md` §2). run1783's leaf on
        // 2983, world range 6479, record 2358 (cell (18, 39)), `block.body`
        // bytes 21, 23, 25: ours 0xc0, 0x80, 0x80, theirs 0x40, 0, 0. Range
        // 6553 (record 2418) bytes 20, 22, 24: ours 1, theirs 0. who=1's
        // President `1/14` (TI 355 -> 357, aboard Freighter `1/120`) kills
        // crew figure 2 on the ocean half of coastal cell (18, 40): world
        // 2,983 -> 3,295, first 2983 -> none. **Row 103 whole, 12 of 12.**
        moved: Some(Score {
            states: 3295,
            agreed: [
                3295, 3295, 3295, 3295, 3295, 3295, 3295, 3295, 3295, 3295, 3295, 3295,
            ],
            first: &[],
            over: None,
        }),
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
        // Moved by item 4546 (`find_build`'s lap resets a cursor past the
        // leader's count to 2000 and starts at 2001; run1785's leaf, builds
        // range 261, record 2000, `queued` on 2977, ours 3, theirs 2: who=2
        // left `find_counters[0x1c]` at 2011, who=1's count (its mark),
        // and who=1's three `train_unit_with_cost` calls on 2976 went to
        // 2000, 2007, 2000 in ours, 2007, 2000, 2007 in theirs): builds and
        // groups 2,977 -> 6,001, units 3,020 -> 6,001, and every later line
        // with them; twelve lines whole.
        moved: Some(Score {
            states: 6001,
            agreed: [
                6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001, 6001,
            ],
            first: &[],
            over: None,
        }),
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
/// Item 4538 made row 23 whole: 1 -> 2. Item 4546 made row 123 whole: 2 -> 3.
/// Item 4563 made row 63 whole: 3 -> 4. Item 4567 made row 103 whole: 4 -> 5.
/// Item 4572 made row 3 whole: 5 -> 6.
#[cfg(test)]
pub(crate) const HELD_OUT_FIFTH_LINE: (usize, usize) = (6, 8);

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
    frames: 6000,
};

/// **The spent batteries, folded into the headline** (DECISIONS 150 §1).
/// A battery is spent once a lane has read a parting off one of its rows
/// (entry 145 §2); its `now` column then counts in `Parity:` row by row,
/// by [`whole_now`], beside the six and the sample, and its found column
/// never folds. The first, second and third were spent on `43ee59434`;
/// the fourth since item 4517 read row 75's ammo (`b52f075dd`); the fifth
/// since item 4534 read row 23's leaders off run1781 (`3875a22b9`), added
/// by the pass of 2026-11-22 seven landings late (DECISIONS 151 §2); the
/// sixth since item 4575 read row 133's leaders off run1797; the seventh
/// since item 4613 read row 28's units off run1813. A
/// battery declared and not yet spent stays off this list, and
/// `a_battery_with_a_row_s_leaf_test_is_spent` says when one joins.
#[cfg(test)]
pub(crate) const SPENT: &[&Battery] = &[&FIRST, &SECOND, &THIRD, &FOURTH, &FIFTH, &SIXTH, &SEVENTH];

/// **The sixth eight, as declared** (item 4566, before any file): rows 13,
/// 33, …, 153 of `lobbies_on_disk`, each the human (tribe 4, slot 0)
/// against the row's computer, with [`HELD_OUT_SIXTH_SEATS`]' seat and team
/// added on the page; `MAP_SIZE 2` every row. Each is played to
/// [`SIXTH_FRAMES`] with the human scripted ([`HELD_OUT_SIXTH_HUMANS`]),
/// as `run1788 + k` for `HELD_OUT_SIXTH[k]`.
#[cfg(test)]
pub(crate) const HELD_OUT_SIXTH: [HeldOut; 8] = [
    HeldOut {
        row: 13,
        capture: "gamelog-run1099-amazon-cover1-clickfree-diagonal.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 8; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 0; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 0; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 11; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; William the Conqueror",
            "flags 1; tribe 9; who 2; team 0; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; King Philip II",
        ],
        digest: 0xa3ba0971242490d1,
    },
    HeldOut {
        row: 33,
        capture: "gamelog-run1158-greatsahara-turkish-enlightenment-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 7; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 3; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 4; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 4; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 8; who 1; team 1; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Mehmed II Khan Gazi",
            "flags 1; tribe 10; who 2; team 1; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; King Louis XIV",
        ],
        digest: 0x6881f8e264299440,
    },
    HeldOut {
        row: 53,
        capture: "gamelog-run1259-eastindies-inca-classical-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 18; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 1; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 1; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 0; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 2; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Emperor Huayna Capac",
            "flags 1; tribe 13; who 2; team 0; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; Tsar Peter the Great",
        ],
        digest: 0x46151db7b179fa2a,
    },
    HeldOut {
        row: 73,
        capture: "gamelog-run1335-greatsahara-british-industrial-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 7; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 4; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 5; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 5; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 11; who 1; team 1; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; William the Conqueror",
            "flags 1; tribe 14; who 2; team 1; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; Emperor Shi Huangdi",
        ],
        digest: 0xe82e30c3c76842f0,
    },
    HeldOut {
        row: 93,
        capture: "gamelog-run1396-greatlakes-german-modern-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 14; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 4; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 6; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 6; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 0; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 12; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Alaric the Visigoth",
            "flags 1; tribe 15; who 2; team 0; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; Queen Himiko",
        ],
        digest: 0xe88fb61345008795,
    },
    HeldOut {
        row: 113,
        capture: "gamelog-run1515b-lane2-lobby-leak-cover0-clickfree.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 14; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 4; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 4; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 11; who 1; team 1; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; William the Conqueror",
            "flags 1; tribe 16; who 2; team 1; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; King Dangun",
        ],
        digest: 0x62787d8643bf3323,
    },
    HeldOut {
        row: 133,
        capture: "gamelog-run831-greatsahara-mongol-alltech-trace.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 7; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 3; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 8; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 0; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 17; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Batu Khan",
            "flags 1; tribe 18; who 2; team 0; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; Tadodaho",
        ],
        digest: 0x3f5c8efb3780555a,
    },
    HeldOut {
        row: 153,
        capture: "gamelog-run983-greatlakes-roman-diff2-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 14; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 2; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 6; who 1; team 1; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Emperor Marcus Aurelius",
            "flags 1; tribe 19; who 2; team 1; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; Crazy Horse",
        ],
        digest: 0xcf0f26b2da326655,
    },
];

/// **The sixth's humans, as declared** (item 4566, before any file): each
/// row's [`Scripted`] human as [`script_rule`] reads it off the row's start
/// built from the page's block. No row is idle (a row the rule refused
/// would be recorded idle, and this table would say so): every start has a unit `0`
/// with a heading in its own region, a building `2000`, the Citizen and
/// the Farm available and affordable, and a clear Farm site within
/// [`FARM_REACH`] tiles of the capital.
#[cfg(test)]
pub(crate) const HELD_OUT_SIXTH_HUMANS: [Scripted; 8] = [
    // row 13
    Scripted {
        scout: 0,
        scout_to: sim::Pos { x: 37176, y: 35736 },
        capital: 2000,
        builder: 5,
        farm_at: sim::Pos { x: 38112, y: 34464 },
    },
    // row 33
    Scripted {
        scout: 0,
        scout_to: sim::Pos { x: 8184, y: 31128 },
        capital: 2000,
        builder: 13,
        farm_at: sim::Pos { x: 5280, y: 30048 },
    },
    // row 53
    Scripted {
        scout: 0,
        scout_to: sim::Pos { x: 22008, y: 41880 },
        capital: 2000,
        builder: 3,
        farm_at: sim::Pos { x: 18912, y: 39840 },
    },
    // row 73
    Scripted {
        scout: 0,
        scout_to: sim::Pos { x: 10488, y: 30360 },
        capital: 2000,
        builder: 10,
        farm_at: sim::Pos { x: 6240, y: 29088 },
    },
    // row 93
    Scripted {
        scout: 0,
        scout_to: sim::Pos { x: 43320, y: 41112 },
        capital: 2000,
        builder: 10,
        farm_at: sim::Pos { x: 42912, y: 41568 },
    },
    // row 113
    Scripted {
        scout: 0,
        scout_to: sim::Pos { x: 6648, y: 29592 },
        capital: 2000,
        builder: 11,
        farm_at: sim::Pos { x: 3168, y: 29856 },
    },
    // row 133
    Scripted {
        scout: 0,
        scout_to: sim::Pos { x: 34872, y: 37272 },
        capital: 2000,
        builder: 8,
        farm_at: sim::Pos { x: 37728, y: 34464 },
    },
    // row 153
    Scripted {
        scout: 0,
        scout_to: sim::Pos { x: 5880, y: 31896 },
        capital: 2000,
        builder: 4,
        farm_at: sim::Pos { x: 2208, y: 30624 },
    },
];

/// **The sixth battery's scores**, `HELD_OUT_SIXTH[k]` played as
/// `run1788 + k` (item 4566): `None` until the original has played it. The
/// found column is pinned once and never re-pinned; it is the estimate
/// (DECISIONS 151 §6, 152 §4). Scored on `38bfbab38`'s tree and pinned on
/// the merged tree `3b59df526`, where 43 commits of the base moved later
/// lines of rows 53, 93 and 133 and no row's first parting.
#[cfg(test)]
pub(crate) const SCORES_SIXTH: [Option<FirstScore>; 8] = [
    // run1788
    // row 13, as found 2026-11-22: 2 lines whole; units 7,675 of 12,001, first 7675; builds 7,675
    // of 12,001, first 7675; ammo 11,075 of 12,001, first 9001; groups 7,675 of 12,001, first 7675;
    // guys 7,675 of 12,001, first 7675; leaders 7,751 of 12,001, first 7751; cities 8,001 of
    // 12,001, first 8001; items 8,561 of 12,001, first 8561; goods 8,473 of 12,001, first 8286;
    // world 7,676 of 12,001, first 7676; reach 7,675 of 12,001 states, 6,001 of 6,001 in the first
    // 6,001.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 12001,
            agreed: [
                7675, 7675, 12001, 11075, 12001, 7675, 7675, 7751, 8001, 8561, 8473, 7676,
            ],
            first: &[
                ("units", (7675, Some(1817632919), 4097706533)),
                ("builds", (7675, Some(2607812443), 456200029)),
                ("ammo", (9001, Some(1), 2054097555)),
                ("groups", (7675, Some(2026795161), 1961455771)),
                ("guys", (7675, Some(3912017497), 3171329874)),
                ("leaders", (7751, Some(3296775262), 3204434973)),
                ("cities", (8001, Some(3206817740), 3243124685)),
                ("items", (8561, Some(759187207), 770721545)),
                ("goods", (8286, Some(1321043981), 1349879823)),
                ("world", (7676, Some(2649672179), 2530724234)),
            ],
            over: None,
        },
        moved: None,
    }),
    // run1789
    // row 33, as found 2026-11-22: 1 line whole; units 2,411 of 3,944, first 2411; builds 2,483 of
    // 3,944, first 2448; ammo 3,279 of 3,944, first 2849; deaths 2,910 of 3,944, first 2910; groups
    // 2,566 of 3,944, first 2566; guys 2,412 of 3,944, first 2412; leaders 2,433 of 3,944, first
    // 2433; cities 2,411 of 3,944, first 2411; items 3,414 of 3,944, first 3103; goods 3,119 of
    // 3,944, first 2907; world 2,414 of 3,944, first 2414; reach 2,411 of 3,944 states, 2,411 of
    // 3,944 in the first 6,001.
    Some(FirstScore {
        last: Some(3943),
        found: Score {
            states: 3944,
            agreed: [
                2411, 2483, 3944, 3279, 2910, 2566, 2412, 2433, 2411, 3414, 3119, 2414,
            ],
            first: &[
                ("units", (2411, Some(3090837024), 2794807023)),
                ("builds", (2448, Some(835850070), 3087470893)),
                ("ammo", (2849, Some(2591427256), 1)),
                ("deaths", (2910, Some(1951075586), 1)),
                ("groups", (2566, Some(1395743986), 2512936357)),
                ("guys", (2412, Some(1974607008), 839983040)),
                ("leaders", (2433, Some(2744254649), 2938699901)),
                ("cities", (2411, Some(2537762736), 2569744296)),
                ("items", (3103, Some(529023019), 3376233514)),
                ("goods", (2907, Some(3516364582), 3470227234)),
                ("world", (2414, Some(3160934879), 3182954975)),
            ],
            over: None,
        },
        // Moved by item 4583 (`do_trade`'s link gate, `5ed97a`..`5ed9b9`,
        // and `LAB_005eda11`'s ungated `think_caravan(1)`, run1800): the row's
        // first parting 2411 (units and cities) is gone. Every line but ammo
        // is whole to 3,943; ammo 3,279 -> 3,927, parting first on 3386 (ours'
        // ammo sum 3321501423, theirs 3301971689).
        // Moved by item 4591 (piece 49's bays, the original's `get_position`
        // sweep on run1579's packet; run1801's ammo range 4 on 3386, who=2's
        // Musketeers `2/25` `launch.x` ours 6906, theirs 6907): ammo 3,927 ->
        // 3,944. **Whole**, twelve lines to the original's end 3,943.
        moved: Some(Score {
            states: 3944,
            agreed: [
                3944, 3944, 3944, 3944, 3944, 3944, 3944, 3944, 3944, 3944, 3944, 3944,
            ],
            first: &[],
            over: None,
        }),
    }),
    // run1790
    // row 53, as found 2026-11-22: 1 line whole; units 3,358 of 12,001, first 3358; builds 3,732 of
    // 12,001, first 3732; ammo 3,992 of 12,001, first 3795; deaths 3,819 of 12,001, first 3707;
    // groups 3,716 of 12,001, first 3716; guys 3,358 of 12,001, first 3358; leaders 3,707 of
    // 12,001, first 3707; cities 3,825 of 12,001, first 3825; items 9,173 of 12,001, first 4574;
    // goods 3,923 of 12,001, first 3832; world 3,707 of 12,001, first 3707; reach 3,358 of 12,001
    // states, 3,358 of 6,001 in the first 6,001.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 12001,
            agreed: [
                3358, 3732, 12001, 3992, 3819, 3716, 3358, 3707, 3825, 9173, 3923, 3707,
            ],
            first: &[
                ("units", (3358, Some(992329593), 1194311501)),
                ("builds", (3732, Some(2175428037), 986670026)),
                ("ammo", (3795, Some(684788378), 1)),
                ("deaths", (3707, Some(1220611515), 3817282588)),
                ("groups", (3716, Some(3464512650), 3450219541)),
                ("guys", (3358, Some(479939638), 3928181788)),
                ("leaders", (3707, Some(3179845721), 3311573076)),
                ("cities", (3825, Some(4087239103), 2717798815)),
                ("items", (4574, Some(3301772440), 3298888854)),
                ("goods", (3832, Some(2135290454), 2227565146)),
                ("world", (3707, Some(2727186382), 2158397478)),
            ],
            over: None,
        },
        // Moved by item 4589 (`Guy::move`'s moving walk, row 73's word),
        // past the row's first parting 3358, which stands: ammo 3,992 ->
        // 3,918 and items 9,173 -> 9,154, named falls; deaths 3,819 -> 3,880.
        // No first parting moved.
        // Then item 4608 (`Guy::set_all_pivots` bears an `ATTACK_GROUND`
        // order's figures on the order's point, aimed or not): ammo 3,918 -> 3,880, deaths 3,880 ->
        // 3,810, groups 3,716 -> 3,726, leaders 3,707 -> 3,715, cities 3,825 -> 3,951, items 9,154
        // -> 8,940, goods 3,923 -> 3,865, world 3,707 -> 3,721; all past the row's first parting
        // 3358, which stands.
        moved: Some(Score {
            states: 12001,
            agreed: [
                3358, 3732, 12001, 3880, 3810, 3726, 3358, 3715, 3951, 8940, 3865, 3721,
            ],
            first: &[
                ("units", (3358, Some(992329593), 1194311501)),
                ("builds", (3732, Some(2175428037), 986670026)),
                ("ammo", (3807, Some(1), 2355366420)),
                ("deaths", (3713, Some(3855424552), 2127964779)),
                ("groups", (3714, Some(1263612184), 908409090)),
                ("guys", (3358, Some(479939638), 3928181788)),
                ("leaders", (3713, Some(2512362065), 969579214)),
                ("cities", (3951, Some(4088156871), 4088615623)),
                ("items", (4596, Some(3298888854), 3301772440)),
                ("goods", (3832, Some(2135290454), 2227565146)),
                ("world", (3713, Some(2100724504), 1088191604)),
            ],
            over: None,
        }),
    }),
    // run1791
    // row 73, as found 2026-11-22: 1 line whole; units 1,563 of 3,438, first 1563; builds 1,569 of
    // 3,438, first 1569; ammo 2,593 of 3,438, first 2557; deaths 2,604 of 3,438, first 2604; groups
    // 1,678 of 3,438, first 1678; guys 1,564 of 3,438, first 1564; leaders 1,576 of 3,438, first
    // 1576; cities 1,563 of 3,438, first 1563; items 2,589 of 3,438, first 1949; goods 2,460 of
    // 3,438, first 2353; world 1,566 of 3,438, first 1566; reach 1,563 of 3,438 states, 1,563 of
    // 3,438 in the first 6,001.
    Some(FirstScore {
        last: Some(3437),
        found: Score {
            states: 3438,
            agreed: [
                1563, 1569, 3438, 2593, 2604, 1678, 1564, 1576, 1563, 2589, 2460, 1566,
            ],
            first: &[
                ("units", (1563, Some(4059076565), 2125891403)),
                ("builds", (1569, Some(146391282), 1875296499)),
                ("ammo", (2557, Some(4190309544), 1)),
                ("deaths", (2604, Some(1), 2113539492)),
                ("groups", (1678, Some(3126200451), 1300891790)),
                ("guys", (1564, Some(374263833), 1839451683)),
                ("leaders", (1576, Some(1096030747), 4201978914)),
                ("cities", (1563, Some(4048038473), 2512136249)),
                ("items", (1949, Some(1693995504), 1774735860)),
                ("goods", (2353, Some(326399801), 337934141)),
                ("world", (1566, Some(563759280), 2466466211)),
            ],
            over: None,
        },
        // Moved by item 4574 (the Manchu Riflemen's three keys, piece 67, from
        // 2689, past the row's first parting 1563, which stands): ammo 2,593
        // -> 2,592 (on 2784 ours' ammo sum 1 -> 3490584490, theirs 1) and
        // items 2,589 -> 2,557, named falls; goods 2,460 -> 2,476 (3041..=3056,
        // ours 711751501 -> 665614155, theirs 665614155). No first parting moved.
        // Moved by item 4583 (`do_trade`'s link gate, run1800: on 1563 who=2's
        // Caravan `2/31`, homed on who=1's city 3, is refused and given its
        // order again, partnerless, as theirs): units 1,563 -> 2,779 and
        // cities 1,563 -> 2,801, the row's first parting now units 2779 (ours'
        // sum 2333664493, theirs 4092650740) and guys 2779. Every column
        // moved; see `docs/journal/2026-11-23-item-4583.md`.
        // Moved by item 4589 (`Guy::move`'s moving walk is `CHAR_ATTACKWALK`
        // for an aimed fire-on-the-move type, `docs/ANIM.md` §16: on 2779
        // who=2's ARMOREDCAR `2/70`, idled by its guard stand, is put back on
        // the attack walk; guy record 70 `cur_anim` ours 7 -> 10, theirs 10,
        // `end_time` 26 -> 30, theirs 30): units and guys 2,779 -> 3,438, and
        // every line but ammo whole to the original's end 3437; ammo 2,788
        // -> 3,280, the row's first parting now ammo 3279 (ours' sum
        // 3061199017, theirs 3074240683). Every column moved; see
        // `docs/journal/2026-11-23-item-4589.md`.
        // Moved by item 4595 (a release at a gone target still takes a
        // `graph_index`, `docs/COMBAT.md` §24.4: on 3278 who=2's Armored Car
        // `2/32`, on `CHAR_ATTACKWALK` at the gone Citizen `0/4`, reaches
        // `Objects::add_ammo`, which counts 1008 while `Ammo::init` refuses
        // the round; on 3279 range 7, record 4, `graph_index` ours 1008 ->
        // 1009, theirs 1009): ammo 3,280 -> 3,438, and the row whole on all
        // twelve lines to the original's end 3437.
        moved: Some(Score {
            states: 3438,
            agreed: [
                3438, 3438, 3438, 3438, 3438, 3438, 3438, 3438, 3438, 3438, 3438, 3438,
            ],
            first: &[],
            over: None,
        }),
    }),
    // run1792
    // row 93, as found 2026-11-22: 1 line whole; units 2,342 of 12,001, first 2342; builds 2,688 of
    // 12,001, first 2686; ammo 3,799 of 12,001, first 2692; deaths 3,464 of 12,001, first 2775;
    // groups 2,740 of 12,001, first 2740; guys 2,686 of 12,001, first 2686; leaders 2,698 of
    // 12,001, first 2689; cities 2,974 of 12,001, first 2871; items 4,832 of 12,001, first 3127;
    // goods 4,140 of 12,001, first 3157; world 2,691 of 12,001, first 2691; reach 2,342 of 12,001
    // states, 2,342 of 6,001 in the first 6,001.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 12001,
            agreed: [
                2342, 2688, 12001, 3799, 3464, 2740, 2686, 2698, 2974, 4832, 4140, 2691,
            ],
            first: &[
                ("units", (2342, Some(1169777861), 259155158)),
                ("builds", (2686, Some(547053547), 644243435)),
                ("ammo", (2692, Some(1859606587), 696997629)),
                ("deaths", (2775, Some(2882887954), 2321178662)),
                ("groups", (2740, Some(4287796956), 1715573666)),
                ("guys", (2686, Some(384424200), 3607616169)),
                ("leaders", (2689, Some(3893770918), 226245285)),
                ("cities", (2871, Some(3617272752), 3487511460)),
                ("items", (3127, Some(2121282860), 2092447016)),
                ("goods", (3157, Some(2924123420), 3056768288)),
                ("world", (2691, Some(2608492097), 2791665259)),
            ],
            over: None,
        },
        // Moved by item 4574 (the Fighters 242 and 243 from 4749, the
        // Anti-Aircraft Battery 233 from 4894, the Howitzer 222 from 8297 and
        // the Asian Heavy Machine Gun 1131 from 8406, all past the row's first
        // parting 2342, which stands): ammo 3,799 -> 3,924, 386 states gained
        // and 261 lost, the first on 5210 (ours' ammo sum 3991153570 -> 1,
        // theirs 1: no round in flight). No first parting moved.
        //
        // Moved by item 4577 (run1799's ranges: `process_healing`'s
        // President arm, `005e0670:179`, heals on an allied or unowned cell;
        // who=1's Volksgrenadiers `1/39` beside the President `1/17` take
        // `healing` 20 on 2342, units range 1359, record 39, ours 0, theirs
        // 20, and `1/40`'s point goes): units 2,342 -> 2,686, the row's first
        // parting 2342 -> 2686 (units, builds and guys on one state); ammo
        // 3,924 -> 4,263; deaths 3,464 -> 3,406, items 4,832 -> 3,127 and
        // goods 4,140 -> 3,056 (first 3157 -> 3056), all past 2686; ours'
        // walk now holds `game_over` from 9522, theirs plays to 12,000.
        //
        // Moved by item 4583 (`do_trade`'s link gate and `scale`'s wrapped
        // product, measured on the tree merged with 4577): ammo 4,263 ->
        // 4,267, past the row's first parting 2686, which stands; the arm is
        // not split between the two. No first parting moved.
        // Then item 4585 (a tower is halved only while its city has never
        // been attacked), on the tree merged with 4583: ammo 4,267 ->
        // 3,785, deaths 3,406 -> 2,967, ours' `game_over` 9522 -> 9533; the
        // row's first parting 2686 stands.
        // Then item 4594 (a computer's city past half its hits calls a
        // citizen to repair it), on the tree merged with the base at
        // `9d1f92690`: ammo 3,785 -> 3,426, deaths 2,967 -> 2,914, ours'
        // `game_over` 9533 -> 11572; no first parting moved.
        // Then item 4590 (`compare_target`'s raid `/20` asks the target's
        // finished bit, `WallData::is_active`; run1803's packet scores the
        // unfinished Small City `2/2027` at 1613 for the Bazooka `1/31`):
        // the first parting 2686 -> 2962 (units range 6364, record 77,
        // `order.type` ours 10, theirs 0x80); builds 2,688 -> 3,078, guys
        // 2,686 -> 2,967, every line forward; ours' `game_over` 11572 -> 9802.
        // Then item 4606 (the units line walks an `ATTACK_GROUND` order,
        // `AttackGroundOrder::walk_data@00487a60`: who=2's `2/77`, TI 271,
        // land, fired, `(12984, 22872, 0, 1)`): units 2,962 -> 2,965, the
        // row's first parting 2962 -> 2965 (range 1786, record 38,
        // `targeted` ours 2, theirs 1); no other line moved. Cities 3,367 ->
        // 3,376 (first 3376) was measured on the base `b6c90c0a9` before the
        // fix, a merge's, not this item's.
        // Then item 4608 (`Guy::set_all_pivots` bears an `ATTACK_GROUND`
        // order's figures on the order's point, aimed or not): builds 3,078 -> 3,026, ammo 3,483 ->
        // 3,323, deaths 3,632 -> 3,008, groups 3,002 -> 3,004, items 5,437 -> 3,954 (first 3468 ->
        // 3146), goods 4,397 -> 4,117, ours' `game_over` 9802 -> 11109; all past the row's first
        // parting 2965 (units), which stands, and guys 2966 stands.
        moved: Some(Score {
            states: 12001,
            agreed: [
                2965, 3026, 12001, 3323, 3008, 3004, 2967, 2983, 3376, 3954, 4117, 2975,
            ],
            first: &[
                ("units", (2965, Some(2944560567), 523660729)),
                ("builds", (3004, Some(2482202651), 4285163131)),
                ("ammo", (2978, Some(1714572981), 3065139014)),
                ("deaths", (3008, Some(3817289519), 3817489508)),
                ("groups", (2982, Some(1074681641), 115365832)),
                ("guys", (2966, Some(96889624), 2207673204)),
                ("leaders", (2983, Some(3337828037), 36057006)),
                ("cities", (3376, Some(897138611), 842153906)),
                ("items", (3146, Some(2121282860), 2092447016)),
                ("goods", (3157, Some(2924123420), 3056768288)),
                ("world", (2975, Some(4068279797), 2248541463)),
            ],
            over: Some(11109),
        }),
    }),
    // run1793
    // row 113, as found 2026-11-22: 1 line whole; units 2,259 of 4,573, first 2259; builds 2,338 of
    // 4,573, first 2323; ammo 3,335 of 4,573, first 2941; deaths 3,087 of 4,573, first 2993; groups
    // 2,470 of 4,573, first 2470; guys 2,259 of 4,573, first 2259; leaders 2,351 of 4,573, first
    // 2351; cities 2,259 of 4,573, first 2259; items 3,176 of 4,573, first 2526; goods 4,081 of
    // 4,573, first 2666; world 2,261 of 4,573, first 2261; reach 2,259 of 4,573 states, 2,259 of
    // 4,573 in the first 6,001; ours' game over from 4118.
    Some(FirstScore {
        last: Some(4572),
        found: Score {
            states: 4573,
            agreed: [
                2259, 2338, 4573, 3335, 3087, 2470, 2259, 2351, 2259, 3176, 4081, 2261,
            ],
            first: &[
                ("units", (2259, Some(3462201175), 3106729391)),
                ("builds", (2323, Some(52058802), 98261619)),
                ("ammo", (2941, Some(1), 3925806276)),
                ("deaths", (2993, Some(1), 1927744677)),
                ("groups", (2470, Some(939459940), 872023246)),
                ("guys", (2259, Some(1132056778), 3749433370)),
                ("leaders", (2351, Some(417477691), 3343529196)),
                ("cities", (2259, Some(6368709), 3892456893)),
                ("items", (2526, Some(2178306644), 2195608150)),
                ("goods", (2666, Some(1648268551), 1581946117)),
                ("world", (2261, Some(1332653642), 414756426)),
            ],
            over: Some(4118),
        },
        // Moved by item 4574 (the Carabineer, piece 140 as 139's rows, from
        // 3047, past the row's first parting 2259, which stands): ammo 3,335
        // -> 3,336, on 4017 (ours' ammo sum 1731922939 -> 1, theirs 1). No
        // first parting moved.
        // Moved by item 4583 (`do_trade`'s link gate, run1800): the row's
        // first parting 2259 is gone and ours' game over at 4118 with it. Every
        // line but cities is whole to 4,572; cities 2,259 -> 3,995, parting on
        // 3995 (ours' sum 1934313053, theirs 1933788765).
        // Moved by item 4592 (a city's `vans` in the order the links were
        // made, run1802's ranges: cities range 11, record 0, `vans[k]`, byte
        // 4, `who` ours 1, theirs 2 on 3995; who=1's city 0 holds who=2's
        // `2/3` before who=1's own `1/3`): cities 3,995 -> 4,573. **The row
        // is whole, twelve lines of twelve, 4,573 of 4,573 to the original's
        // end.**
        moved: Some(Score {
            states: 4573,
            agreed: [
                4573, 4573, 4573, 4573, 4573, 4573, 4573, 4573, 4573, 4573, 4573, 4573,
            ],
            first: &[],
            over: None,
        }),
    }),
    // run1794
    // row 133, as found 2026-11-22: 1 line whole; units 1,162 of 6,085, first 1162; builds 958 of
    // 6,085, first 958; ammo 2,517 of 6,085, first 1485; deaths 1,484 of 6,085, first 1484; groups
    // 1,242 of 6,085, first 1242; guys 1,162 of 6,085, first 1162; leaders 956 of 6,085, first 956;
    // cities 1,376 of 6,085, first 1376; items 1,614 of 6,085, first 1614; goods 1,758 of 6,085,
    // first 1758; world 1,197 of 6,085, first 1197; reach 956 of 6,085 states, 956 of 6,001 in the
    // first 6,001.
    Some(FirstScore {
        last: Some(6084),
        found: Score {
            states: 6085,
            agreed: [
                1162, 958, 6085, 2517, 1484, 1242, 1162, 956, 1376, 1614, 1758, 1197,
            ],
            first: &[
                ("units", (1162, Some(3743709245), 4104174444)),
                ("builds", (958, Some(696106489), 657638887)),
                ("ammo", (1485, Some(1244795671), 1)),
                ("deaths", (1484, Some(1), 3470790278)),
                ("groups", (1242, Some(2943953166), 3106154756)),
                ("guys", (1162, Some(23336544), 1735796580)),
                ("leaders", (956, Some(1209252435), 1776794036)),
                ("cities", (1376, Some(1412642287), 1429550575)),
                ("items", (1614, Some(3614982968), 3623633722)),
                ("goods", (1758, Some(4286740555), 139819094)),
                ("world", (1197, Some(1966726756), 3748257716)),
            ],
            over: None,
        },
        // Moved by item 4575 (the nuke arm skips an enemy researching the
        // Missile Shield: who=2 lists a Jet Fighter, not an ICBM, on 956),
        // the first parting 956 -> 1757; then by item 4585 (a tower is
        // halved only while its city has never been attacked: who=2 lists
        // no Bunker in its capital on 1757): units 1,758 -> 1,905, builds
        // 1,758 -> 1,905, ammo 2,738 -> 2,680 (first 2179 -> 2184), deaths
        // 2,524 -> 2,382 (first 2375 -> 2382), groups 1,821 -> 2,067, guys
        // 1,758 -> 1,906, leaders 1,757 -> 1,905, cities 1,758 -> 1,951,
        // items 2,185 -> 2,676 (first 2051 -> 2310), goods 1,827 -> 2,418
        // (first 1758 -> 2418), world 1,758 -> 1,906; the first parting is
        // units, builds and leaders on 1905. Then by item 4594 (a computer's
        // city past half its hits calls a citizen to repair it: who=2's
        // `2/72` repairs its capital `2/2000` from 1904): units 1,905 ->
        // 2,126 (first 1905 -> 1906), builds 1,905 -> 2,166, ammo 2,680 ->
        // 2,770 (first 2184 -> 2198), deaths 2,382 -> 2,529 (first 2382 ->
        // 2380), groups 2,067 -> 2,319, guys 1,906 -> 2,199, leaders 1,905
        // -> 2,228 (first 2212), cities 1,951 -> 2,526 (first 2297), items
        // 2,676 -> 2,464 (first 2310 -> 2464), goods 2,418 -> 2,403 (first
        // 2418 -> 2403), world 1,906 -> 2,234 (first 2215); the first
        // parting is units alone on 1906. Then by item 4597 (a computer's
        // repair swarm walks under `MOVE_TO`, `local_40` being 3 for
        // `BUILD_AT` alone: `2/72`'s `order.type` ours 3, theirs 1 on 1906,
        // both 1 now): units 2,126 -> 2,199 (first 1906 -> 2199); no other
        // line moved, and the first parting is builds alone on 2166. Then by
        // item 4602 (an anti-air building winds on its sighting, `near_o`,
        // not its target, and its turret turns at its aim: who=2's SAM
        // `2/2027` on 2166 `recharging` ours 0, theirs 1, on 2167
        // `job_counter` ours 0x40000000, theirs 0x42040000; both agree
        // now): units 2,199 -> 2,245 (first 2199 -> 2245), builds 2,166 ->
        // 2,351 (first 2166 -> 2351), ammo 2,770 -> 2,794, deaths 2,529 ->
        // 2,598 (first 2380 -> 2443), groups 2,319 -> 2,351 (first 2319 ->
        // 2351), guys 2,199 -> 2,351 (first 2199 -> 2351), leaders 2,228 ->
        // 2,245 (first 2212 -> 2245), cities 2,526 -> 2,442 (a fall past its
        // first parting, 2297 on both), items 2,464 -> 3,222 (first 2464 ->
        // 3035), goods 2,403 -> 2,721 (first 2403 -> 2721), world 2,234 ->
        // 2,352 (first 2215 -> 2352); the first parting is ammo alone on
        // 2198. Then by item 4604 (an anti-air building's `missile="1"`
        // round flies `Ammo::init`'s spline at 150 from its node's
        // measured launch: the SAM `2/2027`'s first rocket on 2198,
        // launch ours (9024, 27264, 479), theirs (9079, 27270, 688), traj
        // ours 1, theirs 2; both agree now): units 2,245 -> 2,351 (first
        // 2245 -> 2351), ammo 2,794 -> 3,015 (first 2198 -> 2408), deaths
        // first 2443 -> 2442 (past the parting), leaders 2,245 -> 2,367
        // (first 2245 -> 2297), cities 2,442 -> 2,543, items 3,222 -> 3,007
        // (a fall past the parting, first 3035 -> 3007), goods 2,721 ->
        // 4,316 (first 2721 -> 3258); the first parting is cities and
        // leaders on 2297. Then by item 4607 (every Temple's close fixes
        // the borders, built or not, `Build::close@00628980`'s `is(0x1b5)`
        // at `0x628d2c`: who=1 razes who=2's Temple site `2/2009` on frame
        // 2295, and on 2297 who=0's `reg_terr[1]` ours 741, theirs 0, city
        // 1's `bordering` ours 3, theirs 0; both agree now): leaders 2,367
        // -> 2,423 (first 2297 -> 2413), cities 2,543 -> 2,550 (first 2297
        // -> 2448); the first parting is units, builds, groups and guys on
        // 2351. Then by item 4610 (`check_orphaned_buildings` walks the
        // sites by object number, `006c9f20`'s `iVar9` from `obj_base`: on
        // 2350 who=2's one idle citizen `2/11` went to the started site
        // `2/2003` (uid 112) in theirs and to `2/2033` (uid 69, an earlier
        // slot) in ours; on 2351 units range 3699, record 11, `x` ours
        // 10632, theirs 10608, `y` 33096 against 33104; builds range 2085,
        // record 2003, `build_masks` ours 8584, theirs 392; group 132's
        // `facing` ours 1, theirs 0; all agree now): units 2,351 -> 2,376,
        // builds 2,351 -> 2,370, ammo 3,015 -> 2,980 and deaths 2,598 ->
        // 2,442 (falls past the parting), groups 2,351 -> 2,447, guys 2,351
        // -> 2,399, goods 4,316 -> 4,136, world 2,352 -> 2,400; the first
        // parting is builds alone on 2370. Then by item 4620
        // (`City::find_buildings` walks by owner and object number,
        // `007384c0`'s `+0x1ec`/`+0x1f0` lists from `obj_base`: who=2's
        // Village `2/2026` finishes on 2370 and sweeps `2/2008` (slot 28)
        // and the site `2/2007` (slot 92); builds range 2661, record 2026,
        // `city_down` ours 2008, theirs 2007; range 2221, record 2007, ours
        // -1, theirs 2008; range 2259, record 2008, ours 2007, theirs -1; all
        // agree now): builds 2,370 -> 2,432 (first 2370 -> 2432); no other
        // line moved, and the first parting is units alone on 2376. Then by
        // item 4628 (`get_damage`'s rocky step reads the target's cell
        // `WData.flags & 8`, `644c85`..`644cf2`, run1819's packet: the
        // Redoubt `1/2025`'s blow on the Elite Special Forces `2/0` on 2375
        // is 18 -> 11; units range 3283, record 0, `damage` ours 47, theirs
        // 40 on 2376; both 40 now): units 2,376 -> 2,440 (first 2376 ->
        // 2440), builds 2,432 -> 2,451 (first 2432 -> 2451), ammo 2,980 ->
        // 3,045 (first 2408 -> 2440), deaths 2,442 -> 2,612 (first 2442 ->
        // 2457), groups 2,447 -> 2,445 (a fall past the parting, first 2447
        // -> 2445), guys 2,399 -> 2,440 (first 2399 -> 2440), leaders 2,423
        // -> 2,447 (first 2413 -> 2445), cities 2,550 -> 2,583 (first 2448
        // -> 2583), items 3,007 -> 2,753 and goods 4,136 -> 3,674 (falls
        // past the parting, first 3007 -> 2753 and 3258 -> 2685), world
        // 2,400 -> 2,440 (first 2400 -> 2440); the first parting is units,
        // ammo, guys and world on 2440. Then by item 4632 (the emplaced
        // weapon's latch takes its head order's `FIGHT_REENTRY`,
        // `Unit::fight@005fd4d0:92`-`100`: who=1's Flamethrower `1/115`, TI
        // 131, on 2439 with `recharging` 30 and its attack's `flags` 16
        // walks past the reload gate in theirs and turns; ours stood and
        // swung, one draw `Guy::set_anim+0xf2f < Guy::move+0x166` over
        // theirs' 25; units range 3114, record 115, `angle` ours
        // 1206124544, theirs 1183514624 on 2440, the guy's `cur_anim` 12
        // against 8 and `hold_attack` 0 against 1; all agree now): units
        // 2,440 -> 2,543 (first 2440 -> 2543), ammo 3,045 -> 3,143 (first
        // 2440 -> 2547), deaths 2,612 -> 2,563 (a fall past the parting,
        // first 2457 stands), groups 2,445 -> 2,685 (first 2445 -> 2685),
        // guys 2,440 -> 2,543 (first 2440 -> 2543), leaders 2,447 -> 2,749
        // (first 2445 -> 2608), cities 2,583 -> 2,978 (first 2583 -> 2978),
        // items 2,753 -> 3,148 (first 2753 -> 3007), goods 3,674 -> 3,983
        // (first 2685 -> 3322), world 2,440 -> 2,680 (first 2440 -> 2680);
        // the first parting is builds alone on 2451.
        // Then item 4608 (`Guy::set_all_pivots` bears an `ATTACK_GROUND`
        // order's figures on the order's point, aimed or not): units 2,543 -> 2,643 (first 2543 ->
        // 2566) and guys 2,543 -> 2,677 (first 2543 -> 2677), past the row's first parting 2451
        // (builds), which stands.
        // Then item 4655 (`get_damage`'s recapture step reads the target
        // city's race, `:477`-`:494`, whoever owns the building: on 2450 the
        // SAM `2/2027`'s rocket, its target `1/70` dead, strikes who=2's own
        // capital `2/2000` for 44, not 22; builds range 2180, record 2000,
        // `damage` ours 5,963, theirs 5,985 on 2451, both 5,985 now): builds
        // 2,451 -> 2,768 (first 2451 -> 2689), ammo 3,143 -> 3,144, leaders
        // 2,749 -> 2,751 (first 2608 -> 2751), goods 3,983 -> 3,961 (a fall
        // past the parting), ours' end none -> 6040; the first parting is
        // deaths alone on 2457.
        // Then item 4659 (`get_damage`'s building arm writes `dtype` 3 when
        // `BuildData::get_shot` is 4, the Redoubt's alone: on 2456 who=1's
        // Redoubt `1/2025` kills who=2's Elite Special Forces `2/0`; deaths
        // slot 7 on 2457, `cur_anim` ours 17, theirs 19, `place.angle` ours
        // 0xbd69188d, theirs 0xe9b90000, impact angle ours 100.0, theirs
        // 38.0, all agree now): deaths 2,563 -> 3,206 (first 2457 -> 3206);
        // the first parting is ammo alone on 2547.
        moved: Some(Score {
            states: 6085,
            agreed: [
                2643, 2768, 6085, 3144, 3206, 2685, 2677, 2751, 2978, 3148, 3961, 2680,
            ],
            first: &[
                ("units", (2566, Some(1723402503), 567609615)),
                ("builds", (2689, Some(2754752570), 3680907824)),
                ("ammo", (2547, Some(3276077082), 122615738)),
                ("deaths", (3206, Some(1000348558), 2108296578)),
                ("groups", (2685, Some(293453109), 2102443375)),
                ("guys", (2677, Some(1670959914), 1725420330)),
                ("leaders", (2751, Some(2269206203), 3698742979)),
                ("cities", (2978, Some(1121539707), 1131566715)),
                ("items", (3007, Some(3525321571), 3415154359)),
                ("goods", (3322, Some(569473142), 586774648)),
                ("world", (2680, Some(3147089146), 4135110218)),
            ],
            over: Some(6040),
        }),
    }),
    // run1795
    // row 153, as found 2026-11-22: 12 lines whole; reach 12,001 of 12,001 states, 6,001 of 6,001
    // in the first 6,001.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 12001,
            agreed: [
                12001, 12001, 12001, 12001, 12001, 12001, 12001, 12001, 12001, 12001, 12001, 12001,
            ],
            first: &[],
            over: None,
        },
        moved: None,
    }),
];

/// The sixth battery's line: games whole of the eight, and games taken.
/// As found: row 153 whole to 12,000. Item 4591: row 33 whole to the
/// original's end 3,943 (piece 49's bays), 1 -> 2. Item 4595 made row 73
/// whole (a release at a gone target takes a `graph_index`): 2 -> 3.
/// Item 4592 made row 113 whole (a city's `vans` in the order the links
/// were made): 3 -> 4.
#[cfg(test)]
pub(crate) const HELD_OUT_SIXTH_LINE: (usize, usize) = (4, 8);

/// The sixth battery (item 4566): three seats, a team and a scripted human
/// a row, recorded to [`SIXTH_FRAMES`].
#[cfg(test)]
pub(crate) const SIXTH: Battery = Battery {
    label: "Parity held out, sixth",
    item: 4566,
    run: 1788,
    table: &HELD_OUT_SIXTH,
    scores: &SCORES_SIXTH,
    scores_name: "SCORES_SIXTH",
    line: HELD_OUT_SIXTH_LINE,
    frames: SIXTH_FRAMES,
};

/// **Every battery, in order** (item 4566): the handoff's lines read them,
/// the closed ones on one line and each open one on its own.
#[cfg(test)]
pub(crate) const BATTERIES: &[&Battery] =
    &[&FIRST, &SECOND, &THIRD, &FOURTH, &FIFTH, &SIXTH, &SEVENTH];

/// **The seventh's additions** (DECISIONS 154 §7, item 4600), row for row
/// with [`HELD_OUT_SEVENTH_ROWS`], by the sixth's recipe: the human (seat
/// 0, tribe 4) against the row's computer (seat 1, tribe 8, 13, 16, 11,
/// 17, 5, 13 and 3), and a computer of a nation not already seated on its
/// row: 20, 21, 22 and 23, which no battery has added, then 11, 17 and 0.
/// **Row 108 adds no seat** ([`NO_SEAT`]): its own lobby holds three
/// (Pericles, tribe 5, and Ramesses II, tribe 7, at seats 1 and 2), the
/// most `MAP_SIZE 2` holds, so it takes the team alone. The team
/// alternates as the sixth's: on rows 8, 48, 88 and 128 the human and the
/// added computer are team 0; on rows 28, 68, 108 and 148 the two
/// computers are team 1.
#[cfg(test)]
pub(crate) const HELD_OUT_SEVENTH_SEATS: [Seated; 8] = [
    Seated {
        tribe: 20,
        mates: [0, 2],
        team: 0,
    },
    Seated {
        tribe: 21,
        mates: [1, 2],
        team: 1,
    },
    Seated {
        tribe: 22,
        mates: [0, 2],
        team: 0,
    },
    Seated {
        tribe: 23,
        mates: [1, 2],
        team: 1,
    },
    Seated {
        tribe: 11,
        mates: [0, 2],
        team: 0,
    },
    Seated {
        tribe: NO_SEAT,
        mates: [1, 2],
        team: 1,
    },
    Seated {
        tribe: 17,
        mates: [0, 2],
        team: 0,
    },
    Seated {
        tribe: 0,
        mates: [1, 2],
        team: 1,
    },
];

/// **The seventh eight, as declared** (item 4600, before any file): rows
/// 8, 28, …, 148 of `lobbies_on_disk`, each with [`HELD_OUT_SEVENTH_SEATS`]'
/// seat and team on the page; `MAP_SIZE 2` every row. Each is played to
/// [`SEVENTH_FRAMES`] with the human ordering ([`HELD_OUT_SEVENTH_HUMANS`]),
/// as `run1804 + k` for `HELD_OUT_SEVENTH[k]`.
#[cfg(test)]
pub(crate) const HELD_OUT_SEVENTH: [HeldOut; 8] = [
    HeldOut {
        row: 8,
        capture: "gamelog-run1010-greatsahara-turkish-diff3-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 7; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 3; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 0; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 8; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Mehmed II Khan Gazi",
            "flags 1; tribe 20; who 2; team 0; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; George Washington",
        ],
        digest: 0xb46a53f83576e8d5,
    },
    HeldOut {
        row: 28,
        capture: "gamelog-run1148-eastindies-russian-modern-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 18; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 6; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 6; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 13; who 1; team 1; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Tsar Ivan the Terrible",
            "flags 1; tribe 21; who 2; team 1; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; King Vikramaditya VI",
        ],
        digest: 0x8c91b0903a594bff,
    },
    HeldOut {
        row: 48,
        capture: "gamelog-run1249-greatsahara-korean-classical-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 7; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 5; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 1; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 1; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 0; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 16; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; King Jinheung",
            "flags 1; tribe 22; who 2; team 0; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; Beatrix Wilhelmina Armgard",
        ],
        digest: 0x87fafc72dd921039,
    },
    HeldOut {
        row: 68,
        capture: "gamelog-run1315-eastindies-british-classical-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 18; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 4; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 1; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 1; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 11; who 1; team 1; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; William the Conqueror",
            "flags 1; tribe 23; who 2; team 1; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; Darius",
        ],
        digest: 0xa502654d68779f02,
    },
    HeldOut {
        row: 88,
        capture: "gamelog-run1377-eastindies-mongol-medieval-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 18; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 4; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 2; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 2; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 0; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 17; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Batu Khan",
            "flags 1; tribe 11; who 2; team 0; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; King Henry VIII",
        ],
        digest: 0xbca810f8e4adbabd,
    },
    HeldOut {
        row: 108,
        capture: "gamelog-run1449-greatsahara-classical-melee-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 7; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 0; STARTING_TOWN 2; STARTING_RESOURCES 7; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 1; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 1; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 5; who 1; team 1; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Pericles",
            "flags 1; tribe 7; who 2; team 1; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; Pharaoh Ramesses II",
        ],
        digest: 0x2c1c2747021fa626,
    },
    HeldOut {
        row: 128,
        capture: "gamelog-run644-greatlakes-russian-diff2-24k-trace.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 14; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 2; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 0; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 13; who 1; team 8; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; Tsar Ivan the Terrible",
            "flags 1; tribe 17; who 2; team 0; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; Genghis Khan",
        ],
        digest: 0x8d17c675445baf7b,
    },
    HeldOut {
        row: 148,
        capture: "gamelog-run972-greatsahara-bantu-diff4-start.txt",
        info: "TEAM_STYLE 1; MAP_STYLE 7; MAP_SIZE 2; PLAYERS 2; MAX_OBSERVERS 0; GAME_SPEED 2; GAME_RULES 1; DIFFICULTY 4; STARTING_TOWN 2; STARTING_RESOURCES 1; STARTING_RESOURCES2 1; TECH_COST 3; REVEAL_MAP 1; POP_LIMIT 5; RUSH_RULES 0; CANNON_TIMES 0; STARTING_TECHNOLOGY 0; STARTING_TECHNOLOGY2 1; ENDING_TECHNOLOGY 7; ELIMINATION 1; VICTORY 0; WONDERWIN 5; SCORE_GOAL 4; POPWIN 8; TIME_LIMIT 1; CHAIRS 3; ECONWIN 2; SCENARIO_TYPE 0; SCRIPT_TYPE 0; MODS 0; (int)seed 12345; checksum_deep 0; checksum_window_size 256; checksum_failure_threshold 0; flags 0",
        players: &[
            "flags 7; tribe 4; who 0; team 8; handicap 0; play 0; pauses 0; diff 2; pop_cap_frame 0; Player",
            "flags 1; tribe 3; who 1; team 1; handicap 0; play 1; pauses 0; diff 2; pop_cap_frame 0; King Shaka",
            "flags 1; tribe 0; who 2; team 1; handicap 0; play 2; pauses 0; diff 2; pop_cap_frame 0; Emperor Acamapichtli",
        ],
        digest: 0xfd25e03af6273440,
    },
];

/// **The seventh's humans** (item 4600), row for row with
/// [`HELD_OUT_SEVENTH`]: each row's [`Ordering`] as [`ordering_rule`]
/// reads it off the row's start built from the page's block. No row is
/// idle. Unit `0` is a land unit on every row: Scout (TI 69) on 8, 48, 68,
/// 108, 128 and 148; Special Forces (75) on 28; Explorer (71) on 88. On
/// every row its first heading is the first of the four (toward the
/// centre on x), and all four corners of its square are on the map and in
/// its own region.
#[cfg(test)]
pub(crate) const HELD_OUT_SEVENTH_HUMANS: [Ordering; 8] = [
    // row 8
    Ordering {
        first: Scripted {
            scout: 0,
            scout_to: sim::Pos { x: 34872, y: 37272 },
            capital: 2000,
            builder: 5,
            farm_at: sim::Pos { x: 37344, y: 34464 },
        },
        square: [
            sim::Pos { x: 34872, y: 34872 },
            sim::Pos { x: 37272, y: 34872 },
            sim::Pos { x: 37272, y: 37272 },
            sim::Pos { x: 34872, y: 37272 },
        ],
    },
    // row 28
    Ordering {
        first: Scripted {
            scout: 0,
            scout_to: sim::Pos { x: 12024, y: 12696 },
            capital: 2000,
            builder: 8,
            farm_at: sim::Pos { x: 9120, y: 10464 },
        },
        square: [
            sim::Pos { x: 12024, y: 15096 },
            sim::Pos { x: 9624, y: 15096 },
            sim::Pos { x: 9624, y: 12696 },
            sim::Pos { x: 12024, y: 12696 },
        ],
    },
    // row 48
    Ordering {
        first: Scripted {
            scout: 0,
            scout_to: sim::Pos { x: 34872, y: 34200 },
            capital: 2000,
            builder: 10,
            farm_at: sim::Pos { x: 37728, y: 34464 },
        },
        square: [
            sim::Pos { x: 34872, y: 31800 },
            sim::Pos { x: 37272, y: 31800 },
            sim::Pos { x: 37272, y: 34200 },
            sim::Pos { x: 34872, y: 34200 },
        ],
    },
    // row 68
    Ordering {
        first: Scripted {
            scout: 0,
            scout_to: sim::Pos { x: 11256, y: 10392 },
            capital: 2000,
            builder: 10,
            farm_at: sim::Pos { x: 9120, y: 10848 },
        },
        square: [
            sim::Pos { x: 11256, y: 12792 },
            sim::Pos { x: 8856, y: 12792 },
            sim::Pos { x: 8856, y: 10392 },
            sim::Pos { x: 11256, y: 10392 },
        ],
    },
    // row 88
    Ordering {
        first: Scripted {
            scout: 0,
            scout_to: sim::Pos { x: 22776, y: 41112 },
            capital: 2000,
            builder: 10,
            farm_at: sim::Pos { x: 19104, y: 39840 },
        },
        square: [
            sim::Pos { x: 22776, y: 38712 },
            sim::Pos { x: 20376, y: 38712 },
            sim::Pos { x: 20376, y: 41112 },
            sim::Pos { x: 22776, y: 41112 },
        ],
    },
    // row 108
    Ordering {
        first: Scripted {
            scout: 0,
            scout_to: sim::Pos { x: 10488, y: 29592 },
            capital: 2000,
            builder: 9,
            farm_at: sim::Pos { x: 5280, y: 30048 },
        },
        square: [
            sim::Pos { x: 10488, y: 27192 },
            sim::Pos { x: 8088, y: 27192 },
            sim::Pos { x: 8088, y: 29592 },
            sim::Pos { x: 10488, y: 29592 },
        ],
    },
    // row 128
    Ordering {
        first: Scripted {
            scout: 0,
            scout_to: sim::Pos { x: 43320, y: 41880 },
            capital: 2000,
            builder: 5,
            farm_at: sim::Pos { x: 42912, y: 41568 },
        },
        square: [
            sim::Pos { x: 43320, y: 39480 },
            sim::Pos { x: 45720, y: 39480 },
            sim::Pos { x: 45720, y: 41880 },
            sim::Pos { x: 43320, y: 41880 },
        ],
    },
    // row 148
    Ordering {
        first: Scripted {
            scout: 0,
            scout_to: sim::Pos { x: 8952, y: 31896 },
            capital: 2000,
            builder: 5,
            farm_at: sim::Pos { x: 7392, y: 30048 },
        },
        square: [
            sim::Pos { x: 8952, y: 29496 },
            sim::Pos { x: 6552, y: 29496 },
            sim::Pos { x: 6552, y: 31896 },
            sim::Pos { x: 8952, y: 31896 },
        ],
    },
];

/// **The seventh battery's scores**, `HELD_OUT_SEVENTH[k]` played as
/// `run1804 + k` (item 4600): `None` until the original has played it. The
/// found column is pinned once and never re-pinned; it is the estimate
/// (DECISIONS 154 §7). Rows 8 to 108 scored on `f7d2e41c6`'s tree; rows
/// 128 and 148, untaken there (the disk read 59 GB before run1810), were
/// played by item 4611 and scored on `6fa43acc3`'s.
#[cfg(test)]
pub(crate) const SCORES_SEVENTH: [Option<FirstScore>; 8] = [
    // run1804
    // row 8, as found 2026-11-23: 2 lines whole; units 9,896 of 12,001, first 9896; builds 9,922 of
    // 12,001, first 9922; ammo 11,146 of 12,001, first 9899; deaths 10,438 of 12,001, first 9929;
    // groups 9,947 of 12,001, first 9932; guys 9,898 of 12,001, first 9898; leaders 9,908 of
    // 12,001, first 9904; cities 10,091 of 12,001, first 9924; goods 11,576 of 12,001, first 11576;
    // world 9,900 of 12,001, first 9900; reach 9,896 of 12,001 states, 6,001 of 6,001 in the first
    // 6,001.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 12001,
            agreed: [
                9896, 9922, 12001, 11146, 10438, 9947, 9898, 9908, 10091, 12001, 11576, 9900,
            ],
            first: &[
                ("units", (9896, Some(171630208), 2211249348)),
                ("builds", (9922, Some(3512185841), 1350677998)),
                ("ammo", (9899, Some(1426297917), 1293259664)),
                ("deaths", (9929, Some(1508386744), 2245866602)),
                ("groups", (9932, Some(2627816952), 467422711)),
                ("guys", (9898, Some(2849304947), 1015869527)),
                ("leaders", (9904, Some(2155332895), 1047774493)),
                ("cities", (9924, Some(4210113495), 4140645333)),
                ("goods", (11576, Some(3252516634), 3292886812)),
                ("world", (9900, Some(4188360961), 2333888676)),
            ],
            over: None,
        },
        moved: None,
    }),
    // run1805
    // row 28, as found 2026-11-23: 2 lines whole; units 3,956 of 4,248, first 3956; builds 3,956 of
    // 4,248, first 3956; ammo 4,030 of 4,248, first 3963; deaths 4,021 of 4,248, first 4021; groups
    // 3,973 of 4,248, first 3973; guys 3,956 of 4,248, first 3956; leaders 3,958 of 4,248, first
    // 3958; cities 3,983 of 4,248, first 3983; goods 4,216 of 4,248, first 4011; world 3,956 of
    // 4,248, first 3956; reach 3,956 of 4,248 states, 3,956 of 4,248 in the first 6,001.
    Some(FirstScore {
        last: Some(4247),
        found: Score {
            states: 4248,
            agreed: [
                3956, 3956, 4248, 4030, 4021, 3973, 3956, 3958, 3983, 4248, 4216, 3956,
            ],
            first: &[
                ("units", (3956, Some(3894155877), 1909180687)),
                ("builds", (3956, Some(157090054), 3057844487)),
                ("ammo", (3963, Some(2790001298), 2854947519)),
                ("deaths", (4021, Some(3224243742), 3228634655)),
                ("groups", (3973, Some(4290454040), 1327899175)),
                ("guys", (3956, Some(617490755), 841753209)),
                ("leaders", (3958, Some(26406777), 35648267)),
                ("cities", (3983, Some(4199698130), 16469717)),
                ("goods", (4011, Some(2255352392), 2572546636)),
                ("world", (3956, Some(16483), 1319911691)),
            ],
            over: None,
        },
        // Moved by item 4619 (`Unit::target_opportunity`'s anti-siege shield
        // read whole: the five group-66 Bazookas yield their chase of `0/7`
        // to `1/25`'s `0/0` on 3955): 2 -> 7 lines whole (deaths, groups,
        // guys, goods and world join walls and items); lowest parting 3956
        // -> 4027 (ammo). Moved by item 4626 (piece 1849, the India
        // Bazooka, releases from its bay: who=2's `2/34` round of 4026 at
        // `0/2000` leaves (12208, 12465, 296), not its own square): 7 -> 9
        // lines whole (units and cities join); units 4,247 -> 4,248, first
        // 4122 -> whole; builds 4,219 -> 4,246, first 4037 -> 4179; ammo
        // 4,120 -> 4,184, first 4027 -> 4167; leaders 4,037 -> 4,246, first
        // 4037 -> 4179; cities 4,247 -> 4,248, first 4037 -> whole. Lowest
        // parting 4027 -> 4167 (ammo), 81 states from the original's end.
        // Moved by item 4630 (piece 1812, the India Infantry, releases from
        // its bay: who=2's `2/50` round of 4166 at `0/2000` leaves (12973,
        // 9025, 144), not its own square (13032, 9000, 24)): 9 -> 12 lines
        // whole, the row whole; builds 4,246 -> 4,248, first 4179 -> whole;
        // ammo 4,184 -> 4,248, first 4167 -> whole; leaders 4,246 -> 4,248,
        // first 4179 -> whole.
        moved: Some(Score {
            states: 4248,
            agreed: [
                4248, 4248, 4248, 4248, 4248, 4248, 4248, 4248, 4248, 4248, 4248, 4248,
            ],
            first: &[],
            over: None,
        }),
    }),
    // run1806
    // row 48, as found 2026-11-23: 1 line whole; units 7,150 of 12,001, first 5940; builds 7,170 of
    // 12,001, first 7170; ammo 8,363 of 12,001, first 5607; deaths 8,721 of 12,001, first 8721;
    // groups 7,186 of 12,001, first 7186; guys 7,182 of 12,001, first 6555; leaders 7,176 of
    // 12,001, first 7176; cities 7,183 of 12,001, first 7183; items 7,947 of 12,001, first 7947;
    // goods 7,542 of 12,001, first 7430; world 7,183 of 12,001, first 7183; reach 5,607 of 12,001
    // states, 5,607 of 6,001 in the first 6,001.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 12001,
            agreed: [
                7150, 7170, 12001, 8363, 8721, 7186, 7182, 7176, 7183, 7947, 7542, 7183,
            ],
            first: &[
                ("units", (5940, Some(506938227), 3241886403)),
                ("builds", (7170, Some(2387042259), 2315018235)),
                ("ammo", (5607, Some(2828101927), 3052169677)),
                ("deaths", (8721, Some(1), 2503022540)),
                ("groups", (7186, Some(621207000), 1310973398)),
                ("guys", (6555, Some(3089962720), 556209851)),
                ("leaders", (7176, Some(405918586), 1583338567)),
                ("cities", (7183, Some(2785309316), 2502718081)),
                ("items", (7947, Some(3978827278), 3964409356)),
                ("goods", (7430, Some(3608639283), 3559618353)),
                ("world", (7183, Some(2594571357), 1902774908)),
            ],
            over: None,
        },
        moved: None,
    }),
    // run1807
    // row 68, as found 2026-11-23: 12 lines whole to the original's end; 3,949 of 3,949 states.
    Some(FirstScore {
        last: Some(3948),
        found: Score {
            states: 3949,
            agreed: [
                3949, 3949, 3949, 3949, 3949, 3949, 3949, 3949, 3949, 3949, 3949, 3949,
            ],
            first: &[],
            over: None,
        },
        moved: None,
    }),
    // run1808
    // row 88, as found 2026-11-23: 1 line whole; units 2,741 of 12,001, first 2741; builds 2,783 of
    // 12,001, first 2783; ammo 2,988 of 12,001, first 2791; deaths 2,741 of 12,001, first 2741;
    // groups 2,751 of 12,001, first 2751; guys 2,741 of 12,001, first 2741; leaders 2,741 of
    // 12,001, first 2741; cities 2,951 of 12,001, first 2951; items 7,315 of 12,001, first 2906;
    // goods 2,879 of 12,001, first 2879; world 2,741 of 12,001, first 2741; reach 2,741 of 12,001
    // states, 2,741 of 6,001 in the first 6,001.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 12001,
            agreed: [
                2741, 2783, 12001, 2988, 2741, 2751, 2741, 2741, 2951, 7315, 2879, 2741,
            ],
            first: &[
                ("units", (2741, Some(3725650265), 852594544)),
                ("builds", (2783, Some(998530289), 3813760236)),
                ("ammo", (2791, Some(1), 2513898174)),
                ("deaths", (2741, Some(2957319091), 2550737442)),
                ("groups", (2751, Some(471656992), 1841031624)),
                ("guys", (2741, Some(4290938662), 1298234755)),
                ("leaders", (2741, Some(1601810301), 1853075446)),
                ("cities", (2951, Some(1452619764), 1435711476)),
                ("items", (2906, Some(3288077661), 3279426907)),
                ("goods", (2879, Some(2093806050), 2246636004)),
                ("world", (2741, Some(1926680119), 3718366906)),
            ],
            over: None,
        },
        // Moved by item 4614 (run1815's ranges). `kill_contents` brings each
        // rider out before its carrier's death (`Object::kill_contents
        // @00645550`): on 2740 who=2's Peltasts `2/38`..`2/40` come ashore off
        // the sunk barge `2/71` and die there, and the world's block 1101,
        // `block.present`, reads ours 0 -> 1, theirs 1. A fire raft dies with
        // its shot (`Unit::fight@005fd4d0:900`): who=1's Heavy Fire Raft `1/87`
        // takes deaths slot 0 on 2741, ours alive -> dead, theirs dead. Deaths
        // 2,741 -> 2,862, world 2,741 -> 2,862, builds 2,783 -> 2,887, ammo
        // 2,988 -> 3,039, groups 2,751 -> 2,883, guys 2,741 -> 2,851, items
        // 7,315 -> 7,799, goods 2,879 -> 2,998.
        // Moved by item 4622 (run1815's ranges): a self-destructing shooter
        // splashes from the shot's middle (`Ammo::do_damage@00678060`
        // `678652`..`6786b0`), so who=2's barge `2/61` beside the Heavy Fire
        // Raft `1/87`'s landing on 2740 takes, ours 12 -> 0, theirs 0. Its
        // `flags` on 2741 read ours 0x11 -> 0x01, theirs 0x01. Units 2,741 ->
        // 2,807, leaders 2,741 -> 2,879, guys 2,851 -> 2,882, deaths 2,862 ->
        // 2,882, world 2,862 -> 2,882, builds 2,887 -> 2,919, goods 2,998 ->
        // 2,999; ammo 3,039 -> 2,930 and items 7,799 -> 4,108 past the first
        // parting. The first parting moves 2741 -> 2791, on ammo alone.
        // Moved by item 4625 (run1818's ranges): the Carrack's release keys
        // (piece 292) take `BAYS` rows, so who=1's Carrack `1/82`'s round on
        // 2790 leaves its starboard battery. Ammo record 0 on 2791: `sx, sy,
        // sz` ours (16824, 11640, 0) -> (16866, 11729, 35), theirs (16866,
        // 11729, 35); `ex, ey` (16800, 13142) -> (16817, 13159), `total_time`
        // 18 -> 17, as theirs. Units 2,807 -> 2,902, builds 2,919 -> 2,953,
        // ammo 2,930 -> 3,056, deaths 2,882 -> 2,922, groups 2,883 -> 2,928,
        // guys 2,882 -> 2,907, leaders 2,879 -> 2,883, cities 2,951 -> 3,063,
        // items 4,108 -> 7,829, world 2,882 -> 2,909. The first parting moves
        // 2791 -> 2883, on leaders alone.
        // Moved by item 4631 (run1815's ranges): a transport's kill plunders
        // as its cargo (`Object::take_damage@00652020` `6530a5`..`6530ba`).
        // who=2's barge `2/61` dies on 2882 carrying the Despot `2/24`, and
        // who=1's Despot `1/14` is in reach: who=1's `encrypt` on 2883 reads
        // `bucket[1]` (timber) ours 15,500 -> 15,485, theirs 15,485, and
        // `bucket[2]` (wealth) ours 17,667 -> 17,767, theirs 17,767
        // (`GENERAL_PLUNDER` 100 over the Despot's 15). Leaders 2,883 ->
        // 2,926. The first parting moves 2883 -> 2902, on units alone.
        // Then item 4608 (`Guy::set_all_pivots` bears an `ATTACK_GROUND`
        // order's figures on the order's point, aimed or not): builds 7,170 -> 7,183 (first 7170 ->
        // 7183) and ammo 8,363 -> 8,372, past the row's first parting 5607 (ammo), which stands.
        // Then item 4608 (`Guy::set_all_pivots` bears an `ATTACK_GROUND`
        // order's figures on the order's point, aimed or not): ammo 3,056 -> 3,062 and items 7,829
        // -> 7,968, and ours' walk now ends on `game_over` 7465 where it played on; past the row's
        // first parting 2902, which stands.
        // Moved by item 4633 (run1815's ranges): `Unit::same_damage@005f9400`
        // walks the receiving squad from its tail. On 2901 who=2's barge `2/65`
        // (TI 320, 44 of 50) unloads the Elite Javelineers `2/29`..`2/31` (TI
        // 84), and the boat's 30/256 is 12 of 110: units range 3709, record 29
        // (`2/29`, the captain), `damage` ours 12 -> 0, theirs 0; record 31
        // (`2/31`, the tail) ours 0 -> 12, theirs 12. Units 2,902 -> 2,985,
        // builds 2,953 -> 3,015, ammo 3,062 -> 3,170, deaths 2,922 -> 2,986,
        // groups 2,928 -> 2,999, guys 2,907 -> 2,985, leaders 2,926 -> 3,011,
        // cities 3,063 -> 3,161, goods 2,999 -> 3,543, world 2,909 -> 2,988;
        // items 7,968 -> 7,838 (first 3201 -> 3126) past the parting. The
        // first parting moves 2902 -> 2985, on units, guys and leaders.
        // Moved by item 4662 (run1815's ranges): `Unit::work@0060d180`'s
        // attack uid test. On 2984 who=1's Trebuchet `1/61` (TI 266) ends its
        // attack-ground, and the Attack under it names `2/63` at uid 93, a
        // number another barge now holds: theirs marks that barge in danger,
        // writes `ox`/`whom` -1 and returns; `fight` searches on 2985. Units
        // range 2125, record 61 `order.target` ours (64, 2, 94) -> (-1, -1,
        // 93), theirs (-1, -1, 93); who=2's `2/63` `unit_masks` 8650826 ->
        // 8650830, theirs 8650830. Units 2,985 -> 3,033, builds 3,015 ->
        // 3,144, ammo 3,170 -> 3,391 (first 2994 -> 3028), deaths 2,986 ->
        // 3,463, groups 2,999 -> 3,095, guys 2,985 -> 3,069, leaders 3,011 ->
        // 3,134 (first 2985 -> 3075), cities 3,161 -> 3,313, items 7,838 ->
        // 8,590 (first 3126 -> 4135), goods 3,543 -> 3,500 (first 3114 ->
        // 3116), world 2,988 -> 3,086. The first parting moves 2985 -> 3028,
        // on ammo.
        moved: Some(Score {
            states: 12001,
            agreed: [
                3033, 3144, 12001, 3391, 3463, 3095, 3069, 3134, 3313, 8590, 3500, 3086,
            ],
            first: &[
                ("units", (3033, Some(3546334183), 3496526911)),
                ("builds", (3144, Some(3531851268), 1209124357)),
                ("ammo", (3028, Some(2577858340), 392952667)),
                ("deaths", (3463, Some(4004785731), 1478238545)),
                ("groups", (3095, Some(3354359777), 3950803518)),
                ("guys", (3069, Some(2438684783), 234578274)),
                ("leaders", (3075, Some(2998777053), 2372580574)),
                ("cities", (3313, Some(4151394957), 891306669)),
                ("items", (4135, Some(308941102), 314708274)),
                ("goods", (3116, Some(2794516976), 3169382901)),
                ("world", (3086, Some(183371053), 233702701)),
            ],
            over: None,
        }),
    }),
    // run1809
    // row 108, as found 2026-11-23: 2 lines whole; units 7,421 of 12,001, first 7421; builds 7,421
    // of 12,001, first 7421; ammo 11,543 of 12,001, first 8370; groups 7,421 of 12,001, first 7421;
    // guys 7,421 of 12,001, first 7421; leaders 7,551 of 12,001, first 7551; cities 7,583 of
    // 12,001, first 7583; items 8,392 of 12,001, first 7585; goods 9,009 of 12,001, first 8784;
    // world 7,422 of 12,001, first 7422; reach 7,421 of 12,001 states, 6,001 of 6,001 in the first
    // 6,001.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 12001,
            agreed: [
                7421, 7421, 12001, 11543, 12001, 7421, 7421, 7551, 7583, 8392, 9009, 7422,
            ],
            first: &[
                ("units", (7421, Some(3393241865), 1901678766)),
                ("builds", (7421, Some(331793807), 2606286225)),
                ("ammo", (8370, Some(1), 921766682)),
                ("groups", (7421, Some(2172338358), 1917123)),
                ("guys", (7421, Some(4128971196), 1422531110)),
                ("leaders", (7551, Some(1473370479), 814143512)),
                ("cities", (7583, Some(539043893), 624109624)),
                ("items", (7585, Some(1328428020), 1345729526)),
                ("goods", (8784, Some(3873929020), 3876812606)),
                ("world", (7422, Some(4189256067), 281868672)),
            ],
            over: None,
        },
        moved: None,
    }),
    // run1810
    // row 128, as found 2026-11-23 (item 4611, on `6fa43acc3`'s tree): 11 lines whole; leaders
    // 8,154 of 12,001, first 3338; reach 3,338 of 12,001 states, 3,338 of 6,001 in the first
    // 6,001.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 12001,
            agreed: [
                12001, 12001, 12001, 12001, 12001, 12001, 12001, 8154, 12001, 12001, 12001, 12001,
            ],
            first: &[("leaders", (3338, Some(2712824495), 2415749278))],
            over: None,
        },
        // Moved by item 4635 (`Leader::gain_tech`'s border arm: who=1, the
        // Russians, enters the Classical age on 3336 and theirs' fix
        // restarts every region's count; on 3338 `reg_terr[1]` ours 198,
        // 495, 447 against theirs 0, 104, 0): 11 -> 12 lines whole, the row
        // whole; leaders 8,154 -> 12,001, first 3338 -> whole.
        moved: Some(Score {
            states: 12001,
            agreed: [
                12001, 12001, 12001, 12001, 12001, 12001, 12001, 12001, 12001, 12001, 12001, 12001,
            ],
            first: &[],
            over: None,
        }),
    }),
    // run1811
    // row 148, as found 2026-11-23 (item 4611, on `6fa43acc3`'s tree): 1 line whole; units 10,879
    // of 12,001, first 10879; builds 10,930 of 12,001, first 10908; ammo 10,970 of 12,001, first
    // 10888; deaths 10,994 of 12,001, first 10994; groups 11,045 of 12,001, first 10933; guys
    // 10,909 of 12,001, first 10909; leaders 10,944 of 12,001, first 10908; cities 10,964 of
    // 12,001, first 10908; items 11,556 of 12,001, first 11459; goods 11,695 of 12,001, first
    // 11198; world 10,913 of 12,001, first 10912; reach 10,879 of 12,001 states, 6,001 of 6,001
    // in the first 6,001.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 12001,
            agreed: [
                10879, 10930, 12001, 10970, 10994, 11045, 10909, 10944, 10964, 11556, 11695, 10913,
            ],
            first: &[
                ("units", (10879, Some(3473404783), 2285826991)),
                ("builds", (10908, Some(3337499215), 3915461255)),
                ("ammo", (10888, Some(2756447483), 379062916)),
                ("deaths", (10994, Some(2387741884), 2392132797)),
                ("groups", (10933, Some(907293491), 3971560240)),
                ("guys", (10909, Some(1753099358), 2173644070)),
                ("leaders", (10908, Some(1527486154), 257660620)),
                ("cities", (10908, Some(3068282017), 1805141196)),
                ("items", (11459, Some(1489378100), 1460542256)),
                ("goods", (11198, Some(4110382930), 4015224652)),
                ("world", (10912, Some(304981228), 1792976066)),
            ],
            over: None,
        },
        // Item 4618 (piece 37's three bays, the Xopilli Atlatl's 143 rounds
        // from 10888): 1 -> 11 lines whole; units part first on 10879 still.
        moved: Some(Score {
            states: 12001,
            agreed: [
                10879, 12001, 12001, 12001, 12001, 12001, 12001, 12001, 12001, 12001, 12001, 12001,
            ],
            first: &[("units", (10879, Some(3473404783), 2285826991))],
            over: None,
        }),
    }),
];

/// The seventh battery's line: games whole of the eight, and games taken.
/// As found: row 68 whole to the original's end 3,948, six taken.
/// Item 4630 made row 28 whole to the original's end 4,247 (piece 1812's
/// bays): 1 -> 2. Item 4611 took rows 128 and 148 (run1810, run1811):
/// six taken -> eight. Item 4635 made row 128 whole to 12,000 (the
/// Russian age's border fix): 2 -> 3.
#[cfg(test)]
pub(crate) const HELD_OUT_SEVENTH_LINE: (usize, usize) = (3, 8);

/// The seventh battery (item 4600): the sixth's recipe, the human ordering
/// every 600 frames to 11,400.
#[cfg(test)]
pub(crate) const SEVENTH: Battery = Battery {
    label: "Parity held out, seventh",
    item: 4600,
    run: 1804,
    table: &HELD_OUT_SEVENTH,
    scores: &SCORES_SEVENTH,
    scores_name: "SCORES_SEVENTH",
    line: HELD_OUT_SEVENTH_LINE,
    frames: SEVENTH_FRAMES,
};
