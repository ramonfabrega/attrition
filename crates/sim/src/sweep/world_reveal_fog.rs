//! `World::reveal_fog@006b3d30` under the emulator against the port (item 3337; tools/sweep_gen.py, obj with windows).

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

fn u16_at(w: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([w[off], w[off + 1]])
}

/// The emulated original's windows as [`crate::rares::RevealFogView`].
///
/// Windows: `m` the world (`+0x0` the cell row length, `+0x18` the tile row
/// length), `c0` the console (`+0x298` its player), `c1` the game (`+0x30`
/// `reveal_map`, `+0x550` the frame), `c3` `div_3_table`, `c5` the goods
/// count, `c6` the message string's base word, `c7` the message window's
/// word; `p22..` the goods table (`p23..27` the goods, `p58..62` their type
/// records), `p28..` the items (`p29..31`), `p33..`/`p45..` each player's units
/// and objects (three windows a player), `p56` the world's cells, `p57` its
/// tiles, `p63` the goods-type name table, `p2..p21` the leaders' slices (five
/// a player) and `p64..p67` their `oil_patches` data. A stub's call is a
/// token in `log`, as the recorder prints it.
struct Fog<'a> {
    a: &'a [i32],
    m: &'a mut [u8],
    c: &'a mut [Vec<u8>],
    p: &'a mut [Vec<u8>],
    log: &'a mut Vec<String>,
    deref: Option<(i32, i32)>,
}

impl Fog<'_> {
    fn cell_at(&self, cx: i32, cy: i32) -> usize {
        ((cy * rd(self.m, 0) + cx) * 0x1c) as usize
    }
    /// The leader's row base, as the recorder names the `ecx` it is handed.
    fn leader(who: i32) -> String {
        format!("p{}+0x0", 2 + 5 * who)
    }
}

impl crate::rares::RevealFogView for Fog<'_> {
    fn tile_marked(&mut self, fx: i32, fy: i32) -> bool {
        let i = ((2 * fy + 1) * rd(self.m, 0x18) + 2 * fx + 1) as usize;
        u16_at(&self.p[57], 2 * i) & 0x200 != 0
    }
    fn find_good_at(&mut self, qx: i32, qy: i32, who: i32) -> i32 {
        self.log.push(format!("s0:{qy},{qx},{qy},{who},1,0"));
        self.a[4]
    }
    fn good_alive(&mut self, gi: i32) -> bool {
        self.p[23 + gi as usize][8] & 1 != 0
    }
    fn new_rare(&mut self, who: i32, gi: i32) {
        self.log.push(format!("s1:{},{gi}", Self::leader(who)));
    }
    fn console_who(&mut self) -> i32 {
        rd(&self.c[0], 0x298)
    }
    fn game_started(&mut self) -> bool {
        rd(&self.c[1], 0x550) != 0
    }
    fn good_ever_seen(&mut self, gi: i32) -> u8 {
        self.p[23 + gi as usize][0x20]
    }
    fn leader_seen_mask(&mut self, who: i32) -> u8 {
        self.p[(4 + 5 * who) as usize][1]
    }
    fn good_type(&mut self, gi: i32) -> i32 {
        rd(&self.p[58 + gi as usize], 4)
    }
    fn reveal_map(&mut self) -> u8 {
        self.c[1][0x30]
    }
    fn leader_flags(&mut self, who: i32) -> u32 {
        rd(&self.p[(2 + 5 * who) as usize], 0) as u32
    }
    fn leader_w59e4(&mut self, who: i32) -> u16 {
        u16_at(&self.p[(3 + 5 * who) as usize], 0)
    }
    fn cell_owner(&mut self, cx: i32, cy: i32) -> i32 {
        i32::from(self.p[56][self.cell_at(cx, cy) + 0xf] as i8)
    }
    fn type_avail(&mut self, who: i32, ty: i32) -> bool {
        self.log.push(format!("s2:{},{ty},1", Self::leader(who)));
        self.a[4 + 2] != 0
    }
    fn found_rare_message(&mut self, who: i32, ty: i32, wx: i32, wy: i32) {
        let msg = 15_558_448; // the `String` global, 0xed6730
        let base = rd(&self.c[6], 0).wrapping_add(0x123e0);
        let name = rd(&self.p[63], 4 * ty as usize).wrapping_add(0x74);
        let colour = i32::from(self.p[(4 + 5 * who) as usize][0]);
        let win = rd(&self.c[7], 0);
        self.log.push(format!("s3:{msg},{base}"));
        self.log.push(format!("s4:{msg},{name}"));
        self.log.push(format!("s5:{msg},39"));
        self.log
            .push(format!("s6:{},sp-0x44", 0xe3_9ce0 + colour * 0x64));
        self.log.push(format!(
            "s7:{win},{msg},{wx},{wy},-9,{},-1,p1+0x0,0,0,p1+0x0",
            self.a[4 + 6]
        ));
    }
    fn mark_good_seen(&mut self, gi: i32, who: i32) {
        self.p[23 + gi as usize][0x20] |= 1 << who;
    }
    fn cell_flags(&mut self, cx: i32, cy: i32) -> u16 {
        u16_at(&self.p[56], self.cell_at(cx, cy))
    }
    fn cell_kind(&mut self, cx: i32, cy: i32) -> i32 {
        i32::from(self.p[56][self.cell_at(cx, cy) + 2] as i8)
    }
    fn goods_len(&mut self) -> i32 {
        rd(&self.c[5], 0)
    }
    fn good_cell(&mut self, i: i32) -> (i32, i32) {
        let g = &self.p[23 + i as usize];
        let axis = |off: usize| {
            let ix = ((rd(g, off) ^ 0x63637) >> 8) as usize;
            rd(&self.c[3], 4 * ix)
        };
        (axis(0x10), axis(0x14))
    }
    fn oil_count(&mut self, who: i32) -> i32 {
        rd(&self.p[(5 + 5 * who) as usize], 0)
    }
    fn oil_at(&mut self, who: i32, k: i32) -> i32 {
        rd(&self.p[64 + who as usize], 4 * k as usize)
    }
    fn oil_add(&mut self, who: i32, gi: i32) {
        self.log.push(format!("s8:{},{gi}", 0xe4_1218 + who * 0x6eec));
    }
    fn find_goody_at(&mut self, cx: i32, cy: i32) -> i32 {
        self.log.push("s9:sp-0x34,sp-0x38".to_string());
        self.deref = Some((cy, cx));
        self.a[4 + 9]
    }
    fn mark_item_seen(&mut self, gi: i32, who: i32) {
        self.p[29 + gi as usize][0x20] |= 1 << who;
    }
    fn unit_is_unit(&mut self, who: i32, unit: i32) -> bool {
        self.log.push(format!("s11:p{}+0x0", 45 + 3 * who + unit));
        self.a[4 + 11] != 0
    }
    fn unit_masks(&mut self, who: i32, unit: i32) -> u32 {
        rd(&self.p[(33 + 3 * who + unit) as usize], 0x68) as u32
    }
    fn get_goody_box(&mut self, who: i32, unit: i32, cx: i32, cy: i32) {
        self.log
            .push(format!("s10:p{}+0x0,{cx},{cy}", 33 + 3 * who + unit));
    }
}

/// `World::reveal_fog@006b3d30`, run by `tools/emu/sweep_world_reveal_fog.py` under unicorn
/// (tools/emu): 320 random bytes of object, 4 argument(s), `p` windows in order: p0..p1 pointed argument(s), p2..p21 sparse table 0 (4 rows × 5 slice(s), row-major), p22..p67 the place(s), in flag order; the global cell 0x00c06210 points at 672 random bytes; the global cell 0x00c061ec points at 1368 random bytes; the global cell 0x00c061ac points at 32 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; the global cell 0x00c0618c points at 256 random bytes; 4 random bytes at 0x00c0acc4 (each word in 0..5); 4 random bytes at 0x00c8cd00; 4 random bytes at 0x00c06208; argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x0065bec0 (5 args) stubbed; callee 0x006d9e70 (1 args) stubbed; callee 0x006e33a0 (2 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; callee 0x00a1d440 (1 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x008c2cc0 (1 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x0042daf0 (1 args) stubbed; callee 0x0065c040 (1 args) stubbed; callee 0x005f7690 (2 args) stubbed; vtable slot +0x18 (0 args) stubbed; the word at `0xc0a0f0` points at 20 random bytes less `0` (placed); the word at `p22+0` points at 48 random bytes less `0` (placed); the word at `p22+4` points at 48 random bytes less `0` (placed); the word at `p22+8` points at 48 random bytes less `0` (placed); the word at `p22+12` points at 48 random bytes less `0` (placed); the word at `p22+16` points at 48 random bytes less `0` (placed); the word at `0xc0a110` points at 12 random bytes less `0` (placed); the word at `p28+0` points at 48 random bytes less `0` (placed); the word at `p28+4` points at 48 random bytes less `0` (placed); the word at `p28+8` points at 48 random bytes less `0` (placed); the word at `0xc0aec0` points at 8 random bytes less `0` (placed); the word at `p32+0` points at 112 random bytes less `0` (placed); the word at `p32+4` points at 112 random bytes less `0` (placed); the word at `0xc0aedc` points at 8 random bytes less `0` (placed); the word at `p35+0` points at 112 random bytes less `0` (placed); the word at `p35+4` points at 112 random bytes less `0` (placed); the word at `0xc0aef8` points at 8 random bytes less `0` (placed); the word at `p38+0` points at 112 random bytes less `0` (placed); the word at `p38+4` points at 112 random bytes less `0` (placed); the word at `0xc0af14` points at 8 random bytes less `0` (placed); the word at `p41+0` points at 112 random bytes less `0` (placed); the word at `p41+4` points at 112 random bytes less `0` (placed); the word at `c4+0x14` points at 8 random bytes less `0` (placed); the word at `p44+0` points at 16 random bytes less `0` (placed); the word at `p44+4` points at 16 random bytes less `0` (placed); the word at `c4+0x30` points at 8 random bytes less `0` (placed); the word at `p47+0` points at 16 random bytes less `0` (placed); the word at `p47+4` points at 16 random bytes less `0` (placed); the word at `c4+0x4c` points at 8 random bytes less `0` (placed); the word at `p50+0` points at 16 random bytes less `0` (placed); the word at `p50+4` points at 16 random bytes less `0` (placed); the word at `c4+0x68` points at 8 random bytes less `0` (placed); the word at `p53+0` points at 16 random bytes less `0` (placed); the word at `p53+4` points at 16 random bytes less `0` (placed); the word at `m+0x134` points at 256 random bytes less `0` (placed); the word at `m+0x138` points at 256 random bytes less `0` (placed); the word at `p23+0x18` points at 8 random bytes less `0` (placed); the word at `p24+0x18` points at 8 random bytes less `0` (placed); the word at `p25+0x18` points at 8 random bytes less `0` (placed); the word at `p26+0x18` points at 8 random bytes less `0` (placed); the word at `p27+0x18` points at 8 random bytes less `0` (placed); the word at `c2+0x10` points at 64 random bytes less `0` (placed); the word at `p6+0` points at 16 random bytes less `0` (placed); the word at `p11+0` points at 16 random bytes less `0` (placed); the word at `p16+0` points at 16 random bytes less `0` (placed); the word at `p21+0` points at 16 random bytes less `0` (placed); stub 2 answered from [0, 1, 1, 1, 1] (90%); stub 11 answered from [0, 1, 1] (90%); the stubbed vtable laid at `m+0x13c`; stub 0 answers -1..4 every row; stub 9 answers -1..2 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 9 prints the 8 bytes its argument 0 points at, hex, after the record (`/` a call); the global cell 0x00c06188 holds `this`; the global cell 0x00c061d0 holds `this`; 4 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x59e4:0x4, +0x6928:0x4, +0x6e8c:0x4, +0x6e98:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p45+0` as well as at `this`; the stubbed vtable laid at `p46+0` as well as at `this`; the stubbed vtable laid at `p48+0` as well as at `this`; the stubbed vtable laid at `p49+0` as well as at `this`; the stubbed vtable laid at `p51+0` as well as at `this`; the stubbed vtable laid at `p52+0` as well as at `this`; the stubbed vtable laid at `p54+0` as well as at `this`; the stubbed vtable laid at `p55+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 6000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_reveal_fog.py") else {
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
            // **The port is the crate's own steps**,
            // `crate::rares::reveal_fog_steps`, which `Sim::reveal_fog` runs over
            // its `SimRevealFog`; here the view is the emulated original's
            // windows (`Fog`). `a`: the two `FCoord` pointers (their windows
            // `p0`, `p1` hold the values), the player, the unit slot, then
            // the twelve stubs' answers in stub order.
            let (fx, fy) = (rd(&p[0], 0), rd(&p[1], 0));
            let mut v = Fog {
                a,
                m: &mut *m,
                c: &mut *c,
                p: &mut *p,
                log: &mut log,
                deref: None,
            };
            crate::rares::reveal_fog_steps(&mut v, fx, fy, a[2], a[3]);
            // Stub 9 prints the 8 bytes its argument points at (the caller's
            // two locals, `cy` then `cx`) after the record, `-` when not called.
            let tail = match v.deref {
                Some((cy, cx)) => cy
                    .to_le_bytes()
                    .iter()
                    .chain(cx.to_le_bytes().iter())
                    .map(|b| format!("{b:02x}"))
                    .collect(),
                None => "-".to_string(),
            };
            if log.is_empty() {
                log.push("-".to_string());
            }
            log.push(tail);
            0
        };
        let _ = (got, want_eax); // void: `eax` is not held
        assert_eq!(log, want_log, "reveal_fog — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "reveal_fog — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "reveal_fog — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[9 + i], "reveal_fog — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 6000, "only {n} rows");
    eprintln!("World::reveal_fog@006b3d30: {n} rows agree");
}
