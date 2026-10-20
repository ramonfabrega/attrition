//! `Object::eject_contents@0064cd20` under the emulator against the port (item 3356; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

use crate::eject::{EjectView, eject_steps};

// **The windows** (item 3356; `tools/emu/sweep_object_eject_contents.py`). `m`
// is `this`: who 0, index 3, and `units[0][3]`, `objects[0][3]` too (aliased
// every row). The passengers are who 0 index 0..2 (`p7..p9`) and who 1 index
// 0..3 (`p12..p15`); their type records are `p30..p32`, `p34..p37` (`p38` is
// `this`'s). Cells: `c0` objects, `c1` game, `c2` spelltypes, `c3` types, `c4`
// console, `c5` leader options, `c6` the selection, `c7` the groups. The
// leaders' rows are `p0..p5` (flags, `+8`, the type bitset), the cities `p17,
// p18` and `p20, p21`, the spell record `p24`, the types `p25..p28` and
// `p29` (type `0x42`), `this`'s path stack `p39` and its two list nodes
// `p40, p41`, the passengers' stacks `p42..p47`, the caravans `p49..p51`
// (index -1, 0, 1), the data records `p52..p54`.
//
// **Held out of the draw, by name.** `Slot 0x18` of the selection's
// `GroupData` takes three arguments and `this`'s `is_unit` takes none, and the
// stubbed vtable has one slot 0x18: the stub pops none, so the selection's
// `(s, who, 0)` is not recorded (only its `ecx`); the frame pointer restores
// `esp` at the epilogue. The passengers' chains are cut so a walk ends:
// each passenger's `+0x28` is a link below its own index or -1 and the
// unlinking stubs lower every container's head at once (the stubs write
// every chain head, as `Unit::come_out` unlinks the one it is handed).
// `Stack::increase_size` and every other callee is a stub: what they do to
// the lists is the callee's, and the stubs here leave the lists alone.

const S_CAN_HOLD: usize = 0;
const S_PLAIN: usize = 1;
const S_DATA: usize = 2;
const S_IS: usize = 3;
const S_DIE_SLOT: usize = 4;
const S_SET_TYPE: usize = 5;
const S_SET_TYPE_K: usize = 6;
const S_ON_SCREEN: usize = 7;
const S_ME_PRED: usize = 8;
const S_CARAVAN: usize = 9;
const S_CARAVAN_K: usize = 10;
const S_TYPE68: usize = 11;
const S_IS_K: usize = 12;
const S_IS_UNIT: usize = 13;
const S_G10: usize = 14;
const S_CAN_CARRY: usize = 15;
const S_GET_ACTION: usize = 16;
const S_REPATH: usize = 17;
const S_IS_CASTABLE: usize = 18;
const S_ADD_CAST: usize = 19;
const S_CLOSE_ORDERS: usize = 20;
const S_CLEAR_PARTIAL: usize = 21;
const S_UPDATE_ACTION: usize = 22;
const S_COME_OUT: usize = 23;
const S_DIE_DIRECT: usize = 24;
const S_TRIBE_CAN: usize = 25;
const S_CUR_UPGRADE: usize = 26;
const S_CLEAR_ORDERS: usize = 27;
const S_FIND_CITY: usize = 28;
const S_ADD_MOVE: usize = 29;
const S_ADD_STRAFE: usize = 30;
const S_SOUND: usize = 31;
const S_SAME_DAMAGE: usize = 32;
const S_REMOVE_CUR: usize = 33;
const S_LIST_ADD: usize = 34;
const S_INVERT: usize = 35;
const S_INC_SIZE: usize = 36;
const S_PUSH: usize = 37;
const S_RESET_MOVE: usize = 38;
const S_GET_ARMY: usize = 39;
const S_G_CLEAR: usize = 40;
const S_G_ADD: usize = 41;
const S_G_SETUP: usize = 42;
const S_G_KILL: usize = 43;
const S_PUSH_GROUP: usize = 44;
const S_FINISH_INSERT: usize = 45;
const S_LIST_CLEAR: usize = 46;
const S_TRANSPORT: usize = 47;
const S_MEMBER: usize = 48;
const S_REPLACE_HOT: usize = 49;

/// The answers: the four arguments, then one a stub in the line's order.
const fn ans(stub: usize) -> usize {
    4 + stub
}

/// The `--seq` runs follow the 50 stubs: plain (4), is (16), is_k (16),
/// come_out (4), tribe_can (4), find_city (3); then the five keep flags.
const Q_PLAIN: usize = 54;
const Q_IS: usize = 58;
const Q_IS_K: usize = 74;
const Q_COME: usize = 90;
const Q_TRIBE: usize = 94;
const Q_CITY: usize = 98;
const F_IS: usize = 101;
const F_ON_SCREEN: usize = 102;
const F_CARAVAN: usize = 103;
const F_DIE: usize = 104;
const F_SET_TYPE: usize = 105;

/// The windows whose chain head `come_out` and `die` stubs lower: `this` and
/// the six passengers a walk can stand in.
const HEADS: [usize; 6] = [7, 8, 9, 12, 13, 14];
const DATA0: usize = 52;
const SPELL: usize = 24;
const T42: usize = 29;

#[derive(Clone, Copy)]
enum Rec {
    M,
    P(usize),
}

fn rec(w: i32, s: i32) -> Rec {
    match (w, s) {
        (0, 3) => Rec::M,
        (0, 0..=2) => Rec::P(7 + s as usize),
        (1, 0..=3) => Rec::P(12 + s as usize),
        _ => panic!("no record ({w}, {s}) in the draw"),
    }
}

fn type_win(w: i32, s: i32) -> usize {
    match (w, s) {
        (0, 0) => 30,
        (0, 1) => 31,
        (0, 2) => 32,
        (0, 3) => 38,
        (1, 0) => 34,
        (1, 1) => 35,
        (1, 2) => 36,
        (1, 3) => 37,
        _ => panic!("no type window ({w}, {s})"),
    }
}

fn name(r: Rec) -> String {
    match r {
        Rec::M => "m".to_string(),
        Rec::P(i) => format!("p{i}"),
    }
}

fn unit(w: i32, s: i32) -> String {
    format!("{}+0x0", name(rec(w, s)))
}

fn stack_win(r: Rec) -> usize {
    match r {
        Rec::M => 39,
        Rec::P(7) => 42,
        Rec::P(8) => 43,
        Rec::P(9) => 44,
        Rec::P(12) => 45,
        Rec::P(13) => 46,
        Rec::P(14) => 47,
        Rec::P(i) => panic!("no stack for p{i}"),
    }
}

struct Emu<'a> {
    a: &'a [i32],
    m: &'a mut [u8],
    c: &'a mut [Vec<u8>],
    p: &'a mut [Vec<u8>],
    log: &'a mut Vec<String>,
    /// Calls so far of: plain, is, is_k, come_out, tribe_can, find_city, data.
    n: [usize; 7],
    derefs: Vec<String>,
}

impl<'a> Emu<'a> {
    fn new(
        a: &'a [i32],
        m: &'a mut [u8],
        c: &'a mut [Vec<u8>],
        p: &'a mut [Vec<u8>],
        log: &'a mut Vec<String>,
    ) -> Self {
        Emu {
            a,
            m,
            c,
            p,
            log,
            n: [0; 7],
            derefs: Vec::new(),
        }
    }

    fn buf(&mut self, r: Rec) -> &mut [u8] {
        match r {
            Rec::M => &mut self.m[..],
            Rec::P(i) => &mut self.p[i][..],
        }
    }

    fn stub(&mut self, idx: usize, ecx: &str, args: &[i32]) {
        let mut t = format!("s{idx}:{ecx}");
        for x in args {
            t.push(',');
            t.push_str(&x.to_string());
        }
        self.log.push(t);
    }

    /// The `k`th answer of a `--seq` run of `n` values (the last repeated).
    fn seq(&mut self, which: usize, base: usize, n: usize) -> i32 {
        let k = self.n[which].min(n - 1);
        self.n[which] += 1;
        self.a[base + k]
    }

    /// `Unit::come_out` and `die` unlink what they are handed; the stubs lower
    /// every window's chain head (a short at `+0x28`).
    fn dec_heads(&mut self) {
        let lower = |b: &mut [u8]| {
            let v = u16::from_le_bytes([b[0x28], b[0x29]]).wrapping_sub(1);
            b[0x28..0x2a].copy_from_slice(&v.to_le_bytes());
        };
        lower(&mut self.m[..]);
        for &i in &HEADS {
            lower(&mut self.p[i][..]);
        }
    }

    fn is_on(&mut self, r: Rec, tw: usize, ty: i32) -> bool {
        if self.a[F_IS] != 0 {
            self.stub(S_IS_K, &format!("p{tw}+0x0"), &[ty, 0]);
            self.seq(2, Q_IS_K, 16) != 0
        } else {
            self.stub(S_IS, &format!("{}+0x0", name(r)), &[ty, 0]);
            self.seq(1, Q_IS, 16) != 0
        }
    }

    fn data_win(&mut self) -> usize {
        let k = self.n[6].min(2);
        self.n[6] += 1;
        self.stub(S_DATA, "m+0x0", &[]);
        DATA0 + k
    }

    fn who_i(&self) -> i32 {
        i32::from(self.m[9])
    }

    fn idx_i(&self) -> i32 {
        i32::from(i16::from_le_bytes([self.m[10], self.m[11]]))
    }

    fn play(&self) -> String {
        format!("c6+{:#x}", rd(&self.c[4], 0x2a0) as usize * 0x9f0)
    }
}

impl EjectView for Emu<'_> {
    fn who(&mut self) -> i32 {
        self.who_i()
    }
    fn index(&mut self) -> i32 {
        self.idx_i()
    }
    fn can_hold(&mut self) -> bool {
        self.stub(S_CAN_HOLD, "m+0x0", &[]);
        self.a[ans(S_CAN_HOLD)] != 0
    }
    fn is_plain_building(&mut self) -> bool {
        self.stub(S_PLAIN, "m+0x0", &[]);
        self.seq(0, Q_PLAIN, 4) != 0
    }
    fn is_unit(&mut self) -> bool {
        self.stub(S_IS_UNIT, "m+0x0", &[]);
        self.a[ans(S_IS_UNIT)] != 0
    }
    fn game_flag_8(&mut self) -> bool {
        self.c[1][0x821] & 8 != 0
    }
    fn can_carry(&mut self, kind: i32) -> bool {
        self.stub(S_CAN_CARRY, "m+0x0", &[kind]);
        self.a[ans(S_CAN_CARRY)] != 0
    }
    fn mark_data_4000(&mut self) {
        let d = self.data_win();
        let v = u16::from_le_bytes([self.p[d][0x60], self.p[d][0x61]]) | 0x4000;
        self.p[d][0x60..0x62].copy_from_slice(&v.to_le_bytes());
    }
    fn container_head(&mut self, w: i32, s: i32) -> i32 {
        let b = self.buf(rec(w, s));
        i32::from(i16::from_le_bytes([b[0x28], b[0x29]]))
    }
    fn container_owner(&mut self, w: i32, s: i32) -> i32 {
        i32::from(self.buf(rec(w, s))[0x3e] as i8)
    }
    fn obj_is(&mut self, w: i32, s: i32, ty: i32) -> bool {
        self.is_on(rec(w, s), type_win(w, s), ty)
    }
    fn this_is(&mut self, ty: i32) -> bool {
        self.is_on(Rec::M, 38, ty)
    }
    fn type_index(&mut self, ty: i32) -> i32 {
        rd(&self.p[25 + ty as usize], 4)
    }
    fn get_action(&mut self, w: i32, s: i32) -> bool {
        self.stub(S_GET_ACTION, &unit(w, s), &[]);
        self.a[ans(S_GET_ACTION)] != 0
    }
    fn repath(&mut self, w: i32, s: i32) {
        self.stub(S_REPATH, &unit(w, s), &[]);
    }
    fn is_castable(&mut self, w: i32, s: i32) -> bool {
        let ecx = rd(&self.p[SPELL], 0xa4c).to_string();
        self.stub(S_IS_CASTABLE, &ecx, &[s, w, 0]);
        self.a[ans(S_IS_CASTABLE)] != 0
    }
    fn add_cast_order(&mut self, w: i32, s: i32) {
        let (x, y) = (rd(self.m, 0x10) ^ 0x63637, rd(self.m, 0x14) ^ 0x63637);
        self.stub(S_ADD_CAST, &unit(w, s), &[-1, -1, x, y, 0x293, 0, 1]);
    }
    fn unit_reset_flags(&mut self, w: i32, s: i32) {
        let b = self.buf(rec(w, s));
        let f = rd(b, 0x68) & 0xfbff_ffffu32 as i32;
        wr(b, 0x68, f);
        wr(b, 0xc0, 0);
    }
    fn close_orders(&mut self, w: i32, s: i32) {
        self.stub(S_CLOSE_ORDERS, &unit(w, s), &[0]);
    }
    fn clear_partial_path(&mut self, w: i32, s: i32) {
        self.stub(S_CLEAR_PARTIAL, &unit(w, s), &[]);
    }
    fn update_action(&mut self, w: i32, s: i32) {
        self.stub(S_UPDATE_ACTION, &unit(w, s), &[]);
    }
    fn come_out(&mut self, w: i32, s: i32) -> bool {
        self.stub(S_COME_OUT, &unit(w, s), &[0]);
        self.dec_heads();
        self.seq(3, Q_COME, 4) != 0
    }
    fn die(&mut self, w: i32, s: i32) {
        let idx = if self.a[F_DIE] != 0 { S_DIE_DIRECT } else { S_DIE_SLOT };
        self.stub(idx, &unit(w, s), &[0, -1, 0]);
        self.dec_heads();
    }
    fn unit_type(&mut self, w: i32, s: i32) -> i32 {
        rd(&self.p[type_win(w, s)], 4)
    }
    fn unit_type_308(&mut self, w: i32, s: i32) -> i32 {
        rd(&self.p[type_win(w, s)], 0x308)
    }
    fn data_city(&mut self) -> i32 {
        let d = self.data_win();
        i32::from(i16::from_le_bytes([self.p[d][0x72], self.p[d][0x73]]))
    }
    fn tribe_can_type(&mut self, w: i32) -> bool {
        self.stub_str(S_TRIBE_CAN, &format!("p{}+0x0", 3 * w), &[format!("p{T42}+0x0")]);
        self.seq(4, Q_TRIBE, 4) != 0
    }
    fn leader_type68(&mut self, w: i32) -> i32 {
        let arg = rd(&self.p[3 * w as usize + 1], 0);
        self.stub(S_TYPE68, &format!("p{T42}+0x0"), &[arg]);
        self.a[ans(S_TYPE68)]
    }
    fn leader_bit(&mut self, w: i32, r: i32) -> bool {
        let byte = self.p[3 * w as usize + 2][(r >> 3) as usize];
        byte & (1u8 << (r & 7)) != 0
    }
    fn current_upgrade(&mut self, w: i32, ty: i32) -> i32 {
        self.stub(S_CUR_UPGRADE, &format!("p{}+0x0", 3 * w), &[ty]);
        self.a[ans(S_CUR_UPGRADE)]
    }
    fn unit_set_type(&mut self, w: i32, s: i32, ty: i32) {
        let idx = if self.a[F_SET_TYPE] != 0 { S_SET_TYPE_K } else { S_SET_TYPE };
        self.stub(idx, &unit(w, s), &[ty, 0]);
    }
    fn unit_clear_orders(&mut self, w: i32, s: i32) {
        self.stub(S_CLEAR_ORDERS, &unit(w, s), &[]);
    }
    fn find_city(&mut self, w: i32, l20: i32) -> i32 {
        let who = self.who_i();
        let (x, y) = (rd(self.m, 0x10) ^ 0x63637, rd(self.m, 0x14) ^ 0x63637);
        let city = &self.p[(17 + 3 * who + l20) as usize];
        let short = i32::from(i16::from_le_bytes([city[8], city[9]]));
        self.stub(S_FIND_CITY, &l20.to_string(), &[x, y, 1, w, l20, 0x200, 3, short, who]);
        self.seq(5, Q_CITY, 3)
    }
    fn add_move_order(&mut self, w: i32, s: i32, city: i32) {
        let c = &self.p[(17 + 3 * w + city) as usize];
        let (cx, cy) = (rd(c, 0xc), rd(c, 0x10));
        let slot = format!("p{}+{:#x}", if w == 0 { 6 } else { 11 }, 4 * s);
        let args = [cx.to_string(), cy.to_string(), "4".into(), "0".into(), "2".into(), "0".into(), slot, "-1".into(), "-1".into()];
        self.stub_str(S_ADD_MOVE, &unit(w, s), &args);
    }
    fn add_strafe_order(&mut self, w: i32, s: i32) {
        self.stub(S_ADD_STRAFE, &unit(w, s), &[-1, -1, -1, -1, 1, 2, 0]);
    }
    fn me_on_screen(&mut self) -> bool {
        if self.a[F_ON_SCREEN] != 0 {
            self.m[0x1c] != 0
        } else {
            self.stub(S_ON_SCREEN, "m+0x0", &[]);
            self.a[ans(S_ON_SCREEN)] != 0
        }
    }
    fn me_pred(&mut self) -> bool {
        let who = rd(&self.c[4], 0x298);
        self.stub(S_ME_PRED, "m+0x0", &[who, 0]);
        self.a[ans(S_ME_PRED)] != 0
    }
    fn sound_unload(&mut self) {
        self.stub(S_SOUND, "m+0x0", &[0x134]);
    }
    fn same_damage(&mut self, w: i32, s: i32) {
        let (i, who) = (self.idx_i(), self.who_i());
        self.stub(S_SAME_DAMAGE, &unit(w, s), &[i, who]);
    }
    fn me_is_caravan(&mut self) -> bool {
        if self.a[F_CARAVAN] != 0 {
            self.stub(S_CARAVAN_K, "p38+0x0", &[]);
            self.a[ans(S_CARAVAN_K)] != 0
        } else {
            self.stub(S_CARAVAN, "m+0x0", &[]);
            self.a[ans(S_CARAVAN)] != 0
        }
    }
    fn hand_over_caravan(&mut self, w: i32, s: i32) {
        let c = i32::from(i16::from_le_bytes([self.m[0x86], self.m[0x87]]));
        let r = rec(w, s);
        let low = (c as u16).to_le_bytes();
        self.buf(r)[0x86..0x88].copy_from_slice(&low);
        let cv = (50 + c) as usize;
        self.p[cv][0xa..0xc].copy_from_slice(&(s as u16).to_le_bytes());
        self.m[0x86..0x88].copy_from_slice(&0xffffu16.to_le_bytes());
        let mine = rd(self.m, 0x68);
        let b = self.buf(r);
        let f = rd(b, 0x68);
        wr(b, 0x68, if mine & 0x200 != 0 { f | 0x200 } else { f & !0x200 });
    }
    fn me_list_len(&mut self) -> i32 {
        rd(self.m, 0xd8)
    }
    fn me_list_advance(&mut self) {
        if rd(self.m, 0xdc) != 0 {
            let node = rd(&self.p[40], 4);
            wr(self.m, 0xd4, node);
            let data = rd(&self.p[41], 8);
            wr(self.m, 0xcc, data);
            self.m[0xd0] = self.p[41][0xc];
        }
    }
    fn me_list_data(&mut self) -> i32 {
        rd(self.m, 0xcc)
    }
    fn me_list_remove(&mut self) {
        self.stub(S_REMOVE_CUR, "m+0xcc", &[]);
        let v = rd(self.m, 0xd8).wrapping_sub(1);
        wr(self.m, 0xd8, v);
    }
    fn list_add(&mut self, w: i32, s: i32, data: i32) {
        self.stub(S_LIST_ADD, &format!("{}+0xcc", name(rec(w, s))), &[data]);
    }
    fn me_stack_invert(&mut self) {
        self.stub(S_INVERT, "m+0xb8", &[]);
    }
    fn me_stack_count(&mut self) -> i32 {
        rd(self.m, 0xc0)
    }
    fn me_stack_set_count(&mut self, n: i32) {
        wr(self.m, 0xc0, n);
    }
    fn me_stack_get(&mut self, i: i32) -> [i32; 4] {
        let l = &self.p[stack_win(Rec::M)];
        let at = 16 * i as usize;
        [rd(l, at), rd(l, at + 4), rd(l, at + 8), rd(l, at + 12)]
    }
    fn stack_count(&mut self, w: i32, s: i32) -> i32 {
        rd(self.buf(rec(w, s)), 0xc0)
    }
    fn stack_cap(&mut self, w: i32, s: i32) -> i32 {
        rd(self.buf(rec(w, s)), 0xbc)
    }
    fn stack_grow_arg(&mut self, w: i32, s: i32) -> i32 {
        i32::from(self.buf(rec(w, s))[0xc4] as i8)
    }
    fn stack_set_count(&mut self, w: i32, s: i32, n: i32) {
        wr(self.buf(rec(w, s)), 0xc0, n);
    }
    fn stack_get(&mut self, w: i32, s: i32, i: i32) -> [i32; 4] {
        let l = &self.p[stack_win(rec(w, s))];
        let at = 16 * i as usize;
        [rd(l, at), rd(l, at + 4), rd(l, at + 8), rd(l, at + 12)]
    }
    fn stack_set(&mut self, w: i32, s: i32, i: i32, r: [i32; 4]) {
        let l = &mut self.p[stack_win(rec(w, s))];
        for (k, v) in r.iter().enumerate() {
            wr(l, 16 * i as usize + 4 * k, *v);
        }
    }
    fn stack_increase(&mut self, w: i32, s: i32) {
        let arg = self.stack_grow_arg(w, s);
        self.stub(S_INC_SIZE, &format!("{}+0xb8", name(rec(w, s))), &[arg]);
    }
    fn stack_push(&mut self, w: i32, s: i32, r: [i32; 4]) {
        self.stub_str(
            S_PUSH,
            &format!("{}+0xb8", name(rec(w, s))),
            &["sp-0x58".to_string()],
        );
        let bytes: Vec<u8> = r.iter().flat_map(|v| v.to_le_bytes()).collect();
        self.derefs.push(bytes.iter().map(|b| format!("{b:02x}")).collect());
    }
    fn me_reset_move_orders(&mut self) {
        self.stub(S_RESET_MOVE, "m+0x0", &[]);
    }
    fn unit_flags68(&mut self, w: i32, s: i32) -> i32 {
        rd(self.buf(rec(w, s)), 0x68)
    }
    fn get_army(&mut self, w: i32, s: i32) -> i32 {
        self.stub(S_GET_ARMY, &unit(w, s), &[]);
        self.a[ans(S_GET_ARMY)]
    }
    fn group_insert(&mut self, w: i32, s: i32) {
        let (idx, who) = (self.idx_i(), self.who_i());
        let g = "sp-0xa48";
        let l = "sp-0x74";
        self.stub_str(S_G_CLEAR, g, &["-1".to_string()]);
        self.stub(S_G_ADD, g, &[idx, who, 0, 0]);
        self.stub_str(S_G_SETUP, g, &[l.to_string()]);
        self.stub(S_G_KILL, g, &[idx, who, 0, 0]);
        self.stub(S_G_ADD, g, &[s, w, 0, 0]);
        self.stub_str(S_PUSH_GROUP, g, &[who.to_string(), g.to_string(), "1".to_string()]);
        let at = self.a[ans(S_PUSH_GROUP)] as usize * 0x9d4;
        self.stub_str(S_FINISH_INSERT, &format!("c7+{at:#x}"), &[l.to_string()]);
        self.stub(S_LIST_CLEAR, "sp-0x70", &[]);
    }
    fn leader_flags(&mut self) -> i32 {
        rd(&self.p[3 * self.who_i() as usize], 0)
    }
    fn transport_type(&mut self, w: i32, s: i32) -> i32 {
        self.stub(S_TRANSPORT, &unit(w, s), &[]);
        self.a[ans(S_TRANSPORT)]
    }
    fn unit_is(&mut self, w: i32, s: i32, ty: i32) -> bool {
        self.is_on(rec(w, s), type_win(w, s), ty)
    }
    fn leader_option_2(&mut self) -> bool {
        self.c[5][self.who_i() as usize * 32 + 0x1c] & 2 != 0
    }
    fn unit_set_flag_800000(&mut self, w: i32, s: i32) {
        let b = self.buf(rec(w, s));
        let f = rd(b, 0x68) | 0x80_0000;
        wr(b, 0x68, f);
    }
    fn selection_member(&mut self) -> bool {
        let (g, idx, who) = (self.play(), self.idx_i(), self.who_i());
        self.stub(S_MEMBER, &g, &[idx, who, 1]);
        self.a[ans(S_MEMBER)] != 0
    }
    fn selection_swap(&mut self, w: i32, s: i32) {
        let (g, idx, who) = (self.play(), self.idx_i(), self.who_i());
        let _ = (w, s);
        self.stub(S_G10, &g, &[idx, who, 0, 0]);
        self.stub(S_IS_UNIT, &g, &[]);
    }
    fn replace_hotunit(&mut self, w: i32, s: i32) {
        self.stub(S_REPLACE_HOT, "m+0x0", &[s, w]);
    }
}

impl Emu<'_> {
    fn stub_str(&mut self, idx: usize, ecx: &str, args: &[String]) {
        let mut t = format!("s{idx}:{ecx}");
        for x in args {
            t.push(',');
            t.push_str(x);
        }
        self.log.push(t);
    }
}

/// `Object::eject_contents@0064cd20`, run by `tools/emu/sweep_object_eject_contents.py` under unicorn
/// (tools/emu): 256 random bytes of object, 4 argument(s), `p` windows in order: p0..p5 sparse table 0 (2 rows × 3 slice(s), row-major), p6..p51 the place(s), in flag order, p52..p54 the stubs' answer window(s); the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00c061ec points at 2304 random bytes; the global cell 0x00c061a8 points at 32 random bytes; the global cell 0x00e85ddc points at 272 random bytes; the global cell 0x00c06210 points at 688 random bytes; the global cell 0x00c061b4 points at 64 random bytes; the global cell 0x00e8d444 points at 5088 random bytes; the global cell 0x00e85f20 points at 7260 random bytes; vtable slot +0xbc (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x158 (3 args) stubbed; vtable slot +0x80 (2 args) stubbed; vtable slot +0x84 (2 args) stubbed; vtable slot +0x70 (0 args) stubbed; vtable slot +0x48 (2 args) stubbed; vtable slot +0xd0 (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; vtable slot +0x68 (1 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0x10 (4 args) stubbed; callee 0x00646c40 (1 args) stubbed; callee 0x00608450 (0 args) stubbed; callee 0x005e29b0 (0 args) stubbed; callee 0x00675bc0 (3 args) stubbed; callee 0x005e4a60 (7 args) stubbed; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x00617c10 (1 args) stubbed; callee 0x00647080 (3 args) stubbed; callee 0x006d9380 (1 args) stubbed; callee 0x006e3140 (1 args) stubbed; callee 0x005e3860 (0 args) stubbed; callee 0x0065ba90 (9 args) stubbed; callee 0x00616ed0 (9 args) stubbed; callee 0x005e48c0 (7 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x005f9400 (2 args) stubbed; callee 0x0046d620 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; callee 0x0046d9a0 (0 args) stubbed; callee 0x0046e8c0 (1 args) stubbed; callee 0x0046d820 (1 args) stubbed; callee 0x005fd080 (0 args) stubbed; callee 0x00649d70 (0 args) stubbed; callee 0x00713e80 (1 args) stubbed; callee 0x00714350 (4 args) stubbed; callee 0x0070e520 (1 args) stubbed; callee 0x00714110 (4 args) stubbed; callee 0x0070f9e0 (3 args) stubbed; callee 0x0070e620 (1 args) stubbed; callee 0x0046f600 (0 args) stubbed; callee 0x0046f790 (0 args) stubbed; callee 0x0070f8f0 (3 args) stubbed; callee 0x00643b70 (2 args) stubbed; the word at `0xc0aec0+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p6+0` points at 256 random bytes less `0` (placed); the word at `p6+4` points at 256 random bytes less `0` (placed); the word at `p6+8` points at 256 random bytes less `0` (placed); the word at `p6+12` points at 256 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p11+0` points at 256 random bytes less `0` (placed); the word at `p11+4` points at 256 random bytes less `0` (placed); the word at `p11+8` points at 256 random bytes less `0` (placed); the word at `p11+12` points at 256 random bytes less `0` (placed); the word at `0xc09970+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p16+0` points at 32 random bytes less `0` (placed); the word at `p16+4` points at 32 random bytes less `0` (placed); the word at `0xc09970+0x1c*1` points at 8 random bytes less `0` (placed); the word at `p19+0` points at 32 random bytes less `0` (placed); the word at `p19+4` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*0` points at 16 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 16 random bytes less `0` (placed); the word at `c2+0x10` points at 2656 random bytes less `0` (placed); the word at `c3+0x0` points at 16 random bytes less `0` (placed); the word at `c3+0x4` points at 16 random bytes less `0` (placed); the word at `c3+0x8` points at 16 random bytes less `0` (placed); the word at `c3+0xc` points at 16 random bytes less `0` (placed); the word at `c3+0x108` points at 16 random bytes less `0` (placed); the word at `p7+0x18` points at 784 random bytes less `0` (placed); the word at `p8+0x18` points at 784 random bytes less `0` (placed); the word at `p9+0x18` points at 784 random bytes less `0` (placed); the word at `p10+0x18` points at 784 random bytes less `0` (placed); the word at `p12+0x18` points at 784 random bytes less `0` (placed); the word at `p13+0x18` points at 784 random bytes less `0` (placed); the word at `p14+0x18` points at 784 random bytes less `0` (placed); the word at `p15+0x18` points at 784 random bytes less `0` (placed); the word at `m+0x18` points at 784 random bytes less `0` (placed); the word at `m+0xb8` points at 128 random bytes less `0` (placed); the word at `m+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p40+4` points at 16 random bytes less `0` (placed); the word at `p7+0xb8` points at 128 random bytes less `0` (placed); the word at `p8+0xb8` points at 128 random bytes less `0` (placed); the word at `p9+0xb8` points at 128 random bytes less `0` (placed); the word at `p12+0xb8` points at 128 random bytes less `0` (placed); the word at `p13+0xb8` points at 128 random bytes less `0` (placed); the word at `p14+0xb8` points at 128 random bytes less `0` (placed); the word at `0xe3a2a0+0x1c*0` points at 12 random bytes less `-4` (placed); the word at `p48+0` points at 16 random bytes less `0` (placed); the word at `p48+4` points at 16 random bytes less `0` (placed); the word at `p48+8` points at 16 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0x70 laid as 0x00662670 or its stub, a flag after the answers; slot +0xd0 laid as 0x0046ce90 or its stub, a flag after the answers; slot +0x158 laid as 0x0060eda0 or its stub, a flag after the answers; slot +0x80 laid as 0x0046ce10 or its stub, a flag after the answers; stub 0 answered from [1, 1, 1, 1, 1, 1, 1, 1, 0] (90%); stub 1 answered from [0, 1, 1, 1, 1] (90%); stub 3 answered from [0, 1, 1, 0] (90%); stub 7 answered from [0, 1, 1] (90%); stub 8 answered from [0, 1, 1] (90%); stub 9 answered from [0, 1, 1] (90%); stub 10 answered from [0, 1, 1] (90%); stub 12 answered from [0, 1, 1, 0] (90%); stub 13 answered from [1, 1, 1, 0] (90%); stub 15 answered from [0, 1] (90%); stub 16 answered from [0, 1] (90%); stub 18 answered from [0, 1] (90%); stub 23 answered from [0, 0, 0, 0, 0, 1] (90%); stub 25 answered from [0, 1] (90%); stub 39 answered from [-1, 0, 3] (90%); stub 48 answered from [0, 1] (90%); stub 2 answers a window of its own to each call, in call order (128 bytes, ...); the word at `p6+12` laid as the address of `m` every row; the word at `p22+0` laid as the address of `p7` every row; the word at `p22+4` laid as the address of `p8` every row; the word at `p22+8` laid as the address of `p9` every row; the word at `p22+12` laid as the address of `m` every row; the word at `p23+0` laid as the address of `p12` every row; the word at `p23+4` laid as the address of `p13` every row; the word at `p23+8` laid as the address of `p14` every row; the word at `p23+12` laid as the address of `p15` every row; stub 1 answers 4 values in call order, printed after the answers; stub 3 answers 16 values in call order, printed after the answers; stub 12 answers 16 values in call order, printed after the answers; stub 23 answers 4 values in call order, printed after the answers; stub 25 answers 4 values in call order, printed after the answers; stub 28 answers 3 values in call order, printed after the answers; stub 11 answers 0..127 every row; stub 28 answers -1..1 every row; stub 44 answers 0..2 every row; stub 47 answers 0..4 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 27 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 28 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 29 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 30 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 31 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 32 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 33 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 34 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 35 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 36 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 37 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 38 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 39 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 40 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 41 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 42 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 43 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 44 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 45 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 46 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 47 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 48 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 49 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 37 prints the 16 bytes its argument 0 points at, hex, after the record (`/` a call); the stack 1024 bytes under the call is zeroed before each row; stub 23 lowers `m+0x28` (2 bytes) by one each call; stub 23 lowers `p7+0x28` (2 bytes) by one each call; stub 23 lowers `p8+0x28` (2 bytes) by one each call; stub 23 lowers `p9+0x28` (2 bytes) by one each call; stub 23 lowers `p12+0x28` (2 bytes) by one each call; stub 23 lowers `p13+0x28` (2 bytes) by one each call; stub 23 lowers `p14+0x28` (2 bytes) by one each call; stub 24 lowers `m+0x28` (2 bytes) by one each call; stub 24 lowers `p7+0x28` (2 bytes) by one each call; stub 24 lowers `p8+0x28` (2 bytes) by one each call; stub 24 lowers `p9+0x28` (2 bytes) by one each call; stub 24 lowers `p12+0x28` (2 bytes) by one each call; stub 24 lowers `p13+0x28` (2 bytes) by one each call; stub 24 lowers `p14+0x28` (2 bytes) by one each call; stub 4 lowers `m+0x28` (2 bytes) by one each call; stub 4 lowers `p7+0x28` (2 bytes) by one each call; stub 4 lowers `p8+0x28` (2 bytes) by one each call; stub 4 lowers `p9+0x28` (2 bytes) by one each call; stub 4 lowers `p12+0x28` (2 bytes) by one each call; stub 4 lowers `p13+0x28` (2 bytes) by one each call; stub 4 lowers `p14+0x28` (2 bytes) by one each call; stub 33 lowers `m+0xd8` (4 bytes) by one each call; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x8:0x4, +0x6c18:0x10 (`p` windows after the pointed buffers, before the places); figures p7..p10 link at +0x28 (2 bytes) below their own index or to -1; figures p12..p15 link at +0x28 (2 bytes) below their own index or to -1; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `c6+0` as well as at `this`; the stubbed vtable laid at `c6+0x9f0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_eject_contents.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(8);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            let derefs = {
                let mut emu = Emu::new(a, m, c, p, &mut log);
                eject_steps(&mut emu, a[0], a[1], a[2], a[3]);
                emu.derefs.join("/")
            };
            log.push(if derefs.is_empty() { "-".to_string() } else { derefs });
            want_eax
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "eject_contents — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "eject_contents — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "eject_contents — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[9 + i], "eject_contents — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Object::eject_contents@0064cd20: {n} rows agree");
}
