//! `Build::plunder@00623660` as steps over a view (item 3402): the good a
//! building's death, or its razing by its owner, puts in a leader's bucket,
//! and the messages that tell the console player. [`build_plunder_steps`] is
//! the original's control flow in its order; [`BuildPlunderView`] is what it
//! reads and the callees it makes, which `sweep/build_plunder.rs` answers
//! from the emulated original's windows.
//!
//! The function's arguments are the building (`this`), a second argument
//! the body never reads, and `who`: the leader that plunders. Its steps:
//!
//! * **the price**: the type's plunder value in the type's plunder good,
//!   refused when the owner is a Lakota razing it, when the good is not
//!   available to `who`, and when a German's mine or a Tikal temple has
//!   its discount taken to nothing;
//! * **the share**: an unfinished building pays its built fraction, in
//!   single precision (`job counter / build time`, clamped to one); a
//!   finished one pays `PLUNDER` percent, unless the owner razes it and it
//!   stands in a city it owns (`get_town`, the town's city's owner byte);
//! * **the hero and the Aztecs**: an Aztec plunderer pays `AZTEC_PLUNDER`
//!   more; a plunderer with The Despot or Spitamenes near the building
//!   adds `THEDESPOT_PLUNDER` percent of it;
//! * **the books**: the scenario's plunder scale, then the bucket, or for a
//!   Russian owner the building's price to the owner and the hero's share
//!   to `who`; and a message to each side that is the console player.
//!
//! **What the crate carries.** `Sim::plunder_kill` is a namesake, not a
//! caller: it carries the arms in integers (its built fraction is an exact
//! `i64` quotient where this is a single, its `foreign` test reads the
//! building's city where this walks `get_town`), and none of the messages.
//! `docs/journal/2026-10-21-item-3402.md` names the inputs that part.

use crate::single::Single;

/// The goods are indexed by `TypeIndex`; these are the type identities the
/// function names.
const MINE: i32 = 0x1a3;
const TEMPLE: i32 = 0x1b5;
/// `TIKAL`'s wonder.
const TIKAL_WONDER: i32 = 0x214;
const DESPOT: i32 = 0x160;
const SPITAMENES: i32 = 0x16d;
/// The sound a plunder plays (`S_BUILDING_PLUNDERED`).
const SOUND: i32 = 0x90;

/// The two local `String`s the original builds a message in: `A` is
/// `local_48`, the line; `B` is `local_5c`, the bubble's.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Str {
    A,
    B,
}

/// What `Build::plunder` reads of the building, its type, the leaders, the
/// game and the tables, and the callees it makes.
pub trait BuildPlunderView {
    type Bub: Copy;
    /// `this->who` (the byte at `+9`).
    fn this_who(&mut self) -> i32;
    /// `this`'s position, the words at `+0x10` and `+0x14` unscrambled.
    fn this_coord(&mut self) -> (i32, i32);
    /// `this->flags & 0x1000` (the word at `+0x60`): the building is up.
    fn activated(&mut self) -> bool;
    /// The word at `+0x4c`: the build's job counter.
    fn job_counter(&mut self) -> i32;
    /// Slot `+0x18c` of `this`, handed zero: the build time.
    fn build_time(&mut self) -> i32;
    /// The type's `+0x2d4`: the plunder value.
    fn type_value(&mut self) -> i32;
    /// The type's `+0x2d8`: the plunder good.
    fn type_good(&mut self) -> i32;
    /// The type's `+4`: its identity.
    fn type_ident(&mut self) -> i32;
    /// The type's `+0x234` and `+0x238`: its footprint.
    fn type_size(&mut self) -> (i32, i32);
    /// `LAKOTA_RAZE_PRICE`.
    fn lakota_raze_price(&mut self) -> i32;
    /// `GERMAN_MINE_COST`.
    fn german_mine_cost(&mut self) -> i32;
    /// `TIKAL_TEMPLE_COST`.
    fn tikal_temple_cost(&mut self) -> i32;
    /// `PLUNDER`.
    fn plunder(&mut self) -> i32;
    /// `AZTEC_PLUNDER`.
    fn aztec_plunder(&mut self) -> i32;
    /// `THEDESPOT_PLUNDER`.
    fn thedespot_plunder(&mut self) -> i32;
    /// `RUSSIAN_PLUNDER_STEAL`.
    fn russian_plunder_steal(&mut self) -> i32;
    /// `LeaderData::has_tribe_bonus(bonus)`.
    fn has_tribe_bonus(&mut self, who: i32, bonus: i32) -> bool;
    /// `LeaderData::type_avail(good, 1)`.
    fn type_avail(&mut self, who: i32, good: i32) -> bool;
    /// `LeaderData::has_wonder(wonder)`.
    fn has_wonder(&mut self, who: i32, wonder: i32) -> bool;
    /// `div_3_table[c >> 6]`, `c` a coordinate.
    fn div3(&mut self, c: i32) -> i32;
    /// `BuildTypeData::get_town` of the type `ident`, handed the two table
    /// entries and `who`.
    fn get_town(&mut self, ident: i32, x: i32, y: i32, who: i32) -> i32;
    /// `objects[who].list[town]`'s slot `+0xac`, then the short at `+0x72`
    /// of what it answers: the city's index.
    fn town_city(&mut self, who: i32, town: i32) -> i32;
    /// `cities[who].list[city]`'s signed byte at `+0x5f`.
    fn city_owner(&mut self, who: i32, city: i32) -> i32;
    /// The short at leader `+0x59be` (`ty` The Despot) or `+0x59d8`
    /// (Spitamenes): how many heroes of the type the leader has.
    fn hero_count(&mut self, who: i32, ty: i32) -> i32;
    /// `HeroesData::find_hero@0073a1b0`: the hero of type `ty` within
    /// `slack` of `(x, y)`, or negative.
    fn find_hero(&mut self, x: i32, y: i32, who: i32, ty: i32, slack: i32) -> i32;
    /// `Game::is_scenario`.
    fn is_scenario(&mut self) -> bool;
    /// `ScenarioData::plunder`.
    fn scenario_plunder(&mut self) -> bool;
    /// The float at leader `+0x914`, its bits: the scenario's plunder scale.
    fn plunder_scale(&mut self, who: i32) -> u32;
    /// `LeaderData::bucket_add(good, n)`.
    fn bucket_add(&mut self, who: i32, good: i32, n: i32);
    /// `console->who`.
    fn console_who(&mut self) -> i32;
    /// `String::String` of a local.
    fn str_new(&mut self, s: Str);
    /// `s = loc_str_array[off]` (a localised string).
    fn str_assign_loc(&mut self, s: Str, off: i32);
    /// `s = s.parse(goodtypes[good]->name, goodtypes[good]->name, n)`:
    /// `String::parse` into a temporary, the assignment, the temporary's
    /// close.
    fn str_parse(&mut self, s: Str, good: i32, n: i32);
    /// `String::close`.
    fn str_close(&mut self, s: Str);
    /// `Recycler<TextBubble>::pop`.
    fn pop_bubble(&mut self) -> Self::Bub;
    /// `bubble->string = s`.
    fn bubble_text(&mut self, b: Self::Bub, s: Str);
    /// `bubble->who = who`.
    fn bubble_who(&mut self, b: Self::Bub, who: i32);
    /// `TeamColor::get_neon` of `who`'s team colour: the colour.
    fn neon(&mut self, who: i32) -> i32;
    /// `MessageWin::add_message(s, x, y, 0, color, 1, tc, 0, bubble, tc)`,
    /// `tc` `who`'s team colour.
    fn add_message(&mut self, s: Str, b: Self::Bub, color: i32, who: i32);
    /// `SoundGlobal::play(sound)`.
    fn play(&mut self, sound: i32);
}

/// `x * v / 100`, the multiply wrapping and the division signed: the
/// original's `imul` and the multiply-by-reciprocal that stands for `idiv`.
fn pct(x: i32, v: i32) -> i32 {
    x.wrapping_mul(v) / 100
}

/// The message to the console player, once: the line's first string at
/// the localised offset `line`, the amount, the colour of `color_of`'s team.
fn message<V: BuildPlunderView>(v: &mut V, who: i32, line: i32, good: i32, n: i32, color_of: i32) {
    v.str_new(Str::A);
    v.str_new(Str::B);
    v.str_assign_loc(Str::A, line);
    v.str_assign_loc(Str::B, 0x8c0);
    v.str_parse(Str::A, good, n);
    v.str_parse(Str::B, good, n);
    let b = v.pop_bubble();
    v.bubble_text(b, Str::B);
    v.bubble_who(b, who);
    let color = v.neon(color_of);
    v.add_message(Str::A, b, color, color_of);
    v.play(SOUND);
    v.str_close(Str::B);
    v.str_close(Str::A);
}

/// `Build::plunder@00623660(this, _, who)`.
pub fn build_plunder_steps<V: BuildPlunderView>(v: &mut V, who: i32) {
    let owner = v.this_who();
    if who == owner && v.has_tribe_bonus(owner, 0x13) && v.lakota_raze_price() != 0 {
        return;
    }
    let mut value = v.type_value();
    if value == 0 {
        return;
    }
    let good = v.type_good();
    if good < 0 {
        return;
    }
    if !v.type_avail(who, good) {
        return;
    }
    let owner = v.this_who();
    let german = v.has_tribe_bonus(owner, 0xc);
    if german && v.type_ident() == MINE {
        value = pct(100i32.wrapping_sub(v.german_mine_cost()), value);
        if value == 0 {
            return;
        }
    }
    if who == owner {
        if !v.activated() {
            return;
        }
        if v.type_ident() == TEMPLE && v.has_wonder(owner, TIKAL_WONDER) {
            value = pct(100i32.wrapping_sub(v.tikal_temple_cost()), value);
        }
    }
    if !v.activated() {
        // The built fraction: both counters as unsigned, to single, divided,
        // clamped to one by `minss` (a NaN falls through), the price scaled
        // in single and truncated.
        let ct = v.build_time();
        let job = v.job_counter();
        let f = Single::from_u32(job as u32).divss(Single::from_u32(ct as u32));
        let one = Single::from_bits(0x3f80_0000);
        let f = if f.gt(one) { one } else { f };
        value = Single::from_i32(value).mulss(f).to_i32();
    } else {
        let mut share = true;
        if who == owner {
            let (x, y) = v.this_coord();
            let tx = v.div3(x);
            let ty = v.div3(y);
            let ident = v.type_ident();
            let town = v.get_town(ident, tx, ty, who);
            if town >= 0 {
                let city = v.town_city(who, town);
                if city < 0 || v.city_owner(who, city) == owner {
                    share = false;
                }
            } else {
                share = false;
            }
        }
        if share {
            value = pct(v.plunder(), value);
        }
    }
    if value == 0 {
        return;
    }
    let aztec = v.aztec_plunder();
    if aztec != 0 && who != owner && v.has_tribe_bonus(who, 0) {
        value = pct(aztec.wrapping_add(100), value);
    }
    let mut hero = 0;
    if who != owner {
        let (x, y) = v.this_coord();
        let (xs, ys) = v.type_size();
        let slack = ys.wrapping_div(2).wrapping_add(xs).wrapping_mul(0xc0);
        if v.hero_count(who, DESPOT) != 0 && v.find_hero(x, y, who, DESPOT, slack) >= 0 {
            hero = pct(v.thedespot_plunder(), value);
        }
        if v.hero_count(who, SPITAMENES) != 0 {
            let (x, y) = v.this_coord();
            let (xs, ys) = v.type_size();
            let slack = ys.wrapping_div(2).wrapping_add(xs).wrapping_mul(0xc0);
            if v.find_hero(x, y, who, SPITAMENES, slack) >= 0 {
                hero = pct(v.thedespot_plunder(), value);
            }
        }
    }
    if v.is_scenario() && !v.scenario_plunder() {
        return;
    }
    let scale = Single::from_bits(v.plunder_scale(who));
    if scale.gt(Single::ZERO) || scale.eq_value(Single::ZERO) {
        value = Single::from_i32(value).mulss(scale).to_i32();
        hero = Single::from_i32(hero).mulss(scale).to_i32();
    }
    let owner = v.this_who();
    let (first, second);
    if v.has_tribe_bonus(owner, 0xd) && v.russian_plunder_steal() != 0 && who != owner {
        v.bucket_add(owner, good, value);
        v.bucket_add(who, good, hero);
        (first, second) = (hero, value);
    } else {
        let all = value.wrapping_add(hero);
        v.bucket_add(who, good, all);
        (first, second) = (all, 0);
    }
    if who == v.console_who() && first != 0 {
        let line = if who == owner { 0x898 } else { 0x8ac };
        message(v, owner, line, good, first, who);
    }
    if owner == v.console_who() && second != 0 {
        message(v, owner, 0x898, good, second, who);
    }
}
