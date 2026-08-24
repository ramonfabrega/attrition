//! The production AI's driver over the simulation — `Leaders::strategy_all`,
//! `Leader::plan_strategy`'s cadence and `Leader::production_ai`'s step
//! machine, with [`crate::ai`]'s pure half doing the deciding.
//!
//! `docs/AI.md` §2.1–§2.5. What is wired today is the shape: every computer
//! leader takes the frame-0 sweep and the sweep on its own phase, the sweep
//! arms the step machine, and the machine runs one step a frame — the
//! script at step 1 (over [`crate::ai_host`]), the goods picture at step 2,
//! and the producers as the seams they are until the census lands (§12.1
//! item 5). The census itself, `check_orphaned_buildings`, `compute_sites`,
//! `check_explore`, `compute_score` and `diplomacy` are not stepped yet.

use crate::ai::{self, GoodsSetup, ScriptResult, Step, cadence};
use crate::world::Terrain;
use crate::{Player, Sim};

impl Sim {
    /// `world+0x34` as the AI reads it: the land regions.
    fn landmasses(&self) -> i32 {
        self.world
            .regions()
            .filter(|(_, t)| *t == Terrain::Land)
            .count() as i32
    }

    /// `Leader::init`'s AI tail for a computer leader (`docs/AI.md` §3
    /// item 1, §6): `prod_script_run = 1`, `random_personality`, then the
    /// script choice — [`ai::Leader::choose_script`]. `player_flags` is the
    /// lobby's per-player flags word (`GameInfo.player[i].flags`). Draws
    /// from the sync stream at the point `Setup::build_game` calls it.
    pub fn init_leader_ai(&mut self, who: Player, player_flags: u32) {
        let w = who as usize;
        let tribe = self.tech[w].tribe;
        let rivals: Vec<ai::Rival> = (0..self.players.len())
            .filter(|&i| {
                i != w
                    && !self.nation[i].human
                    && !self.defeated[i]
                    && !(self.allied[w][i] && self.allied[i][w])
            })
            .map(|i| ai::Rival {
                tribe: self.tech[i].tribe,
            })
            .collect();
        let setup = ai::RollSetup {
            landmasses: self.landmasses(),
            rush_rules_off: self.lobby.rush_rules == 8,
        };
        let pers = ai::Personality::roll(&mut self.rng, tribe, &rivals, &setup);
        let lakota = self
            .tech_tree
            .has_tribe_bonus(&self.setup, &self.tech[w], 0x13);
        let team = self.tech[w].team;
        let team_style = self.lobby.team_style;
        let leader = &mut self.ai[w];
        *leader = ai::Leader::new();
        leader.pers = pers;
        leader.choose_script(&mut self.rng, player_flags, team_style, team, lakota);
    }

    /// `Leaders::strategy_all@006ed430`: every active computer leader, every
    /// frame — `check_explore`, `plan_strategy`, `compute_score`,
    /// `diplomacy`. Only `plan_strategy` decides anything the simulation
    /// models; the other three are recounts and the diplomacy pass.
    pub fn strategy_all(&mut self) {
        for w in 0..self.players.len() {
            if self.nation[w].human || self.defeated[w] {
                continue;
            }
            self.plan_strategy(w as Player);
        }
    }

    /// `Leader::plan_strategy@006b9620`'s head (`docs/AI.md` §2.2): a
    /// running step machine pre-empts everything; otherwise the sweep at
    /// frame 0 and on the leader's phase, which ends by arming the machine.
    /// The cheap research tick between sweeps reads the make list's head;
    /// nothing fills the list yet, so it has nothing to buy.
    fn plan_strategy(&mut self, who: Player) {
        let w = who as usize;
        if self.ai[w].step != Step::Idle {
            self.production_ai(who);
            return;
        }
        if !cadence::sweep_due(w, self.frame, self.ai_speed) {
            return;
        }
        // The sweep: the census, the orphan check and `compute_sites` are
        // the next tranche's; its one act modelled today is the tail.
        self.ai[w].step = Step::Script;
    }

    /// `Leader::production_ai@006c1960` — one step a frame (`docs/AI.md`
    /// §2.4).
    fn production_ai(&mut self, who: Player) {
        let w = who as usize;
        let unlimited = self.lobby.resources_unlimited();
        self.ai[w].enter(unlimited);
        match self.ai[w].step {
            Step::Idle => {}
            Step::Script => {
                let name = self.ai[w].script.clone();
                let step = self.ai[w].script_step;
                let rush = self.ai[w].pers.rush;
                // `(who + 1, ref step, pers.rush + 2, num_loops = 5)`.
                let r = match name {
                    Some(n) => match self.run_script(who, &n, step, rush + 2, 5) {
                        Ok((ret, back)) => {
                            self.ai[w].script_step = back;
                            ScriptResult::Returned(ret)
                        }
                        Err(_) => ScriptResult::Failed,
                    },
                    None => ScriptResult::Failed,
                };
                self.ai[w].after_script(r);
            }
            Step::Setup => {
                let setup = self.goods_setup(who);
                let leader = &mut self.ai[w];
                ai::goods_picture(leader, &mut self.ledgers[w], &setup);
                // `market_speculation`: the market is not modelled.
                leader.make_list.clear();
                leader.after_producer();
            }
            Step::Cities
            | Step::Research
            | Step::Upgrades
            | Step::Units
            | Step::Buildings
            | Step::Units2 => {
                // `found_cities`, `research_techs`, `upgrade_units`,
                // `create_units`, `create_buildings`: the producers fill the
                // make list; none is built yet (§12.1 item 5).
                self.ai[w].after_producer();
                if unlimited {
                    self.ai[w].make_list.clear();
                }
            }
            Step::Make => {
                // `make_stuff` over an empty list buys nothing.
                self.ai[w].after_make(false, unlimited);
            }
            Step::Buildings2 | Step::Make2 => self.ai[w].after_second_pass(),
        }
    }

    /// The inputs `production_ai_setup` reads (`docs/AI.md` §2.5): the
    /// difficulty, the next age's price per good, the goods available and
    /// their caps.
    fn goods_setup(&self, who: Player) -> GoodsSetup {
        let w = who as usize;
        let p = &self.tech[w];
        let next_age = usize::try_from(p.ages)
            .ok()
            .and_then(|a| self.tech_tree.ages.get(a).copied().flatten());
        GoodsSetup {
            city_num: self.city_num(who),
            difficulty: self.lobby.difficulty,
            resources_unlimited: self.lobby.resources_unlimited(),
            lakota: self.tech_tree.has_tribe_bonus(&self.setup, p, 0x13),
            next_age_cost: next_age.map(|t| self.tech_price(who, t)),
            available: self.holdings[w].available,
            cap: self.ledgers[w].cap,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Tuning, World};

    /// A computer leader with no script takes the frame-0 sweep, walks the
    /// eleven steps one a frame and disarms; a human never enters.
    #[test]
    fn the_step_machine_runs_one_step_a_frame_from_the_sweep() {
        let mut sim = Sim::new(Tuning::RON, World::new(2, 2), 2);
        sim.nation[1].human = false;
        sim.ai[1].script_live = false;
        let mut steps = Vec::new();
        for _ in 0..12 {
            sim.tick();
            steps.push(sim.ai[1].step.number());
        }
        // Frame 0 arms; frame 1 enters as Script, skips to Setup and runs
        // it (→ 3); then one producer a frame to Make (8), which disarms.
        assert_eq!(steps, [1, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0]);
        assert_eq!(sim.ai[0].step, Step::Idle, "a human is never stepped");
        // The next sweep for who = 1 is frame 175.
        for _ in 12..175 {
            sim.tick();
        }
        assert_eq!(sim.ai[1].step, Step::Idle);
        sim.tick();
        assert_eq!(sim.ai[1].step, Step::Script, "armed at frame 175");
    }
}
