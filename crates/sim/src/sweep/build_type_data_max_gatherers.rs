//! `BuildTypeData::max_gatherers@0063c430` under the emulator against the port (item 2368; tools/sweep_gen.py, obj with windows).

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

/// `BuildTypeData::max_gatherers@0063c430`, run by `tools/emu/sweep_build_type_data_max_gatherers.py` under unicorn
/// (tools/emu): 720 random bytes of object, 4 argument(s), vtable slot +0x94 (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x3c (0 args) stubbed; callee 0x00639e40 (13 args) stubbed; the word at `0xc0ab84+max(0,a[1])*0x1c` points at 4 random bytes less `a[0]*4` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); slot +0x94 laid as 0x00472b20 or its stub, a flag after the answers; stub 0 answered from [0, 0, 0, 0, 1, 268435456] (90%); stub 1 answered from [0, 1, 1] (90%); stub 2 answered from [0, 1, 1] (90%); stub 3 answers the address of 160 random bytes, a `p` window after the places; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 4 writes a value in -5..300 through its argument 1, printed after the answers (`o` in a place); the stubbed vtable laid at `p1+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_max_gatherers.py") else {
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
                // Slot +0x94 the base `is_flat` (the flag, a[10]) → `build_flags` (+0x2c0) &
                // 0x10000000; else its stub (s0, a[4]). Placed (a0, a1 ≥ 0): objects[a1][a0] (p1)
                // active when its flag bit 0, slot +0x20 (s1) and slot +0x4c (s2) all answer, its
                // record (slot +0x3c, s3 → p2) giving the city (+0x72, a short); the record asked
                // again for the list (+0x98). `calc_gather` (s4) writes the count (`o`, a[9]).
                let flat = if a[10] != 0 {
                    rd(m, 0x2c0) & 0x1000_0000 != 0
                } else {
                    a[4] != 0
                };
                let cell = std::cell::RefCell::new(&mut log);
                let log = &cell;
                let p = &*p;
                let r = i64::from(crate::gather::max_gatherers_steps(
                    flat,
                    a[0] >= 0 && a[1] >= 0,
                    || {
                        p[1][8] & 1 != 0
                            && {
                                log.borrow_mut().push("s1:p1+0x0".into());
                                a[5] != 0
                            }
                            && {
                                log.borrow_mut().push("s2:p1+0x0".into());
                                a[6] != 0
                            }
                    },
                    || {
                        log.borrow_mut().push("s3:p1+0x0".into());
                        i32::from(i16::from_le_bytes([p[2][0x72], p[2][0x73]]))
                    },
                    || {
                        log.borrow_mut().push("s3:p1+0x0".into());
                        "p2+0x98"
                    },
                    |city, list| {
                        log.borrow_mut().push(format!(
                            "s4:m+0x0,sp-0x3c,sp-0x20,sp-0x24,0,{city},{},0,{},{},1,1,0,{}",
                            a[1],
                            a[2],
                            a[3],
                            list.unwrap_or("0")
                        ));
                        a[9]
                    },
                ));
                // No recorded call at all prints `-`.
                if log.borrow().is_empty() {
                    log.borrow_mut().push("-".into());
                }
                r
            }
        };
        assert_eq!(got, want_eax, "max_gatherers — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "max_gatherers — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "max_gatherers — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "max_gatherers — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "max_gatherers — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("BuildTypeData::max_gatherers@0063c430: {n} rows agree");
}
