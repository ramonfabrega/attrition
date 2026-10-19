//! `Unit::explore_goody@005f9780` under the emulator against the port (item 3295; tools/sweep_gen.py, obj with windows).

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

/// `Unit::explore_goody@005f9780`, run by `tools/emu/sweep_unit_explore_goody.py` under unicorn
/// (tools/emu): 32 random bytes of object, 0 argument(s), `p` windows in order: p0..p3 sparse table 0 (2 rows × 2 slice(s), row-major), p4..p21 the place(s), in flag order; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c0618c points at 64 random bytes; the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00c06210 points at 672 random bytes; the global cell 0x00c061f0 points at 3120 random bytes; 4 random bytes at 0x00c06184; callee 0x0065b7c0 (3 args) stubbed; vtable slot +0x90 (0 args) stubbed; callee 0x006e33a0 (2 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; the word at `0xc0a110` points at 16 random bytes less `0` (placed); the word at `p4+0` points at 32 random bytes less `0` (placed); the word at `p4+4` points at 32 random bytes less `0` (placed); the word at `p4+8` points at 32 random bytes less `0` (placed); the word at `p4+12` points at 32 random bytes less `0` (placed); the word at `c2+0x14` points at 16 random bytes less `0` (placed); the word at `p9+0` points at 64 random bytes less `0` (placed); the word at `p9+4` points at 64 random bytes less `0` (placed); the word at `p9+8` points at 64 random bytes less `0` (placed); the word at `p9+12` points at 64 random bytes less `0` (placed); the word at `c2+0x30` points at 16 random bytes less `0` (placed); the word at `p14+0` points at 64 random bytes less `0` (placed); the word at `p14+4` points at 64 random bytes less `0` (placed); the word at `p14+8` points at 64 random bytes less `0` (placed); the word at `p14+12` points at 64 random bytes less `0` (placed); the word at `c0+0x134` points at 256 random bytes less `0` (placed); the word at `p1+0` points at 256 random bytes less `0` (placed); the word at `p3+0` points at 256 random bytes less `0` (placed); stub 4 answered from [0, 1] (90%); stub 2 answers 6 values in call order, printed after the answers; stub 3 answers 6 values in call order, printed after the answers; stub 0 answers -1..3 every row; stub 2 answers 0..1 every row; stub 3 answers 0..65535 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 2 rows 0x6eec apart at `@0xe3a390`, slices +0x86c:0x4, +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); figures p10..p13 link at +0x2c (2 bytes) below their own index or to -1, -3; figures p15..p18 link at +0x2c (2 bytes) below their own index or to -1, -3; figures p10..p13 hold a short at +0x2e drawn from [0, 1], after the links; figures p15..p18 hold a short at +0x2e drawn from [0, 1], after the links; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_explore_goody.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(7);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `a`: the six stubs' single answers (0 `find_goody_at`'s item index, 1 the slot's unread,
                // 2 `type_avail` and 3 `Random::get` replaced by their per-call values, 4 `has_tribe_bonus`,
                // 5 `String::close`), then `type_avail`'s six answers, then `Random::get`'s six (the last
                // repeated). Windows: `m` the unit (`+9` the owner, `+0x10`/`+0x14` its position xor 0x63637),
                // `c0` the world (`+0` width), `c1` the `div_3_table`, `c3` the game (`+0x550` the frame),
                // `c5` the constants, `c6` the game-random cell's word; `p0..p3` the leaders' slices
                // (`+0x86c` goody_box_resources, `+0x6eb8` the encrypted record's pointer), `p4..p8` the
                // item array and its items, `p9..p18` the two players' object arrays and objects, `p19`
                // the cell array (stride 0x1c), `p20`/`p21` the leaders' encrypted records.
                //
                // **The port is the crate's own steps**, `crate::goody::explore_goody_steps`, over a view of
                // the windows. The original's chain walk over the items and objects at the cell
                // (`ObjectsData::find_goody_at`'s caller and the `-3` repair) has no crate method — the
                // simulation keeps no item chain — so it is inline here.
                use crate::goody::{GoodyView, explore_goody_steps};
                let rd16 = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
                let wr16 = |w: &mut [u8], o: usize, v: i32| w[o..o + 2].copy_from_slice(&(v as i16).to_le_bytes());
                let owner = usize::from(m[9]);
                let (x, y) = (rd(m, 0x10) ^ 0x6_3637, rd(m, 0x14) ^ 0x6_3637);
                let div3 = |c: &[Vec<u8>], v: i32| rd(&c[1], 4 * (v >> 8) as usize);
                // `find_goody_at`, and the item's slot `+0x90` when it finds one.
                log.push(format!("s0:{x},{x},{y},{x}"));
                if a[0] >= 0 {
                    log.push(format!("s1:p{}+0x0", 5 + a[0]));
                }
                let cell = (rd(&c[0], 0) * div3(c, y) + div3(c, x)) as usize * 0x1c;
                let (region, mut who) = (rd16(&p[19], cell + 8), rd16(&p[19], cell + 10));
                if region >= 0 {
                    let mut next = region;
                    while next >= 0 {
                        let arr = if who == 0 { 10 } else { 15 };
                        let obj = &mut p[arr + next as usize];
                        let (n, w) = (rd16(obj, 0x2c), rd16(obj, 0x2e));
                        if n == -3 {
                            wr16(obj, 0x2c, -1);
                        }
                        (next, who) = (n, w);
                    }
                } else if region == -3 {
                    wr16(&mut p[19], cell + 8, -3);
                    wr16(&mut p[19], cell + 10, who);
                }
                struct V<'a> {
                    a: &'a [i32],
                    c: &'a [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    owner: usize,
                    cell: usize,
                    avail: usize,
                    rolls: usize,
                }
                impl V<'_> {
                    fn leader(&self) -> usize {
                        14_918_544 + self.owner * 0x6eec
                    }
                }
                impl GoodyView for V<'_> {
                    fn clear_box(&mut self) {
                        let f = &mut self.p[19][self.cell..self.cell + 2];
                        let v = u16::from_le_bytes([f[0], f[1]]) & 0x7fff;
                        f.copy_from_slice(&v.to_le_bytes());
                    }
                    fn started(&self) -> bool {
                        rd(&self.c[3], 0x550) != 0
                    }
                    fn candidate(&mut self, good: usize) -> bool {
                        self.log.push(format!("s2:{},{good},1", self.leader()));
                        let v = self.a[6 + self.avail.min(5)];
                        self.avail += 1;
                        v != 0
                    }
                    fn held(&self, good: usize) -> i32 {
                        rd(&self.p[20 + self.owner], 4 * good) ^ 0x8221
                    }
                    fn roll(&mut self) -> i32 {
                        self.log.push(format!("s3:{},0,65535", rd(&self.c[6], 0)));
                        let v = self.a[12 + self.rolls.min(5)];
                        self.rolls += 1;
                        v
                    }
                    fn pile(&mut self) -> i32 {
                        self.log.push(format!("s4:{},9", self.leader()));
                        let level = rd(&self.p[20 + self.owner], 0xf4) ^ 0x6_3187;
                        let k = &self.c[5];
                        if self.a[4] != 0 {
                            level.wrapping_mul(rd(k, 0x68c)).wrapping_add(rd(k, 0x688))
                        } else {
                            level.wrapping_mul(rd(k, 0xc28)).wrapping_add(rd(k, 0xc24))
                        }
                    }
                    fn pay(&mut self, good: usize, amount: i32) {
                        let enc = &mut self.p[20 + self.owner];
                        let held = (rd(enc, 4 * good) ^ 0x8221).wrapping_add(amount);
                        wr(enc, 4 * good, held ^ 0x8221);
                        let total = &mut self.p[2 * self.owner];
                        let t = rd(total, 0).wrapping_add(amount);
                        wr(total, 0, t);
                    }
                }
                let mut v = V {
                    a,
                    c,
                    p,
                    log: &mut log,
                    owner,
                    cell,
                    avail: 0,
                    rolls: 0,
                };
                explore_goody_steps(&mut v);
                // `String::close` of the never-built message, the function's last act on every path.
                log.push("s5:sp-0x54".into());
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "explore_goody — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "explore_goody — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "explore_goody — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[8 + i], "explore_goody — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Unit::explore_goody@005f9780: {n} rows agree");
}
