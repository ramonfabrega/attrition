//! Territory, supply and attrition on the `Sim`: the holdings, the borders,
//! the supply slots and a unit's attrition step (`docs/ATTRITION.md`,
//! `docs/SUPPLY.md`; `docs/STRUCTURE.md` §3, `territory/`).
//!
//! Moved out of `lib.rs` unchanged by item 4014 (`docs/STRUCTURE.md` §5, move 6).

use crate::*;

impl Sim {
    /// `Leader::calc_attrition@006cdea0`: the attrition this player's
    /// territory inflicts, from the leading run of `ATTRITION1..4` held
    /// ([`tech::Roles::attrition_preq`]) and the Russian scaling. The
    /// original calls it from `gain_tech`, `calc_unit_stats` and
    /// `Leader::init`; here it runs from [`Sim::apply_gained`], which both
    /// tech paths reach.
    ///
    /// **Unwired until item 552.** Phase 1 built [`attrition::strength`]
    /// and tested it against hand-set [`attrition::PlayerState`]s, and
    /// nothing ever wrote `strength` from the tech tree, so every war-zone
    /// refresh came out at the sentinel 0. Golden chapter four's Allegiance
    /// at 550 gave the original's squad a period of 48 on block 601 and
    /// this crate's 0. The Colosseum and Kremlin arms stay false: this
    /// crate holds no wonder a player owns.
    pub(crate) fn calc_attrition(&mut self, who: Player) {
        let w = who as usize;
        if w >= self.players.len() {
            return;
        }
        let steps = self
            .tables
            .tech_tree
            .roles
            .attrition_preq
            .iter()
            .take_while(|row| {
                row.is_some_and(|t| {
                    self.tables
                        .tech_tree
                        .has_tech(&self.tables.setup, &self.tech[w], t)
                })
            })
            .count();
        let mods = attrition::StrengthMods {
            russian: self.nation.get(w).is_some_and(|n| n.russians),
            ..attrition::StrengthMods::default()
        };
        self.players[w].strength = attrition::strength(&self.tables.tuning, steps, &mods);
    }

    /// Gives a dead unit's supply slot back — `Unit::close` — and takes it
    /// out of both collision indices, which is `Object::remove_from_world`
    /// (`docs/COLLISION.md` §2, §3). This is every death path's tail.
    pub(crate) fn close_supply(&mut self, unit: usize) {
        self.close_supply_in(unit, false);
    }

    /// [`Sim::close_supply`], with `keep_disc` for `Object::close@00647160`'s
    /// `blocks_while_dead` arm: a combat death of a type that carries
    /// `uflags::BLOCKS_WHILE_DEAD` skips `remove_from_world`, and its
    /// occupancy disc stands (`docs/GOLDEN.md` §125).
    pub(crate) fn close_supply_in(&mut self, unit: usize, keep_disc: bool) {
        let owner = self.units[unit].owner as usize;
        if let Some(slot) = self.units[unit].supply_slot.take() {
            self.supply[owner].list.close(slot);
        }
        // `Unit::close@0060ee50`'s own: the route goes back with the unit.
        if let Some(slot) = self.units[unit].caravan.take() {
            let who = self.units[unit].owner;
            self.close_caravan(who, slot);
        }
        // `Unit::close@0060ee50`'s merchant arm: the leader's `0x2000000`
        // for any merchant, and the footprint `cast_unpack` blocked given
        // back when the trader is deployed — neither packed nor packing
        // (`docs/MERCHANT.md` §3.2).
        if self.is_merchant(unit) {
            let who = self.units[unit].owner;
            self.economy_changed(who);
            let packing = matches!(
                self.units[unit].orders.front().map(|o| &o.body),
                Some(orders::Body::Cast(c)) if orders::spell::is_pack(c.spell)
            );
            if !self.units[unit].combat.packed && !packing {
                self.merchant_footprint(unit, false);
            }
        }
        if keep_disc {
            self.units[unit].coll_at = None;
        } else {
            self.coll_remove(unit);
        }
        self.chain_remove(unit);
    }

    /// Whether anything of `owner`'s supplies a unit standing at `at`.
    ///
    /// A source's liveness is resolved here rather than stored, exactly as the
    /// original resolves it: the supply record holds an index and nothing
    /// else, so a wagon that dies or boards a transport stops supplying with
    /// no bookkeeping anywhere.
    pub fn supplied_at(&self, owner: Player, at: Pos) -> bool {
        let upgrades = self.supply_upgrade_level(owner);
        self.supply[owner as usize].supplies(&self.tables.tuning, upgrades, at, |u| {
            self.units.get(u).map(|w| supply::Wagon {
                pos: w.pos,
                active: w.alive(),
                on_map: w.on_map,
            })
        })
    }

    /// Copies the territory the border pass produced into the holdings the
    /// territory tax reads, and tells the world how much land there is.
    ///
    /// The original keeps both numbers itself — `LeaderData::territory` and
    /// `WorldData::land_size` — and the tax is their ratio. **The unit
    /// cancels**, so counting cells here where the original counts tiles gives
    /// the same wealth; what would not survive is counting one of them in one
    /// unit and the other in the other.
    ///
    /// Marks every player's economy dirty, since this is exactly the kind of
    /// change the original's flag exists for.
    pub fn update_territory_holdings(&mut self) {
        let mut land = 0;
        let mut owned = vec![0; self.players.len()];
        for (region, terrain) in self.world.regions().collect::<Vec<_>>() {
            if terrain != Terrain::Land {
                continue;
            }
            for cell in self.world.cells_in(region) {
                land += 1;
                if let Some(p) = self.world.owner(cell).player() {
                    owned[p as usize] += 1;
                }
            }
        }
        for (who, holdings) in self.holdings.iter_mut().enumerate() {
            holdings.territory = owned[who];
            holdings.land_size = land;
        }
        for l in &mut self.ledgers {
            l.dirty = true;
        }
    }

    /// The per-player half of the border bonus, read from live state —
    /// `World::compute_reg_territory@006b0bb0`'s own opening loop, which
    /// rebuilds all eight rows on every pass.
    ///
    /// The Civic level is `data_encrypted->epoch[1] ^ 0x63187`, the same
    /// field `LeaderData::get_city_limit` reads, and it feeds both
    /// `CIVIC_UPGRADE_TERR` and the Russians' per-step flat bonus.
    ///
    /// **The gem term is live**, and it is the one of the four seams a
    /// capture ever reached: Great Lakes' AI has a Merchant on the map's
    /// one Gems deposit by frame 23,999 (run80), so its border is wider
    /// than a gem-less table's. `compute_reg_territory`'s arm is an
    /// inlined `LeaderData::has_rare(`[`economy::GEMS`]`)` over
    /// `rare | rare_conquest`, which is what [`Sim::has_rare`] answers.
    ///
    /// **The temple and fort border levels** are `1` plus the highest of
    /// `TEMPLEBORDERS2..4` / `FORTBORDERS2..4` held
    /// ([`tech::Roles::temple_borders_preq`]), since item 552: chapter
    /// four's Religion at 400 moved run132's border 296 → 327 and this
    /// crate's not at all while the level was a constant 1.
    ///
    /// **The three wonders are `has_wonder`'s** (item 2106): Tikal
    /// (`0x214`), the Colosseum (`0x212`) and the Eiffel Tower (`0x21c`),
    /// as [`Sim::wonders_held`] answers. Tikal read `Nation::tikal`, which
    /// nothing wrote, and the other two a constant `false`.
    ///
    /// **Still a seam**: the AI handicap allowance, which is unreachable in
    /// any game run here (`docs/ATTRITION.md`, "Territory").
    pub(crate) fn player_borders(&self, who: Player) -> territory::PlayerBorders {
        let w = who as usize;
        let civic = self.tech[w].epoch[tech::Line::Civic.index()].max(0);
        let roles = &self.tables.tech_tree.roles;
        let temple_level = 1 + self.bonus_level(who, &roles.temple_borders_preq) as i32;
        let fort_level = 1 + self.bonus_level(who, &roles.fort_borders_preq) as i32;
        let n = &self.nation[w];
        let held = self.wonders_held(who);
        territory::PlayerBorders::new(
            &self.tables.tuning,
            civic as usize,
            temple_level,
            fort_level,
            &territory::Wonders {
                colosseum: held & (1 << tech::wonder::COLOSSEUM) != 0,
                tikal: held & (1 << tech::wonder::TIKAL) != 0,
                eiffel_tower: held & (1 << tech::wonder::EIFFEL_TOWER) != 0,
            },
            &territory::NationBonuses {
                roman: n.romans,
                russian: n.russians,
                gems: self.has_rare(who, economy::GEMS),
                civic,
            },
            0,
        )
    }

    /// Recomputes every border from scratch.
    ///
    /// Wholesale, which is what the original does at game setup and **not**
    /// what it does in play: there, `GameDaemon::check_borders` recomputes
    /// invalidated regions on a shared budget of 256 cells a frame, so a cell
    /// can carry stale ownership for up to `land cells / 256` frames after a
    /// city changes hands. In play this is the *target* a fix computes, and
    /// [`crate::border_pass`] copies it onto the map at that pace (item 1106,
    /// `docs/ATTRITION.md`, "Territory").
    pub fn recompute_territory(&mut self) {
        let players = u8::try_from(self.players.len()).expect("too many players");
        let by_source = territory::compute_all_territory_won(
            &mut self.world,
            &self.tables.tuning,
            &self.sources,
            players,
        );
        let mut city_of = vec![-1; self.sources.len()];
        for (c, city) in self.cities.iter().enumerate() {
            if let Some(si) = city.source.filter(|&si| si < city_of.len()) {
                city_of[si] = c as i32;
            }
        }
        self.territory_won = by_source
            .into_iter()
            .map(|si| usize::try_from(si).map_or(-1, |si| city_of[si]))
            .collect();
    }

    /// One unit's attrition for one frame — the refresh, the supply veto, and
    /// the damage.
    pub(crate) fn process_attrition(&mut self, i: usize, frame: i64) -> Option<Tick> {
        let phase = self.units[i].phase(frame);

        // The refresh. `process_attrition` clears the period on entry, so an
        // exempt unit comes out of it with nothing pending.
        if phase % ATTRITION_REFRESH_FRAMES == 0 {
            let outcome = self.attrition_for(i);
            let unit = &mut self.units[i];
            unit.sheltered = false;
            match outcome {
                attrition::Outcome::Exempt(_) => {
                    unit.attrition = 0;
                    unit.ignores_supply = false;
                }
                attrition::Outcome::Period {
                    frames,
                    ignores_supply,
                } => {
                    unit.attrition = frames;
                    unit.ignores_supply = ignores_supply;
                }
            }
        }

        let unit = &self.units[i];
        if unit.attrition == 0 || phase % i64::from(unit.attrition) != 0 {
            return None;
        }
        // Supply gets first refusal — `Unit::process_supply`. A unit inside a
        // friendly supply radius takes no attrition at all, which is the whole
        // reason an army can campaign abroad. The two refusals before the
        // search are the interesting ones: a peace or assassin bleed gives up
        // immediately, and militia and supply units are never sheltered.
        let sheltered = !unit.ignores_supply
            && unit.kind.shelterable()
            && self.supplied_at(unit.owner, unit.pos);

        if sheltered {
            self.units[i].sheltered = true;
            return None;
        }
        Some(self.suffer_attrition(i, frame))
    }

    /// **`Unit::suffer_attrition@005e1a10`**: one tick's damage, for the
    /// period's bleed above and a detected decoy's
    /// ([`Sim::recover_mana`]). Its argument gates the graphic alone.
    pub(crate) fn suffer_attrition(&mut self, i: usize, frame: i64) -> Tick {
        // The walk is [`suffer_view::suffer_attrition_steps`], the swept one
        // (item 3372): the damage is sixteenths of a hit point per figure,
        // carried through a fractional accumulator the way
        // `Object::take_damage` carries it (a lone figure loses one whole
        // point a tick, a figure in a squad of four one every fourth tick),
        // **and the squad is counted, not stored** (item 552): the view asks
        // `curr_uber_size`, which walks up to the captain and counts down the
        // chain, and a figure falls at its share of the squad's hit points
        // (`docs/COMBAT.md` §7.2 step 8, `docs/GOLDEN.md` §135, run936).
        let mut view = suffer_view::SimSuffer {
            sim: self,
            unit: i,
            outcome: None,
        };
        let killed = suffer_view::suffer_attrition_steps(&mut view, false);
        let (sixteenths, lost, _) = view.outcome.expect("the hurt call is always made");
        if killed {
            // `Object::take_damage`'s `die` with `dtype` 1 — the whole combat
            // tail, `Unit::close`'s draw and orders and the thirty-frame hold
            // on the number (`docs/COMBAT.md` §59.3, `docs/GOLDEN.md` §135).
            self.die_by_attrition(i, frame);
        }
        Tick {
            unit: i,
            frame,
            sixteenths,
            lost,
            killed,
        }
    }

    pub(crate) fn attrition_for(&self, i: usize) -> attrition::Outcome {
        let unit = &self.units[i];
        let ground = self.world.owner_at(unit.pos);
        let victim = &self.players[unit.owner as usize];
        let at_war = ground
            .player()
            .is_none_or(|o| self.at_war[unit.owner as usize][o as usize]);
        let situation = attrition::Situation {
            ground,
            in_free_zone: false,
            mutual_treaty: false,
            at_war,
            in_war_grace: false,
            assassin: false,
            conquest_exempt: false,
        };
        attrition::process(
            &self.tables.tuning,
            &unit.kind,
            unit.owner,
            victim,
            &situation,
            |p| self.players[p as usize],
        )
    }
}
