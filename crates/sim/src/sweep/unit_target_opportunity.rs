//! `Unit::target_opportunity@005fffc0` under the emulator against the port (item 3348; tools/sweep_gen.py, obj with windows).

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

/// `Unit::target_opportunity@005fffc0`, run by `tools/emu/sweep_unit_target_opportunity.py` under unicorn
/// (tools/emu): 256 random bytes of object, 3 argument(s), `p` windows in order: p0..p15 sparse table 0 (8 rows × 2 slice(s), row-major), p16..p23 sparse table 1 (8 rows × 1 slice(s), row-major), p24..p64 the place(s), in flag order, p65..p67 the stubs' answer window(s); the global cell 0x00e85f20 points at 16 random bytes; the global cell 0x00c0618c points at 256 random bytes; vtable slot +0xe8 (0 args) stubbed; callee 0x0070f8f0 (3 args) stubbed; callee 0x007107d0 (4 args) stubbed; callee 0x00648ba0 (3 args) stubbed; vtable slot +0xf4 (0 args) stubbed; vtable slot +0xa8 (0 args) stubbed; the word at `c1+20` points at 16 random bytes less `0` (placed); the word at `p24+0` points at 64 random bytes less `0` (placed); the word at `p24+4` points at 64 random bytes less `0` (placed); the word at `p24+8` points at 64 random bytes less `0` (placed); the word at `p24+12` points at 64 random bytes less `0` (placed); the word at `c1+48` points at 16 random bytes less `0` (placed); the word at `p29+0` points at 64 random bytes less `0` (placed); the word at `p29+4` points at 64 random bytes less `0` (placed); the word at `p29+8` points at 64 random bytes less `0` (placed); the word at `p29+12` points at 64 random bytes less `0` (placed); the word at `c1+76` points at 16 random bytes less `0` (placed); the word at `p34+0` points at 64 random bytes less `0` (placed); the word at `p34+4` points at 64 random bytes less `0` (placed); the word at `p34+8` points at 64 random bytes less `0` (placed); the word at `p34+12` points at 64 random bytes less `0` (placed); the word at `c1+104` points at 16 random bytes less `0` (placed); the word at `p39+0` points at 64 random bytes less `0` (placed); the word at `p39+4` points at 64 random bytes less `0` (placed); the word at `p39+8` points at 64 random bytes less `0` (placed); the word at `p39+12` points at 64 random bytes less `0` (placed); the word at `c1+132` points at 16 random bytes less `0` (placed); the word at `p44+0` points at 64 random bytes less `0` (placed); the word at `p44+4` points at 64 random bytes less `0` (placed); the word at `p44+8` points at 64 random bytes less `0` (placed); the word at `p44+12` points at 64 random bytes less `0` (placed); the word at `c1+160` points at 16 random bytes less `0` (placed); the word at `p49+0` points at 64 random bytes less `0` (placed); the word at `p49+4` points at 64 random bytes less `0` (placed); the word at `p49+8` points at 64 random bytes less `0` (placed); the word at `p49+12` points at 64 random bytes less `0` (placed); the word at `c1+188` points at 16 random bytes less `0` (placed); the word at `p54+0` points at 64 random bytes less `0` (placed); the word at `p54+4` points at 64 random bytes less `0` (placed); the word at `p54+8` points at 64 random bytes less `0` (placed); the word at `p54+12` points at 64 random bytes less `0` (placed); the word at `c1+216` points at 16 random bytes less `0` (placed); the word at `p59+0` points at 64 random bytes less `0` (placed); the word at `p59+4` points at 64 random bytes less `0` (placed); the word at `p59+8` points at 64 random bytes less `0` (placed); the word at `p59+12` points at 64 random bytes less `0` (placed); the word at `m+0x18` points at 768 random bytes less `0` (placed); stub 5 answers a window of its own to each call, in call order (256 bytes, ...); the word at `p65+0x18` laid as the address of `p64` every row; the word at `p66+0x18` laid as the address of `p64` every row; the word at `p67+0x18` laid as the address of `p64` every row; stub 0 answers 3 values in call order, the last from [1], printed after the answers; stub 1 answers 3 values in call order, printed after the answers; stub 1 answers 0..1 every row; stub 3 answers 0..1 every row; stub 4 answers 5..5 every row; stub 0 answers 0..1 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x8:0x4, +0x74:0x20 (`p` windows after the pointed buffers, before the places); 8 rows 0x6eec apart at `0xc061e0`, slices +0x74:0x20 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p65+0` as well as at `this`; the stubbed vtable laid at `p66+0` as well as at `this`; the stubbed vtable laid at `p67+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p40+0` as well as at `this`; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p43+0` as well as at `this`; the stubbed vtable laid at `p45+0` as well as at `this`; the stubbed vtable laid at `p46+0` as well as at `this`; the stubbed vtable laid at `p47+0` as well as at `this`; the stubbed vtable laid at `p48+0` as well as at `this`; the stubbed vtable laid at `p50+0` as well as at `this`; the stubbed vtable laid at `p51+0` as well as at `this`; the stubbed vtable laid at `p52+0` as well as at `this`; the stubbed vtable laid at `p53+0` as well as at `this`; the stubbed vtable laid at `p55+0` as well as at `this`; the stubbed vtable laid at `p56+0` as well as at `this`; the stubbed vtable laid at `p57+0` as well as at `this`; the stubbed vtable laid at `p58+0` as well as at `this`; the stubbed vtable laid at `p60+0` as well as at `this`; the stubbed vtable laid at `p61+0` as well as at `this`; the stubbed vtable laid at `p62+0` as well as at `this`; the stubbed vtable laid at `p63+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_target_opportunity.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().filter(|t| *t != "-").collect();
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
            let _ = &c;
            use crate::topp::{Head, TargetOppView, target_opportunity_head};
            // `p[0..24]`: p[2r] the team slice of `leaders[r]` (+8), p[2r+1] its treaty row (+0x74);
            // p[16+r] the treaty row of `leadersc[r]`. p[24..64] the objects tables (not read here:
            // the walk's `get_captain` is stub 5, answering the next of p[65..], in call order);
            // p[64] the type every unit names; p[65..68] the captains the walk reaches.
            // `a`: the arguments (`param_1..3`), stubs 0..=5's answers (5: the window address), then
            // stub 0's and stub 1's three answers in call order (the last repeated).
            struct W<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                s0: usize,
                s1: usize,
            }
            // The harness's address of the cell at 0xe85f20's target (`GroupData::member`'s
            // `ecx` is that plus the group times 0x9d4); it is the same on every row.
            const GROUP_CELL: i64 = 536_871_712;
            impl W<'_> {
                fn bytes(&self, u: usize) -> &[u8] {
                    if u == 0 { self.m } else { &self.p[64 + u] }
                }
                fn name(u: usize) -> String {
                    if u == 0 {
                        "m+0x0".to_string()
                    } else {
                        format!("p{}+0x0", 64 + u)
                    }
                }
                fn group(&self, u: usize) -> i64 {
                    let b = self.bytes(u);
                    i64::from(i16::from_le_bytes([b[0x80], b[0x81]]))
                }
            }
            impl TargetOppView for W<'_> {
                type U = usize;
                type T = ();
                fn owner(&self, u: usize) -> i32 {
                    i32::from(self.bytes(u)[9])
                }
                fn team(&self, owner: i32) -> i32 {
                    rd(&self.p[2 * owner as usize], 0)
                }
                fn target_owner(&self, _: ()) -> i32 {
                    self.a[1]
                }
                fn treaty(&self, a: i32, b: i32) -> bool {
                    rd(&self.p[2 * a as usize + 1], 4 * b as usize) != 0
                }
                fn treaty_c(&self, p2: i32, team: i32) -> bool {
                    rd(&self.p[16 + p2 as usize], 4 * team as usize) != 0
                }
                fn group_noncombat(&mut self, u: usize) -> bool {
                    self.group(u) >= 0 && rd(&self.p[64], 0x2c8) & 0x10000 == 0
                }
                fn group_member(&mut self, u: usize) -> bool {
                    let b = self.bytes(u);
                    let short = i16::from_le_bytes([b[0xa], b[0xb]]);
                    let line = format!(
                        "s1:{},{},{},1",
                        GROUP_CELL + self.group(u) * 0x9d4,
                        short,
                        b[9]
                    );
                    self.log.push(line);
                    let k = self.s1.min(2);
                    self.s1 += 1;
                    self.a[12 + k] != 0
                }
                fn group_handoff(&mut self, u: usize, _: ()) {
                    let b = self.bytes(u);
                    let short = i16::from_le_bytes([b[0xa], b[0xb]]);
                    let line = format!(
                        "s2:{},{},{},{},0",
                        GROUP_CELL + self.group(u) * 0x9d4,
                        self.a[0],
                        self.a[1],
                        short
                    );
                    self.log.push(line);
                }
                fn is_captain(&mut self, _: usize) -> bool {
                    let k = self.s0.min(2);
                    self.s0 += 1;
                    self.a[9 + k] != 0
                }
                fn up(&mut self, u: usize) -> Option<usize> {
                    let up = i16::from_le_bytes([self.bytes(u)[0x8e], self.bytes(u)[0x8f]]);
                    assert!((0..4).contains(&up), "the sweep keeps o_up in the table");
                    Some(u + 1)
                }
                fn valid_target(&mut self, u: usize, _: ()) -> bool {
                    self.log.push(format!(
                        "s3:{},{},{},0",
                        Self::name(u),
                        self.a[0],
                        self.a[1]
                    ));
                    self.a[6] != 0
                }
                fn stance(&mut self, u: usize) -> i32 {
                    self.log.push(format!("s4:{}", Self::name(u)));
                    self.a[7]
                }
            }
            let mut w = W {
                a,
                m,
                p,
                log: &mut log,
                s0: 0,
                s1: 0,
            };
            // The head's end is what the stubs cannot see past: `Go` is the point the original
            // enters `order_type` (`600149`), which the sweep's slice stops before.
            let head = target_opportunity_head(&mut w, 0, (), a[2] != 0);
            assert!(
                !matches!(head, Head::Go(_)),
                "stub 4 answers 5: the slice ends at the hold-fire return"
            );
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "target_opportunity — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "target_opportunity — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "target_opportunity — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "target_opportunity — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Unit::target_opportunity@005fffc0: {n} rows agree");
}
