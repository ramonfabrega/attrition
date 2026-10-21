//! `HeroData::get_radius@00739e50` under the emulator against the port (item 3396; tools/sweep_gen.py, obj with windows).

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

/// `HeroData::get_radius@00739e50`, run by `tools/emu/sweep_hero_data_get_radius.py` under unicorn
/// (tools/emu): 64 random bytes of object, 0 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p4 the place(s), in flag order; the global cell 0x00c061e4 points at 3328 random bytes; callee 0x006e0830 (0 args) stubbed; callee 0x006ebc10 (1 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `m+0x30` points at 64 random bytes less `0` (placed); the word at `p2+0x18` points at 64 random bytes less `0` (placed); the word at `0xc0aec0+u8(m,0x29)*0x1c` points at 4 random bytes less `i16(m,0x26)*4` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); the word at `p4+0` laid as the address of `p2` every row; stub 2 answers 12 values in call order, printed after the answers; stub 3 answers 12 values in call order, printed after the answers; stub 0 answers -6..12 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 2 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_hero_data_get_radius.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Windows: `m` the `HeroData` (`who` the signed byte `+0x29` -> row, `o` the short `+0x26`);
                // `c[0]` `constantsc` (the seven constants); `p[0..2]` the leaders' rows (the ecx of the
                // two leader callees). `a`: the four stubs' single answers (a[0] `get_general_upgrade`,
                // a[1] `has_wonder`; a[2], a[3] unused by the seqs), then `is` answers by slot 0xb8
                // (a[4..16]) and by the type's 0x60 (a[16..28]) in call order, then the slot flag
                // (a[28]: nonzero = slot 0xb8 is `ObjectData::is`, so the type's 0x60 answers).
                use sim::hero_radius_view::{HeroConstants, HeroRadiusView, hero_radius_steps};
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a [u8],
                    c: &'a [Vec<u8>],
                    log: &'a mut Vec<String>,
                    turn: usize,
                }
                impl HeroRadiusView for V<'_> {
                    fn general_upgrade(&mut self) -> i32 {
                        self.log.push(format!("s0:r{}+0x0", self.m[0x29]));
                        self.a[0]
                    }
                    fn is(&mut self, ty: i32, mode: i32) -> bool {
                        let base = self.a[28] != 0;
                        let n = self.turn.min(11);
                        self.turn += 1;
                        if base {
                            self.log.push(format!("s3:p3+0x0,{ty},{mode}"));
                            self.a[16 + n] != 0
                        } else {
                            self.log.push(format!("s2:p2+0x0,{ty},{mode}"));
                            self.a[4 + n] != 0
                        }
                    }
                    fn has_wonder(&mut self, ty: i32) -> bool {
                        self.log.push(format!("s1:r{}+0x0,{ty}", self.m[0x29]));
                        self.a[1] != 0
                    }
                    fn constants(&mut self) -> HeroConstants {
                        let k = |o: usize| rd(&self.c[0], o);
                        HeroConstants {
                            general_radius: k(0xc58),
                            parmenio_radius_adjust: k(0xb54),
                            wellington_radius: k(0xbb4),
                            kutosov_radius: k(0xbdc),
                            terra_cotta_range: k(0x45c),
                            mil_patriot_radius_bonus: k(0xc68),
                            econ_patriot_radius_bonus: k(0xc6c),
                        }
                    }
                }
                let mut v = V {
                    a,
                    m,
                    c,
                    log: &mut log,
                    turn: 0,
                };
                i64::from(hero_radius_steps(&mut v))
            }
        };
        assert_eq!(got, want_eax, "get_radius — eax, row `{line}`");
        assert_eq!(log, want_log, "get_radius — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_radius — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_radius — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "get_radius — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("HeroData::get_radius@00739e50: {n} rows agree");
}
