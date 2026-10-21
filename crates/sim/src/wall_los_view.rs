//! **`Wall::update_los@0063eeb0`'s steps** (item 3391): a building's line of
//! sight as a pure walk over a [`WallLosView`]. The swept steps read the world
//! only through the view, so the sweep's port is a view over the emulated
//! original's windows (`docs/VISION.md` §2.1).
//!
//! Not wired: [`crate::Sim::build_los`] carries the finished building's
//! `LOS + epoch * SCIENCE_LOS + x_size / 2` and none of the fort, tower,
//! colosseum and furs terms, which need tech bits, a wonder test and the
//! constants block the crate does not model. This is the whole function, so
//! the sweep pins what the crate would have to carry.
//!
//! Every sum is in the **byte** the original keeps at `+0x3c`: the products
//! and the additions wrap at 8 bits, and the answer is that byte
//! sign-extended.

/// What `Wall::update_los` asks of the building, its owner and the constants.
pub trait WallLosView {
    /// Vslot `+0x50`, `WallData::is_started` (`flags & 2`).
    fn is_started(&mut self) -> bool;
    /// Vslot `+0x4c`, `WallData::is_active` (`flags & 4`).
    fn is_active(&mut self) -> bool;
    /// Vslot `+0x2c`: `BuildData::is_wonder` on a `Build`.
    fn is_wonder(&mut self) -> bool;
    /// `leaders[who]`'s first byte, bit 0.
    fn leader_live(&self) -> bool;
    /// The type's `LOS`, the byte at `+0x21c`.
    fn type_los(&self) -> i8;
    /// The type's `SCIENCE_LOS`, the byte at `+0x220`.
    fn type_science_los(&self) -> i8;
    /// The type's `X_SIZE`, the dword at `+0x234`.
    fn type_x_size(&self) -> i32;
    /// The byte at `+0xf4` of the record `leaders[who] +0x6eb8` points at,
    /// before the `^ 0x87`: the science epoch, stored encrypted.
    fn science_epoch_raw(&self) -> u8;
    /// The type's vslot `+0xfc`: a fortress.
    fn is_fort(&mut self) -> bool;
    /// This building's vslot `+0xb8` (`ObjectData::is(0x1b7, 0)`, inline the
    /// type's slot `+0x60`): a tower.
    fn is_tower(&mut self) -> bool;
    /// `LeaderData::get_fort_los`, 1 to 4.
    fn fort_los(&mut self) -> i32;
    /// `LeaderData::get_tower_fort_los`, 1 to 4.
    fn tower_fort_los(&mut self) -> i32;
    /// `LeaderData::has_wonder(0x212)`.
    fn has_colosseum(&mut self) -> bool;
    /// Low byte of `constants.fort_upgrade_range[i]`.
    fn fort_upgrade_range(&self, i: i32) -> i8;
    /// Low byte of `constants.tower_fort_range[i]`.
    fn tower_fort_range(&self, i: i32) -> i8;
    /// Low byte of `constants.colosseum_fort_range`.
    fn colosseum_fort_range(&self) -> i8;
    /// Low byte of `constants.furs_los`.
    fn furs_los(&self) -> i8;
    /// Bit 7 of `leaders[who] +0x6da5` or `+0x6dcd`.
    fn has_furs(&self) -> bool;
}

/// The byte `Wall::update_los` leaves at `+0x3c`; its return is the same byte
/// sign-extended.
pub fn wall_update_los<V: WallLosView>(v: &mut V) -> i8 {
    if !v.is_started() {
        return 0;
    }
    if !v.is_active() && !v.is_wonder() {
        return 0;
    }
    let mut los: i8;
    if !v.is_active() {
        // A wonder site: the base is one, then the footprint.
        los = 1;
    } else {
        let base: i8 = if v.leader_live() { v.type_los() } else { 0 };
        let sci = i32::from((v.science_epoch_raw() ^ 0x87) as i8) * i32::from(v.type_science_los());
        los = (sci as u8).wrapping_add(base as u8) as i8;
        let ranged = if v.is_fort() {
            let i = v.fort_los();
            los = los.wrapping_add(v.fort_upgrade_range(i + 4));
            true
        } else if v.is_tower() {
            let i = v.tower_fort_los();
            los = los.wrapping_add(v.tower_fort_range(i + 3));
            true
        } else {
            false
        };
        // After the range term the fort test and the tower test are asked
        // again; the colosseum is added when either holds.
        if ranged && (v.is_fort() || v.is_tower()) && v.has_colosseum() {
            los = los.wrapping_add(v.colosseum_fort_range());
        }
        if v.has_furs() {
            los = los.wrapping_add(v.furs_los());
        }
    }
    los.wrapping_add((v.type_x_size() / 2) as i8)
}
