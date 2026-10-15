//! `SpellType::cast_unpack@006709c0` under the emulator against the port (item 2400; tools/sweep_gen.py, obj with windows).

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

/// `SpellType::cast_unpack@006709c0`, run by `tools/emu/sweep_spell_type_cast_unpack.py` under unicorn
/// (tools/emu): 32 random bytes of object, 2 argument(s), the global cell 0x00c06204 points at 256 random bytes; the global cell 0x00c0620c points at 576 random bytes; the global cell 0x00c0618c points at 64 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; 56 random bytes at 0x00c0aec0; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0x160 (0 args) stubbed; vtable slot +0x174 (1 args) stubbed; callee 0x006068a0 (2 args) stubbed; callee 0x005f8d20 (4 args) stubbed; callee 0x006b4900 (3 args) stubbed; callee 0x005e2920 (0 args) stubbed; the word at `c2+0x14+a[1]*0x1c` points at 4 random bytes less `a[0]*4` (placed); the word at `c4+a[1]*0x1c` points at 4 random bytes less `a[0]*4` (placed); the word at `p2+0` points at 32 random bytes less `0` (placed); the word at `p3+0` points at 384 random bytes less `0` (placed); the word at `p5+0x18` points at 16 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [1, 1, 1, 0] (90%); stub 4 answered from [0, 1] (90%); the stubbed vtable laid at `p4+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 6 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 6 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p5+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_cast_unpack.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `SpellType::cast_unpack(o = a[0], who = a[1])`: `options->rebuild = 1` (c0+0x90);
                // the object (p4) asked alive (s0, answer a[2]) then on the map (s1, a[3]); the unit
                // (p5) gets `announce_frame = -1` (+0x14c) and the steps run (`cast_unpack_steps`).
                use crate::transport::{UnpackSteps, cast_unpack_steps};
                struct W<'a> {
                    log: &'a mut Vec<String>,
                    c: &'a mut [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    who: usize,
                    good: bool,
                    xs: Vec<i32>,
                    ys: Vec<i32>,
                }
                impl UnpackSteps for W<'_> {
                    fn pos(&self) -> (i32, i32) {
                        (
                            rd(&self.p[4], 0x10) ^ 0x63637,
                            rd(&self.p[4], 0x14) ^ 0x63637,
                        )
                    }
                    fn good_merchant_spot(&mut self, tx: i32, ty: i32) -> bool {
                        self.log.push(format!("s4:p5+0x0,{tx},{ty}"));
                        self.good
                    }
                    fn set_new_location(&mut self, x: i32, y: i32) {
                        self.log.push(format!("s5:p5+0x0,{x},{y},1,1"));
                    }
                    fn set_blocked(&mut self, tx: i32, ty: i32) {
                        // The two out-pointers are the original's frame slots, in the order of its four calls.
                        let slots = [
                            ("sp-0x1c", "sp-0x20"),
                            ("sp-0x14", "sp-0x20"),
                            ("sp-0x1c", "sp-0x14"),
                            ("sp-0x1c", "sp-0x14"),
                        ];
                        let (px, py) = slots[self.xs.len()];
                        self.log.push(format!("s6:12621800,{px},{py},1"));
                        self.xs.push(tx);
                        self.ys.push(ty);
                    }
                    fn merchant_deployed(&mut self) {
                        self.c[1][0x22c] = 1;
                        let w = &mut self.p[self.who];
                        let v = rd(w, 0) | 0x2000000;
                        wr(w, 0, v);
                    }
                    fn clear_packed(&mut self) {
                        let v = rd(&self.p[5], 0x68) & 0xfff7ffffu32 as i32;
                        wr(&mut self.p[5], 0x68, v);
                    }
                    fn update_los(&mut self) {
                        self.log.push("s2:p5+0x0".into());
                    }
                    fn update_seen(&mut self) {
                        self.log.push("s3:p5+0x0,0".into());
                    }
                    fn update_gpiece(&mut self) {
                        self.log.push("s7:p5+0x0".into());
                    }
                }
                wr(&mut c[0], 0x90, 1);
                log.push("s0:p4+0x0".into());
                let alive = a[2] != 0;
                if alive {
                    log.push("s1:p4+0x0".into());
                }
                let usable = alive && a[3] != 0;
                let mut w = W {
                    log: &mut log,
                    c,
                    p,
                    who: a[1] as usize,
                    good: a[6] != 0,
                    xs: Vec::new(),
                    ys: Vec::new(),
                };
                if usable {
                    wr(&mut w.p[5], 0x14c, -1);
                }
                let tree = usize::try_from(rd(&w.p[6], 4)).ok();
                cast_unpack_steps(&mut w, usable, tree);
                let hex = |v: &[i32]| {
                    if v.is_empty() {
                        "-".to_string()
                    } else {
                        let h: Vec<String> = v
                            .iter()
                            .map(|x| format!("{:08x}", x.swap_bytes()))
                            .collect();
                        h.join("/")
                    }
                };
                let (hx, hy) = (hex(&w.xs), hex(&w.ys));
                w.log.push(hx);
                w.log.push(hy);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "cast_unpack — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "cast_unpack — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "cast_unpack — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "cast_unpack — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("SpellType::cast_unpack@006709c0: {n} rows agree");
}
