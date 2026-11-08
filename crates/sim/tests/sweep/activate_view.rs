//! The emulated original's windows as `Build::activate`'s views (item 3364): the one
//! port the `build_activate_arms` and `build_activate_senate` sweeps share (hand-written;
//! `sweep_gen.py --regen` redraws each test's body and keeps this file).
//!
//! Row: `a[0..3]` captured, announce, counted; `a[3..18]` the fifteen stubs' answers
//! (s0 type slot `+0x60`, s1 slot `+0xb8`, s2 `new_library`, s3 `has_tribe_bonus`, s4
//! `Wall::activate`, s5 `find_unit`, s6 `get_gov_hero`, s7 `train`, s8 `find_capital`, s9
//! `count_buildings`, s10 `fix_borders`, s11 `granary_level`, s12 `lumber_level`, s13
//! `regen_roads`, s14 slot `+0xfc`, where the call ends); `a[18..20]` `find_capital`'s two
//! out values (the city's index, the owner); `a[20..28]` slot `+0x60`'s eight `is` answers,
//! `a[28..36]` slot `+0xb8`'s, `a[36..42]` `has_tribe_bonus`'s six.
//!
//! Windows: `m` the building (`+8` flags, `+9` owner, `+0xa` index, `+0x10`/`+0x14` the
//! coordinates, `+0x72` the city's index); `c0` the game (`+0x550` the frame), `c1` the world
//! (`+0` its width, `+0x134` the regions), `c2` `constantsc`, `c3` `div_3_table`; per owner
//! `o`: `c(4+4o)` the leader's flags, `c(5+4o)` `+0x408` its cities, `c(6+4o)` `+0x85c`
//! senates built, `c(7+4o)` `+0xa48` the gov hero frame; `c12`/`c13` the `+0x59f8` tables,
//! `c14` `last_build_built_o`; `p0` the type (`+4` its index), `p(2+4o+i)` the owner's city
//! `i`, `p9` the region records.

use crate::activate::{
    ActivateArmsView, ActivateHeadView, activate_arms_steps, activate_head_steps,
};

fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

struct V<'a> {
    a: &'a [i32],
    m: &'a mut [u8],
    c: &'a mut [Vec<u8>],
    p: &'a mut [Vec<u8>],
    log: &'a mut Vec<String>,
    is_calls: usize,
    tribe_calls: usize,
    cap: (i32, i32),
}

impl V<'_> {
    fn owner(&self) -> usize {
        usize::from(self.m[9])
    }
    fn leader(&self) -> String {
        format!("c{}+0x0", 4 + 4 * self.owner())
    }
    fn city(&self) -> i32 {
        i32::from(i16::from_le_bytes([self.m[0x72], self.m[0x73]]))
    }
    /// The `p` window of the owner's city `i`.
    fn rec(&self, i: i32) -> usize {
        2 + 4 * self.owner() + i as usize
    }
    fn rec_name(&self, i: i32) -> String {
        format!("p{}+0x0", self.rec(i))
    }
}

impl ActivateHeadView for V<'_> {
    fn is_type(&mut self, id: i32) -> bool {
        let k = self.is_calls.min(7);
        self.is_calls += 1;
        if rd(self.m, 0) == 0x00b4_2174 {
            self.log.push(format!("s0:p0+0x0,{id},0"));
            self.a[20 + k] != 0
        } else {
            self.log.push(format!("s1:m+0x0,{id},0"));
            self.a[28 + k] != 0
        }
    }
    fn new_library(&mut self) {
        self.log.push("s2:m+0x0".to_string());
    }
    fn clear_recharging(&mut self) {
        self.m[0x7a] = 0;
        self.m[0x7b] = 0;
    }
    fn city_flag(&mut self) -> bool {
        self.m[8] & 0x20 != 0
    }
    fn cities_add(&mut self, _d: i32) {
        unreachable!("the arms sweeps never set the city flag");
    }
    fn tribe_bonus(&mut self, bonus: i32) -> bool {
        let who = self.leader();
        self.log.push(format!("s3:{who},{bonus}"));
        let k = self.tribe_calls.min(5);
        self.tribe_calls += 1;
        self.a[36 + k] != 0
    }
    fn chinese_large_cities(&mut self) -> bool {
        unreachable!("the arms sweeps never set the city flag");
    }
    fn set_type(&mut self, _id: i32) {
        unreachable!("the arms sweeps never set the city flag");
    }
    fn mask_me(&mut self) {
        unreachable!("the arms sweeps never set the city flag");
    }
    fn flag_wall_stats(&mut self) {
        unreachable!("the arms sweeps never set the city flag");
    }
    fn record_last_built(&mut self) {
        let o = self.owner();
        let idx = rd(&self.p[0], 4) as usize;
        let short = i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]]));
        wr(&mut self.c[12 + o], idx * 4, short);
        wr(&mut self.c[14], o * 4, short);
    }
    fn wall_activate(&mut self, captured: i32, announce: i32) {
        self.log.push(format!("s4:m+0x0,{captured},{announce},0"));
    }
}

impl ActivateArmsView for V<'_> {
    fn city_index(&mut self) -> i32 {
        self.city()
    }
    fn city_flags_or(&mut self, mask: u16) {
        let r = self.rec(self.city());
        let f = u16::from_le_bytes([self.p[r][4], self.p[r][5]]) | mask;
        self.p[r][4..6].copy_from_slice(&f.to_le_bytes());
    }
    fn granary_bonus(&mut self) {
        let r = self.rec(self.city());
        let n = self.rec_name(self.city());
        self.log.push(format!("s11:{n}"));
        let level = self.a[3 + 11] as usize;
        self.p[r][0x56] = self.c[2][0x2b8 + 4 * level];
    }
    fn lumber_bonus(&mut self) {
        let r = self.rec(self.city());
        let n = self.rec_name(self.city());
        self.log.push(format!("s12:{n}"));
        let level = self.a[3 + 12] as usize;
        self.p[r][0x57] = self.c[2][0x2cc + 4 * level];
    }
    fn leader_flags_or(&mut self, mask: u32) {
        let at = 4 + 4 * self.owner();
        let w = rd(&self.c[at], 0) | mask as i32;
        wr(&mut self.c[at], 0, w);
    }
    fn fix_borders(&mut self) {
        // `div_3_table[(y ^ 0x63637) >> 8] + world * div_3_table[...x...]`, the region index.
        let slot = |w: i32| ((w ^ 0x63637) >> 8) as usize;
        let (x, y) = (rd(self.m, 0x10), rd(self.m, 0x14));
        let idx = rd(&self.c[1], 0)
            .wrapping_mul(rd(&self.c[3], 4 * slot(y)))
            .wrapping_add(rd(&self.c[3], 4 * slot(x)));
        let at = 28 * idx as usize + 4;
        if i16::from_le_bytes([self.p[9][at], self.p[9][at + 1]]) >= 0 {
            self.log.push(format!("s10:{}", idx.wrapping_mul(7)));
        }
    }
    fn gov_hero_frame_set(&mut self) -> bool {
        rd(&self.c[7 + 4 * self.owner()], 0) >= 0
    }
    fn gov_hero_near(&mut self) -> bool {
        let (x, y) = (rd(self.m, 0x10) ^ 0x63637, rd(self.m, 0x14) ^ 0x63637);
        let who = self.owner();
        self.log.push(format!(
            "s5:12626800,{x},{y},1,{who},-1,131072,23,0,0,0,0,0,{y}"
        ));
        self.a[3 + 5] >= 0
    }
    fn gov_hero_due(&mut self) -> bool {
        rd(&self.c[7 + 4 * self.owner()], 0) <= rd(&self.c[0], 0x550)
    }
    fn get_gov_hero(&mut self) -> i32 {
        let l = self.leader();
        self.log.push(format!("s6:{l},{l}"));
        self.a[3 + 6]
    }
    fn train(&mut self, ty: i32) {
        self.log.push(format!("s7:m+0x0,{ty}"));
    }
    fn senates_built_inc(&mut self) {
        let at = 6 + 4 * self.owner();
        let w = rd(&self.c[at], 0).wrapping_add(1);
        wr(&mut self.c[at], 0, w);
    }
    fn city_race_is_owner(&mut self) -> bool {
        let r = self.rec(self.city());
        i32::from(self.p[r][0x5f] as i8) == self.owner() as i32
    }
    fn find_capital(&mut self) -> Option<i32> {
        let l = self.leader();
        self.log.push(format!("s8:{l},sp-0x30,sp-0x34,-1,-1"));
        self.cap = (self.a[18], self.a[19]);
        (self.cap.1 == self.owner() as i32 && self.cap.0 >= 0).then_some(self.cap.0)
    }
    fn count_senates(&mut self, city: i32) -> i32 {
        let n = self.rec_name(city);
        self.log.push(format!("s9:{n},438,0,1"));
        self.a[3 + 9]
    }
    fn own_cities(&mut self) -> i32 {
        rd(&self.c[5 + 4 * self.owner()], 0)
    }
    fn own_city_flags(&mut self, i: i32) -> u16 {
        let r = self.rec(i);
        u16::from_le_bytes([self.p[r][4], self.p[r][5]])
    }
    fn set_own_city_flags(&mut self, i: i32, f: u16) {
        let r = self.rec(i);
        self.p[r][4..6].copy_from_slice(&f.to_le_bytes());
    }
    fn city_make_capital(&mut self) {
        let r = self.rec(self.city());
        let f = u16::from_le_bytes([self.p[r][4], self.p[r][5]]) | 0x10;
        self.p[r][4..6].copy_from_slice(&f.to_le_bytes());
        self.p[r][0x60] = self.m[9];
    }
    fn senate_announce(&mut self) {
        unreachable!("the arms sweeps hold `announce` at 0");
    }
    fn regen_roads(&mut self) {
        let n = self.rec_name(self.city());
        self.log.push(format!("s13:{n}"));
    }
    fn fort_probe(&mut self) {
        self.log.push("s14:p0+0x0".to_string());
    }
}

/// One row: the head, then the arms; `eax` is the last stub's answer.
pub(super) fn run_row(
    a: &[i32],
    m: &mut [u8],
    c: &mut [Vec<u8>],
    p: &mut [Vec<u8>],
    log: &mut Vec<String>,
) -> i64 {
    let mut v = V {
        a,
        m,
        c,
        p,
        log,
        is_calls: 0,
        tribe_calls: 0,
        cap: (0, 0),
    };
    activate_head_steps(&mut v, a[0], a[1], a[2]);
    activate_arms_steps(&mut v, a[0], a[1]);
    i64::from(a[3 + 14])
}
