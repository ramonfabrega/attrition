//! The emulated original's side of `BuildTypeData::blocked_location@006375b0`
//! (item 3364): the rows of `tools/emu/sweep_build_type_data_blocked_location*.py`
//! as a [`LocationView`], answered call by call to
//! [`blocked_location_steps`]. Shared by the sweep's variants, whose windows
//! are laid alike (`genflags.py`'s layout, three live leaders of eight (0, 1 and 7), three
//! cities and two forts a leader, two objects a leader, two kinds of three
//! `get_build` objects).
//!
//! Row: `a[0..7]` the arguments (`x`, `y`, `tx`, `ty`, `who`, `exclude`, the
//! `param_7` pointer), `a[7..35]` the 28 stubs' single answers (unread),
//! `a[35]` the count `calc_gather` writes, then each stub's `--seq` run in
//! [`SEQ`]'s order and last the five slot flags (`1`: the image function).
//!
//! Stubs: 0 `is`, 1 `is_city`, 2 `is_fort`, 3 `is_gather_type`, 4 `is_flat`,
//! 5 `is_wonder_type`, 6 `get_build` (slot `+0xac`, answers from the object's
//! word at `+0x40`), 7 the object's slot `+0x3c` (the same), 8 the build's
//! slot `+0x50`, 9 `get_tregion`, 10 `non_friendly_territory`, 11
//! `has_tribe_bonus`, 12 `in_enemy_territory`, 13 `has_preq`, 14 `valid`, 15
//! `is_ocean`, 16 `get_town`, 17 `get_queue`, 18 `count_buildings`, 19
//! `get_radius`, 20 `get_farm_limit`, 21 `num_wonders`, 22 `count_water`, 23
//! `check_land_adjacent`, 24 `count_mountains_adjacent`, 25
//! `count_trees_adjacent`, 26 `was_seen`, 27 `calc_gather`.
//!
//! Windows: `m` the type, `c0` the world (`+0` width, `+0x134` the cells), `c1`
//! the game, `c2` the constants, `c3` the cities' table, `c4` `div_3_table`,
//! `c5` `GameAccess::objects`, `c6` the regions. `p0` the `param_7` buffer;
//! `p1..` the leaders' slices (six a leader), then the unbuilt tables (two a
//! leader, cities then forts), the built forts' and objects' table words, and
//! the placed records — see the `W_*` constants.

use crate::location::{Konst, LocArgs, LocationView, blocked_location_steps};

const R: usize = 3;
/// The leaders that live: slot `k` of the placed records is leader `LIVE[k]`.
const LIVE: [i32; R] = [0, 1, 7];

fn slot(i: i32) -> usize {
    LIVE.iter().position(|&p| p == i).expect("a live leader")
}
const NC: usize = 3;
const NF: usize = 2;
const NO: usize = 2;
const NG: usize = 3;

const P_LEAD: usize = 1;
const P_UCITY_TAB: usize = P_LEAD + 8 * 6;
const P_UFORT_TAB: usize = P_UCITY_TAB + 2 * 8;
const P_FORT_WORD: usize = P_UFORT_TAB + 2 * 8;
const P_OBJ_WORD: usize = P_FORT_WORD + 8;
const W_CITIES: usize = P_OBJ_WORD + 8; // 97
const W_FORTS: usize = W_CITIES + R * (1 + NC);
const W_OBJS: usize = W_FORTS + R * (1 + NF);
const W_GOBJS: usize = W_OBJS + R * (1 + NO);
const W_UCITY: usize = W_GOBJS + 2 * (1 + NG);
const W_OBJX: usize = W_UCITY + 2 * R;
const W_GBUILD: usize = W_OBJX + R * NO * 2;
const W_CELLS: usize = W_GBUILD + 2 * NG;

/// `--seq` runs, in the line's order: (stub, length).
const SEQ: [(usize, usize); 26] = [
    (0, 10),
    (1, 4),
    (2, 3),
    (3, 1),
    (4, 1),
    (5, 1),
    (8, 40),
    (9, 1),
    (10, 1),
    (11, 2),
    (12, 1),
    (13, 1),
    (14, 10),
    (15, 10),
    (16, 1),
    (17, 3),
    (18, 6),
    (19, 3),
    (20, 1),
    (21, 1),
    (22, 1),
    (23, 2),
    (24, 1),
    (25, 1),
    (26, 1),
    (27, 1),
];

const BUILD_VFTABLE: u32 = 0xb4_2174;

fn rd(w: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(w[off..off + 4].try_into().expect("four bytes"))
}
fn i16_at(w: &[u8], off: usize) -> i32 {
    i32::from(i16::from_le_bytes([w[off], w[off + 1]]))
}
fn hex4(v: i32) -> String {
    v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
}

struct Emu<'a> {
    a: &'a [i32],
    m: &'a [u8],
    c: &'a [Vec<u8>],
    p: &'a mut [Vec<u8>],
    base: [usize; 28],
    len: [usize; 28],
    n: [usize; 28],
    /// `--deref` columns: get_tregion (0, 1), valid (2, 3), is_ocean (4, 5),
    /// was_seen (6, 7).
    der: [Vec<String>; 8],
    log: &'a mut Vec<String>,
    tx: i32,
    ty: i32,
}

impl Emu<'_> {
    fn ans(&mut self, stub: usize) -> i32 {
        let i = self.n[stub].min(self.len[stub] - 1);
        self.n[stub] += 1;
        self.a[self.base[stub] + i]
    }
    fn flag(&self, k: usize) -> bool {
        self.a[36 + 108 + k] != 0
    }
    fn deref(&mut self, col: usize, x: i32, y: i32) {
        self.der[col].push(hex4(x));
        self.der[col + 1].push(hex4(y));
    }
    fn lead(&self, i: i32, s: usize) -> &[u8] {
        &self.p[P_LEAD + 6 * i as usize + s]
    }
    fn width(&self) -> i32 {
        rd(&self.c[0], 0)
    }
    fn div3t(&self, i: i32) -> i32 {
        rd(&self.c[4], 4 * i as usize)
    }
    fn list_win(fort: bool, i: i32) -> usize {
        W_UCITY + 2 * slot(i) + usize::from(fort)
    }
    fn tab_win(fort: bool, i: i32) -> usize {
        (if fort { P_UFORT_TAB } else { P_UCITY_TAB }) + 2 * i as usize
    }
    /// The entry's `(object window, build window)`.
    fn entry(&self, fort: bool, i: i32, k: i32) -> (usize, usize) {
        let w = &self.p[Self::list_win(fort, i)];
        let o = i16_at(w, 4 * k as usize);
        let kind = i32::from(w[4 * k as usize + 2] as i8);
        assert!((0..2).contains(&kind) && (0..NG as i32).contains(&o), "entry {kind} {o}");
        let gobj = W_GOBJS + (1 + NG) * kind as usize;
        (gobj + 1 + o as usize, W_GBUILD + NG * kind as usize + o as usize)
    }
    fn is_id(&mut self, id: i32) -> bool {
        self.log.push(format!("s0:m+0x0,{id},0"));
        self.ans(0) != 0
    }
    fn city_rec(i: i32, j: i32) -> usize {
        W_CITIES + (1 + NC) * slot(i) + 1 + j as usize
    }
    fn fort_rec(i: i32, j: i32) -> usize {
        W_FORTS + (1 + NF) * slot(i) + 1 + j as usize
    }
    fn obj_rec(i: i32, idx: i32) -> usize {
        assert!((0..NO as i32).contains(&idx), "object {idx}");
        W_OBJS + (1 + NO) * slot(i) + 1 + idx as usize
    }
    fn obd(i: i32, idx: i32) -> usize {
        W_OBJX + 4 * slot(i) + 2 * idx as usize + 1
    }
    fn finish(&mut self) {
        // `--record-order` prints `-` for a row that called nothing.
        if self.log.is_empty() {
            self.log.push("-".to_string());
        }
        for d in &self.der {
            self.log.push(if d.is_empty() {
                "-".to_string()
            } else {
                d.join("/")
            });
        }
    }
}

impl LocationView for Emu<'_> {
    type B = usize;

    fn type_index(&self) -> i32 {
        rd(self.m, 4)
    }
    fn flags(&self) -> u32 {
        rd(self.m, 0x2c0) as u32
    }
    fn domain(&self) -> i32 {
        rd(self.m, 0x218)
    }
    fn area(&self) -> i32 {
        rd(self.m, 0x238).wrapping_mul(rd(self.m, 0x234))
    }
    fn is(&mut self, id: i32) -> bool {
        self.is_id(id)
    }
    fn is_city(&mut self) -> bool {
        if self.flag(0) {
            return self.is_id(0x19e);
        }
        self.log.push("s1:m+0x0".to_string());
        self.ans(1) != 0
    }
    fn is_fort(&mut self) -> bool {
        if self.flag(1) {
            return self.is_id(0x1bb);
        }
        self.log.push("s2:m+0x0".to_string());
        self.ans(2) != 0
    }
    fn gathers(&mut self) -> bool {
        if self.flag(2) {
            return self.flags() & 0x40 != 0;
        }
        self.log.push("s3:m+0x0".to_string());
        self.ans(3) != 0
    }
    fn flat(&mut self) -> bool {
        if self.flag(3) {
            return self.flags() & 0x1000_0000 != 0;
        }
        self.log.push("s4:m+0x0".to_string());
        self.ans(4) != 0
    }
    fn is_wonder(&mut self) -> bool {
        if self.flag(4) {
            return (0x20e..0x21f).contains(&self.type_index());
        }
        self.log.push("s5:m+0x0".to_string());
        self.ans(5) != 0
    }

    fn div3(&self, i: i32) -> i32 {
        self.div3t(i)
    }
    fn sem8(&self) -> bool {
        self.c[1][0x821] & 8 != 0
    }
    fn konst(&self, k: Konst) -> i32 {
        rd(
            &self.c[2],
            match k {
                Konst::CitySpacing => 0x138,
                Konst::RelaxCitySpacing => 0x13c,
                Konst::FirstCityNearCoast => 0x140,
                Konst::FortSpacing => 0x144,
                Konst::FortToEnemyCitySpacing => 0x148,
            },
        )
    }
    fn cell_region(&self, cx: i32, cy: i32) -> i32 {
        let idx = self.width().wrapping_mul(cy).wrapping_add(cx);
        i16_at(&self.p[W_CELLS], 0x1c * idx as usize + 4)
    }
    fn region_size(&self, r: i32) -> i32 {
        rd(&self.c[6], 0x88 * r as usize + 0x14)
    }

    fn alive(&self, i: i32) -> bool {
        self.lead(i, 0)[0] & 1 != 0
    }
    fn city_num(&self, i: i32) -> i32 {
        rd(self.lead(i, 2), 0)
    }
    fn team(&self, i: i32) -> i32 {
        rd(self.lead(i, 0), 8)
    }
    fn rel(&self, i: i32, k: i32) -> i32 {
        rd(self.lead(i, 1), 4 * k as usize)
    }
    fn reg_cities(&self, i: i32, tr: i32) -> i32 {
        i16_at(self.lead(i, 3), 2 * tr as usize)
    }
    fn reg_forts(&self, i: i32, tr: i32) -> i32 {
        i16_at(self.lead(i, 4), 2 * tr as usize)
    }
    fn reg_terr(&self, i: i32, tr: i32) -> i32 {
        let w = self.lead(i, 5);
        i32::from(u16::from_le_bytes([w[2 * tr as usize], w[2 * tr as usize + 1]]))
    }
    fn city_count(&self, i: i32) -> i32 {
        rd(self.lead(i, 2), 0x10)
    }
    fn fort_count(&self, i: i32) -> i32 {
        rd(self.lead(i, 2), 0x30)
    }

    fn tregion(&mut self, x: i32, y: i32) -> i32 {
        self.log.push("s9:c0+0x0,sp-0x4c,sp-0x50".to_string());
        self.deref(0, x, y);
        self.ans(9)
    }
    fn non_friendly(&mut self, who: i32) -> i32 {
        self.log
            .push(format!("s10:m+0x0,{},{},{who}", self.tx, self.ty));
        self.ans(10)
    }
    fn in_enemy(&mut self, who: i32) -> bool {
        self.log
            .push(format!("s12:m+0x0,{},{},{who}", self.tx, self.ty));
        self.ans(12) != 0
    }
    fn has_tribe_bonus(&mut self, who: i32, bonus: i32) -> bool {
        self.log.push(format!("s11:r{who}+0x0,{bonus}"));
        self.ans(11) != 0
    }
    fn has_preq(&mut self, who: i32) -> bool {
        self.log.push(format!("s13:r{who}+0x0,687"));
        self.ans(13) != 0
    }
    fn valid(&mut self, x: i32, y: i32) -> bool {
        self.log.push("s14:c0+0x0,sp-0x48,sp-0x4c".to_string());
        self.deref(2, x, y);
        self.ans(14) != 0
    }
    fn is_ocean(&mut self, x: i32, y: i32) -> bool {
        self.log.push("s15:c0+0x0,sp-0x48,sp-0x4c".to_string());
        self.deref(4, x, y);
        self.ans(15) != 0
    }
    fn get_radius(&mut self, owner: i32, t: i32) -> i32 {
        self.log.push(format!("s19:r{owner}+0x0,{t}"));
        self.ans(19)
    }
    fn get_town(&mut self, x: i32, y: i32, who: i32) -> i32 {
        self.log.push(format!("s16:m+0x0,{x},{y},{who}"));
        self.ans(16)
    }
    fn count_water(&mut self) -> i32 {
        self.log
            .push(format!("s22:m+0x0,{},{}", self.tx, self.ty));
        self.ans(22)
    }
    fn check_land_adjacent(&mut self, who: i32) -> i32 {
        self.log
            .push(format!("s23:m+0x0,{},{},{who}", self.tx, self.ty));
        self.ans(23)
    }
    fn count_mountains(&mut self, ring: i32) -> i32 {
        self.log
            .push(format!("s24:m+0x0,{},{},{ring}", self.tx, self.ty));
        self.ans(24)
    }
    fn count_trees(&mut self, ring: i32) -> i32 {
        self.log
            .push(format!("s25:m+0x0,{},{},{ring}", self.tx, self.ty));
        self.ans(25)
    }
    fn was_seen(&mut self, hx: i32, hy: i32, who: i32) -> bool {
        self.log
            .push(format!("s26:c0+0x0,sp-0x58,sp-0xc,{who}"));
        self.deref(6, hx, hy);
        self.ans(26) != 0
    }
    fn calc_gather(&mut self, who: i32, md: Option<usize>) -> i32 {
        let md = md.map_or_else(|| "0".to_string(), |b| format!("p{b}+0x98"));
        self.log.push(format!(
            "s27:m+0x0,sp-0x70,sp-0xc,sp-0x8,0,-1,{who},0,{},{},1,1,0,{md}",
            self.tx, self.ty
        ));
        let _ = self.ans(27);
        self.a[35]
    }
    fn store_gather(&mut self, n: i32) {
        self.p[0][0..4].copy_from_slice(&n.to_le_bytes());
    }

    fn unbuilt_count(&self, fort: bool, i: i32) -> i32 {
        rd(&self.p[Self::tab_win(fort, i)], 0)
    }
    fn unbuilt_o(&self, fort: bool, i: i32, k: i32) -> i32 {
        i16_at(&self.p[Self::list_win(fort, i)], 4 * k as usize)
    }
    fn unbuilt_started(&mut self, fort: bool, i: i32, k: i32) -> bool {
        let (ow, bw) = self.entry(fort, i, k);
        self.log.push(format!("s6:p{ow}+0x0"));
        if rd(&self.p[bw], 0) as u32 == BUILD_VFTABLE {
            self.p[bw][8] & 2 != 0
        } else {
            self.log.push(format!("s8:p{bw}+0x0"));
            self.ans(8) != 0
        }
    }
    fn unbuilt_tx(&mut self, fort: bool, i: i32, k: i32) -> i32 {
        let (ow, bw) = self.entry(fort, i, k);
        self.log.push(format!("s6:p{ow}+0x0"));
        self.div3t((rd(&self.p[bw], 0x10) ^ 0x63637) >> 6)
    }
    fn unbuilt_ty(&mut self, fort: bool, i: i32, k: i32) -> i32 {
        let (ow, bw) = self.entry(fort, i, k);
        self.log.push(format!("s6:p{ow}+0x0"));
        self.div3t((rd(&self.p[bw], 0x14) ^ 0x63637) >> 6)
    }

    fn city_alive(&self, i: i32, j: i32) -> bool {
        self.p[Self::city_rec(i, j)][4] & 1 != 0
    }
    fn city_reg(&self, i: i32, j: i32) -> i32 {
        i16_at(&self.p[Self::city_rec(i, j)], 0xa)
    }
    fn city_tx(&self, i: i32, j: i32) -> i32 {
        self.div3t(rd(&self.p[Self::city_rec(i, j)], 0xc) >> 6)
    }
    fn city_ty(&self, i: i32, j: i32) -> i32 {
        self.div3t(rd(&self.p[Self::city_rec(i, j)], 0x10) >> 6)
    }
    fn city_owner(&self, i: i32, j: i32) -> i32 {
        i32::from(self.p[Self::city_rec(i, j)][0x5e] as i8)
    }
    fn city_bld(&self, i: i32, j: i32) -> i32 {
        i16_at(&self.p[Self::city_rec(i, j)], 8)
    }
    fn object_type(&self, owner: i32, idx: i32) -> i32 {
        rd(&self.p[Self::obd(owner, idx) - 1], 4)
    }
    fn count_buildings(&mut self, i: i32, j: i32) -> i32 {
        self.log.push(format!(
            "s18:p{}+0x0,{},0,0",
            Self::city_rec(i, j),
            self.type_index()
        ));
        self.ans(18)
    }
    fn farm_limit(&mut self, i: i32, j: i32) -> i32 {
        self.log.push(format!("s20:p{}+0x0", Self::city_rec(i, j)));
        self.ans(20)
    }
    fn num_wonders(&mut self, i: i32, j: i32) -> i32 {
        self.log
            .push(format!("s21:p{}+0x0,1", Self::city_rec(i, j)));
        self.ans(21)
    }
    fn fort_alive(&self, i: i32, j: i32) -> bool {
        self.p[Self::fort_rec(i, j)][6] & 1 != 0
    }
    fn fort_tile(&self, i: i32, j: i32) -> (i32, i32) {
        let idx = i16_at(&self.p[Self::fort_rec(i, j)], 2);
        let w = &self.p[Self::obj_rec(i, idx)];
        (
            self.div3t((rd(w, 0x10) ^ 0x63637) >> 6),
            self.div3t((rd(w, 0x14) ^ 0x63637) >> 6),
        )
    }

    fn bdata(&mut self, who: i32, idx: i32) -> usize {
        self.log
            .push(format!("s7:p{}+0x0", Self::obj_rec(who, idx)));
        Self::obd(who, idx)
    }
    fn bd_queue_len(&self, b: usize) -> i32 {
        i32::from(self.p[b][0x82])
    }
    fn get_queue(&mut self, b: usize, q: i32) -> i32 {
        self.log.push(format!("s17:p{b}+0x0,{q}"));
        self.ans(17)
    }
    fn bd_city(&self, b: usize) -> i32 {
        i16_at(&self.p[b], 0x72)
    }
}

/// One row: the verdict, the recorded calls and the `--deref` columns left in
/// `log`, `param_7`'s buffer written in `p[0]`.
pub(super) fn run(a: &[i32], m: &[u8], c: &[Vec<u8>], p: &mut [Vec<u8>], log: &mut Vec<String>) -> i64 {
    let mut base = [0usize; 28];
    let mut len = [1usize; 28];
    let mut at = 36;
    for (s, l) in SEQ {
        base[s] = at;
        len[s] = l;
        at += l;
    }
    let args = LocArgs {
        px: a[0],
        py: a[1],
        tx: a[2],
        ty: a[3],
        who: a[4],
        exclude: a[5],
        has_out: a[6] != 0,
    };
    let mut e = Emu {
        a,
        m,
        c,
        p,
        base,
        len,
        n: [0; 28],
        der: Default::default(),
        log,
        tx: a[2],
        ty: a[3],
    };
    let r = blocked_location_steps(&mut e, &args);
    e.finish();
    i64::from(r)
}
