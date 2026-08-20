//! The combat table's generation — `docs/COMBAT.md` §5.
//!
//! The original builds its 493×493 percentage table once per data load from
//! two halves: a hardcoded chain of flag-pair multipliers
//! (`Balance::type_damage`, [`type_damage`] here) and a 399×399 category table
//! read from `balance.xml` and multiplied through every category pair
//! ([`xml_product`]). The simulation never reads this module at run time; it
//! takes the finished [`crate::combat::Table`]. `rondata` calls this against
//! the user's install to build that table, and a `RULES=1` logged start of the
//! original is the oracle it is checked against.
//!
//! Nothing here is the shipped data: the chain is the rule, and the numbers in
//! it are the original's literals.

use crate::attrition::Domain;
use crate::combat::mask;
use crate::tuning::Tuning;

/// The named lineages `type_damage` tests with `ObjectTypeData::is(x, strict)`.
/// A type carries the bits of every lineage it belongs to; a `_STRICT` bit is
/// the strict (`graft`-only) sense of the same name.
pub mod line {
    pub const ARMOREDCAR: u64 = 1 << 0;
    pub const LIGHTTANK: u64 = 1 << 1;
    pub const MACHINEGUN: u64 = 1 << 2;
    pub const FLAMETHROWER: u64 = 1 << 3;
    pub const FLAMETHROWER_STRICT: u64 = 1 << 4;
    pub const MILITIA: u64 = 1 << 5;
    /// `MERCHANT`, `MERCHANTDUTCH`, `FURTRAPPER` — by id, not lineage.
    pub const MERCHANT: u64 = 1 << 6;
    /// `PEASANTS`, `PEASANTSKOREAN`, `SCHOLARS`, `SCHOLARSKOREAN` — by id.
    pub const CITIZEN: u64 = 1 << 7;
    pub const BOMBARDSHIP: u64 = 1 << 8;
    pub const BARK: u64 = 1 << 9;
    pub const SUB: u64 = 1 << 10;
    pub const FIRERAFT: u64 = 1 << 11;
    pub const TRIREME: u64 = 1 << 12;
    pub const BOMBER: u64 = 1 << 13;
    pub const FIGHTERBOMBER: u64 = 1 << 14;
    pub const HELICOPTER: u64 = 1 << 15;
    pub const V2ROCKET: u64 = 1 << 16;
    pub const SUPPLYWAGON: u64 = 1 << 17;
    pub const AIRBASE: u64 = 1 << 18;
    pub const BALAMOBSLINGERS: u64 = 1 << 19;
    pub const KUSHITEARCHERS: u64 = 1 << 20;
    pub const KUSHITEARCHERS_STRICT: u64 = 1 << 21;
    pub const INTICLUBMEN: u64 = 1 << 22;
    pub const CAMELRANGE2: u64 = 1 << 23;
    pub const CHARIOT: u64 = 1 << 24;
    pub const NOMAD: u64 = 1 << 25;
    pub const RUSINYLANCER: u64 = 1 << 26;
    pub const LONGBOWMEN_STRICT: u64 = 1 << 27;
    pub const ELONGBOWMEN_STRICT: u64 = 1 << 28;
    pub const KINGSYEOMANRY_STRICT: u64 = 1 << 29;
    pub const ECOMPANION: u64 = 1 << 30;
    pub const LEGIONS: u64 = 1 << 31;
    pub const SAMURAI_STRICT: u64 = 1 << 32;
    pub const HALBERDIERS: u64 = 1 << 33;
    pub const TERCIOS: u64 = 1 << 34;
    pub const RECOILGUN: u64 = 1 << 35;
    pub const HIGHLANDERS: u64 = 1 << 36;
    pub const MG42_STRICT: u64 = 1 << 37;
    pub const TIGERTANK_STRICT: u64 = 1 << 38;
    pub const LEOPARDTANK_STRICT: u64 = 1 << 39;
    pub const FLAMINGARROW: u64 = 1 << 40;
    pub const BASILICABOMBARD: u64 = 1 << 41;
    pub const MORTAR: u64 = 1 << 42;
    /// The building lines `return_pack` sorts a building into.
    pub const FORT: u64 = 1 << 43;
    pub const CITY: u64 = 1 << 44;
    pub const LOOKOUT: u64 = 1 << 45;
    pub const TOWER: u64 = 1 << 46;
}

/// What `type_damage` and `return_pack` read of one type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Kind {
    pub masks: u32,
    pub age: i32,
    /// `TypeIndex` in `0x32..0x19e` — a unit (gaia included; gaia never reach
    /// the chain).
    pub unit: bool,
    /// `TypeIndex` in `0x19e..0x21f`.
    pub build: bool,
    /// `TypeIndex` in `0x20e..0x21f`.
    pub wonder: bool,
    pub gaia: bool,
    /// `UnitTypeData::is_siege` — `unit_flags & 0x20000`; false for buildings.
    pub siege: bool,
    /// `UnitTypeData::is_caravan` — `unit_flags2 & 8`; false for buildings.
    pub caravan: bool,
    pub domain: Domain,
    pub lines: u64,
    /// The unit's index in the unit table (`TypeIndex − 0x32`), for its own
    /// `balance.xml` row; `None` for a building.
    pub unit_index: Option<usize>,
}

impl Kind {
    const fn is(&self, l: u64) -> bool {
        self.lines & l != 0
    }
}

/// `v = (v * n) / 100`, truncating.
const fn pct(v: i32, n: i32) -> i32 {
    (v * n) / 100
}

/// `Balance::type_damage(a, b)` — the hardcoded half (§5.3), in execution
/// order.
pub fn type_damage(t: &Tuning, a: &Kind, b: &Kind) -> i32 {
    use line as l;
    use mask as m;
    let (am, bm) = (a.masks, b.masks);
    let mut v = 100;

    // 1. The age bonus.
    if a.unit && b.unit && b.age < a.age {
        let c = match a.age - b.age {
            1 => t.one_age_down,
            2 => t.two_ages_down,
            3 => t.three_ages_down,
            4 => t.four_ages_down,
            _ => t.five_ages_down,
        };
        v = ((c + 100) * 100) / 100;
    }
    // 2. LIGHT_INF.
    if am & m::LIGHT_INF != 0 {
        if bm & m::FOOT_ARCHER != 0 {
            v = pct(v, 162);
        }
        if bm & m::HORSE_ARCHER != 0 {
            v = pct(v, 152);
        }
    }
    // 3. HEAVY_INF.
    if am & m::HEAVY_INF != 0 {
        if am & m::GUN == 0 {
            if bm & m::MOUNTED != 0 {
                v = pct(v, 166);
            }
            if b.build {
                v = pct(v, 130);
            }
            if bm & m::FOOT_ARCHER != 0 {
                v = pct(v, 86);
            }
            if bm & m::MUSKET_INF != 0 {
                v = pct(v, 75);
            }
            if bm & m::MODERN_INF != 0 {
                v = pct(v, 75);
            }
        } else {
            if bm & m::MOUNTED != 0 {
                v = pct(v, 257);
            }
            if bm & m::HORSE_ARCHER != 0 {
                v = pct(v, 66);
            }
            if bm & m::LIGHT_CAV != 0 {
                v = pct(v, 114);
            }
            if b.is(l::ARMOREDCAR) {
                v = pct(v, 225);
            }
            if b.is(l::LIGHTTANK) {
                v = pct(v, 132);
            }
            if bm & m::MODERN_INF != 0 {
                v = pct(v, 80);
            }
        }
    }
    // 4. FOOT_ARCHER.
    if am & m::FOOT_ARCHER != 0 {
        if bm & m::LIGHT_INF != 0 {
            v = pct(v, 65);
        }
        if bm & m::MUSKET_INF != 0 {
            v = pct(v, 90);
        }
        if bm & m::HEAVY_INF != 0 {
            v = pct(v, 253);
        }
        if bm & m::HORSE_ARCHER != 0 {
            v = pct(v, 138);
        }
        if bm & m::SPARSE != 0 {
            v = pct(v, 80);
        }
        if b.build {
            v = pct(v, 33);
        }
        if b.siege {
            v = pct(v, 75);
        }
    }
    // 5. LIGHT_CAV.
    if am & m::LIGHT_CAV != 0 {
        if bm & m::FOOT_ARCHER != 0 {
            v = pct(v, 114);
        }
        if bm & m::LIGHT_INF != 0 {
            v = pct(v, 158);
        }
        if bm & m::MUSKET_INF != 0 {
            v = pct(v, 149);
        }
        if bm & m::MODERN_INF != 0 {
            v = pct(v, 159);
        }
        if b.is(l::MACHINEGUN) {
            v = pct(v, 130);
        }
        if b.is(l::FLAMETHROWER) {
            v = pct(v, 125);
        }
        if bm & m::SIEGE != 0 {
            v = pct(v, 122);
        }
        if bm & m::ARMORED != 0 {
            v = pct(v, 98);
        }
        if b.build {
            v = pct(v, 33);
        }
        if b.caravan || b.is(l::MERCHANT) {
            v /= 3;
        }
        if b.is(l::CITIZEN) || b.is(l::MILITIA) {
            v /= 2;
        }
    }
    // 6. HORSE_ARCHER.
    if am & m::HORSE_ARCHER != 0 {
        if b.build {
            v = pct(v, 33);
        }
        if b.siege {
            v = pct(v, 75);
        }
        if bm & m::LIGHT_CAV != 0 {
            v = pct(v, 57);
        }
        if bm & m::FOOT_ARCHER != 0 {
            v = pct(v, 57);
        }
        if bm & m::LIGHT_INF != 0 {
            v = pct(v, 66);
        }
        if bm & m::MUSKET_INF != 0 {
            v = pct(v, 66);
        }
        if bm & m::HEAVY_INF != 0 {
            v = pct(v, 160);
        }
        if am & m::VEHICLE != 0 {
            if bm & m::LIGHT_CAV != 0 {
                v = pct(v, 160);
            }
            if bm & m::FOOT_ARCHER != 0 {
                v = pct(v, 135);
            }
            if bm & m::LIGHT_INF != 0 {
                v *= 2;
            }
            if bm & m::MUSKET_INF != 0 {
                v *= 2;
            }
            if bm & m::HEAVY_INF != 0 {
                v = pct(v, 50);
            }
            if bm & m::HEAVY_CAV != 0 {
                v = pct(v, 97);
            }
        }
        if am & m::GUN != 0 {
            if b.is(l::MACHINEGUN) {
                v = pct(v, 155);
            }
            if bm & m::ARMORED != 0 {
                v = pct(v, 92);
            }
        }
    }
    // 7. HEAVY_CAV.
    if am & m::HEAVY_CAV != 0 {
        if bm & m::FOOT_ARCHER != 0 {
            v = pct(v, 170);
        }
        if bm & m::LIGHT_INF != 0 {
            v = pct(v, 179);
        }
        if bm & m::MUSKET_INF != 0 {
            v = pct(v, 155);
        }
        if bm & m::MODERN_INF != 0 {
            v = pct(v, 170);
        }
        if b.is(l::MACHINEGUN) {
            v = pct(v, 155);
        }
        if b.is(l::FLAMETHROWER) {
            v = pct(v, 140);
        }
        if b.build {
            v = pct(v, 33);
        }
    }
    // 8. MUSKET_INF.
    if am & m::MUSKET_INF != 0 {
        v = pct(v, 133);
        if bm & m::FOOT_ARCHER != 0 {
            v = pct(v, 125);
        }
        if bm & m::HEAVY_INF != 0 {
            v = pct(v, 180);
        }
        if bm & m::HORSE_ARCHER != 0 {
            v = pct(v, 114);
        }
        if b.build {
            v = pct(v, 66);
        }
        if b.siege {
            v = pct(v, 75);
        }
        if a.age == 4 && bm & m::HEAVY_INF != 0 {
            v = pct(v, 110);
        }
    }
    // 9. ARMORED.
    if am & m::ARMORED != 0 {
        if a.is(l::LIGHTTANK) {
            if bm & m::MUSKET_INF != 0 {
                v = pct(v, 155);
            }
            if bm & m::MODERN_INF != 0 {
                v = pct(v, 185);
            }
            if bm & m::HEAVY_CAV != 0 {
                v = pct(v, 115);
            }
            if bm & m::LIGHT_CAV != 0 {
                v = pct(v, 120);
            }
            if bm & m::HORSE_ARCHER != 0 {
                v = pct(v, 120);
            }
            if b.is(l::MACHINEGUN) {
                v = pct(v, 175);
            }
        }
        if a.is(l::ARMOREDCAR) {
            if bm & m::MODERN_INF != 0 {
                v = pct(v, 175);
            }
            if bm & m::MUSKET_INF != 0 {
                v = pct(v, 175);
            }
            if b.is(l::LIGHTTANK) {
                v = pct(v, 120);
            }
        }
        if b.build {
            v = pct(v, 66);
        }
    }
    // 10. MODERN_INF.
    if am & m::MODERN_INF != 0 {
        if bm & m::HEAVY_INF != 0 {
            v = pct(v, 260);
        }
        if b.is(l::ARMOREDCAR) {
            v = pct(v, 170);
        }
        if b.is(l::LIGHTTANK) {
            v = pct(v, 75);
        }
        if b.is(l::MACHINEGUN) {
            v = pct(v, 66);
        }
        if bm & m::CIVILIAN != 0 {
            v = pct(v, 120);
        }
        if b.siege {
            v = pct(v, 75);
        }
        if b.build {
            v = pct(v, 66);
        }
    }
    // 11. The machine-gun line.
    if a.is(l::MACHINEGUN) {
        if bm & (m::LIGHT_INF | m::MUSKET_INF) != 0 {
            v = pct(v, 330);
        }
        if bm & m::MODERN_INF != 0 {
            v = pct(v, 330);
        }
        if bm & m::HEAVY_INF != 0 {
            v = pct(v, 330);
        }
        if bm & m::CIVILIAN != 0 {
            v *= 5;
        }
        if bm & m::ARMORED != 0 {
            v = pct(v, 50);
        }
        if b.build {
            v = pct(v, 50);
        }
        if b.siege {
            v = pct(v, 75);
        }
    }
    // 12. Flamethrower, strict.
    if a.is(l::FLAMETHROWER_STRICT) {
        if bm & m::MUSKET_INF != 0 {
            v = pct(v, 88);
        }
        if bm & m::MODERN_INF != 0 {
            v = pct(v, 80);
        }
        if bm & m::MOUNTED != 0 {
            v = pct(v, 125);
        }
    }
    // 13. ARMORPIERCE.
    if am & m::ARMORPIERCE != 0 && bm & m::ARMORED != 0 {
        v = pct(v, 180);
    }
    // 14, 15. Militia and citizens double against mounted.
    if a.is(l::MILITIA) && bm & (m::MOUNTED | m::HORSE_ARCHER) != 0 {
        v *= 2;
    }
    if a.is(l::CITIZEN) && bm & (m::MOUNTED | m::HORSE_ARCHER) != 0 {
        v *= 2;
    }
    // 16. NAVAL.
    if am & m::NAVAL != 0 {
        if am & m::SIEGE == 0 && b.build {
            v = pct(v, 33);
        }
        if b.unit && bm & (m::NAVAL | m::AIR) == 0 && !b.siege {
            v = pct(v, 33);
        }
        if b.unit && bm & m::NAVAL == 0 && b.siege {
            v = pct(v, 66);
        }
        if a.is(l::BOMBARDSHIP) && bm & m::NAVAL != 0 {
            v = pct(v, 33);
        }
        if (a.is(l::BARK) || a.is(l::SUB))
            && bm & (m::NAVAL | m::CIVILIAN) == (m::NAVAL | m::CIVILIAN)
        {
            v *= 2;
        }
    }
    // 17–20. Fire rafts and triremes.
    if a.is(l::FIRERAFT) && b.is(l::TRIREME) {
        v = pct(v, 310);
    }
    if a.is(l::FIRERAFT) && b.is(l::BOMBARDSHIP) {
        v *= 4;
    }
    if a.is(l::FIRERAFT) && b.is(l::FIRERAFT) {
        v *= 4;
    }
    if a.is(l::TRIREME) && b.is(l::BARK) {
        v = pct(v, 160);
    }
    // 21. Land units against ships.
    if a.unit && am & m::NAVAL == 0 {
        if am & m::SIEGE == 0 && bm & m::NAVAL != 0 {
            v = pct(v, 33);
        }
        if a.siege && bm & m::NAVAL != 0 {
            v = pct(v, 350);
        }
    }
    // 22. Bombers against anti-air buildings.
    if (a.is(l::BOMBER) || a.is(l::FIGHTERBOMBER)) && b.build && bm & m::ANTI_AIR != 0 {
        v = pct(v, 25);
    }
    // 23. Anti-air aircraft.
    if am & m::AIR != 0 && am & m::MISSILE == 0 && am & m::ANTI_AIR != 0 {
        if b.build && bm & m::ANTI_AIR == 0 {
            v = pct(v, 15);
        }
        if b.build && bm & m::ANTI_AIR != 0 {
            v = pct(v, 85);
        }
        if b.is(l::BOMBER) {
            v *= 2;
        }
        if b.siege {
            v = pct(v, 125);
        }
        if bm & m::NAVAL != 0 {
            v = pct(v, 350);
        }
    }
    // 24, 25. Helicopters.
    if a.is(l::HELICOPTER) {
        if b.is(l::FIRERAFT) {
            v *= 8;
        }
        if b.is(l::LIGHTTANK) {
            v = pct(v, 450);
        }
    }
    if a.unit && am & m::ANTI_AIR == 0 && b.is(l::HELICOPTER) {
        v = pct(v, 25);
    }
    // 26. V2 against wonders.
    if a.is(l::V2ROCKET) && b.build && b.wonder {
        v /= 2;
    }
    // 27. Supply wagons.
    if am & m::MELEE != 0 && b.is(l::SUPPLYWAGON) {
        v = pct(v, 180);
    }
    if a.build && b.is(l::SUPPLYWAGON) {
        v = (v * 14) / 5;
    }
    // 28, 29. Siege and bombard against buildings.
    if am & m::SIEGE != 0 && b.build {
        v = pct(v, 430);
    }
    if am & m::BOMBARD != 0 && b.build {
        v = pct(v, 250);
    }
    // 30. Airbases.
    if (!matches!(a.domain, Domain::Land) || am & m::SIEGE != 0) && b.is(l::AIRBASE) {
        v /= 3;
    }
    // 31–33. Named archers and clubmen.
    if a.is(l::BALAMOBSLINGERS) && bm & m::LIGHT_INF != 0 {
        v = pct(v, 150);
    }
    if a.is(l::KUSHITEARCHERS) {
        if bm & m::FOOT_ARCHER != 0 {
            v = pct(v, 125);
        }
        if bm & m::HORSE_ARCHER != 0 {
            v = pct(v, 125);
        }
    }
    if a.is(l::INTICLUBMEN) {
        if bm & m::HEAVY_CAV != 0 {
            v = pct(v, 115);
        }
        if bm & m::LIGHT_CAV != 0 {
            v = pct(v, 115);
        }
        if bm & m::HORSE_ARCHER != 0 {
            v = pct(v, 115);
        }
    }
    // 34. Named horse archers.
    if am & m::HORSE_ARCHER != 0 {
        if a.is(l::CAMELRANGE2) {
            if bm & m::LIGHT_INF != 0 {
                v = pct(v, 135);
            }
            if bm & m::MUSKET_INF != 0 {
                v = pct(v, 120);
            }
        }
        if a.is(l::CHARIOT) && bm & m::HORSE_ARCHER != 0 {
            v = pct(v, 150);
        }
        if a.is(l::NOMAD) {
            if bm & m::LIGHT_INF != 0 {
                v = pct(v, 160);
            }
            if bm & m::MUSKET_INF != 0 {
                v = pct(v, 145);
            }
        }
    }
    // 35. Rusiny lancers.
    if am & m::LIGHT_CAV != 0 && a.is(l::RUSINYLANCER) {
        v = pct(v, 105);
    }
    // 36. Named foot archers.
    if am & m::FOOT_ARCHER != 0 {
        if a.is(l::LONGBOWMEN_STRICT) && bm & m::HEAVY_INF != 0 {
            v = pct(v, 130);
        }
        if a.is(l::ELONGBOWMEN_STRICT) && bm & m::HEAVY_INF != 0 {
            v = pct(v, 130);
        }
        if a.is(l::KINGSYEOMANRY_STRICT) && bm & m::HEAVY_INF != 0 {
            v = pct(v, 130);
        }
        if a.is(l::KUSHITEARCHERS_STRICT) && bm & m::FOOT_ARCHER != 0 {
            v = pct(v, 135);
        }
    }
    // 37. Companions.
    if am & m::HEAVY_CAV != 0 && a.is(l::ECOMPANION) {
        v = pct(v, 105);
    }
    // 38. Named heavy infantry.
    if am & m::HEAVY_INF != 0 {
        if a.is(l::LEGIONS) && bm & m::HEAVY_INF != 0 {
            v *= 2;
        }
        if a.is(l::SAMURAI_STRICT) {
            v = pct(v, 50);
        }
        if a.is(l::HALBERDIERS) {
            if bm & m::LIGHT_CAV != 0 {
                v = pct(v, 180);
            }
            if bm & m::HEAVY_CAV != 0 {
                v = pct(v, 180);
            }
            if bm & m::HORSE_ARCHER != 0 {
                v = pct(v, 110);
            }
        }
        if a.is(l::TERCIOS) {
            v = pct(v, 130);
        }
        if a.is(l::RECOILGUN) && bm & m::VEHICLE != 0 {
            v = pct(v, 135);
        }
    }
    // 39–42.
    if a.is(l::HIGHLANDERS) && bm & m::FOOT != 0 {
        v = pct(v, 130);
    }
    if a.is(l::MG42_STRICT) && bm & m::FOOT != 0 {
        v = pct(v, 130);
    }
    if a.is(l::TIGERTANK_STRICT) {
        if bm & (m::LIGHT_INF | m::MODERN_INF | m::MUSKET_INF) != 0 {
            v = pct(v, 115);
        }
        if b.is(l::MACHINEGUN) {
            v = pct(v, 120);
        }
    }
    if a.is(l::LEOPARDTANK_STRICT) {
        if bm & (m::LIGHT_INF | m::MODERN_INF | m::MUSKET_INF) != 0 {
            v = pct(v, 115);
        }
        if b.is(l::MACHINEGUN) {
            v = pct(v, 120);
        }
    }
    // 43. The only early return.
    if am & m::SIEGE == 0 {
        return v;
    }
    // 44–46. Named siege against buildings.
    if a.is(l::FLAMINGARROW) && b.build {
        v = pct(v, 135);
    }
    if a.is(l::BASILICABOMBARD) && b.build {
        v = pct(v, 110);
    }
    if a.is(l::MORTAR) && b.build {
        v = pct(v, 110);
    }
    v
}

/// The 399 `balance.xml` category indices a type belongs to, in the order
/// `compute_modifier` walks them (§5.1): its own unit row, its line, its
/// object class, its age, then one per `obj_masks` bit ascending.
pub fn categories(k: &Kind) -> Vec<usize> {
    let mut c = Vec::new();
    if k.unit
        && let Some(i) = k.unit_index
    {
        c.push(i);
    }
    let line = if k.unit {
        if k.siege { Some(0) } else { None }
    } else if k.is(line::FORT) {
        Some(1)
    } else if k.is(line::CITY) {
        Some(3)
    } else if k.is(line::LOOKOUT) {
        Some(4)
    } else if k.is(line::TOWER) {
        Some(2)
    } else {
        None
    };
    if let Some(l) = line {
        c.push(0x160 + l);
    }
    if k.unit || k.build {
        c.push(if k.unit { 0x166 } else { 0x165 });
    }
    if k.age >= 0 {
        c.push(0x167 + k.age as usize);
    }
    for bit in 0..32 {
        if k.masks & (1 << bit) != 0 {
            c.push(0x16f + bit);
        }
    }
    c
}

/// The XML half of one entry: `p = 100; for r in rows: for c in cols: p =
/// (xml(r, c) * p) / 100`, truncating each step.
pub fn xml_product(rows: &[usize], cols: &[usize], xml: impl Fn(usize, usize) -> i32) -> i32 {
    let mut p = 100;
    for &r in rows {
        for &c in cols {
            p = (xml(r, c) * p) / 100;
        }
    }
    p
}

/// `Balance::compute_modifier(a, b)`: 100 for any gaia type, else
/// `type_damage × xml_product / 100`.
pub fn compute_modifier(t: &Tuning, a: &Kind, b: &Kind, xml: impl Fn(usize, usize) -> i32) -> i32 {
    if a.gaia || b.gaia {
        return 100;
    }
    let p = xml_product(&categories(a), &categories(b), xml);
    (type_damage(t, a, b) * p) / 100
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: Tuning = Tuning::RON;

    fn unit(masks: u32, age: i32) -> Kind {
        Kind {
            masks,
            age,
            unit: true,
            domain: Domain::Land,
            unit_index: Some(0),
            ..Kind::default()
        }
    }

    fn building(masks: u32) -> Kind {
        Kind {
            masks,
            build: true,
            domain: Domain::Land,
            ..Kind::default()
        }
    }

    #[test]
    fn the_age_bonus_and_a_few_flag_pairs() {
        let hop = unit(mask::FOOT | mask::HEAVY_INF | mask::MELEE, 0);
        let arch = unit(mask::FOOT | mask::FOOT_ARCHER | mask::ARCHERY, 0);
        // Heavy infantry without guns against foot archers: ×86.
        assert_eq!(type_damage(&T, &hop, &arch), 86);
        // Foot archers against heavy infantry: ×253.
        assert_eq!(type_damage(&T, &arch, &hop), 253);
        // An age ahead: 115 first, then ×253 → 290.
        let arch2 = unit(arch.masks, 1);
        assert_eq!(type_damage(&T, &arch2, &hop), (115 * 253) / 100);
        // Five ages ahead caps at five.
        let arch7 = unit(arch.masks, 7);
        assert_eq!(type_damage(&T, &arch7, &hop), (170 * 253) / 100);
        // Behind: nothing.
        assert_eq!(type_damage(&T, &hop, &arch2), 86);
    }

    #[test]
    fn siege_against_buildings_and_the_early_return() {
        let cat = unit(mask::SIEGE | mask::BOMBARD | mask::WAR_MACHINE, 0);
        let mut cat = cat;
        cat.siege = true;
        let b = building(0);
        // SIEGE ×430, BOMBARD ×250: 430 → 1075.
        assert_eq!(type_damage(&T, &cat, &b), 1075);
        // A mortar line adds ×110 after the early-return gate only because
        // it has SIEGE.
        let mut mortar = cat;
        mortar.lines = line::MORTAR;
        assert_eq!(type_damage(&T, &mortar, &b), (1075 * 110) / 100);
        let mut not_siege = mortar;
        not_siege.masks &= !mask::SIEGE;
        assert_eq!(type_damage(&T, &not_siege, &b), 250);
    }

    #[test]
    fn naval_rules() {
        let ship = unit(mask::NAVAL | mask::LARGE, 0);
        let mut ship = ship;
        ship.domain = Domain::Sea;
        let land = unit(mask::FOOT, 0);
        // A ship against a land unit: ×33.
        assert_eq!(type_damage(&T, &ship, &land), 33);
        // A land unit against a ship: ×33; a siege land unit: ×350.
        assert_eq!(type_damage(&T, &land, &ship), 33);
        let mut siege = unit(mask::SIEGE, 0);
        siege.siege = true;
        assert_eq!(type_damage(&T, &siege, &ship), 350);
    }

    #[test]
    fn categories_follow_return_pack() {
        let mut k = unit(mask::FOOT | mask::HEAVY_INF, 2);
        k.unit_index = Some(40);
        assert_eq!(
            categories(&k),
            vec![40, 0x166, 0x167 + 2, 0x16f + 5, 0x16f + 7]
        );
        let mut s = k;
        s.siege = true;
        assert_eq!(categories(&s)[1], 0x160);
        let mut fort = building(mask::ARCHERY);
        fort.lines = line::FORT;
        fort.age = 1;
        assert_eq!(categories(&fort), vec![0x161, 0x165, 0x168, 0x16f + 17]);
        let mut tower = building(0);
        tower.lines = line::TOWER | line::CITY;
        // CITIES outranks TOWERS in the ladder.
        assert_eq!(categories(&tower)[0], 0x163);
    }

    #[test]
    fn the_xml_product_truncates_step_by_step_and_gaia_is_flat() {
        let xml = |r: usize, c: usize| {
            if r == 1 && c == 2 {
                66
            } else if r == 3 && c == 2 {
                150
            } else {
                100
            }
        };
        // 100 → ×66 → 66 → ×150 → 99.
        assert_eq!(xml_product(&[1, 3], &[2], xml), 99);
        // Order matters only through truncation: 100 → 150 → ×66 = 99 too
        // here, but 33 then 150: 33 → 49 vs 150 → 49.
        assert_eq!(xml_product(&[3, 1], &[2], xml), 99);
        let a = unit(mask::FOOT, 0);
        let mut g = a;
        g.gaia = true;
        assert_eq!(compute_modifier(&T, &a, &g, |_, _| 50), 100);
        assert_eq!(compute_modifier(&T, &a, &a, |_, _| 100), 100);
        // Every pair of categories contributes: a unit with one flag has
        // four categories (its row, UNITS, its age, the flag), so 4 × 4 = 16
        // multiplications of 90 %.
        let p = xml_product(&categories(&a), &categories(&a), |_, _| 90);
        let mut q = 100;
        for _ in 0..16 {
            q = (90 * q) / 100;
        }
        assert_eq!(p, q);
    }
}
