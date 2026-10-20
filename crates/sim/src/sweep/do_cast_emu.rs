//! The emulated original's windows as `Unit::do_cast@005ebfe0`'s view (item
//! 3376), shared by the animation and tail sweeps, which lay the same
//! windows: `m` the caster; `c0` the console (`+0x298` its player), `c1` the
//! message window's cell, `c2` the scene (`+0x22c` `recalc_builds`), `c3` the
//! options (`+0x90` `rebuild`), `c4` `div_3_table`, `c5` the unit types'
//! cell, `c6` the constants; `p1` the caster's figure array and `p2..p4` its
//! figures; `p5` its player's units array and `p6..p8` the three units it
//! holds; `p9` the order record slot `+0x7c` answers; `p10` the cell of
//! `types.list` and `p11` the one type record it holds at that index; `p12`
//! the spell types' array and `p13` its pointers, `p14..p28` the spell-type
//! rows at [`SPELLS`]; `p29` the caster's unit type; `p30` the unit types'
//! record.

use crate::do_cast_view::{DoCastClock, DoCastHead};

/// The spell indices the sweeps' orders draw: the row of `SPELLS[k]` is
/// window `p(14 + k)`.
pub const SPELLS: [i32; 15] = [
    0x279, 0x27b, 0x27c, 0x28a, 0x28b, 0x28c, 0x28d, 0x28e, 0x28f, 0x290, 0x291, 0x292, 0x296,
    0x2ac, 0x2ad,
];

/// The windows, named.
const ORDER: usize = 9;
const TYPE: usize = 11;
const UNIT_TYPE: usize = 29;
const UNIT_TYPES: usize = 30;

pub fn rd(w: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(w[off..off + 4].try_into().expect("four bytes"))
}

pub fn wr(w: &mut [u8], off: usize, v: i32) {
    w[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn rd16(w: &[u8], off: usize) -> i32 {
    i32::from(i16::from_le_bytes([w[off], w[off + 1]]))
}

/// The row's windows, the arguments (`a[0]` the order, `a[1 + k]` stub `k`'s
/// answer, then the vtable flags), and the calls the port makes in order.
pub struct EmuCast<'a> {
    pub a: &'a [i32],
    pub m: &'a mut [u8],
    pub c: &'a mut [Vec<u8>],
    pub p: &'a mut [Vec<u8>],
    pub log: &'a mut Vec<String>,
    /// What `ecx` is on return from the last stub (`set_angle` is handed it).
    pub ecx: String,
    /// The flag columns of `a`: slot `+0x48`, `+0x50` and `+0xc4` are the
    /// image's own (1) or the stub (0); `+0xe8`'s, `usize::MAX` where the
    /// sweep does not ask it.
    pub flags: [usize; 4],
}

impl EmuCast<'_> {
    fn answer(&self, stub: usize) -> i32 {
        self.a[1 + stub]
    }
    fn row_of(craft: i32) -> usize {
        14 + SPELLS
            .iter()
            .position(|&s| s == craft)
            .expect("a drawn spell")
    }
    fn tile(&self, off: usize, shift: u32) -> i32 {
        rd(
            &self.c[4],
            4 * (((rd(self.m, off) ^ 0x63637) >> shift) as usize),
        )
    }
}

impl DoCastHead for EmuCast<'_> {
    fn order_spell(&mut self) -> i32 {
        self.log.push("s0:p0+0x0".into());
        rd(&self.p[ORDER], 0x20)
    }
    fn order_paid(&self) -> bool {
        rd(&self.p[ORDER], 0x1c) != 0
    }
    fn is_spell_type(&mut self, _idx: i32) -> bool {
        if self.a[self.flags[0]] == 1 {
            (0x275..0x2ac).contains(&rd(&self.p[TYPE], 4))
        } else {
            self.log.push("s2:p11+0x0".into());
            self.answer(2) != 0
        }
    }
    fn caster_index(&self) -> i32 {
        rd16(self.m, 0xa)
    }
    fn caster_owner(&self) -> i32 {
        i32::from(self.m[9])
    }
    fn pay_cast_costs(&mut self, idx: i32, index: i32, owner: i32) -> bool {
        self.log
            .push(format!("s1:p{}+0x0,{index},{owner}", Self::row_of(idx)));
        self.answer(1) != 0
    }
    fn mark_paid(&mut self) {
        wr(&mut self.p[ORDER], 0x1c, 1);
    }
}

impl DoCastClock for EmuCast<'_> {
    fn untargeted(&mut self, craft: i32) -> bool {
        rd(&self.p[Self::row_of(craft)], 0x1c8) & 0xe == 0
    }
    fn spell_time(&self) -> i32 {
        rd16(self.m, 0x98)
    }
    fn set_spell_time(&mut self, t: i32) {
        self.m[0x98..0x9a].copy_from_slice(&(t as i16).to_le_bytes());
    }
    fn type_is_pack(&mut self, _idx: i32) -> bool {
        if self.a[self.flags[1]] == 1 {
            matches!(rd(&self.p[TYPE], 4), 0x28b | 0x28d | 0x28f | 0x291)
        } else {
            self.log.push("s3:p11+0x0".into());
            self.answer(3) != 0
        }
    }
    fn type_is_unpack(&mut self, _idx: i32) -> bool {
        self.log.push("s4:p11+0x0".into());
        self.answer(4) != 0
    }
    fn packed(&self) -> bool {
        rd(self.m, 0x68) & 0x80000 != 0
    }
    fn is_merchant(&mut self) -> bool {
        self.log.push("s7:m+0x0".into());
        self.ecx = "m+0x0".into();
        self.answer(7) != 0
    }
    fn merchant_repiece(&mut self) {
        self.c[2][0x22c] = 1;
        let unit = format!("p{}+0x0", 6 + rd16(self.m, 0xa));
        self.log.push(format!("s8:{unit}"));
        self.ecx = unit;
    }
    fn face_guy0(&mut self) {
        let angle = rd(&self.p[2], 0x18);
        self.log.push(format!("s9:m+0x0,{angle},{},0", self.ecx));
    }
    fn caster_is_hero(&mut self) -> bool {
        if self.a[self.flags[2]] == 1 {
            rd(&self.p[UNIT_TYPE], 0x2b8) & 0x20 != 0
        } else {
            self.log.push("s5:m+0x0".into());
            self.answer(5) != 0
        }
    }
    fn set_anim(&mut self, anim: i32, p3: bool) {
        self.log
            .push(format!("s10:m+0x0,{anim},0,{}", i32::from(p3)));
    }
    fn rebuild_options(&mut self) {
        wr(&mut self.c[3], 0x90, 1);
    }
    fn is_rare_collector(&mut self) -> bool {
        self.log.push("s11:m+0x0".into());
        self.ecx = "m+0x0".into();
        self.answer(11) != 0
    }
    fn good_merchant_spot_here(&mut self) -> bool {
        let (x, y) = (self.tile(0x10, 6), self.tile(0x14, 6));
        self.log.push(format!("s12:m+0x0,{x},{y}"));
        self.answer(12) != 0
    }
    fn seat_on_cell(&mut self) {
        let x = self.tile(0x10, 4).wrapping_mul(0x30).wrapping_add(0x18);
        let y = self.tile(0x14, 4).wrapping_mul(0x30).wrapping_add(0x18);
        self.log.push(format!("s17:m+0x0,{x},{y},1,1"));
        self.ecx = "m+0x0".into();
    }
    fn kill_current_order(&mut self) {
        self.log.push("s13:m+0x0,0".into());
    }
    fn console_is_owner(&self) -> bool {
        rd(&self.c[0], 0x298) == i32::from(self.m[9])
    }
    fn feedback(&mut self, msg: i32) {
        self.log.push(format!("s14:{},64", rd(&self.c[0], 0x298)));
        self.log.push(format!("s15:sp-0x6c,{msg}"));
        self.log
            .push("s16:c1+0x0,0,0,0,0,0,-1,13161032".into());
    }
    fn board_spot_blocked(&mut self) -> bool {
        let (x, y) = (rd(self.m, 0x10) ^ 0x63637, rd(self.m, 0x14) ^ 0x63637);
        let ty = rd(&self.p[UNIT_TYPES], 0x500);
        let dist = rd(&self.c[6], 0x98);
        self.log.push(format!(
            "s18:{ty},{x},{y},sp-0x3c,sp-0x44,0,{dist},0,1431655765,3,-1,-1,0,0,-1,0,-1"
        ));
        self.answer(18) != 0
    }
    fn unit_type_gate(&mut self) -> bool {
        self.log.push("s6:p29+0x0".into());
        self.answer(6) != 0
    }
    fn has_general_162(&mut self) -> i32 {
        self.log.push("s19:m+0x0,0,354".into());
        self.answer(19)
    }
    fn crew_hold_bump(&mut self) {
        let n = rd(self.m, 0xe8);
        for i in 0..n.max(0) as usize {
            let w = &mut self.p[2 + i];
            let v = rd(w, 0x74).wrapping_add(1);
            wr(w, 0x74, v);
            if (v as u32) >= rd(w, 0x78) as u32 {
                wr(w, 0x74, v.wrapping_sub(1));
            }
        }
    }
    fn job_time(&mut self, craft: i32) -> i32 {
        let index = rd16(self.m, 0xa);
        let owner = i32::from(self.m[9]);
        self.log
            .push(format!("s20:p{}+0x0,{index},{owner}", Self::row_of(craft)));
        if self.flags[3] == usize::MAX {
            // The animation sweep stops here: a clock that is not due ends the walk.
            return 0x7fff;
        }
        self.answer(20)
    }
    fn is_captain(&mut self) -> bool {
        if self.a[self.flags[3]] == 1 {
            (i32::from(u16::from_le_bytes([self.m[0x8e], self.m[0x8f]])) >> 15) != 0
        } else {
            self.log.push("s21:m+0x0".into());
            self.answer(21) != 0
        }
    }
    fn captain_order_is_cast(&mut self) -> bool {
        self.log.push("s22:m+0x0".into());
        let c = self.answer(22);
        self.log.push(format!("s23:p{}+0x0", 6 + c));
        self.answer(23) == 14
    }
    fn spell_cast(&mut self, idx: i32) {
        let index = rd16(self.m, 0xa);
        let owner = i32::from(self.m[9]);
        self.log.push(format!(
            "s24:p{}+0x0,{index},{owner},0,0",
            Self::row_of(idx)
        ));
    }
    fn non_spell_arm(&mut self, _craft: i32) {
        panic!("the non-spell-type arm is not swept: its stubs are parked");
    }
    fn clear_paid(&mut self) {
        wr(&mut self.p[ORDER], 0x1c, 0);
    }
}
