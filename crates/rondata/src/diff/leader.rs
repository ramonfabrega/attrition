//! The `LEADERDATA` record, whole, over a window — the leader's ledger and
//! its census against the original's own, frame for frame.
//!
//! **Why this module exists.** `bucket` — the AI's stockpile — is written
//! only at `LEADERS=9`, and until item 290 the only thing compared out of
//! that record over a *window* was six of its goods rows
//! (`great_lakes_goods_record_and_its_trade_routes_are_the_original_s`).
//! The rest of a ~250-field record went uncompared on every capture that
//! carries it, which is the shape item 285 names one record up. Great
//! Lakes' word at 7585 is inside it: every gate of `make_stuff` step 6
//! agrees and the AI is short of food (`docs/AI.md` §32), so what parts is
//! a *value* in this record and nothing else.
//!
//! [`rows`] is the whole of the mapping: the crate's leader state keyed by
//! the dump's own field names, so a widening is a line here rather than a
//! new test. What the crate does not model is listed in
//! [`UNMODELLED`] rather than left silent — the widening ledger's rule
//! (`docs/DATALAYER.md` §4).

use crate::diff::setup::Built;
/// `BASE_UNITTYPES` — the `TypeIndex` of the first unit record, which is
/// the offset between this crate's record index and the dump's own
/// `num_queued` index. `docs/DATALAYER.md`.
const BASE_UNITTYPES: usize = 0x32;
/// `BASE_GAIATYPES - BASE_UNITTYPES`: the width of the record's own
/// `num_units` array, which is **352** and stops before the twelve gaia
/// types. This crate's unit table is 364 records for exactly that reason,
/// so the muster rows are cut here rather than naming twelve keys the
/// record cannot carry.
const MUSTER_WIDTH: usize = 0x192 - 0x32;

use crate::gamelog::Block;
use sim::economy::RESOURCES;

/// One scalar of a leader's record: the dump's key, and this crate's value.
pub(crate) type Row = (String, i64);

/// The `LEADERDATA` keys this crate has no value for, and why — the
/// widening ledger's own list, so "not compared" is a statement rather
/// than an omission. Every other scalar the record prints is in [`rows`].
pub(crate) const UNMODELLED: &[(&str, &str)] = &[
    ("score", "no score is kept"),
    ("gov", "governments are not modelled"),
    ("defeated_by", "no defeat path"),
    ("misery", "the misery counter is unread"),
    ("att", "the attrition score counters are unread"),
    ("anti_att", "the attrition score counters are unread"),
    ("blacken", "fog bookkeeping"),
    ("explored", "check_explore's visibility recount is a seam"),
    ("known_rares", "rares are a leader-level seam"),
    ("pop_issues", "the population-pressure counter is unread"),
    (
        "misc_stamps",
        "the twenty-odd *_stamp counters have no writers here",
    ),
    (
        "handicap",
        "`Holdings::handicap` is the income percentage, a different field",
    ),
    ("multi_diff", "the lobby difficulty is an input, not state"),
    (
        "wonder_mark",
        "`Census::wonder_mark` has no writer here and the original DOES move \
         it — 1 on forty of run80's records — so the row would be a \
         constant against a moving field, which is worse than not \
         comparing it. It returns when the wonder bookkeeping lands.",
    ),
    ("DIPLOMACY", "the diplomacy block is not modelled"),
    ("num_bonus_cards", "bonus cards are Conquer-the-World's"),
    ("average_*_rate", "the combat averages are score counters"),
];

/// Whether a good's index is one the record prints in its per-good block.
const GOODS: [&str; RESOURCES] = ["food", "timber", "metal", "wealth", "knowledge", "oil"];

/// This crate's whole leader record, keyed by the dump's own names.
///
/// The per-good rows are flattened as `bucket[0]` … `bucket[5]`, which is
/// what makes a residue line name the good rather than a slot.
pub(crate) fn rows(loaded: &crate::load::Loaded, built: &Built, who: usize) -> Vec<Row> {
    let l = &built.sim.ledgers[who];
    let h = &built.sim.holdings[who];
    let a = &built.sim.ai[who];
    let c = &a.census;
    let mut out: Vec<Row> = Vec::new();
    fn six(out: &mut Vec<Row>, key: &str, v: [i32; RESOURCES]) {
        for (g, name) in GOODS.iter().enumerate() {
            out.push((format!("{key}[{g}:{name}]"), i64::from(v[g])));
        }
    }
    // The encrypted goods block, per good, in the order the record prints
    // it: `bucket leftover resource_cap over_cap resources support income
    // rate bonus`. Six of the nine are modelled; `support` and `bonus` are
    // the support ledger's and are not, and `over_cap` is an enum here.
    six(&mut out, "bucket", l.bucket);
    six(&mut out, "leftover", l.leftover);
    six(&mut out, "resource_cap", l.cap);
    six(&mut out, "resources", l.rate);
    six(&mut out, "income", l.income);
    six(&mut out, "rate", a.rate);
    six(&mut out, "escrow", l.escrow);
    six(&mut out, "escrow_rate", c.escrow_rate);
    six(&mut out, "econ", a.econ);
    six(&mut out, "gather_slots", l.gather_slots);
    six(&mut out, "filled_gather_slots", c.filled_gather_slots);
    six(&mut out, "gather_slots_high", l.gather_slots_high);
    six(&mut out, "bonus_cap", h.bonus_cap);
    // `home_reg` and every `SITE.reg` are region ids, and the two sides
    // number regions differently: `Built::region_map` is the translation
    // and without it the whole family reads as a divergence.
    let region = |r: i32| -> i64 {
        built
            .region_map
            .iter()
            .find(|(_, s)| i64::from(*s) == i64::from(r))
            .map_or(i64::from(r), |(d, _)| *d)
    };
    six(
        &mut out,
        "over_cap",
        std::array::from_fn(|g| match l.over_cap[g] {
            sim::economy::OverCap::Under => 0,
            sim::economy::OverCap::At => 1,
            sim::economy::OverCap::Uncapped => 2,
        }),
    );
    // **The diplomacy pair, whole** — item 382. `diplos[i]` is 0 at war,
    // 1 at peace, 2 allied, and a leader's own slot reads 2; `treaties[i]`
    // carries the met bit in its low bit, set on *first contact* by
    // `Leader::meet@006e1250` and never cleared. This crate's `met` is a
    // seam that answers "yes" for every live leader from frame 1
    // (`ai_census.rs`, `seams`), so `treaties` is the seam's own value and
    // parts on every window until a visibility model exists. It is the
    // gate `weight_total`'s war term hangs on, so it is compared rather
    // than left out (`docs/AI.md` §45).
    for i in 0..built.sim.players.len() {
        let d = if i == who {
            2
        } else if built.sim.at_war[who][i] {
            0
        } else if built.sim.allied[who][i] {
            2
        } else {
            1
        };
        out.push((format!("diplos[{i}]"), d));
        let met = i64::from(built.sim.has_met(who as sim::Player, i));
        out.push((format!("treaties[{i}]"), met));
    }
    // The AI's own step machine and its biases.
    out.push(("production_step".to_string(), i64::from(a.step.number())));
    out.push(("prod_script_run".to_string(), i64::from(a.script_live)));
    out.push(("script_step".to_string(), i64::from(a.script_step)));
    out.push(("worst_good".to_string(), a.worst_good as i64));
    out.push(("best_good".to_string(), a.best_good as i64));
    out.push(("shortages".to_string(), i64::from(a.shortages)));
    out.push(("wonder_mod".to_string(), i64::from(a.wonder_mod)));
    out.push(("ground_mod".to_string(), i64::from(a.ground_mod)));
    out.push(("air_mod".to_string(), i64::from(a.air_mod)));
    out.push(("sea_mod".to_string(), i64::from(a.sea_mod)));
    out.push(("infra_mod".to_string(), i64::from(a.infra_mod)));
    out.push(("defense_mod".to_string(), i64::from(a.defense_mod)));
    out.push(("effective_pop".to_string(), i64::from(a.effective_pop)));
    out.push(("gather_stamp".to_string(), l.gather_stamp));
    out.push(("tech_frame".to_string(), a.tech_frame));
    out.push(("frame_attacked".to_string(), a.frame_attacked));
    out.push(("attacked_by".to_string(), i64::from(a.attacked_by)));
    for (i, f) in a.tech_cat_frame.iter().enumerate() {
        out.push((format!("tech_cat_frame[{i}]"), *f));
    }
    // **`control` is `active`'s twin and it had never been compared** —
    // item 303. The original writes the two on adjacent lines at every
    // writer it has (`Unit::set_type@00612fa0:74,284`,
    // `Objects::init_unit@0065e0c0:92,174`), so a row that moves one and
    // not the other is exactly the bug the pair is here to catch. It
    // lives on [`sim::Muster`] rather than the census because the price
    // ramp reads it.
    out.push((
        "control".to_string(),
        i64::from(built.sim.muster[who].control),
    ));
    // The census — `docs/AI.md` §2.3, under the PDB's names.
    for (k, v) in [
        ("active", c.active),
        ("combat", c.combat),
        ("siege", c.siege),
        ("non_siege", c.non_siege),
        ("sea_combat", c.sea_combat),
        ("defense", c.defense),
        ("attack", c.attack),
        ("naval", c.naval),
        ("transports", c.transports),
        ("peasants", c.peasants),
        ("scholars", c.scholars),
        ("caras", c.caras),
        ("merchants", c.merchants),
        ("scouts", c.scouts),
        ("fighters", c.fighters),
        ("bombers", c.bombers),
        ("cruise", c.cruise),
        ("nuke", c.nuke),
        ("pop", c.pop),
        ("free_peasants", c.free_peasants),
        ("xport_peasants", c.xport_peasants),
        ("gatherers", c.gatherers),
        ("attacked", c.attacked),
        ("full_cities", c.full_cities),
        ("my_team_terr", c.my_team_terr),
        ("other_team_terr", c.other_team_terr),
        ("min_other_team_terr", c.min_other_team_terr),
        ("wars", c.wars),
        ("allies", c.allies),
        ("active_wars", c.active_wars),
        ("peasant_high", c.peasant_high),
        ("scholar_high", c.scholar_high),
        ("merchant_high", c.merchant_high),
        ("caravan_high", c.caravan_high),
        ("village_num", c.village_num),
        ("territory", h.territory),
    ] {
        out.push((k.to_string(), i64::from(v)));
    }
    out.push(("home_reg".to_string(), region(c.home_reg)));
    out.push((
        "active_wars_with".to_string(),
        i64::from(c.active_wars_with),
    ));
    out.push(("ally_mask".to_string(), i64::from(c.ally_mask)));
    // `city_num`: the live cities this leader owns.
    out.push((
        "city_num".to_string(),
        built
            .sim
            .cities
            .iter()
            .filter(|x| x.alive && usize::from(x.owner) == who)
            .count() as i64,
    ));
    // The personality — twenty-four rolls, and `Leader::init` is their one
    // writer, so a parting here is a setup fault rather than a drift.
    let p = &a.pers;
    for (k, v) in [
        ("rush", p.rush),
        ("cities", p.cities),
        ("upgrades", p.upgrades),
        ("arms", p.arms),
        ("army", p.army),
        ("army_size", p.army_size),
        ("raid", p.raid),
        ("invade", p.invade),
        ("target", p.target),
        ("strategy", p.strategy),
        ("raze", p.raze),
        ("spells", p.spells),
        ("forts", p.forts),
        ("nukes", p.nukes),
        ("air", p.air),
        ("naval", p.naval),
        ("market", p.market),
        ("scouts", p.scouts),
        ("civilians", p.civilians),
        ("early_army", p.early_army),
        ("friendly_human", p.friendly_human),
        ("alliance_human", p.alliance_human),
        ("friendly_ai", p.friendly_ai),
        ("alliance_ai", p.alliance_ai),
    ] {
        out.push((format!("PERSONALITY.{k}"), i64::from(v)));
    }
    // **The make list, slot for slot** — eleven `MAKEOBJECT` blocks, ten
    // fields each. It is part of this record and was compared nowhere over
    // a window: `make_stuff`'s whole decision is a function of it
    // (`docs/AI.md` §2.6), so a head that is the wrong type spends the
    // wrong draws with every arithmetic step correct.
    // **`t` is the original's `TypeIndex`, not this crate's tree id.** The
    // tree is laid out gaplessly (`crate::load`, "the type space"), so a
    // good, a unit and a building carry the same number either way and a
    // **tech does not**: the tech block starts at tree id 543 and at
    // `TypeIndex` `0x220` = 544. Comparing the raw id against the dump's
    // therefore reads every tech offer as off by one — which it did, on
    // every window, until item 323 put a scholar and three techs side by
    // side and the techs were the ones that "differed".
    let ti = |t: i32| -> i64 {
        if t < 0 {
            i64::from(t)
        } else {
            i64::from(loaded.type_index(t as sim::tech::TypeId))
        }
    };
    for (i, m) in a.make_list.list.iter().enumerate() {
        for (k, v) in [
            ("t", ti(m.t)),
            ("val", i64::from(m.val)),
            ("escrow", i64::from(m.escrow)),
            ("city", i64::from(m.city)),
            ("up", i64::from(m.up)),
            ("o", i64::from(m.o)),
            ("num", i64::from(m.num)),
            ("cat", i64::from(m.cat)),
            ("wx", i64::from(m.wx)),
            ("wy", i64::from(m.wy)),
        ] {
            out.push((format!("MAKE[{i}].{k}"), v));
        }
    }
    // **The per-type muster, whole** — `num_units` and `num_queued`, the
    // two 352-wide arrays the record prints under those names. Item 302:
    // `MAKE[7].val` is a function of `upgrade_units`' `owned`, which sums
    // `num_units` over a type's predecessor chain, and nothing here
    // compared either array. The dump's index is the record's own — the
    // array is keyed by `TypeIndex - BASE_UNITTYPES` and this crate's
    // record index is the same number (`UnitType::type_index`).
    let m = &built.sim.muster[who];
    for (r, n) in m.by_type.iter().enumerate().take(MUSTER_WIDTH) {
        out.push((format!("num_units[{r}]"), i64::from(*n)));
    }
    for (r, n) in m.queued_by_type.iter().enumerate().take(MUSTER_WIDTH) {
        out.push((format!("num_queued[{r}]"), i64::from(*n)));
    }
    // The ten sites, slot for slot.
    for (i, s) in a.sites.iter().enumerate() {
        for (k, v) in [
            ("wx", i64::from(s.wx)),
            ("wy", i64::from(s.wy)),
            ("val", i64::from(s.val)),
            ("reg", region(s.reg)),
            ("dist", i64::from(s.dist)),
            ("rank", i64::from(s.rank)),
        ] {
            out.push((format!("SITE[{i}].{k}"), v));
        }
    }
    out
}

/// The dump's side of [`rows`] for one `LEADERDATA` block: the same keys,
/// read off the record. A key the block does not carry is absent, which is
/// what keeps a thinner capture from reading as a page of divergences.
pub(crate) fn theirs(block: &Block<'_>) -> std::collections::BTreeMap<String, i64> {
    let mut out = std::collections::BTreeMap::new();
    let all = |k: &str| -> Vec<i64> {
        block
            .all(k)
            .iter()
            .map(|v| v.trim().parse().unwrap_or(i64::MIN))
            .collect()
    };
    // The per-good rows: nine keys repeated once per good in the encrypted
    // block, and the `[scan]` arrays after it.
    for key in [
        "bucket",
        "leftover",
        "resource_cap",
        "over_cap",
        "resources",
        "income",
        "rate",
    ] {
        let v = all(key);
        // `resource_cap` is printed a seventh time after the six goods
        // (the block's own trailer); only the first six are the goods.
        for (g, name) in GOODS.iter().enumerate() {
            if let Some(x) = v.get(g) {
                out.insert(format!("{key}[{g}:{name}]"), *x);
            }
        }
    }
    for key in [
        "escrow",
        "escrow_rate",
        "econ",
        "gather_slots",
        "filled_gather_slots",
        "bonus_cap",
    ] {
        let v = all(&format!("{key}[scan]"));
        for (g, name) in GOODS.iter().enumerate() {
            if let Some(x) = v.get(g) {
                out.insert(format!("{key}[{g}:{name}]"), *x);
            }
        }
    }
    // **`gather_slots_high` is twelve entries and it is `[6][2]`**, not
    // six: run84's block 7000 prints `10 10 12 12 1 1 0 0 0 0 0 0`
    // against a `gather_slots` of `10 12 1 0 0 0`, so the good's own
    // high-water mark is at stride two and the six-in-a-row reading
    // reported five of the six goods as divergences on every frame.
    for key in ["diplos", "treaties"] {
        for (i, x) in all(&format!("{key}[scan]")).iter().enumerate().take(8) {
            out.insert(format!("{key}[{i}]"), *x);
        }
    }
    let v = all("gather_slots_high[scan]");
    for (g, name) in GOODS.iter().enumerate() {
        if let Some(x) = v.get(g * 2) {
            out.insert(format!("gather_slots_high[{g}:{name}]"), *x);
        }
    }
    for key in [
        "production_step",
        "prod_script_run",
        "script_step",
        "worst_good",
        "best_good",
        "shortages",
        "wonder_mod",
        "ground_mod",
        "air_mod",
        "sea_mod",
        "infra_mod",
        "defense_mod",
        "effective_pop",
        "gather_stamp",
        "tech_frame",
        "frame_attacked",
        "attacked_by",
        "active",
        "control",
        "combat",
        "siege",
        "non_siege",
        "sea_combat",
        "defense",
        "attack",
        "naval",
        "transports",
        "peasants",
        "scholars",
        "caras",
        "merchants",
        "scouts",
        "fighters",
        "bombers",
        "cruise",
        "nuke",
        "pop",
        "free_peasants",
        "xport_peasants",
        "gatherers",
        "attacked",
        "full_cities",
        "home_reg",
        "my_team_terr",
        "other_team_terr",
        "min_other_team_terr",
        "wars",
        "allies",
        "active_wars",
        "active_wars_with",
        "ally_mask",
        "peasant_high",
        "scholar_high",
        "merchant_high",
        "caravan_high",
        "village_num",
        "territory",
        "city_num",
    ] {
        if let Some(x) = block.int(key) {
            out.insert(key.to_string(), x);
        }
    }
    // **The per-type muster, whole — and the two arrays are keyed
    // differently.** `num_units` is 352 wide and its index 0 is
    // `BASE_UNITTYPES`; `num_queued` is **806** wide and its index 0 is
    // `TypeIndex` 0, so the same Hoplite sits at 82 in one and 132 in the
    // other. Reading both as record-keyed reported every queued type as a
    // divergence at two indices at once, which is how the offset was
    // found (item 302). Both are re-keyed to this crate's record here.
    for (r, x) in all("num_units[scan]").iter().enumerate() {
        out.insert(format!("num_units[{r}]"), *x);
    }
    for (i, x) in all("num_queued[scan]").iter().enumerate() {
        if let Some(r) = i.checked_sub(BASE_UNITTYPES) {
            out.insert(format!("num_queued[{r}]"), *x);
        }
    }
    let v = all("tech_cat_frame[scan]");
    for (i, x) in v.iter().enumerate().take(4) {
        out.insert(format!("tech_cat_frame[{i}]"), *x);
    }
    if let Some(p) = block.kid("PERSONALITY") {
        for k in [
            "rush",
            "cities",
            "upgrades",
            "arms",
            "army",
            "army_size",
            "raid",
            "invade",
            "target",
            "strategy",
            "raze",
            "spells",
            "forts",
            "nukes",
            "air",
            "naval",
            "market",
            "scouts",
            "civilians",
            "early_army",
            "friendly_human",
            "alliance_human",
            "friendly_ai",
            "alliance_ai",
        ] {
            if let Some(x) = p.int(k) {
                out.insert(format!("PERSONALITY.{k}"), x);
            }
        }
    }
    for (i, m) in block.make_list().iter().enumerate().take(11) {
        for (k, v) in [
            ("t", m.t),
            ("val", m.val),
            ("escrow", m.escrow),
            ("city", m.city),
            ("up", m.up),
            ("o", m.o),
            ("num", m.num),
            ("cat", m.cat),
            ("wx", m.wx),
            ("wy", m.wy),
        ] {
            out.insert(format!("MAKE[{i}].{k}"), v);
        }
    }
    for (i, s) in block.kids("SITE").enumerate().take(10) {
        for k in ["wx", "wy", "val", "reg", "dist", "rank"] {
            if let Some(x) = s.int(k) {
                out.insert(format!("SITE[{i}].{k}"), x);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{rows, theirs};
    use crate::diff::setup::{Built, borrow_from_siblings, build_sim};
    use crate::diff::testkit::with_sibling_initials;
    use crate::gamelog::Log;
    use crate::testenv::{dump, install};
    use sim::Tuning;

    /// Great Lakes' game, stood up from run53's start dump and ticked to
    /// `last`, keeping this crate's whole leader record for both players
    /// on every frame from `first`.
    type Kept = std::collections::BTreeMap<(i64, usize), Vec<(String, i64)>>;

    fn great_lakes(first: i64, last: i64) -> Option<(Kept, Built)> {
        let inst = install()?;
        let state = dump("gamelog-run53-greatlakes-24k-trace.txt").or_else(|| {
            eprintln!("skipping: no run53 capture (set RON_GAMELOG_DIR)");
            None
        })?;
        let loaded = crate::load::load(&inst).unwrap();
        // Only construction borrows capture data. Keep sibling observations,
        // not their whole text, and release all inputs before ticking.
        let mut built = with_sibling_initials(|refs| {
            let text = crate::capture::read(&state);
            let log = Log::parse(&text);
            let mut init = log.initial().unwrap();
            borrow_from_siblings(&mut init, refs);
            build_sim(&loaded, &init, Tuning::RON)
        });
        let mut kept = std::collections::BTreeMap::new();
        for n in 1..=last {
            built.tick();
            if n < first {
                continue;
            }
            for who in 0..2usize {
                kept.insert((n, who), rows(&loaded, &built, who));
            }
        }
        Some((kept, built))
    }

    /// **`active` is the muster's cardinality, and that is what item 303
    /// established.** It reads like a census counter — it is
    /// `LeaderData+0x93c`, the sweep zeroes it and counts into it
    /// (`Leader::plan_strategy@006b9620:127,508`), and the record prints it
    /// among the unit classes — and it is not one. Its live writers are
    /// `Unit::set_type@00612fa0:74,284`, which moves `num_units`, `control`
    /// and `active` in one guarded block in each direction, and
    /// `Objects::init_unit@0065e0c0:92,174`, which undoes all three when
    /// the new unit turns out to be a squad follower. So the original's
    /// `active` equals the sum of its own `num_units` on **every** block,
    /// not only on the frames its sweep runs — which is why modelling the
    /// recount alone left this crate a stale snapshot, 31 against 32 for
    /// 62 of run91's 86 blocks. `docs/AI.md` §37.
    ///
    /// Called from both windows' loops: 332 blocks. Made to fail on
    /// purpose by summing `num_queued` instead.
    fn active_is_the_muster_summed(
        t: &std::collections::BTreeMap<String, i64>,
        n: i64,
        who: usize,
    ) {
        let sum: i64 = t
            .iter()
            .filter(|(k, _)| k.starts_with("num_units["))
            .map(|(_, v)| *v)
            .sum();
        assert_eq!(
            t["active"], sum,
            "block {n}, leader {who}: the original's `active` is not its own \
             per-type muster summed"
        );
    }

    /// **The leader record, whole, over run84's window** — item 290.
    ///
    /// Until this, six of the record's ~250 fields were compared over a
    /// window and the rest of it on no capture at all past frame 1
    /// (`run9_s_frame_1_leader_record_is_the_census_after_the_sweep`).
    /// The record carries the AI's stockpile, its census, its personality
    /// and its ten sites, and Great Lakes' word at 7585 is a *value* in it
    /// (`docs/AI.md` §32) — so what a window of it says is the whole of
    /// what that word is waiting on.
    #[test]
    fn run84_s_window_is_the_original_s_whole_leader_record() {
        const FIRST: i64 = 6950;
        const LAST: i64 = 7029;
        let Some(path) = dump("gamelog-run84-greatlakes-makelist.txt") else {
            eprintln!("skipping: no run84 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let Some((ours, _)) = great_lakes(FIRST, LAST) else {
            return;
        };
        let wtext = crate::capture::read(&path);
        let wlog = Log::parse(&wtext);
        let mut compared = 0usize;
        let mut blocks = 0usize;
        let mut missing: std::collections::BTreeSet<String> = Default::default();
        let mut residue: std::collections::BTreeMap<(usize, String), (usize, i64, i64, i64)> =
            Default::default();
        for n in FIRST..=LAST {
            for who in 0..2usize {
                let Some(block) = wlog.leader_block(n, who as i64) else {
                    continue;
                };
                blocks += 1;
                let t = theirs(&block);
                active_is_the_muster_summed(&t, n, who);
                for (k, mine) in &ours[&(n, who)] {
                    let Some(&yours) = t.get(k) else {
                        missing.insert(k.clone());
                        continue;
                    };
                    compared += 1;
                    if *mine != yours {
                        let e = residue
                            .entry((who, k.clone()))
                            .or_insert((0, *mine, yours, n));
                        e.0 += 1;
                    }
                }
            }
        }
        eprintln!("run84 [{FIRST}, {LAST}]: {blocks} blocks, {compared} field-frames");
        if !missing.is_empty() {
            eprintln!("  the record does not carry: {missing:?}");
        }
        for ((who, k), (n, o, t, first)) in &residue {
            eprintln!("  {who}/{k}: {n} frames, ours {o} theirs {t} on {first}");
        }
        assert_eq!(blocks, 160, "eighty frames, two leaders");
        // [`UNMODELLED`] and [`rows`] are two halves of one statement and
        // may not overlap: a key in both would claim the field is compared
        // and not compared at once. `wonder_mark` was in both for an hour.
        let keys: std::collections::BTreeSet<&str> = ours[&(FIRST, 1)]
            .iter()
            .map(|(k, _)| k.split(['[', '.']).next().unwrap())
            .collect();
        let clash: Vec<&str> = super::UNMODELLED
            .iter()
            .map(|(k, _)| *k)
            .filter(|k| keys.contains(k))
            .collect();
        assert!(clash.is_empty(), "UNMODELLED and rows both carry {clash:?}");
        assert_eq!(
            compared, 167_680,
            "160 blocks of the record, every field the mapping carries"
        );
        assert!(
            missing.is_empty(),
            "the record does not carry {missing:?} — the mapping names a key \
             the original does not print"
        );
        // **The residue is pinned by name, not by count.** Every field
        // below parts on run84's window today and the reason is recorded
        // in `docs/AI.md` §33; a field that leaves this list is a fix and
        // a field that joins it is a regression, and either way the test
        // says which. Made to fail on purpose both ways before it landed.
        let parting: Vec<(usize, &str)> = residue.keys().map(|(w, k)| (*w, k.as_str())).collect();
        assert_eq!(
            parting,
            PARTS_ON_RUN84,
            "run84's leader residue moved: {} fields",
            parting.len()
        );
    }

    /// **The leader record over run91's window, and the word is in it** —
    /// item 290. Great Lakes' 7585 is `make_stuff` step 6's good loop
    /// over food (`docs/AI.md` §32), and this is the first capture on
    /// this map ever to carry `bucket` anywhere near it.
    #[test]
    fn run91_s_window_is_the_leader_s_ledger_at_the_word() {
        const FIRST: i64 = 7514;
        const LAST: i64 = 7599;
        let Some(path) = dump("gamelog-run91-greatlakes-wordledger.txt") else {
            eprintln!("skipping: no run91 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let Some((ours, _)) = great_lakes(FIRST, LAST) else {
            return;
        };
        let wtext = crate::capture::read(&path);
        let wlog = Log::parse(&wtext);
        let mut compared = 0usize;
        let mut blocks = 0usize;
        let mut residue: std::collections::BTreeMap<(usize, String), (usize, i64, i64, i64)> =
            Default::default();
        let mut food: Vec<(i64, i64, i64)> = Vec::new();
        for n in FIRST..=LAST {
            for who in 0..2usize {
                let Some(block) = wlog.leader_block(n, who as i64) else {
                    continue;
                };
                blocks += 1;
                let t = theirs(&block);
                active_is_the_muster_summed(&t, n, who);
                for (k, mine) in &ours[&(n, who)] {
                    let Some(&yours) = t.get(k) else { continue };
                    compared += 1;
                    if *mine != yours {
                        let e = residue
                            .entry((who, k.clone()))
                            .or_insert((0, *mine, yours, n));
                        e.0 += 1;
                    }
                    if who == 1 && k == "bucket[0:food]" {
                        food.push((n, *mine, yours));
                    }
                }
            }
        }
        eprintln!("run91 [{FIRST}, {LAST}]: {blocks} blocks, {compared} field-frames");
        for ((who, k), (n, o, t, first)) in &residue {
            eprintln!("  {who}/{k}: {n} frames, ours {o} theirs {t} on {first}");
        }
        let ladder: Vec<String> = food
            .iter()
            .map(|(n, o, t)| format!("{n}:{o}/{t}"))
            .collect();
        eprintln!("  food ours/theirs: {}", ladder.join(" "));
        // **The two make lists at the word, slot for slot.** This is what
        // `make_stuff` reads on sim-frame 7585, and `docs/AI.md` §34 is
        // written off it.
        for n in [7585i64, 7586] {
            let block = wlog.leader_block(n, 1).expect("run91 carries the block");
            let t = theirs(&block);
            let o: std::collections::BTreeMap<String, i64> =
                ours[&(n, 1)].iter().cloned().collect();
            eprintln!("  block {n}, leader 1's make list:");
            for i in 0..11 {
                let f = |k: &str| -> (i64, i64) {
                    let key = format!("MAKE[{i}].{k}");
                    (o[&key], t[&key])
                };
                let (ot, tt) = f("t");
                let (ov, tv) = f("val");
                let (oc, tc) = f("cat");
                let (on, tn) = f("num");
                let (oy, ty) = f("city");
                eprintln!(
                    "    {i:>2}  ours t {ot:>5} val {ov:>8} cat {oc:>2} num {on} city {oy:>2}   \
                     theirs t {tt:>5} val {tv:>8} cat {tc:>2} num {tn} city {ty:>2}"
                );
            }
        }
        assert_eq!(blocks, 172, "86 frames, two leaders");
        assert_eq!(
            compared, 180_256,
            "172 blocks of the record, every field the mapping carries"
        );

        // **The per-type muster, whole — and it is what settled item
        // 302.** `num_units` is the original's own per-type count, 352
        // wide, and `num_queued` its 806-wide sibling; neither had ever
        // been compared. They agree on every type, every block and both
        // players, and *that* is the answer to "is the original's count
        // over units or over captains": leader 1's one Hoplite squad of
        // three figures counts **one** on both sides and its three
        // Longbowman squads count **three**, which is this crate's own
        // convention. The item was booked as `Muster::by_type` counting
        // squad heads where the original counts units; the original
        // counts heads. `docs/AI.md` §36.
        let muster: Vec<&str> = residue
            .keys()
            .map(|(_, k)| k.as_str())
            .filter(|k| k.starts_with("num_units[") || k.starts_with("num_queued["))
            .collect();
        assert!(
            muster.is_empty(),
            "the per-type muster parts from the original's: {muster:?}"
        );

        // **The refusal, and it was the whole of item 290.** Item 287
        // read Great Lakes' word as the AI's *stockpile* — `88 < 55 + 43
        // + 4` in `make_stuff` step 6, with the original's food inferred
        // at `98 ≤ food < 160` from its own refusals — and this capture
        // was written to test that. The original has **88**, and the
        // ladder is identical tick for tick over every one of the 72
        // blocks up to and including the word's own. The stockpile was
        // never wrong.
        let under: Vec<&(i64, i64, i64)> = food.iter().filter(|(n, _, _)| *n <= 7585).collect();
        assert_eq!(under.len(), 72, "7514..=7585");
        assert!(
            under.iter().all(|(_, o, t)| o == t),
            "the food ladder parts at or below the word: {:?}",
            under.iter().find(|(_, o, t)| o != t)
        );

        // **And both sides spend 43 of it on the word's own frame** —
        // item 295. Block 7586 is the state at the end of sim-frame 7585:
        // the Citizen out of make-list slot 5, at the price this crate's
        // own tables give the next one. ~~`(7586, 88, 45)` — the original
        // pays 43 and we pay nothing~~; the whole ladder agrees now, all
        // 86 blocks of it, and that is the assertion below.
        let at = |n: i64| food.iter().find(|(f, _, _)| *f == n).copied().unwrap();
        assert_eq!(at(7585), (7585, 88, 88), "the word's frame, both sides");
        assert_eq!(at(7586), (7586, 45, 45), "both sides pay the Citizen's 43");
        assert!(
            food.iter().all(|(_, o, t)| o == t),
            "the food ladder parts somewhere in the window: {:?}",
            food.iter().find(|(_, o, t)| o != t)
        );

        // **What the gate reads, and it is the head.** The good loop tests
        // only goods the head's cost is non-zero in
        // (`make_stuff@006c8af0:172`), so a Temple head — which costs no
        // food — never puts food on trial and slot 5 is bought. This
        // crate's head was three Slingers at the `val < 0` overflow guard
        // until the census learned to read `UnitTypeData::role` over
        // captains; it is the original's Temple now, `val`, `cat` and
        // `num` alike, and slot 6's Slingers carry the original's own
        // **360000** at **num 1** — §34's first two oracles, closed by the
        // one change. `docs/AI.md` §34, §35.
        let head = |i: usize, k: &str| -> (i64, i64) {
            let key = format!("MAKE[{i}].{k}");
            let block = wlog.leader_block(7585, 1).unwrap();
            let t = theirs(&block);
            let o: std::collections::BTreeMap<String, i64> =
                ours[&(7585, 1)].iter().cloned().collect();
            (o[&key], t[&key])
        };
        assert_eq!(head(0, "t"), (437, 437), "the head is not the Temple");
        assert_eq!(head(0, "cat"), (8, 8), "a civic building on both sides");
        assert_eq!(head(0, "num"), (1, 1));
        assert_eq!(head(0, "val"), (2_499_999, 2_499_999));
        assert_eq!(head(6, "t"), (82, 82), "slot 6 is not the Slingers");
        assert_eq!(
            head(6, "val"),
            (360_000, 360_000),
            "slot 6's value is not the original's — the wrap is back, or \
             the census is"
        );
        assert_eq!(head(6, "num"), (1, 1), "slot 6's batch is not one");
        // **Slot 7's factor of two, closed — item 302.** `t 133` Phalanx
        // is `upgrade_units`' one military offer in this list, and its
        // value was exactly half the original's on every block of both
        // windows. The cause is neither `Muster::by_type` nor the
        // predecessor chain, both of which the muster assertion above
        // shows correct: it is the **denominator**. The original divides
        // `pop * 1000` by `city_num + village_num` — two fields it adds
        // — and this crate recounted "cities that are still villages"
        // instead, answering 2 where the original's `village_num` is 0,
        // so `base` was 500 against 1000 and every value in the list
        // halved. `docs/AI.md` §36.
        assert_eq!(head(7, "t"), (133, 133), "slot 7 is not the Phalanx");
        assert_eq!(
            head(7, "val"),
            (62_976, 62_976),
            "slot 7's value is not the original's — `village_num` is \
             being recounted again, or `owned` moved"
        );

        // The residue, pinned by name the way run84's is.
        let parting: Vec<(usize, &str)> = residue.keys().map(|(w, k)| (*w, k.as_str())).collect();
        assert_eq!(
            parting,
            PARTS_ON_RUN91,
            "run91's leader residue moved: {} fields",
            parting.len()
        );
    }

    /// **The leader record over run19's window, and the scholar in it** —
    /// item 323, `docs/AI.md` §38.
    ///
    /// run19 sat on disk from 2026-08-25 carrying dump-blocks **8174–8191**
    /// of this map's own game — `rngcmp` against run53 is 0 differing and
    /// 8,201 identical, and the exhaustive scan of every `gamelog*.txt`
    /// says it is the **only** capture anywhere that holds blocks 8182 and
    /// 8186. It was read once for `make_stuff`'s head clause (§15.6) and
    /// never compared field for field, which is what this does.
    ///
    /// **What it pins.** Great Lakes' draw sequence parted at 8182 on a
    /// `Leader::make_stuff+0x63d` this crate never spent: the original's
    /// `create_units` offers a **Scholar** at sim-frame 8180 and buys it
    /// out of slot 1 at 8182, so step 6's expiry walk runs and draws. This
    /// crate refused the offer — `civilian_value`'s scholar gate compares
    /// `bucket[knowledge]` against `(resource_cap[food] / 16) * 3 / 2` and
    /// was halving the cap through `get_mod_resource_cap`, which on
    /// Easiest turns 2,000 into 1,000 and 187 into 93 against a stockpile
    /// of 159. Block 8181's slot 1 is the assertion, and block 8183's
    /// `bucket` is the value beside it: the scholar's thirty wealth, 40 →
    /// 10, which this crate did not spend.
    #[test]
    fn run19_s_window_is_the_leader_record_at_the_scholar() {
        const FIRST: i64 = 8174;
        const LAST: i64 = 8191;
        let Some(path) = dump("gamelog-run19-window-8174-8192.txt") else {
            eprintln!("skipping: no run19 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let Some((ours, _)) = great_lakes(FIRST, LAST) else {
            return;
        };
        let wtext = crate::capture::read(&path);
        let wlog = Log::parse(&wtext);
        let mut compared = 0usize;
        let mut blocks = 0usize;
        let mut missing: std::collections::BTreeSet<String> = Default::default();
        let mut residue: std::collections::BTreeMap<(usize, String), (usize, i64, i64, i64)> =
            Default::default();
        // The two rows the item is about, read off the comparison rather
        // than off the simulation, so the dump is on both sides of them.
        let mut scholar: Option<(i64, i64)> = None;
        let mut wealth: Option<(i64, i64)> = None;
        for n in FIRST..=LAST {
            for who in 0..2usize {
                let Some(block) = wlog.leader_block(n, who as i64) else {
                    continue;
                };
                blocks += 1;
                let t = theirs(&block);
                active_is_the_muster_summed(&t, n, who);
                for (k, mine) in &ours[&(n, who)] {
                    let Some(&yours) = t.get(k) else {
                        missing.insert(k.clone());
                        continue;
                    };
                    compared += 1;
                    if who == 1 && n == 8181 && k == "MAKE[1].t" {
                        scholar = Some((*mine, yours));
                    }
                    if who == 1 && n == 8183 && k == "bucket[2:metal]" {
                        wealth = Some((*mine, yours));
                    }
                    if *mine != yours {
                        let e = residue
                            .entry((who, k.clone()))
                            .or_insert((0, *mine, yours, n));
                        e.0 += 1;
                    }
                }
            }
        }
        eprintln!("run19 [{FIRST}, {LAST}]: {blocks} blocks, {compared} field-frames");
        for ((who, k), (n, o, t, first)) in &residue {
            eprintln!("  {who}/{k}: {n} frames, ours {o} theirs {t} on {first}");
        }
        assert_eq!(blocks, 36, "eighteen blocks, two leaders");
        assert!(missing.is_empty(), "the record does not carry {missing:?}");
        assert_eq!(
            compared, 37_728,
            "36 blocks of the record, every field the mapping carries"
        );
        // **The scholar, on the frame `create_units` offers it.** 52 is
        // `SCHOLARS` (`0x34`), and the slot is the head's runner-up.
        assert_eq!(
            scholar,
            Some((52, 52)),
            "block 8181's make-list slot 1 is not the scholar on both              sides — this is `civilian_value`'s knowledge gate reading the              difficulty-modified food cap again, and Great Lakes' sequence              parts at 8182 when it does"
        );
        // **And what it costs.** The good the record names `metal` is
        // index 2, which `resourcerules.xml` calls **Wealth** — the dump's
        // own name for slot 2 and the one §15.6 read as metal.
        assert_eq!(
            wealth,
            Some((10, 10)),
            "block 8183's second good is the scholar's price, and 40 here              is the purchase not made"
        );
        let parting: Vec<(usize, &str)> = residue.keys().map(|(w, k)| (*w, k.as_str())).collect();
        assert_eq!(
            parting,
            PARTS_ON_RUN19,
            "run19's leader residue moved: {} fields",
            parting.len()
        );
    }

    /// **The leader record at the word's own frame** — item 369,
    /// `docs/AI.md` §44. run107 is a `LEADERS=9` window over blocks
    /// 9170–9199 of this map's own game, taken because nothing on the
    /// disk carried the AI's goods or its make list between block 8191
    /// (run19) and block 23959 (run80), and Great Lakes' draw sequence
    /// parts at **9182**. The capture's own checks: 30 blocks with no
    /// gap, `rngcmp` against run53 0 differing over 9,216 frames, and
    /// `samegame.py --exclude LEADERDATA` against run97 — the same
    /// frames at `LEADERS=1` — 30 in common, 0 differing.
    ///
    /// **What it decided.** `docs/AI.md` §43 named two branches the
    /// vector at 9182 could be on, and the capture refuses **both**:
    /// the original's `bucket` is `73 84 35 111 71 0` on blocks
    /// 9181–9184, which is this crate's own value entering the frame,
    /// and it does **not move across 9183** where this crate spends 46
    /// food on a Citizen. The goods are not the difference. The make
    /// list is: the original's head is `t 573 cat 10` and the Scholar
    /// sits under it at slot 1, where this crate has promoted the
    /// Scholar over it.
    #[test]
    fn run107_s_window_is_the_leader_record_at_the_word() {
        const FIRST: i64 = 9170;
        const LAST: i64 = 9199;
        let Some(path) = dump("gamelog-run107-greatlakes-wordledger2.txt") else {
            eprintln!("skipping: no run107 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let Some((ours, _)) = great_lakes(FIRST, LAST) else {
            return;
        };
        let wtext = crate::capture::read(&path);
        let wlog = Log::parse(&wtext);
        let mut compared = 0usize;
        let mut blocks = 0usize;
        let mut missing: std::collections::BTreeSet<String> = Default::default();
        let mut residue: std::collections::BTreeMap<(usize, String), (usize, i64, i64, i64)> =
            Default::default();
        // The two rows the item is about, read off the comparison so the
        // dump is on both sides of them: the head's type on the frame the
        // sequence parts, and the food it did not spend.
        let mut head: Option<(i64, i64)> = None;
        let mut food: Option<(i64, i64)> = None;
        for n in FIRST..=LAST {
            for who in 0..2usize {
                let Some(block) = wlog.leader_block(n, who as i64) else {
                    continue;
                };
                blocks += 1;
                let t = theirs(&block);
                active_is_the_muster_summed(&t, n, who);
                for (k, mine) in &ours[&(n, who)] {
                    let Some(&yours) = t.get(k) else {
                        missing.insert(k.clone());
                        continue;
                    };
                    compared += 1;
                    if who == 1 && n == 9182 && k == "MAKE[0].t" {
                        head = Some((*mine, yours));
                    }
                    if who == 1 && n == 9183 && k == "bucket[0:food]" {
                        food = Some((*mine, yours));
                    }
                    if *mine != yours {
                        let e = residue
                            .entry((who, k.clone()))
                            .or_insert((0, *mine, yours, n));
                        e.0 += 1;
                    }
                }
            }
        }
        eprintln!("run107 [{FIRST}, {LAST}]: {blocks} blocks, {compared} field-frames");
        for ((who, k), (n, o, t, first)) in &residue {
            eprintln!("  {who}/{k}: {n} frames, ours {o} theirs {t} on {first}");
        }
        assert_eq!(blocks, 60, "thirty blocks, two leaders");
        assert!(missing.is_empty(), "the record does not carry {missing:?}");
        assert_eq!(
            compared, 62_880,
            "60 blocks of the record, every field the mapping carries"
        );
        // **The head on the frame the sequence parts.** 573 is the
        // original's; 52 is `SCHOLARS`. The two carry the same `val`
        // 9,999,999, so this is a tie broken the other way — and every
        // draw `make_stuff` spends is a function of the head.
        // **The head on the frame the sequence parts, and it is this
        // item's finding.** 573 is `MERCENARIES`, a tech; 52 is
        // `SCHOLARS`. The original keeps Mercenaries at the head through
        // the whole window and ranks the Scholar under it at slot 1;
        // this crate promotes the Scholar over it on block 9181 and
        // drops Mercenaries out of the ranked four. Every draw
        // `make_stuff` spends is a function of the head, so this is the
        // parting — **not** the goods, which agree. A fix must change
        // this line, and the two values are why (`docs/AI.md` §44).
        assert_eq!(
            head,
            Some((573, 573)),
            "block 9182's head moved — if this crate now offers 573 the \
             item landed; if the original's is no longer 573 the capture \
             is being read wrong"
        );
        // **And the food beside it.** The original's stockpile does not
        // move across this frame; 27 here is the Citizen this crate buys
        // and it does not.
        // **And the food beside it — the value diff for the frame that
        // moved.** The original's stockpile does not move across 9182:
        // `73 84 35 111 71 0` on 9181, 9182, 9183 and 9184 alike. This
        // crate spends 46 food on a Citizen there. Both branches
        // `docs/AI.md` §43 named are refused by this one row, because
        // both were about the *bucket* and the bucket agrees entering
        // the frame.
        assert_eq!(
            food,
            Some((73, 73)),
            "block 9183's food moved — 73 on both sides is the purchase \
             gone, which is what this item's successor is for"
        );
        let parting: Vec<(usize, &str)> = residue.keys().map(|(w, k)| (*w, k.as_str())).collect();
        assert_eq!(
            parting,
            PARTS_ON_RUN107,
            "run107's leader residue moved: {} fields",
            parting.len()
        );
    }

    /// The `(player, field)` pairs that part over run107's window — the
    /// word's own frame. Filled from the first run and then pinned;
    /// `docs/AI.md` §44.
    const PARTS_ON_RUN107: &[(usize, &str)] = &[
        (0, "SITE[0].reg"),
        (0, "SITE[1].reg"),
        (0, "SITE[2].reg"),
        (0, "SITE[3].reg"),
        (0, "SITE[4].reg"),
        (0, "SITE[5].reg"),
        (0, "SITE[6].reg"),
        (0, "SITE[7].reg"),
        (0, "SITE[8].reg"),
        (0, "SITE[9].reg"),
        (0, "active_wars"),
        (0, "active_wars_with"),
        (0, "ally_mask"),
        (0, "filled_gather_slots[0:food]"),
        (0, "filled_gather_slots[1:timber]"),
        (0, "gather_stamp"),
        (0, "gatherers"),
        (0, "min_other_team_terr"),
        (0, "my_team_terr"),
        (0, "other_team_terr"),
        (0, "peasant_high"),
        (0, "peasants"),
        (0, "scouts"),
        (0, "wars"),
        (1, "MAKE[0].city"),
        (1, "MAKE[1].city"),
        (1, "MAKE[1].num"),
        (1, "MAKE[1].val"),
        (1, "MAKE[2].cat"),
        (1, "MAKE[2].city"),
        (1, "MAKE[2].num"),
        (1, "MAKE[2].t"),
        (1, "MAKE[2].val"),
        (1, "MAKE[3].cat"),
        (1, "MAKE[3].city"),
        (1, "MAKE[3].t"),
        (1, "MAKE[3].val"),
        (1, "MAKE[4].city"),
        (1, "MAKE[5].city"),
        (1, "MAKE[7].val"),
        (1, "MAKE[8].city"),
        (1, "SITE[1].dist"),
        (1, "SITE[1].rank"),
        (1, "SITE[1].val"),
        (1, "SITE[1].wx"),
        (1, "SITE[1].wy"),
        (1, "SITE[2].dist"),
        (1, "SITE[2].rank"),
        (1, "SITE[2].val"),
        (1, "SITE[2].wx"),
        (1, "SITE[2].wy"),
        (1, "SITE[3].dist"),
        (1, "SITE[3].rank"),
        (1, "SITE[3].val"),
        (1, "SITE[3].wx"),
        (1, "SITE[3].wy"),
        (1, "SITE[4].dist"),
        (1, "SITE[4].rank"),
        (1, "SITE[4].val"),
        (1, "SITE[4].wx"),
        (1, "SITE[4].wy"),
        (1, "SITE[5].dist"),
        (1, "SITE[5].rank"),
        (1, "SITE[5].val"),
        (1, "SITE[5].wx"),
        (1, "SITE[5].wy"),
        (1, "SITE[6].dist"),
        (1, "SITE[6].rank"),
        (1, "SITE[6].val"),
        (1, "SITE[6].wx"),
        (1, "SITE[6].wy"),
        (1, "SITE[7].dist"),
        (1, "SITE[7].rank"),
        (1, "SITE[7].val"),
        (1, "SITE[7].wx"),
        (1, "SITE[7].wy"),
        (1, "SITE[8].dist"),
        (1, "SITE[8].rank"),
        (1, "SITE[8].val"),
        (1, "SITE[8].wx"),
        (1, "SITE[8].wy"),
        (1, "SITE[9].dist"),
        (1, "SITE[9].rank"),
        (1, "SITE[9].val"),
        (1, "SITE[9].wx"),
        (1, "SITE[9].wy"),
        (1, "bucket[1:timber]"),
        (1, "leftover[1:timber]"),
        (1, "scholars"),
        (1, "scouts"),
        (1, "tech_cat_frame[0]"),
        (1, "tech_cat_frame[1]"),
        (1, "tech_cat_frame[2]"),
        (1, "tech_cat_frame[3]"),
        (1, "tech_frame"),
    ];

    /// The `(player, field)` pairs that part over run19's window. Filled
    /// from the first run and then pinned; `docs/AI.md` §38.
    ///
    /// **Item 368 deleted `1/other_team_terr` and
    /// `1/min_other_team_terr`**, and they are the two that matter to the
    /// make list's *values*: `ai_research::weight_total` adds `ai[5]`
    /// (twice over, under `min_other`) when the leader is behind on
    /// territory, and with both facts at nought that term could never
    /// fire. `plan_strategy@006b9620:159-166`'s loop gates on
    /// `leader_flags & 2`, `i != who` and `i >= 0` — and nothing else:
    /// bit 1 is set for **every** leader every capture prints, the
    /// human's `leader_flags 7` among them, so this crate's "every other
    /// *computer* leader" read `other_team_terr` as nought on a
    /// one-human-one-AI game, which is every capture it is diffed
    /// against. 92 fields to 90; **nothing else in the record moved**,
    /// and the Great Lakes word did not either — the six
    /// `1/MAKE[*].val` rows are unchanged, so for these four techs the
    /// territory term is not what pays. `docs/AI.md` §43.
    ///
    /// The **other two loops of the same defect stay**: `census_wars`
    /// (`1/wars`) and `census_strategy` (`1/active_wars`,
    /// `1/active_wars_with`) skip the human on the same wrong gate, and
    /// `weight_total`'s war term reads `active_wars`. Putting the human
    /// back in either takes this residue from 90 fields to **147** —
    /// the facts land and the make list then parts everywhere, because
    /// `active_wars != 0` reaches the danger and region-strategy words
    /// as well. Measured, not guessed; `docs/AI.md` §43 names it.
    ///
    /// **Item 362 deleted `1/MAKE[0].num`**, and this window is what
    /// confirms the fix against a capture rather than against a draw
    /// count: run19 is a `LEADERS=9` dump of `[8174, 8192]`, eight
    /// hundred frames before the frame the item was taken on, and the
    /// make list's `num` for the head slot has disagreed with the
    /// original's on it since the record was first compared. Carrying
    /// `create_units`' per-arm batch size through `civilian_value`
    /// (`docs/AI.md` §42) makes it agree, and the residue falls from 93
    /// fields to 92. Nothing else in the record moved.
    ///
    /// **Item 328 deleted two of them**: `0/frame_attacked` and
    /// `0/attacked_by`. `Army::find_target`'s probe writes both on the
    /// human leader it pokes (`docs/ARMY.md` §12, "The probe"), and this
    /// crate returned before reaching them until `docs/COMBAT.md` §17 was
    /// implemented — so the residue shrank from 95 fields to 93 on the
    /// same window, which is the shape a landed mechanic leaves here.
    const PARTS_ON_RUN19: &[(usize, &str)] = &[
        (0, "SITE[0].reg"),
        (0, "SITE[1].reg"),
        (0, "SITE[2].reg"),
        (0, "SITE[3].reg"),
        (0, "SITE[4].reg"),
        (0, "SITE[5].reg"),
        (0, "SITE[6].reg"),
        (0, "SITE[7].reg"),
        (0, "SITE[8].reg"),
        (0, "SITE[9].reg"),
        (0, "active_wars"),
        (0, "active_wars_with"),
        (0, "ally_mask"),
        (0, "filled_gather_slots[0:food]"),
        (0, "filled_gather_slots[1:timber]"),
        (0, "gatherers"),
        (0, "min_other_team_terr"),
        (0, "my_team_terr"),
        (0, "other_team_terr"),
        (0, "peasant_high"),
        (0, "peasants"),
        (0, "scouts"),
        (0, "wars"),
        (1, "MAKE[0].city"),
        (1, "MAKE[1].city"),
        (1, "MAKE[2].city"),
        (1, "MAKE[3].cat"),
        (1, "MAKE[3].city"),
        (1, "MAKE[3].t"),
        (1, "MAKE[3].val"),
        (1, "MAKE[8].city"),
        (1, "SITE[0].reg"),
        (1, "SITE[1].reg"),
        (1, "SITE[2].dist"),
        (1, "SITE[2].rank"),
        (1, "SITE[2].val"),
        (1, "SITE[2].wx"),
        (1, "SITE[2].wy"),
        (1, "SITE[3].dist"),
        (1, "SITE[3].rank"),
        (1, "SITE[3].val"),
        (1, "SITE[3].wx"),
        (1, "SITE[3].wy"),
        (1, "SITE[4].dist"),
        (1, "SITE[4].rank"),
        (1, "SITE[4].val"),
        (1, "SITE[4].wx"),
        (1, "SITE[4].wy"),
        (1, "SITE[5].dist"),
        (1, "SITE[5].rank"),
        (1, "SITE[5].val"),
        (1, "SITE[5].wx"),
        (1, "SITE[5].wy"),
        (1, "SITE[6].dist"),
        (1, "SITE[6].rank"),
        (1, "SITE[6].val"),
        (1, "SITE[6].wx"),
        (1, "SITE[6].wy"),
        (1, "SITE[7].dist"),
        (1, "SITE[7].rank"),
        (1, "SITE[7].val"),
        (1, "SITE[7].wx"),
        (1, "SITE[7].wy"),
        (1, "SITE[8].dist"),
        (1, "SITE[8].rank"),
        (1, "SITE[8].val"),
        (1, "SITE[8].wx"),
        (1, "SITE[8].wy"),
        (1, "SITE[9].dist"),
        (1, "SITE[9].rank"),
        (1, "SITE[9].val"),
        (1, "SITE[9].wx"),
        (1, "SITE[9].wy"),
        (1, "gather_stamp"),
        (1, "scouts"),
        (1, "tech_cat_frame[0]"),
        (1, "tech_cat_frame[1]"),
        (1, "tech_cat_frame[2]"),
        (1, "tech_cat_frame[3]"),
        (1, "tech_frame"),
    ];

    /// The `(player, field)` pairs that part over run91's window — the
    /// word's own. `docs/AI.md` §34, §35.
    ///
    /// Everything run84's list holds is here too (the same three families
    /// — a human whose census does not run, the sites, and named
    /// unmodelled state), and on top of it the make-list rows below the
    /// head: the offers still rank differently from the third slot down.
    ///
    /// It was **114 rows** before item 295, thirty-five of them make-list
    /// rows. The census fix took fifteen of those, and with them the
    /// whole of `bucket`, `attack`, `combat` and `non_siege` — the head
    /// is the original's Temple now and the stockpile agrees on every
    /// block of the window.
    /// Three `MAKE[i].t` rows left this list in item 323 — the tree
    /// id/`TypeIndex` correction described on [`PARTS_ON_RUN84`].
    /// **Item 368 deleted `1/other_team_terr` and
    /// `1/min_other_team_terr` here too** — 93 fields to 91. The
    /// territory census counted only computer leaders where
    /// `plan_strategy@006b9620:159` gates on `leader_flags & 2`, which
    /// every capture sets for the human. `docs/AI.md` §43.
    const PARTS_ON_RUN91: &[(usize, &str)] = &[
        (0, "SITE[0].reg"),
        (0, "SITE[1].reg"),
        (0, "SITE[2].reg"),
        (0, "SITE[3].reg"),
        (0, "SITE[4].reg"),
        (0, "SITE[5].reg"),
        (0, "SITE[6].reg"),
        (0, "SITE[7].reg"),
        (0, "SITE[8].reg"),
        (0, "SITE[9].reg"),
        (0, "ally_mask"),
        (0, "filled_gather_slots[0:food]"),
        (0, "filled_gather_slots[1:timber]"),
        (0, "gather_stamp"),
        (0, "gatherers"),
        (0, "min_other_team_terr"),
        (0, "my_team_terr"),
        (0, "other_team_terr"),
        (0, "peasant_high"),
        (0, "peasants"),
        (0, "scouts"),
        (1, "MAKE[0].city"),
        (1, "MAKE[0].escrow"),
        (1, "MAKE[0].t"),
        (1, "MAKE[0].val"),
        (1, "MAKE[1].city"),
        (1, "MAKE[1].escrow"),
        (1, "MAKE[1].t"),
        (1, "MAKE[1].val"),
        (1, "MAKE[2].cat"),
        (1, "MAKE[2].city"),
        (1, "MAKE[2].t"),
        (1, "MAKE[2].val"),
        (1, "MAKE[3].city"),
        (1, "MAKE[3].t"),
        (1, "MAKE[3].val"),
        (1, "MAKE[4].city"),
        (1, "MAKE[4].escrow"),
        (1, "MAKE[4].t"),
        (1, "MAKE[4].val"),
        (1, "MAKE[5].city"),
        (1, "MAKE[6].city"),
        (1, "MAKE[8].city"),
        (1, "SITE[0].reg"),
        (1, "SITE[2].dist"),
        (1, "SITE[2].rank"),
        (1, "SITE[2].val"),
        (1, "SITE[2].wx"),
        (1, "SITE[2].wy"),
        (1, "SITE[3].dist"),
        (1, "SITE[3].rank"),
        (1, "SITE[3].val"),
        (1, "SITE[3].wx"),
        (1, "SITE[3].wy"),
        (1, "SITE[4].dist"),
        (1, "SITE[4].rank"),
        (1, "SITE[4].val"),
        (1, "SITE[4].wx"),
        (1, "SITE[4].wy"),
        (1, "SITE[5].dist"),
        (1, "SITE[5].rank"),
        (1, "SITE[5].val"),
        (1, "SITE[5].wx"),
        (1, "SITE[5].wy"),
        (1, "SITE[6].dist"),
        (1, "SITE[6].rank"),
        (1, "SITE[6].val"),
        (1, "SITE[6].wx"),
        (1, "SITE[6].wy"),
        (1, "SITE[7].dist"),
        (1, "SITE[7].rank"),
        (1, "SITE[7].val"),
        (1, "SITE[7].wx"),
        (1, "SITE[7].wy"),
        (1, "SITE[8].dist"),
        (1, "SITE[8].rank"),
        (1, "SITE[8].val"),
        (1, "SITE[8].wx"),
        (1, "SITE[8].wy"),
        (1, "SITE[9].dist"),
        (1, "SITE[9].rank"),
        (1, "SITE[9].val"),
        (1, "SITE[9].wx"),
        (1, "SITE[9].wy"),
        (1, "gather_stamp"),
        (1, "scouts"),
        (1, "tech_cat_frame[0]"),
        (1, "tech_cat_frame[1]"),
        (1, "tech_cat_frame[2]"),
        (1, "tech_cat_frame[3]"),
        (1, "tech_frame"),
    ];

    /// The `(player, field)` pairs of the leader record that part over
    /// run84's window, and nothing else does — `docs/AI.md` §33.
    ///
    /// Three families, and each is one fact:
    ///
    /// - **Player 0's census is empty here**, because the sweep
    ///   (`plan_strategy`, §2.3) runs only for a computer leader in this
    ///   crate and the original runs it for the human too. Ten fields.
    /// - **The ten sites are a different list** — a known seam (§2.7);
    ///   `reg` is a whole-list row because the original writes `0` in a
    ///   site's region where this crate writes the site's own.
    /// - **The rest is unmodelled state named as such**: `tech_frame` and
    ///   its four categories (written only by `Leader::init` here),
    ///   `other_team_terr`/`min_other_team_terr` (census step 4's
    ///   other-team pass), `attack`/`defense`/`scouts`/`active` (unit
    ///   classes the sweep counts), and `gather_stamp` — the frame of the
    ///   last rate reassembly, which is a cadence rather than a value and
    ///   whose *outputs* (`resources`, `income`, `rate`) all agree.
    ///
    /// **Six `MAKE[i].t` rows left this list in item 323 and none of them
    /// was a fix**: `rows` was emitting this crate's tree id where the
    /// dump prints the original's `TypeIndex`, which agree on a good, a
    /// unit and a building and differ by one on a **tech**. Every tech
    /// offer therefore read as a divergence. run91 lost three the same
    /// way. `MAKE[3].t` here is a real one and stays.
    /// **Item 368 deleted `1/other_team_terr` and
    /// `1/min_other_team_terr` here too** — 83 fields to 81. The
    /// territory census counted only computer leaders where
    /// `plan_strategy@006b9620:159` gates on `leader_flags & 2`, which
    /// every capture sets for the human. `docs/AI.md` §43.
    const PARTS_ON_RUN84: &[(usize, &str)] = &[
        (0, "SITE[0].reg"),
        (0, "SITE[1].reg"),
        (0, "SITE[2].reg"),
        (0, "SITE[3].reg"),
        (0, "SITE[4].reg"),
        (0, "SITE[5].reg"),
        (0, "SITE[6].reg"),
        (0, "SITE[7].reg"),
        (0, "SITE[8].reg"),
        (0, "SITE[9].reg"),
        (0, "ally_mask"),
        (0, "filled_gather_slots[0:food]"),
        (0, "filled_gather_slots[1:timber]"),
        (0, "gatherers"),
        (0, "min_other_team_terr"),
        (0, "my_team_terr"),
        (0, "other_team_terr"),
        (0, "peasant_high"),
        (0, "peasants"),
        (0, "production_step"),
        (0, "scouts"),
        (1, "MAKE[0].city"),
        (1, "MAKE[1].city"),
        (1, "MAKE[1].escrow"),
        (1, "MAKE[2].city"),
        (1, "MAKE[2].val"),
        (1, "MAKE[3].city"),
        (1, "MAKE[3].escrow"),
        (1, "MAKE[3].t"),
        (1, "MAKE[3].val"),
        (1, "MAKE[4].city"),
        (1, "MAKE[4].escrow"),
        (1, "MAKE[4].val"),
        (1, "MAKE[6].city"),
        (1, "MAKE[7].city"),
        (1, "MAKE[8].city"),
        (1, "SITE[0].reg"),
        (1, "SITE[1].reg"),
        (1, "SITE[2].dist"),
        (1, "SITE[2].rank"),
        (1, "SITE[2].val"),
        (1, "SITE[2].wx"),
        (1, "SITE[2].wy"),
        (1, "SITE[3].reg"),
        (1, "SITE[4].dist"),
        (1, "SITE[4].rank"),
        (1, "SITE[4].val"),
        (1, "SITE[4].wy"),
        (1, "SITE[5].dist"),
        (1, "SITE[5].rank"),
        (1, "SITE[5].val"),
        (1, "SITE[5].wx"),
        (1, "SITE[5].wy"),
        (1, "SITE[6].dist"),
        (1, "SITE[6].rank"),
        (1, "SITE[6].val"),
        (1, "SITE[6].wx"),
        (1, "SITE[6].wy"),
        (1, "SITE[7].dist"),
        (1, "SITE[7].rank"),
        (1, "SITE[7].val"),
        (1, "SITE[7].wx"),
        (1, "SITE[7].wy"),
        (1, "SITE[8].dist"),
        (1, "SITE[8].rank"),
        (1, "SITE[8].val"),
        (1, "SITE[8].wx"),
        (1, "SITE[8].wy"),
        (1, "SITE[9].dist"),
        (1, "SITE[9].rank"),
        (1, "SITE[9].val"),
        (1, "SITE[9].wx"),
        (1, "SITE[9].wy"),
        (1, "defense"),
        (1, "gather_stamp"),
        (1, "scouts"),
        (1, "tech_cat_frame[0]"),
        (1, "tech_cat_frame[1]"),
        (1, "tech_cat_frame[2]"),
        (1, "tech_cat_frame[3]"),
        (1, "tech_frame"),
    ];

    /// Item 303's probe: this crate's `active` against this crate's own
    /// per-type muster, frame by frame, and the original's beside them.
    #[test]
    #[ignore]
    fn probe_active_against_the_muster() {
        let Some(path) = dump("gamelog-run91-greatlakes-wordledger.txt") else {
            return;
        };
        let Some((ours, _)) = great_lakes(7514, 7599) else {
            return;
        };
        let wtext = crate::capture::read(&path);
        let wlog = Log::parse(&wtext);
        for n in 7514..=7599i64 {
            for who in 0..2usize {
                let r: std::collections::BTreeMap<String, i64> =
                    ours[&(n, who)].iter().cloned().collect();
                let sum: i64 = r
                    .iter()
                    .filter(|(k, _)| k.starts_with("num_units["))
                    .map(|(_, v)| *v)
                    .sum();
                let block = wlog.leader_block(n, who as i64).unwrap();
                let t = theirs(&block);
                eprintln!(
                    "n {n} who {who}: ours active {} sum {sum} control {}  theirs active {}                      control {}",
                    r["active"],
                    r.get("control").copied().unwrap_or(-1),
                    t["active"],
                    t.get("control").copied().unwrap_or(-1),
                );
            }
        }
    }

    /// The predictions run91's stanza is written from: what this crate
    /// holds for both leaders across run91's own window.
    #[test]
    #[ignore]
    fn probe_the_word_s_leader_record() {
        let Some((ours, _)) = great_lakes(7514, 7599) else {
            return;
        };
        for n in [7514i64, 7585, 7586, 7599] {
            for who in 0..2usize {
                let Some(r) = ours.get(&(n, who)) else {
                    continue;
                };
                let line: Vec<String> = r
                    .iter()
                    .filter(|(k, _)| {
                        k.starts_with("bucket")
                            || k.starts_with("leftover")
                            || k.starts_with("income")
                            || k.starts_with("resources")
                            || k.starts_with("escrow[")
                            || k.starts_with("econ")
                            || k.starts_with("gather_slots[")
                            || k.starts_with("filled_gather")
                    })
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect();
                eprintln!("block {n} who {who}: {}", line.join(" "));
                if who == 1 {
                    let rest: Vec<String> = r
                        .iter()
                        .filter(|(k, _)| {
                            !k.contains('[')
                                || k.starts_with("resource_cap")
                                || k.starts_with("rate[")
                                || k.starts_with("over_cap")
                        })
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect();
                    eprintln!("   scalars: {}", rest.join(" "));
                }
            }
        }
        // The food ladder over the window: every frame the pile moves.
        let mut prev = -1i64;
        let mut ticks = Vec::new();
        for n in 7514..=7599i64 {
            let f = ours[&(n, 1)]
                .iter()
                .find(|(k, _)| k == "bucket[0:food]")
                .map(|(_, v)| *v)
                .unwrap();
            if f != prev {
                ticks.push(format!("{n}:{f}"));
                prev = f;
            }
        }
        eprintln!("food ladder who 1: {}", ticks.join(" "));
    }
}
