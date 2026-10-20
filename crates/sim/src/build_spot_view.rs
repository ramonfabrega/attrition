//! **`Unit::find_build_spot@00603e20`'s steps** (item 3372): the whole function
//! as a pure walk over a [`BuildSpotView`], so the sweep's port is a view over
//! the emulated original's windows and the crate's is [`SimBuildSpot`]
//! (`docs/ORDERS.md` §5.5).
//!
//! The walk is the listing's, call for call:
//!
//! 1. the search range, `UNIT_BUILD_RESPOND_RANGE × 0xc0`, **doubled** when the
//!    worker stance reads 1 or 2 (two reads in the original, one value);
//! 2. `Objects::find_builds(SEARCH_FRIENDLY, …, 0x200, FILTER_CONSTRUCT)` into
//!    the objects' result array, and a pass over it keeping the sites whose
//!    `build_masks & 0x20` is clear and whose owner is the builder's;
//! 3. none kept → 0; else `Objects::find_units(…, FILTER_BUILDREPAIR)` **appends
//!    to the same array**, and a pass over the whole of it counts, for each
//!    element whose action is a `BUILD_AT` (kind 6) on a site of the builder's
//!    owner, one builder on the first kept site it names;
//! 4. the fewest builders, the first of a tie (strict `<` from index 0); a
//!    negative site id answers 0;
//! 5. a one-member `Group` takes `action_swarm_around(site, owner, QUEUE_LAST,
//!    BUILD_AT, 0)`.

/// The leaves of `find_build_spot`, one call or inline read each, asked in the
/// order the listing asks them.
pub trait BuildSpotView {
    /// `GameAccess::constants->unit_build_respond_range`.
    fn respond_range(&self) -> i32;
    /// The worker stance as `UnitData::get_worker_stance` reads it (a unit
    /// whose type has no stance type reads 0).
    fn worker_stance(&mut self) -> i32;
    /// `Objects::find_builds(SEARCH_FRIENDLY, owner, range, 0x200,
    /// FILTER_CONSTRUCT)`: fills the result array.
    fn find_builds(&mut self, range: i32);
    /// `Objects::find_units(SEARCH_FRIENDLY, owner, range, 0x200,
    /// FILTER_BUILDREPAIR)`: appends to the result array.
    fn find_units(&mut self, range: i32);
    /// The result array's length now.
    fn len(&self) -> usize;
    /// Element `i` as the first pass reads it (vtable `+0xac`): its site id
    /// when `build_masks & 0x20` is clear and its owner is mine.
    fn site(&mut self, i: usize) -> Option<i32>;
    /// Element `i` as the second pass reads it (vtable `+0xa8`, `get_action`,
    /// kind `+0x10 == 6`, target `+0xd8`): the target's site id when the
    /// action is a `BUILD_AT` and the target's owner is mine.
    fn building_at(&mut self, i: usize) -> Option<i32>;
    /// The one-member group: `Group::add`, then `action_swarm_around`.
    fn swarm(&mut self, site: i32);
}

/// `Unit::find_build_spot`'s walk; the answer is the original's `eax`.
pub fn build_spot_steps(v: &mut impl BuildSpotView) -> bool {
    let unit = v.respond_range().wrapping_mul(0xc0);
    let mut range = unit;
    let stance = v.worker_stance();
    if stance == 1 || stance == 2 {
        range = unit.wrapping_mul(2);
    }
    v.find_builds(range);
    let mut sites: Vec<i32> = Vec::new();
    for i in 0..v.len() {
        if let Some(s) = v.site(i) {
            sites.push(s);
        }
    }
    if sites.is_empty() {
        return false;
    }
    v.find_units(range);
    let mut counts = vec![0i32; sites.len()];
    for i in 0..v.len() {
        if let Some(t) = v.building_at(i)
            && let Some(k) = sites.iter().position(|&s| s == t)
        {
            counts[k] += 1;
        }
    }
    let mut best = 0;
    for i in 1..sites.len() {
        if counts[i] < counts[best] {
            best = i;
        }
    }
    let site = sites[best];
    if site < 0 {
        return false;
    }
    v.swarm(site);
    true
}

/// [`crate::build_spot_view::BuildSpotView`] over the sim: the result array
/// the original shares between its two searches is the construct sites and
/// then the friendly units, each already past the filter the caller applies
/// (`find_construct_sites` folds the mine/not-under-attack tests in).
pub struct SimBuildSpot<'a> {
    s: &'a mut crate::Sim,
    u: usize,
    sites: Vec<usize>,
    units: Vec<usize>,
}

impl<'a> SimBuildSpot<'a> {
    /// A view of unit `u`'s search over `s`.
    pub fn new(s: &'a mut crate::Sim, u: usize) -> Self {
        Self {
            s,
            u,
            sites: Vec::new(),
            units: Vec::new(),
        }
    }
}

impl BuildSpotView for SimBuildSpot<'_> {
    fn respond_range(&self) -> i32 {
        self.s.tuning.unit_build_respond_range
    }
    fn worker_stance(&mut self) -> i32 {
        i32::from(self.s.units[self.u].stance)
    }
    fn find_builds(&mut self, range: i32) {
        self.sites = self.s.find_construct_sites(self.u, range);
    }
    fn find_units(&mut self, range: i32) {
        self.units = self.s.find_units_friendly(self.u, range);
    }
    fn len(&self) -> usize {
        self.sites.len() + self.units.len()
    }
    fn site(&mut self, i: usize) -> Option<i32> {
        self.sites.get(i).map(|&b| b as i32)
    }
    fn building_at(&mut self, i: usize) -> Option<i32> {
        let o = *self.units.get(i.checked_sub(self.sites.len())?)?;
        let who = self.s.units[self.u].owner;
        let k = self.s.action_of(o)?;
        let crate::orders::Body::Build(b) = self.s.units[o].orders[k].body else {
            return None;
        };
        self.s.buildings.get(b).filter(|bd| bd.owner == who)?;
        Some(b as i32)
    }
    fn swarm(&mut self, site: i32) {
        let b = site as usize;
        self.s
            .swarm_around(self.u, b, crate::orders::Body::Build(b), false);
    }
}
