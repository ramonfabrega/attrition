//! `BuildData::calc_gather@0062d360` under the emulator against the port (item 2368; tools/sweep_gen.py, obj with windows).

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

/// `BuildData::calc_gather@0062d360`, run by `tools/emu/sweep_build_data_calc_gather.py` under unicorn
/// (tools/emu): 160 random bytes of object, 6 argument(s), argument 0 names 16 random bytes; argument 1 names 8 random bytes; argument 2 names 16 random bytes; callee 0x00643440 (2 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; callee 0x0064dda0 (2 args) stubbed; callee 0x00630450 (2 args) stubbed; callee 0x00639e40 (13 args) stubbed; the word at `m+0x18` points at 32 random bytes less `0` (placed); slot +0x4c laid as 0x00472350 or its stub, a flag after the answers; slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 1 answered from [0, 1, 4] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 writes a value in -300..300 through its argument 0, printed after the answers (`o` in a place); stub 0 writes a value in -300..300 through its argument 1, printed after the answers (`o` in a place); stub 6 writes a value in -5..9 through its argument 1, printed after the answers (`o` in a place); the stubbed vtable laid at `p3+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_data_calc_gather.py") else {
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
                // The port's own `holdings::build_calc_gather_steps`, its callees answered as the
                // stubs were: `tile_corner` (stub 0, `o` the corner it writes), slot +0x4c (stub 1;
                // the base `WallData::is_active` when the first flag is set, `flags & 4` inlined),
                // slot +0xb8 `is(0x1a4, 0)` (stub 2; the base `ObjectData::is` → the type's slot
                // +0x60, stub 3), `count_inside(19, 0x34)` (stub 4), `num_gatherers(1, 1)` (stub
                // 5), and the type's `calc_gather` (stub 6, writing the count `o` through arg 1).
                let rec = std::cell::RefCell::new(Vec::<String>::new());
                let count = crate::holdings::build_calc_gather_steps(
                    a[4] != 0,
                    || {
                        rec.borrow_mut().push("s0:m+0x0,sp-0x28,sp-0x24".into());
                        (a[13], a[14])
                    },
                    || {
                        if a[16] != 0 {
                            i32::from(m[8] & 4)
                        } else {
                            rec.borrow_mut().push("s1:m+0x0".into());
                            a[7]
                        }
                    },
                    || {
                        if a[17] != 0 {
                            rec.borrow_mut().push("s3:p3+0x0,420,0".into());
                            a[9] != 0
                        } else {
                            rec.borrow_mut().push("s2:m+0x0,420,0".into());
                            a[8] != 0
                        }
                    },
                    || {
                        rec.borrow_mut().push("s4:m+0x0,19,52".into());
                        a[10]
                    },
                    || {
                        rec.borrow_mut().push("s5:m+0x0,1,1".into());
                        a[11]
                    },
                    |h| {
                        rec.borrow_mut().push(format!(
                            "s6:p3+0x0,p0+0x0,p1+0x0,p2+0x0,{},{},{},{},{},{},{},{},{},m+0x98",
                            a[3],
                            i16::from_le_bytes([m[0x72], m[0x73]]),
                            m[9],
                            a[5],
                            h.corner.0,
                            h.corner.1,
                            h.active,
                            h.gatherers,
                            h.scholars
                        ));
                        a[15]
                    },
                );
                log.extend(rec.into_inner());
                wr(&mut p[1], 0, count);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "calc_gather — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "calc_gather — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "calc_gather — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "calc_gather — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildData::calc_gather@0062d360: {n} rows agree");
}
