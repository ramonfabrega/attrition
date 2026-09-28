//! **The second pair** (DECISIONS 53 §2, item 971): run54's and run53's games
//! with the lobby's difficulty at its top setting, Toughest, and nothing else
//! moved — run346 on East Indies and run347 on Great Lakes. The words are
//! [`SECOND_WORD_EAST_INDIES`] and [`SECOND_WORD_GREAT_LAKES`]; the first
//! pair's `LONG_WORD_*` stay as its closed floor. `docs/AI.md` §80 has the
//! lever, the arms as hypotheses, and what each word turned out to be.

use super::*;

use crate::diff::harness::{attributed_sites, debug_leader, site_window};
use crate::diff::testkit::*;
use crate::testenv::{dump, install};

/// A long capture of the second pair walked from its own start dump: the
/// frame the draw count first parts, the frame the label sequence first
/// parts, and the trace's last frame. `None` when the capture is not on
/// this machine.
pub(crate) struct Walk {
    pub count: i64,
    pub sequence: i64,
    pub last: i64,
    pub difficulty: i32,
}

/// The two maps' setup differs in one borrow each, as run54's and run53's
/// own tests do: East Indies takes run38's start dump and the pasture from
/// its own trace, Great Lakes the sibling dumps.
pub(crate) fn walk_second(gamelog: &str, tracelog: &str, east_indies: bool) -> Option<Walk> {
    let inst = install()?;
    let (Some(path), Some(trace)) = (dump(gamelog), trace(tracelog)) else {
        eprintln!("skipping: no {gamelog} (set RON_GAMELOG_DIR)");
        return None;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    let mut init = log.initial().unwrap();
    let sib_texts: Vec<String> = if east_indies {
        dump("gamelog-run38-islands-start.txt")
            .map(crate::capture::read)
            .into_iter()
            .collect()
    } else {
        sibling_texts()
    };
    let sib_logs: Vec<Log> = sib_texts.iter().map(|t| Log::parse(t)).collect();
    let sib_inits: Vec<Initial> = sib_logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = sib_inits.iter().collect();
    borrow_from_siblings(&mut init, &refs);
    if east_indies {
        borrow_pasture(&mut init, &trace);
    }
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    built.sim.trace_phases = true;
    let difficulty = built.sim.lobby.difficulty;
    let last = trace.frames.last().map_or(0, |(n, _)| *n);
    let window = site_window();
    for f in 0..last {
        built.tick();
        debug_leader(&built, f);
        if window.is_some_and(|(lo, hi)| (lo..=hi).contains(&f)) {
            for (label, who) in attributed_sites(&built) {
                eprintln!("  f{f} {who}: {label}");
            }
            let theirs = trace.labels(f);
            if theirs != built.frame_sites.last().map_or(&[][..], |(_, v)| v) {
                eprintln!("  f{f} PARTS: theirs {theirs:?}");
            }
        }
    }
    let sequence = built
        .frame_sites
        .iter()
        .find(|(f, ours)| *ours != trace.labels(*f))
        .map_or(last, |(f, _)| *f);
    let count = built
        .frame_sites
        .iter()
        .find(|(f, ours)| ours.len() != trace.labels(*f).len())
        .map_or(last, |(f, _)| *f);
    eprintln!("{gamelog}: word parts at {count}, sequence at {sequence} of {last}");
    for (f, ours) in built.frame_sites.iter() {
        if *f != count && *f != sequence {
            continue;
        }
        let theirs = trace.labels(*f);
        let at = (0..ours.len().max(theirs.len()))
            .find(|&i| ours.get(i) != theirs.get(i))
            .unwrap_or(0);
        eprintln!(
            "  frame {f}: ours {} theirs {} — at {at}, ours {:?} theirs {:?}",
            ours.len(),
            theirs.len(),
            ours.get(at),
            theirs.get(at)
        );
        if std::env::var("RON_DEBUG_SITES").is_ok() {
            for (i, l) in ours.iter().enumerate() {
                eprintln!("    ours  {i}: {l}");
            }
            for (i, l) in theirs.iter().enumerate() {
                eprintln!("    thrs  {i}: {l}");
            }
        }
    }
    Some(Walk {
        count,
        sequence,
        last,
        difficulty,
    })
}

/// **The compared pin's window** (item 1061, DECISIONS 54 §2): the newest
/// pair's lower map's word — Great Lakes at Toughest, [`SECOND_WORD_GREAT_LAKES`]
/// — its block and two on either side, on run373 walked from run347's
/// start, with the group record and the attack order's row
/// ([`widen_records`]). `coverage`'s compared pin walks these blocks with
/// the recorder on. Until item 1061 it walked the first pair's Great Lakes
/// window, closed since item 899, where no army marched. `None` when the
/// captures are not on this machine.
pub(crate) fn great_lakes_word_window() -> Option<crate::diff::harness::tests::Widened> {
    let block = SECOND_WORD_GREAT_LAKES + 1;
    crate::diff::harness::tests::widen_great_lakes_on(
        (
            "gamelog-run347-greatlakes-toughest-24k-trace.txt",
            "rontrace-run347.log",
        ),
        "the second pair's word's window",
        &[(
            "gamelog-run373-greatlakes-toughest-4846.txt",
            WIDENING_SECOND_GREAT_LAKES_4846.0,
        )],
        (block - 2, block + 2),
        1,
        &[block],
        true,
    )
}

/// **The group record and the attack order's row, both directions** (item
/// 1061, DECISIONS 54 §2; parked 1062): what the second pair's widening
/// did not compare while its word stood on a group's order. On one block,
/// for every player's 64 pool slots that either side holds, the whole
/// `GROUPDATA` record — the list, `num`, `army`, `form`, `order_num`,
/// `ox`/`oy`, `o_dist`, `o_angle`, `facing`, `form_num`, `speed`,
/// `new_speed`, `stamp`, `buildings`, `disband`, `priority`, `role` and each
/// slot's `off`, `curr` and `angle` — keyed `(who, -3, "group:<id>.<field>")`;
/// and on every unit, each order slot both sides hold as an `ATTACKORDER`,
/// its own row past the target — `mandatory`, `defensive`, `in_range`,
/// `ever_in_range`, `new_ord`, `def_x`, `def_y` — keyed
/// `(who, o, "attack[<slot>].<field>")`; and each figure's aim, `ox` and
/// `whom`, keyed `(who, o, "g.<field>[<k>]")`. A slot one side holds alone
/// is a `held` row. Answers the rows compared.
///
/// `disband` and `priority` are compared against 0: this crate writes
/// neither, and a player-1 pool slot the original sets either on is the
/// finding. `role` is compared on an army's group only, where this crate
/// keeps the word (`Army::role`, `docs/ARMY.md` §3.1); a pushed group
/// carries none here. `think_frame` this crate does not carry.
pub(crate) fn widen_records(
    built: &Built,
    frame: &Frame,
    block: crate::gamelog::Block<'_>,
    players: usize,
    n: i64,
    here: &mut std::collections::BTreeMap<(i64, i64, String), (i64, String)>,
) -> usize {
    let mut rows = 0usize;
    let pool = crate::gamelog::groups(block);
    for who in 0..players as i64 {
        let Ok(w) = sim::Player::try_from(who) else {
            continue;
        };
        for slot in 0..64u8 {
            let id = who * 64 + i64::from(slot);
            let theirs = pool.iter().find(|g| g.id == id).filter(|g| g.num > 0);
            let army = built
                .sim
                .armies
                .get(who as usize)
                .into_iter()
                .flat_map(|x| x.list.iter())
                .find(|a| a.valid && a.group.pool == Some(slot));
            let mut list = built.sim.pool_list(w, slot);
            let mut ours = built.sim.pool_state(w, slot);
            let mut buildings = false;
            if list.is_empty()
                && ours.is_none()
                && let Some((st, b)) = built.sim.pool_building_group(w, slot)
            {
                list = b;
                ours = Some(st);
                buildings = true;
            }
            if theirs.is_none() && list.is_empty() {
                continue;
            }
            let mut row = |key: &str, o: String, t: String| {
                rows += 1;
                if o != t {
                    here.entry((who, -3, format!("group:{id}.{key}")))
                        .or_insert((n, format!("ours {o} theirs {t}")));
                }
            };
            let (Some(t), Some(o)) = (theirs, ours) else {
                row(
                    "held",
                    format!("{list:?}"),
                    format!(
                        "{:?}",
                        theirs.map(|t| t.members.iter().map(|m| m.o).collect::<Vec<_>>())
                    ),
                );
                continue;
            };
            let tl: Vec<i64> = t.members.iter().map(|m| m.o).take(t.num as usize).collect();
            let ol: Vec<i64> = list.iter().map(|&x| i64::from(x)).collect();
            compared::note(
                "GroupDump",
                &[
                    "num",
                    "army",
                    "form",
                    "order_num",
                    "ox",
                    "oy",
                    "o_dist",
                    "o_angle",
                    "facing",
                    "form_num",
                    "speed",
                    "new_speed",
                    "stamp",
                    "buildings",
                    "disband",
                    "priority",
                ],
            );
            row("list", format!("{ol:?}"), format!("{tl:?}"));
            for (k, ov, tv) in [
                ("num", ol.len() as i64, t.num),
                ("army", army.map_or(-1, |a| i64::from(a.army)), t.army),
                ("form", i64::from(o.form), t.form),
                ("order_num", i64::from(o.order_num), t.order_num),
                ("ox", i64::from(o.o.x), t.ox),
                ("oy", i64::from(o.o.y), t.oy),
                ("o_dist", i64::from(o.o_dist), t.o_dist),
                ("o_angle", i64::from(o.o_angle.0), t.o_angle),
                ("facing", i64::from(o.facing), t.facing),
                ("form_num", i64::from(o.form_num), t.form_num),
                ("speed", i64::from(o.speed), t.speed),
                ("new_speed", i64::from(o.new_speed), t.new_speed),
                ("stamp", o.stamp, t.stamp),
                ("buildings", i64::from(buildings), t.buildings),
                ("disband", 0, t.disband),
                ("priority", 0, t.priority),
            ] {
                row(k, ov.to_string(), tv.to_string());
            }
            if let Some(a) = army {
                compared::note("GroupDump", &["role"]);
                row("role", a.role.to_string(), t.role.to_string());
            }
            let slots = (o.form_num.max(0) as usize).max(t.form_num.max(0) as usize);
            if slots > 0 {
                compared::note(
                    "GroupMemberDump",
                    &["off_x", "off_y", "curr_x", "curr_y", "angle"],
                );
            }
            for i in 0..slots {
                let tm = t.members.get(i);
                row(
                    &format!("off[{i}]"),
                    format!("{:?}", o.off.get(i)),
                    format!("{:?}", tm.map(|m| (m.off_x, m.off_y))),
                );
                row(
                    &format!("curr[{i}]"),
                    format!("{:?}", o.curr.get(i).map(|p| (p.x, p.y))),
                    format!("{:?}", tm.map(|m| (m.curr_x, m.curr_y))),
                );
                row(
                    &format!("angle[{i}]"),
                    format!("{:?}", o.angles.get(i)),
                    format!("{:?}", tm.map(|m| m.angle)),
                );
            }
        }
    }
    for them in &frame.units {
        if !(0..players as i64).contains(&them.who) {
            continue;
        }
        let (Ok(who), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
            continue;
        };
        let Some(u) = built.sim.unit_by_o(who, o) else {
            continue;
        };
        let un = &built.sim.units[u];
        // **What each figure is aimed at** (`GuyData +0x8e`/`+0x9f`,
        // printed `ox`/`whom`): chapter one's widening read it on a
        // player's fight, and a war window is where it is written.
        for (k, g) in them.guys.iter().enumerate() {
            let Some(og) = un.guys.get(k) else { continue };
            let aim = match og.aim {
                None => (-1, -1),
                Some(sim::combat::Obj::Unit(t)) => {
                    let tu = &built.sim.units[t];
                    (i64::from(tu.owner), i64::from(tu.index))
                }
                // A building by its `(owner, index)`, the identity
                // `widen_block` links on: `build_ids` names only the ones
                // the start dump placed.
                Some(sim::combat::Obj::Building(b)) => {
                    let x = &built.sim.buildings[b];
                    (i64::from(x.owner), i64::from(x.index))
                }
            };
            for (name, mine, dumped) in [("whom", aim.0, g.whom), ("ox", aim.1, g.ox)] {
                let Some(dumped) = dumped else { continue };
                compared::note("Guy", &[name]);
                rows += 1;
                if mine != dumped {
                    here.entry((them.who, them.o, format!("g.{name}[{k}]")))
                        .or_insert((n, format!("ours {mine} theirs {dumped}")));
                }
            }
        }
        let mut first = true;
        for (slot, (ours, od)) in un.orders.iter().zip(them.orders_front_first()).enumerate() {
            let sim::orders::Body::Attack(a) = ours.body else {
                continue;
            };
            if od.index != i64::from(sim::orders::index::ATTACK) {
                continue;
            }
            let def = a.def.unwrap_or(sim::Pos::new(-1, -1));
            let mut fields = vec![
                ("defensive", i64::from(a.defensive), od.defensive),
                ("in_range", i64::from(a.in_range), od.in_range),
                (
                    "ever_in_range",
                    i64::from(a.ever_in_range),
                    od.ever_in_range,
                ),
                ("new_ord", i64::from(a.new_ord), od.new_ord),
                ("def_x", i64::from(def.x), od.def_x),
                ("def_y", i64::from(def.y), od.def_y),
            ];
            // `mandatory` lives on the unit (`combat::State`), so it is
            // the front-most attack's.
            if first {
                fields.push(("mandatory", i64::from(un.combat.mandatory), od.mandatory));
                first = false;
            }
            for (name, mine, dumped) in fields {
                let Some(dumped) = dumped else { continue };
                compared::note("OrderDump", &[name]);
                rows += 1;
                if mine != dumped {
                    here.entry((them.who, them.o, format!("attack[{slot}].{name}")))
                        .or_insert((n, format!("ours {mine} theirs {dumped}")));
                }
            }
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::harness::tests::widen_east_indies_on;

    /// **The second pair's East Indies word, widened whole** (item 971):
    /// run349 is run346's game at run299's per-frame detail over blocks
    /// 1..250 (`rngcmp.py`: 263 frames, 0 differing), walked from run346's
    /// start by [`widen_east_indies_on`] — every record, every unit, both
    /// leaders, both directions — with the word's block (frame 0 writes
    /// block 1) and the two after it kept standing. It keeps the move's
    /// value diff; East Indies' next word, 10, is inside its window.
    #[test]
    fn run349_s_word_frame_is_widened_whole() {
        use std::collections::BTreeMap;
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run349",
            "gamelog-run349-islands-toughest-open.txt",
            WIDENING_SECOND_EAST_INDIES,
            &[2],
            true,
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        assert_eq!(w.blocks, 250, "run349 whole: blocks 1..250");
        assert!(
            w.missing.is_empty(),
            "run349 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **item 971's value diff**, `docs/AI.md` §80.5: before
        // `think_spellcaster`'s coin the AI scout `1/0`'s walk target on
        // block 1 was 41976/36600 here against 35832/42744, and 28 of its
        // rows parted on blocks 1..3. **Item 979's** (§81): with run38's
        // frame words — an Easiest game's — no longer installed over this
        // one, the scout's walk parting (`orders_x` 38136 against 41976 on
        // block 97) closed too; its first row now is its sight count on
        // block 202.
        assert_eq!(row(1, 0, "orders_x"), None, "the scout's walk agrees");
        assert_eq!(
            row(1, 0, "mylos").as_deref(),
            Some("202: ours 6 theirs 4"),
            "the scout's first parting"
        );
        assert!(
            !w.firsts
                .iter()
                .any(|((who, o, _), (f, _))| (*who, *o) == (1, 0) && *f < 202),
            "none of the scout's rows parts before block 202"
        );
        // **And run38's clocks**: block 1's 46 gaia `cur_anim` rows and
        // `1/1`'s `g.end_time[0]` 232 against 33 were the sibling's
        // figures, installed at frame 0's end; both closed with the words.
        assert_eq!(row(1, 1, "g.end_time[0]"), None, "an AI citizen's clock");
        assert!(
            !w.firsts
                .iter()
                .any(|((who, _, what), _)| *who == 8 && what == "gaia:cur_anim"),
            "no gaia animal's clock parts"
        );
        // **What stands**: block 1's 67 keys (player 0's census,
        // `SITE.reg`, `form`, the city record — `docs/AI.md` §33.4's
        // families) and block 2's tech stamps, `tech_frame` and two
        // `tech_cat_frame`s, which the original's `Build::queue_up` writes
        // and this crate does not (§33.4's named unmodelled state).
        assert_eq!(
            row(1, -1, "leader:tech_frame").as_deref(),
            Some("2: ours 0 theirs 1"),
            "the original's tech stamp, standing"
        );
        assert_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(1, 67), (2, 3), (184, 1)],
            "the blocks keys first part on, the first three"
        );
    }

    /// **The second pair's Great Lakes word, widened whole** (item 971):
    /// run350 is run347's game at run349's detail over blocks 1..250, walked
    /// from run347's start by [`widen_great_lakes_on`]. The word is frame 1,
    /// which writes block 2.
    #[test]
    fn run350_s_word_frame_is_widened_whole() {
        use std::collections::BTreeMap;
        let Some(w) = crate::diff::harness::tests::widen_great_lakes_on(
            (
                "gamelog-run347-greatlakes-toughest-24k-trace.txt",
                "rontrace-run347.log",
            ),
            "run350",
            &[("gamelog-run350-greatlakes-toughest-open.txt", 1)],
            WIDENING_SECOND_GREAT_LAKES,
            1,
            &[2],
            false,
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        assert_eq!(w.blocks, 250, "run350 whole: blocks 1..250");
        assert!(
            w.missing.is_empty(),
            "run350 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **Block 1 stands** with frame 0's draws agreeing: 55 keys, the
        // first pair's families (player 0's census, `SITE.reg`, `form`, the
        // city record; `docs/AI.md` §33.4) and nothing of who=1's economy.
        // **Item 979's value diff** (§81): before the lobby gate, run12's
        // frame-0 word — the Easiest game's, one draw short — was installed
        // over this game's, frame 1's rush roll read 9 where the trace's
        // seeds give 3, and who=1's block 2 parted on 130 keys: `script_step`
        // 11 here against 6, timber 28 against 92, wealth 50 against 100,
        // `gatherers` 4 against 3, `1/2001`'s 90 gather slots, citizen
        // `1/2`'s walk and the rush's second farm `2007`. Every one of them
        // agrees now.
        for what in [
            "leader:script_step",
            "leader:bucket[1:timber]",
            "leader:bucket[2:wealth]",
            "leader:gatherers",
        ] {
            assert_eq!(row(1, -1, what), None, "who=1's {what} agrees");
        }
        assert!(
            !w.firsts.iter().any(|((who, o, _), (f, _))| *who == 1
                && *f >= 2
                && [2, 2001, 2005, 2006, 2007].contains(o)),
            "the citizen, the camp's slots and the rush's sites agree past block 1"
        );
        // **What stands on the old word's block**: the original's tech
        // stamps (`Build::queue_up` writes `tech_frame` and
        // `tech_cat_frame[cat]`; §33.4's named unmodelled state) and two
        // pools of the script's buildings, the library `2005` in 64 and the
        // city `2000` in 66, which this crate never seats. No draw reads
        // either before the word.
        assert_eq!(
            row(1, -1, "leader:tech_frame").as_deref(),
            Some("2: ours 0 theirs 1"),
            "the Art of War's stamp"
        );
        assert_eq!(
            row(1, -2, "pool:64").as_deref(),
            Some("2: ours [] theirs [2005]"),
            "the original's pool 64"
        );
        assert_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(1, 55), (2, 4), (8, 1)],
            "the blocks keys first part on, the first three"
        );
    }

    /// **The second pair's East Indies word, 1576, widened whole** (item
    /// 979): run352 is run346's game at run349's detail over blocks
    /// 1571..1827 (`rngcmp.py`: 1841 frames, 0 differing), walked from
    /// run346's own start. The word's frame writes block 1577.
    #[test]
    fn run352_s_word_frame_is_widened_whole() {
        use std::collections::BTreeMap;
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run352",
            "gamelog-run352-islands-toughest-1576.txt",
            WIDENING_SECOND_EAST_INDIES_1576,
            &[1_577],
            true,
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        assert_eq!(w.blocks, 257, "run352 whole: blocks 1571..1827");
        assert!(
            w.missing.is_empty(),
            "run352 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The word's block, 1577, as item 979 found it** (`docs/AI.md`
        // §81.4): the building `1/2009` parted on 91 of its 93 keys — its
        // gather tiles in another order, damage 1 here against 0, city −1
        // against 1 — and the citizen `1/2` walked to another cell
        // (`orders_x` 31944 against 37752). **Item 989's value diff**
        // (§82): the script's `place_woodcutter` places a camp at (38016,
        // 36480) on frame 976 and destroys it the same frame, and
        // `Build::close` gives its 48 tiles back; this crate kept them
        // marked, so the camp re-placed on 1576 went to (31680, 34752) with
        // 62 tiles (248 draws against 192). And the harness linked the
        // original's live `2009` to this crate's dead frame-976 site, which
        // is where damage 1 and city −1 came from. Every row of the word's
        // block now agrees, and so does capital `2000`'s `ter[1]` on 1571,
        // 0 against 2 → 2 on both: the census's `gather_at` had read the
        // dead camp's marks.
        assert!(
            !w.firsts.iter().any(|((who, o, _), (f, _))| *who == 1
                && [2, 2009].contains(o)
                && (1_572..=1_577).contains(f)),
            "the camp and the citizen agree from 1572 through the word's block"
        );
        assert_eq!(row(1, 2000, "city:ter[1]"), None, "the capital's timber");
        // **What stands, drawing nothing before the next word** (5606):
        // city `2007`'s site picture, one cell apart from its sweep of 1575
        // — `filled` one high here and the three `space` counts one low, so
        // `reg_land[11]` 102 against 103 — and, from the sweep before
        // (block 1571), the same city counted by the original while still a
        // site (`land` 9, `reg_cities[11]` 2) and not here. No mechanism is
        // named (DECISIONS 42).
        assert_eq!(
            row(1, -1, "leader:reg_land[11]").as_deref(),
            Some("1576: ours 102 theirs 103"),
            "the block before the word"
        );
        assert_eq!(
            row(1, 2007, "city:space[0]").as_deref(),
            Some("1576: ours 64 theirs 65"),
            "the city's site picture"
        );
        assert_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(1571, 82), (1576, 5), (1601, 1)],
            "the blocks keys first part on, the first three"
        );
    }

    /// **The second pair's Great Lakes word, 3776, widened whole** (item
    /// 979): run355 is run347's game at run350's detail over blocks
    /// 3771..4027 (`rngcmp.py`: 4041 frames, 0 differing), walked from
    /// run347's start. The word's frame writes block 3777.
    #[test]
    fn run355_s_word_frame_is_widened_whole() {
        use std::collections::BTreeMap;
        let Some(w) = crate::diff::harness::tests::widen_great_lakes_on(
            (
                "gamelog-run347-greatlakes-toughest-24k-trace.txt",
                "rontrace-run347.log",
            ),
            "run355",
            &[(
                "gamelog-run355-greatlakes-toughest-3776.txt",
                WIDENING_SECOND_GREAT_LAKES_3776.0,
            )],
            WIDENING_SECOND_GREAT_LAKES_3776,
            1,
            &[3_777],
            false,
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        assert_eq!(w.blocks, 257, "run355 whole: blocks 3771..4027");
        assert!(
            w.missing.is_empty(),
            "run355 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The word's block, 3777, as item 979 found it** (`docs/AI.md`
        // §81.4), East Indies' shape: a building who=1 places on the word's
        // frame, `1/2010` — 79 of its 80 keys its gather slots' tiles, and
        // damage 1 here against 0 — with `2007`'s `city_down` pointing at
        // it here and not there, the city `2000` one gatherer short (7
        // against 8), and citizens `1/6` and `1/28` on other orders.
        // **Item 989's value diff** (§82): the same close tail — a camp the
        // script destroyed kept its ground marked here — and the same dead
        // site linked in place of the live `2010`. All 127 keys of the
        // word's block agree now, and nothing parts on 3772..3804.
        assert!(
            !w.firsts
                .iter()
                .any(|((who, _, _), (f, _))| *who == 1 && (3772..3805).contains(f)),
            "who=1 agrees from the window's second block through 3804"
        );
        assert_eq!(row(1, 2000, "city:gatherers"), None, "the city's census");
        assert_eq!(row(1, 6, "orders_x"), None, "a citizen's order");
        // **What parts next, drawing nothing before the next word**
        // (4555): a city site `1/2009` the original counts in its sweep of
        // 3804 (`land` 9, `filled` 1) and this crate does not, and who=1's
        // `territory` 488 here against 290 — East Indies' `2007` on its
        // block 1571 has the same shape. No mechanism is named (DECISIONS
        // 42).
        assert_eq!(
            row(1, 2009, "city:land").as_deref(),
            Some("3805: ours 0 theirs 9"),
            "the city site's sweep"
        );
        assert_eq!(
            row(1, -1, "leader:territory").as_deref(),
            Some("3805: ours 488 theirs 290"),
            "and who=1's territory"
        );
        assert_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(3771, 78), (3801, 1), (3805, 4)],
            "the blocks keys first part on, the first three"
        );
    }

    /// **The second pair's East Indies word, 5606, widened whole** (item
    /// 989): run357 is run346's game at run352's detail over blocks
    /// 5601..5857, walked from run346's own start. The word's frame writes
    /// block 5607.
    #[test]
    fn run357_s_word_frame_is_widened_whole() {
        use std::collections::BTreeMap;
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run357",
            "gamelog-run357-islands-toughest-5606.txt",
            WIDENING_SECOND_EAST_INDIES_5606,
            &[SECOND_WORD_EAST_INDIES + 1],
            true,
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        assert_eq!(w.blocks, 257, "run357 whole: blocks 5601..5857");
        assert!(
            w.missing.is_empty(),
            "run357 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The block before the word, 5605, parts on one key**
        // (`docs/AI.md` §82.4): who=1's unit `1/14` has a move order whose
        // `dest` reads 0 here against 1 there, and its path's third slot
        // parts on the word's block. The original's extra draw on 5606 is
        // `Unit::do_move+0xe84`. Block 5601 stands on 159 keys, the
        // families the window opened on. No mechanism is named (DECISIONS
        // 42).
        assert_eq!(
            row(1, 14, "order:move.dest").as_deref(),
            Some(r#"5605: Move { field: "dest", ours: 0, theirs: 1 }"#),
            "the unit's move, the block before the word"
        );
        assert_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(5601, 159), (5605, 1), (5607, 1)],
            "the blocks keys first part on, the first three"
        );
    }

    /// **The second pair's Great Lakes word, 4555, widened whole** (item
    /// 989): run356 is run347's game at run355's detail over blocks
    /// 4550..4806, walked from run347's start. The word's frame writes
    /// block 4556. **Since item 997 the word is 4593**, block 4594, inside
    /// the same window (44 blocks after its first and 212 before its last),
    /// so this capture is its widening too. **Since item 1002 the word is
    /// 4605**, block 4606, inside it again (56 blocks after its first and
    /// 200 before its last). **Since item 1012 the word is 4618**, block
    /// 4619 (69 blocks after its first and 187 before its last). **Since
    /// item 1014 the word is 4673**, block 4674 (124 blocks after its first
    /// and 132 before its last). **Since item 1023 the word is 4688**, block
    /// 4689 (139 blocks after its first and 117 before its last). **Since
    /// item 1034 the word is 4781**, block 4782 (232 blocks after its first
    /// and 24 before its last). **Since item 1040 the word is 4846**, block
    /// 4847, past this window: run373 is its widening.
    #[test]
    fn run356_s_word_frame_is_widened_whole() {
        use std::collections::BTreeMap;
        let Some(w) = crate::diff::harness::tests::widen_great_lakes_on(
            (
                "gamelog-run347-greatlakes-toughest-24k-trace.txt",
                "rontrace-run347.log",
            ),
            "run356",
            &[(
                "gamelog-run356-greatlakes-toughest-4555.txt",
                WIDENING_SECOND_GREAT_LAKES_4555.0,
            )],
            WIDENING_SECOND_GREAT_LAKES_4555,
            1,
            &[SECOND_WORD_GREAT_LAKES + 1],
            true,
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        assert_eq!(w.blocks, 257, "run356 whole: blocks 4550..4806");
        assert!(
            w.missing.is_empty(),
            "run356 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The old word's block, 4555, agrees** (item 997, `docs/COMBAT.md`
        // §64): `1/21`'s attack-move no longer takes the human's unarmed
        // Woodcutter's Camp on 4554, so its `ATTACK_TO` stands on both
        // sides until the army's attack on 4605, and since item 1012 its
        // order agrees through that too; it first parted again on 4761
        // until item 1028, and since then not in the window.
        assert_eq!(
            row(1, 21, "order:kind"),
            None,
            "the unit's order, which parted on 4555 until item 997"
        );
        // **The old word, 4593, agrees** (item 1002, `docs/COMBAT.md`
        // §65): `1/26`'s chase on the human's city `0/2000` stops at
        // (5079, 31628) on block 4592 on both sides — a building target is
        // asked without the `0x90` margin — and holds its `ATTACK` there
        // until block 4605, where both sides drop it; since item 1012 the
        // army's attack puts it back on the city on 4605 on both sides, and
        // neither its order nor its position parts again in the window.
        assert_eq!(
            row(1, 26, "order:kind"),
            None,
            "the unit's order, which parted on 4592 until item 1002"
        );
        assert_eq!(row(1, 26, "pos"), None, "and where it stands");
        // **The old word, 4605, agrees** (item 1012, `docs/COMBAT.md` §66):
        // the army group's attack-move looks on 4605 (`1/15`, `(4605 + 15)
        // % 15 == 0`), finds the human's city, and hands it to the whole
        // group — `Group::action_attack(…, QUEUE_FIRST, 4)` — whose retarget
        // takes a building with the word 2, and `1/17`'s ring starts
        // mid-face. Every member holds the city's `ATTACK` over the
        // re-issued `GROUP_ATTACK_TO` on block 4606 on both sides; `1/9`
        // first parted on 4771 until item 1014, and since then its order
        // agrees through the window.
        assert_eq!(
            row(1, 9, "order:kind"),
            None,
            "an army member's order, which parted on 4606 until item 1012"
        );
        // **The old word, 4618, agrees** (item 1014, `docs/SCOUT.md` §8.3):
        // the AI scout `1/0` chooses its explore target on tick 4506 with
        // the human's cells undoubled — `treaties[0] & 3` is the met bit,
        // set on 4456 — and takes cell (3, 42), (2808, 32760), on both
        // sides, where ours took (2, 40), (2040, 31224), arrived on 4618
        // and idled. Its order agrees from block 4550 to its next explore
        // target on 4737; `1/0`'s group and `1/7`'s gather `wait`, which
        // parted on 4619, no longer part in the window. That target read
        // 1272 here from item 1023 and 2808 again from item 1028, against
        // the original's 504; **since item 1034 it agrees**, and the
        // scout's order agrees through the window. Which read of the city
        // moved it is not established: the fix changed only the city's
        // three flags and its heal (`docs/COMBAT.md` §69).
        assert_eq!(
            row(1, 0, "order:move.x"),
            None,
            "the scout's target, which parted on 4550 until item 1014 and on \
             4737 until item 1034"
        );
        assert_eq!(
            row(1, 0, "group"),
            None,
            "the scout's group, 4619 until 1014"
        );
        assert_eq!(
            row(1, 7, "order:gather.wait"),
            None,
            "the gatherer's wait, 4619 until item 1014"
        );
        // **The old word, 4673, agrees** (item 1023, `docs/COMBAT.md` §67):
        // on 4616, the review's phase for `1/24` (`(4616 + 24) % 16 == 0`),
        // its target, the citizen `0/3`, is walking away and out of reach
        // of the chase spot, so `check_target_path` re-aims: block 4617
        // prints the move at (4104, 31512) on both sides, where ours kept
        // (4200, 31608). `1/24`'s order (kind 10 against 1 on 4673) and
        // `stopped` (4674) no longer part; its rows agree from 4605 to 4688.
        // Its order parted again on 4722 until item 1028, and since then
        // not in the window.
        assert_eq!(
            row(1, 24, "order:kind"),
            None,
            "the chaser's order, which parted on 4673 until item 1023"
        );
        // **The old word, 4688, agrees** (item 1028, `docs/COMBAT.md` §68):
        // `1/24` is a computer's raider (stance 3), and `compare_target`'s
        // RAID arm scores the citizen `0/3` at 9800 and the human's scout
        // `0/0` at 15, as run368's packet does on logger frame 4688. So its
        // one-in-five re-search keeps `0/3`, and it strikes: `recharging`
        // (0 against 33 until 1028) no longer parts on block 4689. Its walk
        // spot does, (3763, 31600) against the cell centre (3768, 31608).
        assert_eq!(
            row(1, 24, "recharging"),
            None,
            "the chaser's strike, on 4689 until item 1028"
        );
        assert_eq!(
            row(1, 24, "orders_x").as_deref(),
            Some("4689: ours 3763 theirs 3768"),
            "and its spot, which parted on 4617 until item 1023"
        );
        // **The old word, 4690, agrees** (item 1034, `docs/COMBAT.md` §69):
        // `1/26`'s first strike on the human's city on 4657 sets its
        // `city_flags` `0x2 | 0x4 | 0x8` on both sides, and the city no
        // longer heals the wound off (ours 1/0 against 2/5 on 4661 until
        // 1034), so 4690's strike is not a first wound and throws no
        // `take_damage+0xe1`. The three bits now agree through the window,
        // `0x4`'s clear on 4801 (`(4800 + 2000) % 200 == 0`) included.
        for bit in ["0x2", "0x4", "0x8"] {
            assert_eq!(
                row(0, 2000, &format!("city:city_flags[{bit}]")),
                None,
                "the human city's {bit}, which 1034 carries"
            );
        }
        // **A Hoplite on the city's north face strikes due south**
        // (item 1040, `docs/COMBAT.md` §70.6): `1/22`'s facing stood parted
        // from 4758 (ours the centre's bearing, 0x959a0000, against
        // 0x80000000) until the side arm.
        assert_eq!(
            row(1, 22, "g.angle[0]"),
            None,
            "the Hoplite's facing, which parted on 4758 until item 1040"
        );
        // What the heal had hidden since 4661: `1/26`'s strikes from the
        // second on landed here a frame after the original's (4689, 4716,
        // 4722 there; 4690, 4717, 4723 here) until item 1040: its stones
        // left the Slinger's own square, not its bay (`launch::BAYS`, piece
        // 32, `docs/COMBAT.md` §70), and flew a frame long across a
        // boundary. Now every strike lands on the original's frame.
        assert_eq!(
            row(0, 2000, "build:damage"),
            None,
            "the city's damage, one strike behind on 4690 until item 1040"
        );
        assert_eq!(
            row(0, 3, "hits:damage").as_deref(),
            Some("4723: ours 3 theirs 2"),
            "the citizen's damage, which first parts on 4723"
        );
        // **The old word, 4781, agrees** (item 1040, `docs/COMBAT.md` §70):
        // `1/24`'s stone at the citizen `0/4`, launched on 4775, leaves the
        // Slinger's bay and flies 5 frames on both sides, so `0/4` is struck
        // on 4779 on both; and a citizen walking to its drop site under its
        // `GATHER` is not on duty (`Unit::on_duty@005fff70`), so it does not
        // turn on the Slinger (`600863`): no `ATTACK` on 4780, no
        // `fight+0x9b0` on 4781. None of `0/4`'s rows parts in the window.
        for what in ["damage_frame", "order:kind", "pos"] {
            assert_eq!(
                row(0, 4, what),
                None,
                "the citizen's {what}, which parted on 4780 until item 1040"
            );
        }
        // **The group record and the attack order's row** (item 1061,
        // `docs/GROUPS.md` §33): three rows stand from the window's first
        // block — army 0's group (slot 65) `role`, ours 0 against the
        // original's `LAND | MILITARY | …` word, and the Town Center's
        // building group (slot 64) `ox`/`oy`, ours −1 against 0 — and two
        // families part inside it: the army group's `curr` on 4600, with
        // `1/19`'s heading on the same block, and the `ATTACKORDER`'s own
        // row on 4753, `1/24`'s `ever_in_range` 1 against 0 and `new_ord` 0
        // against 1.
        assert_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(4550, 79), (4585, 1), (4598, 1)],
            "the blocks keys first part on, the first three ((4550, 76) until \
             item 1061 compared the group record; 86, 18 and 1 on 4550, 4575 \
             and 4585 until item 1014, the scout's ten and eighteen)"
        );
        for (who, o, what, want) in [
            (1, -3, "group:65.role", "4550: ours 0 theirs 1379331"),
            (1, -3, "group:64.ox", "4550: ours -1 theirs 0"),
            (
                1,
                -3,
                "group:65.curr[0]",
                "4600: ours Some((190, 414)) theirs Some((192, 414))",
            ),
            (1, 24, "attack[0].ever_in_range", "4753: ours 1 theirs 0"),
            (1, 24, "attack[0].new_ord", "4753: ours 0 theirs 1"),
        ] {
            assert_eq!(
                row(who, o, what).as_deref(),
                Some(want),
                "{who}/{o} {what}, the record item 1061 compares"
            );
        }
    }

    /// **The second pair's Great Lakes word, 4846, widened whole** (item
    /// 1040): run373 is run347's game at run356's detail over blocks
    /// 4841..5097, walked from run347's start. The word's frame writes
    /// block 4847, six blocks after the window's first. **Within the same
    /// item the word moved to 4852 and then 4877**, block 4878, inside the
    /// window (37 blocks after its first and 219 before its last). **Item
    /// 1052 moved it to 4924**, block 4925, inside the window too (84
    /// blocks after its first and 172 before its last).
    #[test]
    fn run373_s_word_frame_is_widened_whole() {
        use std::collections::BTreeMap;
        let Some(w) = crate::diff::harness::tests::widen_great_lakes_on(
            (
                "gamelog-run347-greatlakes-toughest-24k-trace.txt",
                "rontrace-run347.log",
            ),
            "run373",
            &[(
                "gamelog-run373-greatlakes-toughest-4846.txt",
                WIDENING_SECOND_GREAT_LAKES_4846.0,
            )],
            WIDENING_SECOND_GREAT_LAKES_4846,
            1,
            &[SECOND_WORD_GREAT_LAKES + 1],
            true,
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        assert_eq!(w.blocks, 257, "run373 whole: blocks 4841..5097");
        assert!(
            w.missing.is_empty(),
            "run373 carries every key: {:?}",
            w.missing
        );
        let first = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| (*f, r.clone()))
        };
        // **The old word, 4846, agrees** (item 1040, `docs/COMBAT.md`
        // §70.6): the Hoplite `1/19` stands on the row above the human's
        // city and strikes it due south on both sides — a building is
        // struck square to its side — where ours turned 21° to the centre,
        // deferred the swing and struck a frame late. Its figure first
        // parts on the army's tick, 4861.
        assert_eq!(
            first(1, 19, "g.angle[0]").map(|(f, _)| f),
            Some(4_989),
            "the Hoplite's facing, which parted on 4847 until item 1040 and \
             on the army's tick, 4861, until item 1052"
        );
        // **The old word, 4852, agrees** (§70.7): `1/24`'s one-in-five
        // re-search on 4852 names `0/3` on both sides and freezes the
        // frame, so its attack slot holds at 32 of 33 on block 4853 and
        // wraps on 4854; its clock no longer parts, and its figure first
        // parts on the army's tick too.
        for what in ["g.cur_time[0]", "g.last_time[0]", "g.end_time[0]"] {
            assert!(
                first(1, 24, what).is_none_or(|(f, _)| f > 4_853),
                "the Slinger's {what}, which parted on 4853 until item 1040: {:?}",
                first(1, 24, what)
            );
        }
        // **The old word, 4877, agrees** (item 1052, `docs/ARMY.md` §23):
        // the army's tick on 4860 (`4860 ≡ 252 mod 256`) finds army 0
        // engaged and forms it a cell **behind** its point, `muster_angle −
        // 0x80000000`, before `do_forming` moves it again. The leader
        // `1/12`'s first move hands the second's `get_loc` a start in the
        // original's cell (3996, 30614), and the group's chain plans the
        // original's eight legs, not nine. So on block 4861 every member
        // walks its `ATTACK_TO` on both sides: `1/13` at (3936, 31125)
        // toward (4588, 31018), `1/16` toward (5007, 30915), `1/12` with
        // its move under way, where ours stood `1/12`–`1/14` a frame
        // (`dest` 0) and took a leg 768 west for `1/16`–`1/18`.
        for (o, what) in [
            (12, "order:move.dest"),
            (13, "order:move.dest"),
            (14, "order:move.dest"),
            (13, "pos"),
            (16, "order:move.dest_x"),
            (17, "order:move.dest_x"),
            (18, "order:move.dest_x"),
            (16, "path:length"),
        ] {
            assert!(
                first(1, o, what).is_none_or(|(f, _)| f > SECOND_WORD_GREAT_LAKES + 1),
                "1/{o}'s {what}, which parted on the army's tick, 4861, until \
                 item 1052: {:?}",
                first(1, o, what)
            );
        }
        // **The new word, 4924, writes block 4925** (no mechanism is named,
        // DECISIONS 42): ours 8 draws and the original 7, parting at index
        // 0, where ours spends `Guy::set_anim+0xf2f < Guy::move+0x166` and
        // the original `Guy::set_anim+0x97a < Unit::move_step+0x823`. The
        // earliest block past the window's standing rows that parts on a
        // unit's order is 4923: `1/11` holds its `ATTACK` there (kind 10,
        // two orders) where the original has pushed the chase's move above
        // it (kind 1, three), and it stands at (5046, 30225) against
        // (5032, 30200). Before it only value rows: `1/9`'s and `1/24`'s
        // `orders_x/y` (4868, 4887) and the citizens' and the leader's rows.
        // **The group record and the attack order's row** (item 1061,
        // `docs/GROUPS.md` §33): five more rows stand from the window's
        // first block — army 0's `role`, the Town Center group's `ox`/`oy`
        // (run356 has all three from 4550) and `1/9`'s second `ATTACK`,
        // `ever_in_range` 1 against 0 and `new_ord` 0 against 1 — and the
        // same attack row parts on 4853 on `1/24`, the block after its
        // one-in-five re-search (§70.7), with `in_range` 1 against 0 too.
        assert_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(4_841, 116), (4_853, 3), (4_868, 2)],
            "the blocks keys first part on, the first three ((4841, 111), \
             (4868, 2), (4887, 2) until item 1061 compared the group record \
             and the attack order's row; (4841, 111), (4861, 121), (4862, 13) \
             until item 1052)"
        );
        for (o, what, want) in [
            (9, "attack[1].new_ord", (4_841, "ours 0 theirs 1")),
            (24, "attack[0].in_range", (4_853, "ours 1 theirs 0")),
            (24, "attack[0].new_ord", (4_853, "ours 0 theirs 1")),
            (11, "attack[0].in_range", (4_924, "ours 1 theirs 0")),
            (11, "attack[0].new_ord", (4_924, "ours 0 theirs 1")),
            // The figure's aim, first parting on the word's block: ours
            // `1/11` aims at the citizen `0/2`, the original's still at
            // the Town Center `0/2000`.
            (11, "g.ox[0]", (4_924, "ours 2 theirs 2000")),
        ] {
            assert_eq!(
                first(1, o, what),
                Some((want.0, want.1.to_string())),
                "1/{o}'s {what}, the attack order's row item 1061 compares"
            );
        }
        assert_eq!(
            first(1, 11, "order:kind"),
            Some((4_923, "Kind { ours: 10, theirs: 1 }".to_string())),
            "the new word's order row"
        );
        // A standing row the window opens on: the citizen `0/4` has taken
        // more here since run356's window closed (block 4841: `damage` 6
        // 10/16 here, 5 0/16 there).
        assert_eq!(
            first(0, 4, "hits:damage"),
            Some((4_841, "ours 6 theirs 5".to_string())),
            "the citizen's damage, standing from the window's first block"
        );
    }

    /// **run346 — East Indies at Toughest.** The lobby read back from the
    /// dump's own `GAME INFO` is 5, and the harness stands the simulation up
    /// at it; the word is pinned as a floor.
    #[test]
    fn run346_is_east_indies_at_toughest_and_its_word_holds() {
        let Some(w) = walk_second(
            "gamelog-run346-islands-toughest-24k-trace.txt",
            "rontrace-run346.log",
            true,
        ) else {
            return;
        };
        assert_eq!(w.difficulty, 5, "run346's GAME INFO reads DIFFICULTY 5");
        assert!(
            w.count >= SECOND_WORD_EAST_INDIES && w.sequence >= SECOND_WORD_EAST_INDIES,
            "run346's word fell: count {}, sequence {} of {}; the floor is \
             {SECOND_WORD_EAST_INDIES}",
            w.count,
            w.sequence,
            w.last
        );
    }

    /// **run347 — Great Lakes at Toughest**, on the same terms.
    #[test]
    fn run347_is_great_lakes_at_toughest_and_its_word_holds() {
        let Some(w) = walk_second(
            "gamelog-run347-greatlakes-toughest-24k-trace.txt",
            "rontrace-run347.log",
            false,
        ) else {
            return;
        };
        assert_eq!(w.difficulty, 5, "run347's GAME INFO reads DIFFICULTY 5");
        assert!(
            w.count >= SECOND_WORD_GREAT_LAKES && w.sequence >= SECOND_WORD_GREAT_LAKES,
            "run347's word fell: count {}, sequence {} of {}; the floor is \
             {SECOND_WORD_GREAT_LAKES}",
            w.count,
            w.sequence,
            w.last
        );
    }
}
