//! `Build::start@006273a0` under the emulator against the port (item 3295; tools/sweep_gen.py, obj with windows).

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

/// `Build::start@006273a0`, run by `tools/emu/sweep_build_start.py` under unicorn
/// (tools/emu): 64 random bytes of object, 1 argument(s), `p` windows in order: p0 the place; callee 0x0063e810 (1 args) stubbed; vtable slot +0x2c (0 args) stubbed; vtable slot +0x164 (0 args) stubbed; vtable slot +0x1c (0 args) stubbed; callee 0x0097f770 (1 args) stubbed; the word at `m+0x18` points at 64 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); the word at `m+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p0+0` laid as the image vtable at 0xb42b94 (the stub slots copied there) half the rows; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_start.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `a`: `Build::start`'s argument (the announce flag, also `Wall::start`'s), then the five stubs'
                // answers in the order the line names them: a[1] `Wall::start`, a[2] slot +0x2c (`Build`'s
                // wonder test, on `this`), a[3] slot +0x164 (`update_local_seen`, unread), a[4] the type
                // record's slot +0x1c (its wonder test), a[5] `SoundGlobal::play(0x78)` (unread). `m` the building (the
                // vtable word at +0 either `Build::vftable` or the stubbed table, the type pointer at
                // +0x18), `p0` the type record (vtable word at +0, its index at +4). The interface arm
                // (`iface_current`, a cell the image leaves zero) is not reached.
                //
                // **The port is the crate's own steps**, `crate::city::build_start_steps`, which
                // `Sim::start_building` calls over its `StartSite`; here the view is the emulated windows.
                use crate::city::{BuildStartView, build_start_steps};
                const BUILD_VFTABLE: i32 = 0x00b4_2174;
                const BUILD_TYPE_VFTABLE: i32 = 0x00b4_2b94;
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a [u8],
                    p: &'a [Vec<u8>],
                    log: &'a mut Vec<String>,
                }
                impl BuildStartView for V<'_> {
                    fn wall_start(&mut self) -> bool {
                        self.log.push(format!("s0:m+0x0,{}", self.a[0]));
                        self.a[1] != 0
                    }
                    fn is_wonder(&mut self) -> bool {
                        if rd(self.m, 0) == BUILD_VFTABLE {
                            if rd(&self.p[0], 0) == BUILD_TYPE_VFTABLE {
                                // `BuildData::is_wonder`, inlined: `0x20e..=0x21e`.
                                return (0x20e..=0x21e).contains(&rd(&self.p[0], 4));
                            }
                            self.log.push("s3:p0+0x0".into());
                            self.a[4] != 0
                        } else {
                            self.log.push("s1:m+0x0".into());
                            self.a[2] != 0
                        }
                    }
                    fn update_local_seen(&mut self) {
                        self.log.push("s2:m+0x0".into());
                    }
                    fn announce(&mut self) {
                        self.log.push("s4:m+0x0,120".into());
                    }
                }
                let mut v = V {
                    a,
                    m,
                    p: &*p,
                    log: &mut log,
                };
                i64::from(build_start_steps(&mut v, a[0] != 0))
            }
        };
        assert_eq!(got, want_eax, "start — eax, row `{line}`");
        assert_eq!(log, want_log, "start — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "start — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "start — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "start — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Build::start@006273a0: {n} rows agree");
}
