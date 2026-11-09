//! **`Groups::process@006fa210`'s driver** (item 3396): the leaders walk, the group each
//! in-use leader names this frame and the cursor, as a pure walk over a
//! [`GroupsProcessView`]. The per-group body is `Group::normalize@00711540` inlined, so it is
//! [`crate::normalize_view::normalize_steps`] over the view's group (`docs/GROUPS.md` §3.3, §19).
//! The sweep's port is a view over the emulated original's windows
//! (`sweep::groups_process_driver`, `sweep::groups_process_body`).
//!
//! **Not wired.** The crate's `Sim::groups_process` is a namesake. It takes the pool slot as
//! `frame mod 64` (the cursor starts at 0 and steps once a call, so the two agree on a run that
//! has called it every frame from the first), and walks the armies and pushed groups of each
//! player, where the original walks a table of eight leaders (`0xe3a390`, `0x6eec` apart; the
//! byte at +0, bit 0, is "in use") and, per in-use leader `k`, the group at index
//! `cursor + 64 * k` of the table at `[0xe85f20]` (`0x9d4` bytes a group). The prune is
//! `normalize`'s own, not the crate's alive-and-back-pointer test (the same namesake
//! `normalize_view` records). These steps are the original's reading, not the crate's
//! behaviour; no diff pin reads them.
//!
//! After the walk the cursor steps and wraps: `cursor += 1; if cursor > 63 { cursor = 0 }`.

/// Leaders in the table at `0xe3a390`: `(0xe71af0 - 0xe3a390) / 0x6eec`.
pub const LEADERS: usize = 8;
/// Groups a leader owns: the cursor's modulus, and the leader's stride in the group table.
pub const GROUPS_PER_LEADER: i32 = 64;

/// What `Groups::process` reads and does.
pub trait GroupsProcessView {
    /// Bit 0 of the byte at the start of leader `k`.
    fn leader_in_use(&self, k: usize) -> bool;
    /// `groups._64_4_` (`[0xe85f50]`).
    fn cursor(&self) -> i32;
    /// Writes `[0xe85f50]`.
    fn set_cursor(&mut self, c: i32);
    /// The inlined `Group::normalize` of the group at index `gid` of the table.
    fn normalize_group(&mut self, gid: i32);
}

/// `Groups::process(this)`: the steps.
pub fn groups_process_steps<V: GroupsProcessView>(v: &mut V) {
    for k in 0..LEADERS {
        if v.leader_in_use(k) {
            let gid = v.cursor() + GROUPS_PER_LEADER * k as i32;
            v.normalize_group(gid);
        }
    }
    let c = v.cursor() + 1;
    v.set_cursor(if c > GROUPS_PER_LEADER - 1 { 0 } else { c });
}
