//! **`Group::action_form@00707220`'s steps** (item 3379): a formation
//! command, as a pure walk over an [`ActionFormView`] (`docs/ORDERS.md`
//! §30). The swept steps read the world only through the view, so the
//! sweep's port is a view over the emulated original's windows and the
//! crate's is [`SimActionForm`].
//!
//! **The fourth stack argument is dead.** `finish_insert`'s case `0x12` calls
//! `action_form(form, 0, QUEUE_LAST, <leftover ecx>, 1)` (parked 3372f): the
//! leftover lands in the *fourth* slot (`[ebp + 0x14]`), which the listing
//! never reads, and the queue position is the plain `QUEUE_LAST` (3372's note
//! that `action_form` "takes it as its queue position" was a miscount of the
//! pushes). The sweep draws it at random.
//!
//! The listing's flow, leaf for leaf: the group must be on the map, be a unit
//! group (`+0x49` clear) with members and a leader that is no plane. Then
//! the formation is named: `-3` and `-1` step `get_form_option` back and
//! forward through the five buttons (`-3` falling to 4 below zero, `-1`
//! modulo 5), `-2` keeps the leader's own byte (the group's `form` is set to
//! `-2` and the insert dance is skipped). A `QUEUE_NEW` or `QUEUE_FIRST`
//! call that is not itself the dance's recursion copies the leader's action
//! orders aside, halts, and calls itself with `QUEUE_FIRST` where something
//! was copied and `QUEUE_NEW` where not. Every member whose object answers
//! slot `+0x18` and whose unit is no plane takes the byte `+0xaa`. At
//! `QUEUE_NEW` and `QUEUE_LAST` the group then moves to `get_loc_to`.

/// `QUEUE_NEW`, `QUEUE_LAST`, `QUEUE_FIRST` in the original's numbering
/// (`QUEUE_FIRST` 0, `QUEUE_LAST` 1, `QUEUE_NEW` 2: the listing's `cmovne`).
pub const QUEUE_NEW: i32 = 2;
/// `QUEUE_LAST`.
pub const QUEUE_LAST: i32 = 1;
/// `QUEUE_FIRST`.
pub const QUEUE_FIRST: i32 = 0;
/// `MOVE_TO`.
pub const MOVE_TO: i32 = 1;

/// What `Group::action_form` asks of the group, the objects and the units.
/// `i` is a unit's index (the group's short at `+0x8cc`, or an answer).
pub trait ActionFormView {
    /// `GroupData::is_on_map`.
    fn is_on_map(&mut self) -> bool;
    /// Slot `+0x14` of the group: its refresh.
    fn refresh(&mut self);
    /// `[this + 0x49] != 0`: a building group.
    fn buildings(&mut self) -> bool;
    /// `[this + 0xc]`.
    fn count(&mut self) -> i32;
    /// `GroupData::find_leader(0)`.
    fn find_leader(&mut self) -> i32;
    /// Slot `+0xc0` of `units[who][i]`: the unit is a plane.
    fn unit_plane(&mut self, i: i32) -> bool;
    /// `GroupData::get_form_option(0)`.
    fn get_form_option(&mut self) -> i32;
    /// `this + 0x10 = v`: the group's formation.
    fn set_group_form(&mut self, v: i32);
    /// `GroupData::get_unit`.
    fn get_unit(&mut self) -> i32;
    /// The signed byte `units[who][i] + 0xaa`.
    fn unit_form(&mut self, i: i32) -> i32;
    /// The short `k` of the group's array at `+0x8cc`.
    fn member(&mut self, k: i32) -> i32;
    /// Slot `+0x18` of `objects[who][i]`: the object is a unit.
    fn obj_unit(&mut self, i: i32) -> bool;
    /// `units[who][i] + 0xaa = v` (the low byte).
    fn set_unit_form(&mut self, i: i32, v: i32);
    /// `OrderList::OrderList`: the insert dance's list, empty.
    fn new_list(&mut self);
    /// `Group::set_up_insert(list)`.
    fn set_up_insert(&mut self);
    /// `Group::action_halt(0)`.
    fn action_halt(&mut self);
    /// The list's count (`+0x10`) is not zero.
    fn list_pending(&mut self) -> bool;
    /// `Group::finish_insert(list)`.
    fn finish_insert(&mut self);
    /// `LinkListBase::clear` on the list.
    fn clear_list(&mut self);
    /// `GroupData::get_loc_to`: the point the group will end up at, or
    /// `None` where it writes neither (no leader).
    fn loc_to(&mut self) -> Option<(i32, i32)>;
    /// The group's `+0x18`, `+0x1c`: its `ox`, `oy`.
    fn o_xy(&mut self) -> (i32, i32);
    /// The group's `+0x24`: its `o_angle`.
    fn o_angle(&mut self) -> i32;
    /// The word `units[who][i] + 0x50`: the unit's heading.
    fn unit_heading(&mut self, i: i32) -> i32;
    /// `Group::action_move_to`, its ten stack arguments.
    fn action_move_to(&mut self, a: [i32; 10]);
}

/// `Group::action_form(form, rotate, queue, _, inserting)`: the steps.
pub fn action_form_steps<V: ActionFormView>(
    v: &mut V,
    mut form: i32,
    rotate: i32,
    queue: i32,
    inserting: i32,
) {
    if !v.is_on_map() {
        return;
    }
    v.refresh();
    if v.buildings() || v.count() < 1 {
        return;
    }
    let leader = v.find_leader();
    if leader < 0 || v.unit_plane(leader) {
        return;
    }
    // `-3` and `-1` step the button; the result is tested against `-2`
    // whichever way it was reached, and a non-negative `-3` result skips the
    // wrap to 4.
    let mut keep = false;
    if form == -3 {
        form = v.get_form_option() - 1;
        if form < 0 {
            form = 4;
        }
    } else if form == -1 {
        form = (v.get_form_option() + 1) % 5;
    }
    if form == -2 {
        keep = true;
        v.set_group_form(-2);
        let u = v.get_unit();
        if u < 0 {
            return;
        }
        form = v.unit_form(u);
    }
    if !keep && (queue == QUEUE_NEW || queue == QUEUE_FIRST) && inserting == 0 {
        v.new_list();
        v.set_group_form(-1);
        v.set_up_insert();
        v.action_halt();
        let q = if v.list_pending() {
            QUEUE_FIRST
        } else {
            QUEUE_NEW
        };
        action_form_steps(v, form, rotate, q, 1);
        v.finish_insert();
        v.clear_list();
        return;
    }
    let n = v.count();
    for k in 0..n {
        let i = v.member(k);
        if v.obj_unit(i) && !v.unit_plane(i) {
            v.set_unit_form(i, form);
        }
    }
    if queue != QUEUE_LAST && queue != QUEUE_NEW {
        return;
    }
    let (mut x, mut y) = (queue, form);
    if let Some((lx, ly)) = v.loc_to() {
        (x, y) = (lx, ly);
    }
    if rotate == 0 {
        v.action_move_to([x, y, QUEUE_LAST, 0, 0, MOVE_TO, 0, -1, -1, 0]);
        return;
    }
    let (ox, oy) = v.o_xy();
    let base = if x == ox && y == oy {
        v.o_angle()
    } else {
        let mut l = 0;
        if v.count() > 0 {
            let i = if v.buildings() {
                v.member(0)
            } else {
                v.find_leader()
            };
            if i >= 0 {
                l = v.unit_heading(i);
            }
        }
        l
    };
    v.action_move_to([
        x,
        y,
        QUEUE_LAST,
        1,
        base.wrapping_add(rotate),
        MOVE_TO,
        0,
        -1,
        -1,
        0,
    ]);
}

/// The crate's view: the group's members, units and insert copies.
///
/// - Every group in the simulation is a unit group (`buildings` false) and
///   every member's object answers slot `+0x18`.
/// - SEAM: `get_form_option`'s global `counts[5]` side effect is not modelled
///   (nothing reads it); its `*out` flag is not asked here.
/// - `loc_to` is `None` only where the group has no leader, which the head
///   of the steps has already ruled out.
pub struct SimActionForm<'a> {
    /// The simulation.
    pub sim: &'a mut crate::Sim,
    /// The group.
    pub g: &'a crate::group::Group,
    insert: Option<crate::group::Insert>,
}

impl<'a> SimActionForm<'a> {
    /// A view over `sim` and `g`.
    pub fn new(sim: &'a mut crate::Sim, g: &'a crate::group::Group) -> Self {
        Self {
            sim,
            g,
            insert: None,
        }
    }
}

impl ActionFormView for SimActionForm<'_> {
    fn is_on_map(&mut self) -> bool {
        self.sim.group_is_on_map(self.g)
    }
    fn refresh(&mut self) {}
    fn buildings(&mut self) -> bool {
        false
    }
    fn count(&mut self) -> i32 {
        self.g.list.len() as i32
    }
    fn find_leader(&mut self) -> i32 {
        self.sim
            .group_find_leader(self.g)
            .map_or(-1, |u| u as i32)
    }
    fn unit_plane(&mut self, i: i32) -> bool {
        self.sim.is_plane(i as usize)
    }
    fn get_form_option(&mut self) -> i32 {
        let live: Vec<usize> = self
            .g
            .list
            .iter()
            .copied()
            .filter(|&u| self.sim.units[u].alive())
            .collect();
        match live.len() {
            0 => 0,
            1 => match self.sim.group_find_leader(self.g) {
                Some(l) if self.sim.units[l].alive() => i32::from(self.sim.units[l].form),
                _ => 0,
            },
            _ => {
                let mut counts = [0i32; 5];
                for &u in &live {
                    let f = self.sim.units[u].form;
                    if (0..5).contains(&f) {
                        counts[f as usize] += 1;
                    }
                }
                let mut best = 0;
                for j in 1..5 {
                    if counts[j] > 0 && counts[best] < counts[j] {
                        best = j;
                    }
                }
                best as i32
            }
        }
    }
    fn set_group_form(&mut self, v: i32) {
        if let Some(st) = self.sim.gstate_mut(self.g) {
            st.form = v;
        }
    }
    fn get_unit(&mut self) -> i32 {
        self.find_leader()
    }
    fn unit_form(&mut self, i: i32) -> i32 {
        i32::from(self.sim.units[i as usize].form)
    }
    fn member(&mut self, k: i32) -> i32 {
        self.g.list[k as usize] as i32
    }
    fn obj_unit(&mut self, _i: i32) -> bool {
        true
    }
    fn set_unit_form(&mut self, i: i32, v: i32) {
        self.sim.units[i as usize].form = v as i8;
    }
    fn new_list(&mut self) {
        self.insert = None;
    }
    fn set_up_insert(&mut self) {
        self.insert = Some(self.sim.group_set_up_insert(self.g));
    }
    fn action_halt(&mut self) {
        self.sim.group_action_halt(self.g, 0);
    }
    fn list_pending(&mut self) -> bool {
        self.insert.as_ref().is_some_and(|i| !i.is_empty())
    }
    fn finish_insert(&mut self) {
        if let Some(i) = self.insert.take() {
            self.sim.group_finish_insert(self.g, i);
        }
    }
    fn clear_list(&mut self) {
        self.insert = None;
    }
    fn loc_to(&mut self) -> Option<(i32, i32)> {
        self.sim.group_loc_to(self.g).map(|p| (p.x, p.y))
    }
    fn o_xy(&mut self) -> (i32, i32) {
        let o = self.sim.group_o(self.g);
        (o.x, o.y)
    }
    fn o_angle(&mut self) -> i32 {
        self.sim.group_o_angle(self.g).0
    }
    fn unit_heading(&mut self, i: i32) -> i32 {
        self.sim.units[i as usize].movement.heading.0
    }
    fn action_move_to(&mut self, a: [i32; 10]) {
        self.sim.group_action_move_to(
            self.g,
            crate::world::Pos::new(a[0], a[1]),
            crate::orders::QueuePos::Last,
            a[3] != 0,
            crate::movement::Angle(a[4]),
            crate::orders::MoveKind::MoveTo,
            false,
        );
    }
}
