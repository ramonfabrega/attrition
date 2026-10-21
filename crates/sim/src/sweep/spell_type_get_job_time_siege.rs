//! `SpellTypeData::get_job_time@00675800` under the emulator against the port (item 3395; tools/sweep_gen.py, obj with windows).

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

/// `SpellTypeData::get_job_time@00675800`, run by `tools/emu/sweep_spell_type_get_job_time_siege.py` under unicorn
/// (tools/emu): 16 random bytes of object, 2 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p5 sparse table 1 (1 rows × 4 slice(s), row-major), p6..p11 the place(s), in flag order; 56 random bytes at 0x00c0ab84; 56 random bytes at 0x00c0aec0; callee 0x006e1370 (1 args) stubbed; callee 0x006db810 (1 args) stubbed; callee 0x00646b00 (2 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x10c (0 args) stubbed; the word at `c0+a[1]*0x1c` points at 8 random bytes less `a[0]*4` (placed); the word at `p6+0` points at 32 random bytes less `0` (placed); the word at `p7+0x18` points at 16 random bytes less `0` (placed); the word at `c1+a[1]*0x1c` points at 8 random bytes less `a[0]*4` (placed); the word at `p9+0` points at 32 random bytes less `0` (placed); the word at `p10+0x18` points at 16 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 5, 5, -1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [1, 1, 0] (90%); stub 5 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 7 answered from [1, 1, 0] (90%); the stubbed vtable laid at `p7+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 2 rows 0x6eec apart at `0xc061e0`, slices +0x59cc:0x4 (`p` windows after the pointed buffers, before the places); 1 rows 0xbc0 apart at `0xc061e4`, slices +0x670:0x4, +0x6b8:0x4, +0xb9c:0x4, +0xbac:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_get_job_time_siege.py") else {
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
            // `a`: unit, who | the stubs' answers `a[2..10]` (0 `has_tribe_bonus`, 1 `has_preq`, 2 `has_general`,
            // 3/4 the objects row's slots `+0x8`/`+0x18`, 5 the slot `+0xb8` stubbed, 6 the type's `+0x60` under
            // the real `is`, 7 the type's `+0x10c`) | the slotbase flag `a[10]` (1: slot `+0xb8` is
            // `ObjectData::is`). `p0`/`p1` the leaders' shorts at `+0x59cc`, `p2..p5` the constants (`turk_pack`,
            // `french_entrench`, `antipater_entrench_rate`, `napoleon_pack`), `p7`/`p8` the objects row and its
            // type, `p10`/`p11` the units row and its type; `m` the craft (`+4` its index, `+8` `JOB_TIME`).
            //
            // **The port is the crate's own steps**, `crate::spell_type_get_job_time_view::get_job_time`, over a
            // view of the emulated original's windows; the crate has no whole `get_job_time` to call (that
            // module's head).
            use crate::spell_type_get_job_time_view::{JobTimeView, get_job_time};
            struct V<'a> {
                a: &'a [i32],
                p: &'a [Vec<u8>],
                log: &'a mut Vec<String>,
            }
            impl V<'_> {
                fn ans(&self, k: usize) -> i32 {
                    self.a[2 + k]
                }
                fn leader(who: i32) -> String {
                    format!("r{who}+0x0")
                }
                // `ObjectData::is`: the real thunk (`a[10]`) calls the type's slot `+0x60`, else the slot
                // itself is called on the object.
                fn is(&mut self, obj: usize, ty: usize, t: i32, strict: i32) -> bool {
                    let (k, w) = if self.a[10] != 0 { (6, ty) } else { (5, obj) };
                    self.log.push(format!("s{k}:p{w}+0x0,{t},{strict}"));
                    self.ans(k) != 0
                }
                fn word(&self, w: usize) -> i32 {
                    i32::from_le_bytes(self.p[w][0..4].try_into().expect("four bytes"))
                }
            }
            impl JobTimeView for V<'_> {
                fn has_tribe_bonus(&mut self, who: i32, k: i32) -> bool {
                    self.log.push(format!("s0:{},{k}", Self::leader(who)));
                    self.ans(0) != 0
                }
                fn has_preq(&mut self, who: i32, t: i32) -> bool {
                    self.log.push(format!("s1:{},{t}", Self::leader(who)));
                    self.ans(1) != 0
                }
                fn french_entrench(&mut self) -> i32 {
                    self.word(3)
                }
                fn antipater_entrench_rate(&mut self) -> i32 {
                    self.word(4)
                }
                fn turk_pack(&mut self) -> i32 {
                    self.word(2)
                }
                fn napoleon_pack(&mut self) -> i32 {
                    self.word(5)
                }
                fn leader_short_59cc(&mut self, who: i32) -> i16 {
                    let w = &self.p[usize::try_from(who).expect("a player")];
                    i16::from_le_bytes([w[0], w[1]])
                }
                fn objects_slot_08(&mut self, _who: i32, _unit: i32) -> bool {
                    self.log.push("s3:p7+0x0".to_string());
                    self.ans(3) != 0
                }
                fn objects_slot_18(&mut self, _who: i32, _unit: i32) -> bool {
                    self.log.push("s4:p7+0x0".to_string());
                    self.ans(4) != 0
                }
                fn objects_is(&mut self, _who: i32, _unit: i32, ty: i32, strict: i32) -> bool {
                    self.is(7, 8, ty, strict)
                }
                fn units_is(&mut self, _who: i32, _unit: i32, ty: i32, strict: i32) -> bool {
                    self.is(10, 11, ty, strict)
                }
                fn units_has_general(&mut self, _who: i32, _unit: i32) -> i32 {
                    self.log.push("s2:p10+0x0,0,359".to_string());
                    self.ans(2)
                }
                fn units_type_gate(&mut self, _who: i32, _unit: i32) -> bool {
                    self.log.push("s7:p11+0x0".to_string());
                    self.ans(7) != 0
                }
            }
            let mut v = V {
                a,
                p,
                log: &mut log,
            };
            let r = i64::from(get_job_time(&mut v, rd(m, 4), rd(m, 8), a[0], a[1]));
            if log.is_empty() {
                // The recorder prints `-` for a row that made no recorded call.
                log.push("-".to_string());
            }
            r
        };
        assert_eq!(got, want_eax, "get_job_time — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "get_job_time — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_job_time — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_job_time — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "get_job_time — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("SpellTypeData::get_job_time@00675800: {n} rows agree");
}
