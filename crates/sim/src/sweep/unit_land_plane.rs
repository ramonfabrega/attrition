//! `Unit::land_plane@005e9950` under the emulator against the port (item 3396; tools/sweep_gen.py, obj with windows).

use sim::land_plane_view::{self, LandPlaneView};

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

/// `Unit::land_plane@005e9950`, run by `tools/emu/sweep_unit_land_plane.py` under unicorn
/// (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p25 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00c06210 points at 672 random bytes; vtable slot +0x30 (0 args) stubbed; vtable slot +0x158 (3 args) stubbed; vtable slot +0x188 (0 args) stubbed; vtable slot +0x84 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0x100 (0 args) stubbed; vtable slot +0xa8 (0 args) stubbed; vtable slot +0xf0 (0 args) stubbed; vtable slot +0x34 (0 args) stubbed; vtable slot +0x48 (2 args) stubbed; callee 0x00647080 (3 args) stubbed; callee 0x005e3860 (0 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x006179d0 (0 args) stubbed; callee 0x005e4160 (4 args) stubbed; callee 0x006624f0 (4 args) stubbed; the word at `m+0x18` points at 704 random bytes less `0` (placed); the word at `m+0xdc` points at 16 random bytes less `0` (placed); the word at `p3+4` points at 16 random bytes less `0` (placed); the word at `p4+8` points at 32 random bytes less `0` (placed); the word at `c0+0x14` points at 12 random bytes less `0` (placed); the word at `c0+0x30` points at 12 random bytes less `0` (placed); the word at `p6+0` points at 32 random bytes less `0` (placed); the word at `p6+4` points at 32 random bytes less `0` (placed); the word at `p6+8` points at 32 random bytes less `0` (placed); the word at `p7+0` points at 32 random bytes less `0` (placed); the word at `p7+4` points at 32 random bytes less `0` (placed); the word at `p7+8` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 12 random bytes less `0` (placed); the word at `p1+0` points at 12 random bytes less `0` (placed); the word at `p14+0` points at 32 random bytes less `0` (placed); the word at `p14+4` points at 32 random bytes less `0` (placed); the word at `p14+8` points at 32 random bytes less `0` (placed); the word at `p15+0` points at 32 random bytes less `0` (placed); the word at `p15+4` points at 32 random bytes less `0` (placed); the word at `p15+8` points at 32 random bytes less `0` (placed); stub 3's answer points at 40 random bytes less `0` (placed); stub 5's answer points at 32 random bytes less `0` (placed); stub 6's answer points at 40 random bytes less `0` (placed); stub 16's answer points at 8 random bytes less `0` (placed); slot +0x158 laid as 0x0060eda0 or its stub, a flag after the answers; stub 0 answered from [0, 0, 0, 1] (90%); stub 4 answered from [16, 16, 16, 16, 0, 1, 5] (90%); stub 7 answered from [0, 1] (90%); stub 9 answered from [0, 1] (90%); stub 8 answers -3..9 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 2 rows 0x1c apart at `@0xc0ab84`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_land_plane.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // hand: port
            // `a` is the stubs' answers in order (0 is_animal, 4 get_type, 7 has_repeat_air, 8 the
            // home's slot 0x34, 9 the home's visibility) and the last, `a[19]`, 1 when the animal's
            // slot 0x158 is `Unit::die`.
            let _ = &c;
            let mut view = Rows {
                a,
                m,
                c,
                p,
                log: &mut log,
            };
            land_plane_view::land_plane_steps(&mut view);
            0
            // end hand
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "land_plane — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "land_plane — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "land_plane — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "land_plane — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Unit::land_plane@005e9950: {n} rows agree");
}

// hand: view
/// The row's windows as the view. `m` is the unit, `c[1]` the console cell, `p5` the front order,
/// `p22` its `AirOrder`, `p23` the strafe record, `p24` the special-anim order the stub `0xa8`
/// answers, `p25` what `update_order` answers, `p2` the type record, `p3`/`p4` the order list's two
/// nodes, `p8..p13` the objects the table names (`who` 0..1, `o` 0..2), `p16..p21` the inline
/// table's objects.
struct Rows<'a> {
    a: &'a [i32],
    m: &'a mut [u8],
    c: &'a mut [Vec<u8>],
    p: &'a mut [Vec<u8>],
    log: &'a mut Vec<String>,
}

type Home = (usize, usize);

impl Rows<'_> {
    fn home_win(h: Home) -> usize {
        8 + 3 * h.0 + h.1
    }
    fn name(h: Home) -> String {
        format!("p{}+0x0", Self::home_win(h))
    }
    /// The cache of the current order the original refreshes before each read of it.
    fn refresh(&mut self) {
        let node2 = rd(&self.p[3], 4);
        wr(self.m, 0xd4, node2);
        let order = rd(&self.p[4], 8);
        wr(self.m, 0xcc, order);
        self.m[0xd0] = self.p[4][0xc];
    }
}

impl LandPlaneView for Rows<'_> {
    type Home = Home;

    fn is_animal(&mut self) -> bool {
        self.log.push("s0:m+0x0".to_string());
        self.a[0] != 0
    }
    fn die_slot_is_unit_die(&mut self) -> bool {
        self.a[19] != 0
    }
    fn object_die(&mut self) {
        self.log.push("s10:m+0x0,0,-1,0".to_string());
    }
    fn call_die_slot(&mut self) {
        self.log.push("s1:m+0x0,0,-1,0".to_string());
    }
    fn air_home(&mut self) -> Option<Home> {
        self.refresh();
        self.log.push("s3:p5+0x0".to_string());
        Some((rd(&self.p[22], 8) as usize, rd(&self.p[22], 4) as usize))
    }
    fn clear_returning(&mut self) {
        wr(&mut self.p[22], 0x18, 0);
    }
    fn is_strafe(&mut self) -> bool {
        self.log.push("s4:p22+0x0".to_string());
        self.a[4] == land_plane_view::TYPE_STRAFE
    }
    fn strafe_has_target(&mut self) -> bool {
        self.log.push("s5:p22+0x0".to_string());
        if rd(&self.p[23], 8) < 0 {
            return false;
        }
        self.log.push("s5:p22+0x0".to_string());
        rd(&self.p[23], 0xc) >= 0
    }
    fn strafe_mandatory(&mut self) -> bool {
        self.log.push("s5:p22+0x0".to_string());
        self.p[23][0x1c] != 0
    }
    fn clear_orders(&mut self) {
        self.log.push("s11:m+0x0".to_string());
    }
    fn kill_current_order(&mut self) {
        self.log.push("s12:m+0x0,0".to_string());
    }
    fn repeat_air(&mut self, h: Home) -> bool {
        self.log.push(format!("s7:{}", Self::name(h)));
        self.a[7] != 0
    }
    fn clear_order_action_bit(&mut self) {
        self.log.push("s16:m+0x0".to_string());
        self.p[25][4] &= 0xfb;
    }
    fn clear_path_state(&mut self) {
        let v = rd(self.m, 0x68) & 0xfbff_ffffu32 as i32;
        wr(self.m, 0x68, v);
        wr(self.m, 0xc0, 0);
    }
    fn close_orders(&mut self) {
        self.log.push("s13:m+0x0,0".to_string());
    }
    fn clear_partial_path(&mut self) {
        self.log.push("s14:m+0x0".to_string());
    }
    fn update_action(&mut self) {
        self.log.push("s15:m+0x0".to_string());
    }
    fn is_helicopter(&mut self) -> bool {
        self.p[2][0x2b4] & 0x20 != 0
    }
    fn home_slot_34(&mut self, h: Home) -> i32 {
        self.log.push(format!("s8:{}", Self::name(h)));
        self.a[8]
    }
    fn add_spec_anim_order(&mut self, h: Home, d1: i32, d2: i32) {
        self.log
            .push(format!("s17:m+0x0,0,{d1},{d2},{}", Self::name(h)));
    }
    fn set_special_home(&mut self, h: Home) {
        self.refresh();
        self.log.push("s6:p5+0x0".to_string());
        wr(&mut self.p[24], 0x1c, h.1 as i32);
        self.refresh();
        self.log.push("s6:p5+0x0".to_string());
        wr(&mut self.p[24], 0x20, h.0 as i32);
    }
    fn home_visible(&mut self, h: Home) -> bool {
        let who = rd(&self.c[1], 0x298);
        self.log.push(format!("s9:{},{who},0", Self::name(h)));
        self.a[9] != 0
    }
    fn play_landing_sound(&mut self, h: Home) {
        let o = &self.p[Self::home_win(h)];
        let who2 = usize::from(o[9]);
        let o2 = i32::from(i16::from_le_bytes([o[0xa], o[0xb]]));
        let t = &self.p[16 + 3 * who2 + o2 as usize];
        let (x, y) = (rd(t, 0x10), rd(t, 0x14));
        self.log.push(format!(
            "s18:{},23,{},{},{o2}",
            Self::name(h),
            x ^ 0x63637,
            y ^ 0x63637
        ));
    }
    fn go_inside(&mut self, _h: Home) {
        self.log.push("s2:m+0x0".to_string());
    }
}
// end hand
