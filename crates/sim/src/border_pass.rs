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
            self.update_territory_holdings();
            self.claim_oil_from_owners();
            return;
        }
        let target = self.world.owners();
        self.world.set_owners(&before);
        let index = vec![0; self.world.region_count()];
        self.border_pass = Some(BorderPass { target, index });
    }

    /// Finishes a pass at once — the wholesale recompute the original makes
    /// at setup, before the first frame, and what a test that wants a fix
    /// visible on the same frame asks for.
    pub fn settle_borders(&mut self) {
        if let Some(pass) = self.border_pass.take() {
            self.world.set_owners(&pass.target);
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
mod tests {
    use super::*;
    use crate::world::{Cell, World};

    /// A world of `n` land regions, each one row of `w` cells.
    fn rows(w: i32, n: i32) -> World {
        let mut world = World::new(w, n);
        for y in 0..n {
            world.fill_region(Terrain::Land, Cell::new(0, y), Cell::new(w - 1, y));
        }
        world
    }

    /// **The pass writes 256 cells a frame, regions in order, each from its
    /// resume index**, and a cell it has not reached keeps its old owner:
    /// the shape run407's packet reads on East Indies' 5518..5521.
    #[test]
    fn a_fix_reaches_the_map_at_256_cells_a_frame_in_region_order() {
        let mut sim = Sim::new(crate::Tuning::RON, rows(200, 3), 2);
        sim.in_play = true;
        let before = sim.world.owners();
        let mut target = before.clone();
        target.fill((Owner::Player(1), Owner::None));
        sim.world.set_owners(&target);
        sim.start_border_pass(before);
        assert_eq!(
            sim.world.owner(Cell::new(0, 0)),
            Owner::None,
            "a fix writes no cell"
        );
        sim.advance_border_pass();
        // Frame one: region 0 whole (200) and region 1's first 56.
        assert_eq!(sim.world.owner(Cell::new(199, 0)), Owner::Player(1));
        assert_eq!(sim.world.owner(Cell::new(55, 1)), Owner::Player(1));
        assert_eq!(
            sim.world.owner(Cell::new(56, 1)),
            Owner::None,
            "the budget ends it"
        );
        assert_eq!(sim.border_pass.as_ref().unwrap().index, vec![200, 56, 0]);
        sim.advance_border_pass();
        // Frame two: region 1's other 144, region 2's first 112.
        assert_eq!(sim.border_pass.as_ref().unwrap().index, vec![200, 200, 112]);
        assert_eq!(sim.world.owner(Cell::new(111, 2)), Owner::Player(1));
        assert_eq!(sim.world.owner(Cell::new(112, 2)), Owner::None);
        sim.advance_border_pass();
        assert!(sim.border_pass.is_none(), "the third frame finishes it");
        assert_eq!(sim.world.owner(Cell::new(199, 2)), Owner::Player(1));
    }
}
