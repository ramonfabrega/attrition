//! **The border pass** — `GameDaemon::check_borders@00732060` over
//! `World::compute_reg_territory@006b0bb0`, the frames a border fix takes
//! to reach the map (`docs/ATTRITION.md`, "Territory"; item 1106).
//!
//! A fix (`Regions::fix_all_borders@0067f7d0`, every [`Sim::sync_territory`])
//! resets each region's resume index (`Region +0x2c`) to 0 and changes no
//! cell. The daemon's pass, once a frame before the object loop, then walks
//! the land regions in slot order and, in each, the region's cell list
//! (`Region +0x7c`, `Regions::rebuild_coords`' row-major order) from its
//! index: every cell it reaches has its owner byte (`WorldData +0x134`,
//! `+0xf`) and runner-up (`+0x10`) rewritten from the sources as they stand.
//! **The frame's budget is 256 cells across every region**
//! (`GameDaemon::borders`, zeroed at the pass's head and bumped per cell,
//! `0xff < borders` ending a region's turn). A region that reaches its end
//! raises every live leader's economy flag `0x2000000`; the frame the last
//! one does, each leader's `territory` is summed again.
//!
//! This crate computes the owners the pass will write at the fix itself —
//! the sources change only through a fix, and a second fix restarts the pass
//! from 0 on a new target — and copies them onto the visible grid at the
//! pass's own pace. Until then a cell reads its old owner. East Indies'
//! citizen `1/14` finishes the AI's third city on tick 5517; its cell is
//! region 5's 113th, which the pass reaches on tick 5521, and the colonist
//! gate of `think_civilian_transport` reads that byte (`docs/AI.md` §84).

use crate::Sim;
use crate::world::{Owner, Terrain};

/// The fixed pass's own state: the owners it writes and every region's
/// resume index.
#[derive(Clone, Debug, Default)]
pub struct BorderPass {
    /// `(who, who2)` of every cell as the fix's sources claim it.
    pub target: Vec<(Owner, Owner)>,
    /// `Region +0x2c`, by this crate's region index; sea regions never run.
    pub index: Vec<i32>,
    /// [`Sim::territory_won`] as the fix computed it: the city each cell's
    /// winning claim came from, whose `bordering` its runner-up marks.
    pub won: Vec<i32>,
}

/// `compute_reg_territory`'s budget: `if (0xff < game_daemon->borders)`.
pub const CELLS_PER_FRAME: i32 = 0x100;

impl Sim {
    /// Starts the pass on a new target: the visible owners are put back as
    /// they were before the wholesale recompute that produced it.
    pub(crate) fn start_border_pass(&mut self, before: Vec<(Owner, Owner)>) {
        if !self.in_play {
            // Setup's pass, before the first frame: wholesale, and any pass
            // an earlier setup fix left is superseded by this target.
            self.border_pass = None;
            let target = self.world.owners();
            let won = self.territory_won.clone();
            self.wholesale_bordering(&target, &won);
            self.update_territory_holdings();
            self.claim_oil_from_owners();
            return;
        }
        let target = self.world.owners();
        self.world.set_owners(&before);
        let index = vec![0; self.world.region_count()];
        let won = self.territory_won.clone();
        self.border_pass = Some(BorderPass { target, index, won });
    }

    /// Every city's `bordering` back to zero — `compute_reg_territory:92`,
    /// which every land region's first call makes (its resume index at 0)
    /// **before** its budget test, so the last region to start wins.
    fn clear_bordering(&mut self) {
        for leader in &mut self.ai {
            for c in &mut leader.city_ai {
                c.bordering = 0;
            }
        }
    }

    /// `compute_reg_territory:595`–`607`, at a cell it has just written: the
    /// winner's city (`local_1c`) takes the winner's and the runner-up's
    /// bits in its `bordering` (`+0x65`) when both are players and either
    /// declares war on the other (`diplos == 0` on a side). A fort's win, a
    /// runner-up that is none or `Ambiguous`, or the same player twice, marks
    /// nothing. The `city_flags & 0x1000` arm beside it (a cell within four
    /// cells of the city) is not modelled: no capture reads it.
    fn mark_bordering(&mut self, who: Owner, who2: Owner, won: i32, cell: crate::world::Cell) {
        let (Some(p), Some(q)) = (who.player(), who2.player()) else {
            return;
        };
        let Ok(c) = usize::try_from(won) else {
            return;
        };
        if p == q || !(self.at_war_with(p, q) || self.at_war_with(q, p)) {
            return;
        }
        let owner = self.cities[c].owner as usize;
        let bits = (1i32 << (p & 31)) | (1i32 << (q & 31));
        let at = self.cities[c].pos.cell();
        let near = (cell.x - at.x).abs() + (cell.y - at.y).abs() < 5;
        if let Some(a) = self.ai.get_mut(owner) {
            // A human's leader never sweeps, so its table starts short.
            if a.city_ai.len() <= c {
                a.city_ai.resize(c + 1, crate::ai::CityAi::default());
            }
            let rec = &mut a.city_ai[c];
            rec.bordering = (rec.bordering | bits) & 0xff;
            // `compute_reg_territory@006b0bb0:612`: a marked cell within
            // Manhattan distance four of the city sets `city_flags &
            // 0x1000`, which nothing in the pass clears.
            if near {
                rec.near_border = true;
            }
        }
    }

    /// Every leader's `reg_known_rares[r]` back to zero —
    /// `compute_reg_territory@006b0bb0:78`–`101`, the same block as
    /// [`Self::clear_bordering`]: while region `r`'s resume index is 0 its
    /// call zeroes the slot **before** the budget test, so a region the
    /// pass has not yet reached is zeroed again on every frame until its
    /// first cell is written. A census that falls inside the pass — the
    /// leaders' sweep runs before `check_borders` in the frame — is undone
    /// for every region still waiting (`docs/AI.md` §318.1; East Indies'
    /// who=1 on tick 5575, run1241). The block's third write, the
    /// leader's `ushort` at `+0x145e + 2r`, has no field in this crate.
    fn zero_region_rares(&mut self, r: u16) {
        for leader in &mut self.ai {
            if let Some(slot) = leader.census.reg_known_rares.get_mut(usize::from(r)) {
                *slot = 0;
            }
        }
    }

    /// Setup's `compute_all_territory`: every land region in slot order,
    /// each clearing every city's `bordering` and then marking its own
    /// cells.
    fn wholesale_bordering(&mut self, target: &[(Owner, Owner)], won: &[i32]) {
        let lands: Vec<u16> = self
            .world
            .regions()
            .filter(|&(_, t)| t == Terrain::Land)
            .map(|(r, _)| r)
            .collect();
        for r in lands {
            self.clear_bordering();
            for c in self.world.region_coords_strided(r, 0, 1) {
                let i = (c.y * self.world.width() + c.x) as usize;
                if let (Some(&(who, who2)), Some(&w)) = (target.get(i), won.get(i)) {
                    self.mark_bordering(who, who2, w, c);
                }
            }
        }
    }

    /// Finishes a pass at once — the wholesale recompute the original makes
    /// at setup, before the first frame, and what a test that wants a fix
    /// visible on the same frame asks for.
    pub fn settle_borders(&mut self) {
        if let Some(pass) = self.border_pass.take() {
            self.world.set_owners(&pass.target);
            self.wholesale_bordering(&pass.target, &pass.won);
            self.update_territory_holdings();
            self.claim_oil_from_owners();
        }
    }

    /// One frame of the pass (`check_borders`' region loop): up to
    /// [`CELLS_PER_FRAME`] cells, land regions in order, each from its
    /// resume index.
    pub(crate) fn advance_border_pass(&mut self) {
        let Some(mut pass) = self.border_pass.take() else {
            return;
        };
        let mut budget = 0;
        let mut finished_one = false;
        let lands: Vec<u16> = self
            .world
            .regions()
            .filter(|&(_, t)| t == Terrain::Land)
            .map(|(r, _)| r)
            .collect();
        let mut open = false;
        for r in lands {
            let size = self.world.region_size(r);
            let Some(at) = pass.index.get_mut(r as usize) else {
                continue;
            };
            if size == 0 || *at >= size {
                continue;
            }
            if *at == 0 {
                self.clear_bordering();
                self.zero_region_rares(r);
            }
            if budget < CELLS_PER_FRAME {
                let cells = self
                    .world
                    .region_coords_strided(r, *at, 1)
                    .into_iter()
                    .take((CELLS_PER_FRAME - budget) as usize);
                for c in cells {
                    let i = (c.y * self.world.width() + c.x) as usize;
                    let (who, who2) = pass.target[i];
                    self.world.set_owner(c, who, who2);
                    if let Some(&w) = pass.won.get(i) {
                        self.mark_bordering(who, who2, w, c);
                    }
                    if let Some(p) = who.player() {
                        // `compute_reg_territory`'s per-cell goods scan: an
                        // oil patch on a cell it has just given away joins
                        // the owner's `oil_patches` (`docs/AI.md` §133).
                        self.claim_cell_goods(c, p);
                    }
                    *at += 1;
                    budget += 1;
                }
            }
            if *at >= size {
                // `compute_reg_territory`'s tail: `leader_flags |= 0x2000000`
                // on every live leader, the economy's recompute flag.
                finished_one = true;
                for l in &mut self.ledgers {
                    l.dirty = true;
                }
            } else {
                open = true;
            }
        }
        if open {
            self.border_pass = Some(pass);
        } else if finished_one {
            // `check_borders`' tail: every region done, so `territory` is
            // the sum of the regions' counts again.
            self.update_territory_holdings();
        }
    }
}

#[cfg(test)]
mod tests;
