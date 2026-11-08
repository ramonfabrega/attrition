//! `BuildTypeData::calc_gather@00639e40`'s view over the emulated original's windows (item 3386): the
//! port every slice of the sweep shares (`build_type_data_calc_gather_{a,..}.rs`). The slices run one
//! `sweep_gen.py` line each with the same stubs, windows and columns (`tools/emu/sweep_build_type_data_calc_gather_*.py`)
//! and differ in what they draw and where the call ends.
//!
//! **Row layout.** `a[0..13]` the thirteen stack arguments; `a[13 + k]` stub `k`'s answer (49 stubs, `STUB_*`);
//! `a[62..66]` the four `--outarg` values (`corner_tile`'s two, the mountain's and the cliff's distance);
//! then every `--seq` run, in `SEQS`' order.
//!
//! **Windows.** `m` the type; `c0` the constants (`0xc061e4`), `c1` the game (`0xc061e8`), `c2` the cities
//! cell, `c3` the leaders cell, `c4` the world cell, `c5` the lands cell, `c6` the mountains cell; `p0` the
//! out array, `p1` the count, `p2` the rate, `p3` the mining list; `p4` the `div_3_table`; `p5..p15` the
//! mountain tables (`Q`, `T1`, `T2`, two ranges, their `S` and four coordinate arrays); then the stubs'
//! answer windows `p16` (`ConstArray::operator[]`), `p17` (`Leaders::operator[]`), `p18` (`Lands::operator[]`),
//! `p19` (`Array<TCoordData>::operator[]`), `p20`/`p21` (`mnt_loc_x`/`_y`).
//!
//! **Calls.** The log is `--record-order`'s, one token a call (`s<k>:<ecx>,<args>`), then the 22 `--deref`
//! columns (`9:0`, `9:1`, `10:0`, ..), each `/`-joined or `-`.

use crate::calc_gather_type_view::{Konst, TypeGatherArgs, TypeGatherView, type_calc_gather_steps};

/// The function-local static `MiningList` as the log names it: the guard word at `0xee14a4` is the `--at`
/// window `c7`, so the list at `0xee14a8` is `c7+0x4`.
const STATIC_LIST: &str = "c7+0x4";
const STATIC_GUARD: &str = "c7+0x0";
/// The `atexit` destructor stub's address (`0xab8360`) and the error object (`0xc8d3a8`).
const STATIC_ATEXIT: i32 = 0x00ab_8360;
const ERROR_OBJ: i32 = 0x00c8_d3a8;

/// First answer column and the number of stubs.
const ANS: usize = 13;
/// `--outarg` columns.
const OUTS: usize = 62;
/// `--seq` runs: the stub, the values a row. Columns follow `OUTS + 4` in this order.
const SEQS: [(usize, usize); 18] = [
    (8, 24),
    (2, 6),
    (4, 12),
    (5, 14),
    (9, 12),
    (10, 8),
    (11, 8),
    (13, 12),
    (15, 12),
    (20, 24),
    (21, 16),
    (22, 16),
    (23, 24),
    (24, 24),
    (25, 8),
    (28, 12),
    (34, 6),
    (43, 8),
];
/// The row's column count.
pub const COLUMNS: usize = 312;

/// The panic payload that ends a call at its `--stop-stub`.
pub struct Stop;

/// What a slice varies.
#[derive(Clone, Copy)]
pub struct Cfg {
    /// The `--stop-stub`: the call ends where this stub is first entered.
    pub stop: Option<usize>,
    /// The function-local static is uninitialised (`--tls 8:-2`, the guard word `-1`): the first call
    /// runs the guard header, the constructor, `atexit` and the footer.
    pub first_call: bool,
}

fn rd32(w: &[u8], o: usize) -> i32 {
    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
}

/// A local at `ebp - off` as the recorder names it: its depth under the harness's `esp` (thirteen
/// arguments, the return address and the saved `ebp` sit between).
fn sp(off: usize) -> String {
    format!("sp-{:#x}", 0x3c + off)
}

fn hex(v: i32) -> String {
    v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
}

struct V<'a> {
    a: &'a [i32],
    m: &'a [u8],
    c: &'a [Vec<u8>],
    p: &'a [Vec<u8>],
    log: &'a mut Vec<String>,
    turn: [usize; 64],
    deref: [Vec<String>; 22],
    cfg: Cfg,
    list: bool,
}

impl V<'_> {
    fn answer(&self, k: usize) -> i32 {
        self.a[ANS + k]
    }

    /// A `--seq` stub's next value.
    fn seq(&mut self, k: usize) -> i32 {
        let mut at = OUTS + 4;
        for &(s, n) in &SEQS {
            if s == k {
                let i = self.turn[k].min(n - 1);
                self.turn[k] += 1;
                return self.a[at + i];
            }
            at += n;
        }
        unreachable!("stub {k} is not a --seq stub")
    }

    /// One recorded call, `s<k>:<ecx>,<args>`; the `--stop-stub` ends the call here.
    fn rec(&mut self, k: usize, ecx: &str, args: &[String]) {
        let mut t = format!("s{k}:{ecx}");
        for v in args {
            t.push(',');
            t.push_str(v);
        }
        self.log.push(t);
        if self.cfg.stop == Some(k) {
            std::panic::resume_unwind(Box::new(Stop));
        }
    }

    fn recn(&mut self, k: usize, ecx: &str, args: &[i32]) {
        let args: Vec<String> = args.iter().map(i32::to_string).collect();
        self.rec(k, ecx, &args);
    }

    fn d(&mut self, i: usize, x: i32, y: i32) {
        self.deref[i].push(hex(x));
        self.deref[i + 1].push(hex(y));
    }

    fn konst_at(&self, off: usize) -> i32 {
        rd32(&self.c[0], off)
    }

    /// `Leaders::operator[](who)`: the leader record, `p17`.
    fn leader(&mut self, who: i32) {
        self.recn(3, "c3+0x0", &[who]);
    }
}

impl TypeGatherView for V<'_> {
    fn enhancer_amount(&mut self, who: i32, city: i32, good: i32) -> i32 {
        self.recn(0, "c2+0x0", &[who]);
        let r0 = self.answer(0).to_string();
        self.recn(1, &r0, &[city]);
        let ecx = rd32(&self.p[16], 0).to_string();
        self.recn(2, &ecx, &[good, 100]);
        self.seq(2)
    }

    fn resource_bonuses(&mut self, who: i32, out: &mut [i32; 6]) {
        // `LeaderData::calc_resource_bonuses@006db030`, run for real by the emulator over the leader
        // record `Leaders::operator[]` answered; its own callees are the `has_*` stubs.
        self.leader(who);
        let this = "p17+0x0";
        let pct = |k: i32, v: i32| (k.wrapping_add(100)).wrapping_mul(v) / 100;
        self.recn(4, this, &[0xd]);
        if self.seq(4) != 0 {
            out[5] = pct(self.konst_at(0x754), out[5]);
        }
        for (wonder, goods, off) in [
            (0x20e, 0usize, 0x430usize),
            (0x20f, 2, 0x438),
            (0x210, 3, 0x448),
            (0x217, 4, 0x4d0),
            (0x21b, 2, 0x4fc),
            (0x21c, 5, 0x540),
            (0x214, 1, 0x490),
        ] {
            self.recn(5, this, &[wonder]);
            if self.seq(5) != 0 {
                if wonder == 0x210 {
                    out[goods] = out[goods].wrapping_add(self.konst_at(off).wrapping_shl(4));
                } else {
                    out[goods] = pct(self.konst_at(off), out[goods]);
                }
            }
        }
        self.recn(6, this, &[0x2b7]);
        if self.answer(6) != 0 {
            for (i, o) in out.iter_mut().enumerate() {
                if i != 3 {
                    *o = pct(self.konst_at(0x424), *o);
                }
            }
        }
        assert!(
            rd32(&self.c[1], 0x820) >> 16 & 2 == 0,
            "the sweep keeps the CTW bonus bit clear"
        );
    }

    fn is_flat(&mut self) -> bool {
        self.recn(7, "m+0x0", &[]);
        self.answer(7) != 0
    }

    fn is_type(&mut self, ty: i32) -> bool {
        self.recn(8, "m+0x0", &[ty, 0]);
        self.seq(8) != 0
    }

    fn max_flat_gatherers(&mut self) -> i32 {
        1
    }

    fn x_size(&mut self) -> i32 {
        rd32(self.m, 0x234)
    }

    fn y_size(&mut self) -> i32 {
        rd32(self.m, 0x238)
    }

    fn tile_valid(&mut self, x: i32, y: i32) -> bool {
        self.d(0, x, y);
        self.seq(9) != 0
    }

    fn get_whose(&mut self, cx: i32, cy: i32) -> i32 {
        self.d(2, cx, cy);
        self.seq(10)
    }

    fn is_ally(&mut self, who: i32, other: i32) -> bool {
        self.leader(who);
        self.recn(11, "p17+0x0", &[other]);
        self.seq(11) != 0
    }

    fn get_land(&mut self, x: i32, y: i32) -> i32 {
        self.d(4, x, y);
        self.answer(12)
    }

    fn is_river(&mut self, x: i32, y: i32) -> bool {
        self.d(6, x, y);
        self.seq(13) != 0
    }

    fn land_good(&mut self, land: i32, k: i32) -> i32 {
        self.recn(14, "c5+0x0", &[land]);
        rd32(&self.p[18], 4 + 4 * k as usize)
    }

    fn land_amount(&mut self, land: i32, k: i32) -> i32 {
        self.recn(14, "c5+0x0", &[land]);
        rd32(&self.p[18], 0x14 + 4 * k as usize)
    }

    fn gathers(&mut self, good: i32) -> bool {
        self.recn(15, "m+0x0", &[good]);
        self.seq(15) != 0
    }

    fn has_tribe_bonus(&mut self, who: i32, k: i32) -> bool {
        self.leader(who);
        self.recn(4, "p17+0x0", &[k]);
        self.seq(4) != 0
    }

    fn has_wonder(&mut self, who: i32, k: i32) -> bool {
        self.leader(who);
        self.recn(5, "p17+0x0", &[k]);
        self.seq(5) != 0
    }

    fn max_knowledge_gatherers(&mut self, city: i32, who: i32) -> i32 {
        self.recn(16, "m+0x0", &[city, who]);
        self.answer(16)
    }

    fn get_university(&mut self, who: i32) -> i32 {
        self.leader(who);
        self.recn(17, "p17+0x0", &[]);
        self.answer(17)
    }

    fn gather_radius(&mut self) -> i32 {
        self.recn(18, "m+0x0", &[]);
        self.answer(18)
    }

    fn konst(&mut self, k: Konst) -> i32 {
        let off = match k {
            Konst::PeasantRate => 0x280,
            Konst::OilRate => 0x29c,
            Konst::ScholarRate(i) => 0x284 + 4 * i as usize,
            Konst::JapaneseFishingBoats => 0x7ac,
            Konst::EgyptianFarmWealth => 0x658,
            Konst::RiverResourceValue => 0xc40,
            Konst::IroquoisFood => 0x80c,
            Konst::FrenchWoodies => 0x6c8,
            Konst::TajFarms => 0x508,
            Konst::KremlinFarms => 0x518,
            Konst::GermanMiners => 0x728,
            Konst::IncaWealthPerMiner => 0x59c,
        };
        self.konst_at(off)
    }

    fn static_init(&mut self) {
        if self.cfg.first_call {
            // `_Init_thread_header(&guard)` is entered with the guard word (`-1`) still in `ecx`; the
            // constructor leaves the list in `ecx` for `atexit` and the footer.
            self.rec(31, "-1", &[STATIC_GUARD.to_string()]);
            self.rec(30, STATIC_LIST, &[]);
            self.rec(32, STATIC_LIST, &[STATIC_ATEXIT.to_string()]);
            self.rec(33, STATIC_LIST, &[STATIC_GUARD.to_string()]);
        }
    }

    fn list_clear(&mut self) {
        self.rec(29, STATIC_LIST, &[]);
    }

    fn corner_tile(&mut self, tx: i32, ty: i32, site: i32) -> (i32, i32) {
        // `corner_tile(tx, ty, &x, &y)` writes both through its arguments (`--outarg 19:2`, `19:3`); the
        // camp's locals are at `ebp - 0x78`/`- 0x74`, the mine's at `- 0x64`/`- 0x60`.
        let (x, y) = (self.a[OUTS], self.a[OUTS + 1]);
        let (lx, ly) = if site == 1 {
            (0x78, 0x74)
        } else {
            (0x64, 0x60)
        };
        self.rec(
            19,
            "m+0x0",
            &[tx.to_string(), ty.to_string(), sp(lx), sp(ly)],
        );
        (x, y)
    }

    fn div3(&mut self, i: i32) -> i32 {
        rd32(&self.p[4], 4 * i as usize)
    }

    fn circle_len(&mut self, ring: i32) -> i32 {
        crate::ai_place::circle().radius[ring as usize] as i32
    }

    fn circle_x(&mut self, i: i32) -> i32 {
        crate::ai_place::circle().x[i as usize]
    }

    fn circle_y(&mut self, i: i32) -> i32 {
        crate::ai_place::circle().y[i as usize]
    }

    fn cell_valid(&mut self, cx: i32, cy: i32) -> bool {
        self.d(8, cx, cy);
        self.seq(20) != 0
    }

    fn vector_dist(&mut self, dx: i32, dy: i32) -> i32 {
        crate::world::vector_dist(dx.wrapping_abs(), dy.wrapping_abs())
    }

    fn get_who(&mut self, cx: i32, cy: i32) -> i32 {
        self.d(10, cx, cy);
        self.seq(21)
    }

    fn report_no_who(&mut self, site: i32, who: i32) {
        // `Error::report(who < 0, &msg, line, &file, &ignore_always, 1)`; the two `String`s are locals of
        // the site's arm, the byte is the arm's own `ignore_always` (`0xee1099`..`0xee109b`).
        let (msg, file, ignore) = match site {
            0xa82 => (0x2a0, 0x2b4, 0x00ee_1099),
            0xa18 => (0x278, 0x28c, 0x00ee_109a),
            _ => (0x250, 0x264, 0x00ee_109b),
        };
        self.rec(
            37,
            &ERROR_OBJ.to_string(),
            &[
                i32::from(who < 0).to_string(),
                sp(msg),
                site.to_string(),
                sp(file),
                ignore.to_string(),
                "1".to_string(),
            ],
        );
    }

    fn is_gathered_from(&mut self, tx: i32, ty: i32) -> bool {
        self.d(12, tx, ty);
        self.seq(22) != 0
    }

    fn is_tree_at(&mut self, tx: i32, ty: i32) -> bool {
        self.d(14, tx, ty);
        self.seq(23) != 0
    }

    fn is_mountain_at(&mut self, tx: i32, ty: i32) -> bool {
        self.d(16, tx, ty);
        self.seq(24) != 0
    }

    fn is_forest(&mut self, cx: i32, cy: i32) -> bool {
        self.d(18, cx, cy);
        self.seq(25) != 0
    }

    fn get_land_cell(&mut self, cx: i32, cy: i32) -> i32 {
        self.d(20, cx, cy);
        self.answer(26)
    }

    fn list_add(&mut self, tx: i32, ty: i32) {
        self.recn(27, STATIC_LIST, &[tx, ty]);
    }

    fn list_find(&mut self, tx: i32, ty: i32) -> i32 {
        self.recn(28, "p3+0x0", &[tx, ty]);
        self.seq(28)
    }

    fn total_gather_access(&mut self, own_list: bool, x: i32, y: i32, who: i32) -> i32 {
        let list = if own_list { "p3+0x0" } else { STATIC_LIST }.to_string();
        self.rec(
            34,
            "m+0x0",
            &[list, x.to_string(), y.to_string(), who.to_string()],
        );
        self.seq(34)
    }

    fn list_mtn(&mut self) -> i32 {
        i32::from(self.p[3][0x1c] as i8)
    }

    fn list_cliff(&mut self) -> i32 {
        i32::from(self.p[3][0x1d] as i8)
    }

    fn get_region(&mut self, cx: i32, cy: i32, second: bool) -> i32 {
        let _ = (cx, cy);
        let (lx, ly) = if second {
            (0x12c, 0x128)
        } else {
            (0x124, 0x120)
        };
        self.rec(39, "c4+0x0", &[sp(lx), sp(ly)]);
        self.answer(39)
    }

    fn mountains_find_nearest(&mut self, ox: i32, oy: i32, region: i32) -> (i32, i32) {
        self.rec(
            41,
            "c6+0x4",
            &[ox.to_string(), oy.to_string(), region.to_string(), sp(0x8c)],
        );
        (self.answer(41), self.a[OUTS + 2])
    }

    fn cliffs_find_nearest(&mut self, ox: i32, oy: i32, region: i32) -> (i32, i32) {
        // `get_scary_cliffs()` (stub 40, not recorded: its `ecx` is whatever the caller left) answers the
        // cliffs object, `find_nearest`'s `this` is four past it.
        let ecx = self.answer(40).wrapping_add(4).to_string();
        self.rec(
            42,
            &ecx,
            &[ox.to_string(), oy.to_string(), region.to_string(), sp(0x84)],
        );
        (self.answer(42), self.a[OUTS + 3])
    }

    fn cliff_len(&mut self, cliff: i32) -> i32 {
        let ecx = self.answer(40).wrapping_add(0x40).to_string();
        self.recn(1, &ecx, &[cliff]);
        let obj = rd32(&self.p[16], 0).wrapping_add(0x1c).to_string();
        self.rec(43, &obj, &[]);
        self.seq(43)
    }

    fn cliff_tile(&mut self, cliff: i32, i: i32) -> (i32, i32) {
        let mut at = [0; 2];
        for (k, v) in at.iter_mut().enumerate() {
            let ecx = self.answer(40).wrapping_add(0x40).to_string();
            self.recn(1, &ecx, &[cliff]);
            let obj = rd32(&self.p[16], 0).wrapping_add(0x1c).to_string();
            self.recn(44, &obj, &[i]);
            *v = rd32(&self.p[19], 4 * k);
        }
        (at[0], at[1])
    }

    fn cliff_gather_size(&mut self, cliff: i32, who: i32) -> i32 {
        let ecx = self.answer(40).wrapping_add(4).to_string();
        let list = if self.list { "p3+0x0" } else { "0" }.to_string();
        self.rec(48, &ecx, &[cliff.to_string(), who.to_string(), list]);
        self.answer(48)
    }

    fn range_len(&mut self, mtn: i32) -> i32 {
        let k = rd32(&self.p[6], 4 * mtn as usize);
        self.rec(43, &format!("p{}+0x8", 8 + k), &[]);
        self.seq(43)
    }

    fn range_offset(&mut self, mtn: i32, i: i32) -> (i32, i32) {
        let k = rd32(&self.p[6], 4 * mtn as usize) as usize;
        (
            rd32(&self.p[12 + 2 * k], 4 * i as usize),
            rd32(&self.p[13 + 2 * k], 4 * i as usize),
        )
    }

    fn mnt_loc(&mut self, mtn: i32, y: bool, site: i32) -> i32 {
        let local = match (site, y) {
            (0, false) => 0x1ec,
            (0, true) => 0x1f4,
            (_, false) => 0x1fc,
            (_, true) => 0x1f8,
        };
        self.rec(
            if y { 46 } else { 45 },
            "c6+0x4",
            &[sp(local), mtn.to_string()],
        );
        rd32(&self.p[if y { 21 } else { 20 }], 0)
    }

    fn range_gather_size(&mut self, mtn: i32, x: i32, y: i32, who: i32) -> i32 {
        let k = rd32(&self.p[6], 4 * mtn as usize);
        let list = if self.list { "p3+0x0" } else { "0" }.to_string();
        self.rec(
            47,
            &format!("p{}+0x4", 8 + k),
            &[x.to_string(), y.to_string(), who.to_string(), list],
        );
        self.answer(47)
    }
}

/// Runs `type_calc_gather_steps` over a row's windows and writes the result into `p0..p2`, then pushes the
/// deref columns. A call that ends at the `--stop-stub` leaves the out parameters as far as it got.
pub fn run(a: &[i32], m: &[u8], c: &[Vec<u8>], p: &mut [Vec<u8>], log: &mut Vec<String>, cfg: Cfg) {
    assert_eq!(a.len(), COLUMNS, "the row's column layout drifted");
    let mut out = [0i32; 6];
    for (i, o) in out.iter_mut().enumerate() {
        *o = rd32(&p[0], 4 * i);
    }
    let (mut count, mut rate) = (rd32(&p[1], 0), rd32(&p[2], 0));
    let args = TypeGatherArgs {
        flag4: a[3] != 0,
        city: a[4],
        who: a[5],
        bonuses: a[6] != 0,
        tx: a[7],
        ty: a[8],
        rates: a[9] != 0,
        gatherers: a[10],
        list: a[12] != 0,
    };
    let derefs;
    {
        let mut v = V {
            a,
            m,
            c,
            p,
            log,
            turn: [0; 64],
            deref: Default::default(),
            cfg,
            list: a[12] != 0,
        };
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            type_calc_gather_steps(&mut v, &args, &mut out, &mut count, &mut rate);
        }));
        if let Err(e) = r
            && !e.is::<Stop>()
        {
            std::panic::resume_unwind(e);
        }
        derefs = std::mem::take(&mut v.deref);
    }
    if log.is_empty() {
        // `--record-order` prints `-` for a call that recorded nothing.
        log.push("-".to_string());
    }
    for d in derefs {
        log.push(if d.is_empty() {
            "-".to_string()
        } else {
            d.join("/")
        });
    }
    for (i, o) in out.iter().enumerate() {
        p[0][4 * i..4 * i + 4].copy_from_slice(&o.to_le_bytes());
    }
    p[1][..4].copy_from_slice(&count.to_le_bytes());
    p[2][..4].copy_from_slice(&rate.to_le_bytes());
}
