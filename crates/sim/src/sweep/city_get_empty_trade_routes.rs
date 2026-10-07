//! `CityData::get_empty_trade_routes@007395c0` against the port.

use crate::caravan::Caravan;
use crate::city::City;
use crate::world::{Cell, Terrain, World};
use crate::{Pos, Sim, Tuning, Unit};

fn city(at: Pos) -> City {
    City {
        alive: true,
        owner: 0,
        race: Some(0),
        founder: 0,
        building: 0,
        members: Vec::new(),
        reg: None,
        pos: at,
        capital: false,
        founding_capital: false,
        was_founding_capital: false,
        unassimilated: false,
        no_heal: false,
        attacking: false,
        ever_attacked: false,
        alarm: false,
        no_muster: false,
        was_capital: 0,
        capture_stamp: 0,
        assimilation_timer: 0,
        attack_stamp: 0,
        reduce_stamp: 0,
        capture_strength: 0,
        pop: 0,
        has_citizen: false,
        source: None,
        trade_val: 0,
        traded_with: [0; 8],
    }
}

/// One van of a row: `<list> <slot> <flags> <p0> <p1> <p2> <p3>`.
struct Van {
    list: usize,
    slot: usize,
    flags: u8,
    p: [i64; 4],
}

/// The port's answer to "is the pair of cities 0 and 1 still free": two
/// allied, seen cities of player 0 and one idle caravan, with exactly the
/// row's vans put in `caravans`. The port keeps the test inline in
/// `trade_pair_free`/`trade_route_exists` (private, and answering for the
/// whole `Sim` rather than one city's `vans`), so this drives it through
/// `Sim::think_caravan`, whose `find_trade_city` is `true` exactly when
/// some pair of cities is free — and with two cities there is only the one.
/// `idle` is the wait threshold the caller found by the baseline row.
fn port_free(vans: &[Van], idle: u8) -> bool {
    let mut world = World::new(8, 8);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(7, 7));
    let mut s = Sim::new(Tuning::RON, world, 2);
    s.cities.push(city(Pos::new(2 * 768, 2 * 768)));
    s.cities.push(city(Pos::new(5 * 768, 5 * 768)));
    let u = {
        let mut u = Unit::new(0, 0, Pos::new(3 * 768, 3 * 768), 20);
        u.idle = idle;
        s.add_unit(u)
    };
    let to = |p: i64| usize::try_from(p).ok();
    for v in vans {
        let list = &mut s.caravans[v.list];
        list.mark = list.mark.max(v.slot + 1);
        // The port keeps one city number per end (`city2`, `city3`) and
        // drops the owner shorts (`whom`, `whose`).
        list.slots[v.slot] = Caravan {
            alive: v.flags & 1 != 0,
            linked: v.flags & 2 != 0,
            delivered: v.flags & 4 != 0,
            city_a: to(v.p[0]),
            city_b: to(v.p[2]),
            ..Caravan::default()
        };
    }
    s.think_caravan(u)
}

/// Rows where the original and the port part, by cause, with the original's
/// answer and the port's (1 = the pair is free, 0 = taken). Each is asserted
/// below to be in the rows and to still part.
const PARTED: &[(&str, i64, i64, &str)] = &[
    (
        "1 0 0 1 0 0 3 0 0 1 9 -> 1",
        1,
        0,
        "the original matches `(city, owner)` as a pair of shorts; the port keeps the city and drops the owner, so a van whose far end is (1, owner 9) is the pair (0, 1) to the port only",
    ),
    (
        "1 0 0 1 0 0 2 0 0 1 0 -> 0",
        0,
        1,
        "the original tests only `caravan_flags & 2`; the port also needs bit 0 (`alive`) — a slot still listed in `vans` with bit 0 clear is not a state a game reaches (`close_caravan` clears the record and the list entry)",
    ),
];

/// `CityData::get_empty_trade_routes@007395c0`, run under unicorn by
/// `tools/emu/sweep_city_get_empty_trade_routes.py` (nothing stood in for:
/// the function calls nothing; the `vans` list, the `caravans` global and
/// the records are laid out by hand), against the port's pair test
/// (`caravan.rs` `trade_pair_free`/`trade_route_exists`, private, and driven
/// through `Sim::think_caravan` on two cities of one player, see
/// [`port_free`]). The port's question is `result != 0`, which is what
/// `do_trade@005ed270` asks of both calls; the original answers
/// `1 − count`, so a pair listed twice is `-1` there and "taken" here.
///
/// Compared: rows of the game's shape — arguments `(1 − t, 0)` (a city and
/// its owner), every van of the city a flagged route between cities 0 and 1
/// with owner 0 whose flag byte has bit 0 whenever it has bit 2, one of its
/// ends being the city itself. Rows off that shape that the port can still
/// be asked (the cities 0 and 1, the arguments `(1 − t, 0)`) are tallied
/// and the partings among them are the [`PARTED`] causes; the rest (other
/// cities, other arguments, which the port cannot express) are only
/// counted.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_get_empty_trade_routes.py") else {
        return;
    };
    // The idle threshold of a caravan the port will think about: the one
    // wait at which a pair-free two-city world answers true.
    let idle = [1, 2, 7, 12, 17, 32, 62]
        .into_iter()
        .find(|&i| port_free(&[], i))
        .expect("an idle count at which the baseline caravan thinks");
    let (mut agreed, mut in_domain, mut off_shape, mut off_parted, mut skipped) = (0, 0, 0, 0, 0);
    let mut count2 = 0;
    for line in rows.lines() {
        let (a, original) = super::row(line);
        let (arg_a, arg_b, t, n) = (a[0], a[1], a[2], usize::try_from(a[3]).unwrap());
        assert_eq!(a.len(), 4 + 7 * n, "row shape of `{line}`");
        let vans: Vec<Van> = (0..n)
            .map(|i| {
                let v = &a[4 + 7 * i..11 + 7 * i];
                Van {
                    list: usize::try_from(v[0]).unwrap(),
                    slot: usize::try_from(v[1]).unwrap(),
                    flags: u8::try_from(v[2]).unwrap(),
                    p: [v[3], v[4], v[5], v[6]],
                }
            })
            .collect();
        // The original's own answer, derived from the row, so a script
        // that stopped computing it would be caught here.
        let count = vans
            .iter()
            .filter(|v| {
                v.flags & 2 != 0
                    && ((v.p[0] == arg_a && v.p[1] == arg_b)
                        || (v.p[2] == arg_a && v.p[3] == arg_b))
            })
            .count();
        assert_eq!(original, 1 - i64::try_from(count).unwrap(), "`{line}`");

        let expressible = arg_a == 1 - t
            && arg_b == 0
            && vans
                .iter()
                .all(|v| matches!(v.p[0], 0 | 1) && matches!(v.p[2], 0 | 1));
        if !expressible {
            skipped += 1;
            continue;
        }
        let game_shape = vans.iter().all(|v| {
            (v.flags & 2 == 0 || v.flags & 1 != 0)
                && v.p[1] == 0
                && v.p[3] == 0
                && (v.p[0] == t || v.p[2] == t)
        });
        let free = port_free(&vans, idle);
        let line_part = PARTED.iter().find(|p| p.0 == line);
        if !game_shape {
            off_shape += 1;
            if free != (original != 0) {
                off_parted += 1;
            }
            if let Some(p) = line_part {
                assert_eq!((original, i64::from(free)), (p.1, p.2), "{}", p.3);
            }
            continue;
        }
        in_domain += 1;
        if original < 0 {
            // Two listed vans on one pair: the original's callers read the
            // `-1` as free, the port as taken. Not a state the pairing rule
            // lets a game reach.
            count2 += 1;
            assert!(!free, "`{line}`");
            continue;
        }
        assert_eq!(free, original != 0, "port on row `{line}`");
        agreed += 1;
    }
    for (line, o, p, cause) in PARTED {
        assert!(rows.lines().any(|l| l == *line), "{line} is not a row");
        eprintln!("PARTED `{line}`: original {o}, port {p}: {cause}");
    }
    eprintln!(
        "{agreed} rows agree ({in_domain} in the game's shape, {count2} of those a pair listed twice, parted by design); {off_shape} off-shape rows the port can express, {off_parted} of them part; {skipped} the port cannot express"
    );
    assert!(agreed >= 200, "only {agreed} rows");
    assert!(off_parted >= 2, "the recorded partings no longer part");
}
