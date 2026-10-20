//! **`Object::add_to_world@0064d8c0`** (item 3379): an object put on the
//! world's cell chain and, when it is a unit that stands on the ground, its
//! guys' discs marked in the collision blocks — as a walk over an
//! [`AddToWorldView`].
//!
//! The listing in four parts, in the order it runs them:
//!
//! 1. **the gate** — the object's list byte (`+0x9`) below 9; its `+0x2a`
//!    (the next link) becomes `-1`;
//! 2. **the chain** — the world cell is `width * div3[(y ^ 0x63637) >> 8] +
//!    div3[(x ^ 0x63637) >> 8]` of the raw position (`+0x10`, `+0x14`); the
//!    cell's head `(idx, list)` (`+8`, `+10` of the cell record) is the
//!    object's new *previous* (`+0x2c`, `+0x2e`), the old head's `+0x2a`/`+0x3f`
//!    name this object, and the cell's head becomes this object. A head that
//!    *is* this object, with `ignore_always` clear, is `UNIT LINKED LIST LOOPS
//!    <ADD>` — four calls, `Error::report` answering nonzero is an `int3`;
//! 3. **the gates** — vslot `0x8`, vslot `0xbc`, the type's `+0x218` not 2,
//!    the unit record's type `+0x248` not 0;
//! 4. **the discs** — for each guy (`+0xb5` of the unit record, a signed
//!    char), `get_tregion` of its tile, then `radius[coll_size]` offsets
//!    around its position in sixteenths of a cell, each in the world and in
//!    the guy's region (or the region unasked) and not `-1`-blocked: a null
//!    block is `World::new_coll_block`ed, the bit `(x & 15) * 16 + (y & 15)`
//!    set and the block's `+8` zeroed.
//!
//! The crate paints the same disc as [`crate::Sim::coll_paint`] and pushes the
//! chain as [`crate::Sim::chain_add`], but holds one figure a unit and the
//! chain as unit indices; the steps are therefore not wired into any
//! production path (no `Sim` adapter exists; only the sweep drives them).

/// What the walk reads and writes, one leaf a method.
pub trait AddToWorldView {
    /// `this +0x9`, a byte.
    fn list(&mut self) -> i32;
    /// `this +0xa`, a short.
    fn index(&mut self) -> i32;
    /// `this +0x2a` set.
    fn set_next(&mut self, v: i32);
    /// `this +0x10`, `+0x14`.
    fn pos_raw(&mut self) -> (i32, i32);
    /// `div_3_table[i]`.
    fn div3(&mut self, i: i32) -> i32;
    /// `*world`, `*(world + 4)`.
    fn world_size(&mut self) -> (i32, i32);
    /// The cell record's `+8` and `+10` shorts: `(idx, list)`.
    fn cell_head(&mut self, cell: i32) -> (i32, i32);
    /// ... both set.
    fn set_cell_head(&mut self, cell: i32, idx: i32, list: i32);
    /// The byte `ignore_always`.
    fn ignore_always(&mut self) -> bool;
    /// The four calls of `UNIT LINKED LIST LOOPS <ADD>`; `Error::report`'s
    /// answer is nonzero.
    fn report_loop(&mut self) -> bool;
    /// `objects[list][idx]`: `+0x2a` the short `own_idx`, `+0x3f` the byte `own_list`.
    fn link_previous(&mut self, list: i32, idx: i32, own_idx: i32, own_list: i32);
    /// `this +0x2c`, `+0x2e` set.
    fn set_previous(&mut self, idx: i32, list: i32);
    /// Vslot `0x8`, `!= 0`.
    fn slot8(&mut self) -> bool;
    /// Vslot `0xbc`, `!= 0`.
    fn slot_bc(&mut self) -> bool;
    /// `this`'s type record `+0x218`.
    fn type_state(&mut self) -> i32;
    /// The unit record's type `+0x248`.
    fn coll_size(&mut self) -> i32;
    /// The unit record's `+0xb5`, a signed char.
    fn guys(&mut self) -> i32;
    /// Guy `g`'s `+0xc`, `+0x10`.
    fn guy_pos(&mut self, g: i32) -> (i32, i32);
    /// `WorldData::get_tregion(&tx, &ty)`.
    fn tregion(&mut self, tx: i32, ty: i32) -> i32;
    /// `radius[t]`, `move_x[k]`, `move_y[k]`.
    fn radius(&mut self, t: i32) -> i32;
    fn move_x(&mut self, k: i32) -> i32;
    fn move_y(&mut self, k: i32) -> i32;
    /// The cell record's `+4` short.
    fn cell_region(&mut self, cell: i32) -> i32;
    /// The cell record's `+0x18` word, `-1`, `0` or a block (any other).
    fn cell_block(&mut self, cell: i32) -> i32;
    /// `World::new_coll_block` with `ecx = tregion`, the cell's x then y.
    fn new_block(&mut self, tregion: i32, cx: i32, cy: i32);
    /// Bit `bit` of the cell's block (or the new one when `fresh`) set, its `+8` zeroed.
    fn mark(&mut self, cell: i32, fresh: bool, bit: i32);
}

/// `add_to_world` from its entry to its return.
pub fn add_to_world_steps<V: AddToWorldView>(v: &mut V) {
    let list = v.list();
    if list >= 9 {
        return;
    }
    v.set_next(-1);
    let (w, _) = v.world_size();
    let (px, py) = v.pos_raw();
    let ycell = v.div3((py ^ 0x63637) >> 8);
    let xcell = v.div3((px ^ 0x63637) >> 8);
    let cell = w.wrapping_mul(ycell).wrapping_add(xcell);
    let (hi, hl) = v.cell_head(cell);
    let own = v.index();
    if hi >= 0 {
        if hi == own && hl == list && !v.ignore_always() && v.report_loop() {
            return;
        }
        let now = v.index();
        v.link_previous(hl, hi, now, list);
    }
    v.set_previous(hi, hl);
    let own = v.index();
    v.set_cell_head(cell, own, list);
    if !(v.slot8() && v.slot_bc() && v.type_state() != 2 && v.coll_size() != 0) {
        return;
    }
    if v.guys() <= 0 {
        return;
    }
    let mut g = 0;
    loop {
        let (gx, gy) = v.guy_pos(g);
        let ty = v.div3(gy >> 6);
        let tx = v.div3(gx >> 6);
        let tr = v.tregion(tx, ty);
        let ct = v.coll_size();
        let mut k = 0;
        if v.radius(ct) > 0 {
            loop {
                let (gx, gy) = v.guy_pos(g);
                let ux = v.div3(gx >> 4).wrapping_add(v.move_x(k));
                let uy = v.div3(gy >> 4).wrapping_add(v.move_y(k));
                let (w, h) = v.world_size();
                if ux >= 0 && uy >= 0 && ux < w.wrapping_mul(16) && uy < h.wrapping_mul(16) {
                    let (cy, cx) = (uy >> 4, ux >> 4);
                    let c = cy.wrapping_mul(w).wrapping_add(cx);
                    if tr < 0 || v.cell_region(c) == tr {
                        let b = v.cell_block(c);
                        if b != -1 {
                            let fresh = b == 0;
                            if fresh {
                                v.new_block(tr, cx, cy);
                            }
                            v.mark(c, fresh, (ux & 15) * 16 + (uy & 15));
                        }
                    }
                }
                k += 1;
                if k >= v.radius(ct) {
                    break;
                }
            }
        }
        g += 1;
        if g >= v.guys() {
            break;
        }
    }
}
