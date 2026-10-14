//! `Unit::add_air_patrol_order@005e4350` under the emulator against the port (item 2341; tools/sweep_gen.py, obj with windows).

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

/// `Unit::add_air_patrol_order@005e4350`, run by `tools/emu/sweep_unit_add_air_patrol_order.py` under unicorn
/// (tools/emu): 256 random bytes of object, 6 argument(s), the global cell 0x00cae5fc points at 4096 random bytes; the global cell 0x00c0618c points at 256 random bytes; callee 0x005e55c0 (11 args) stubbed; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x8c (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; callee 0x00434630 (1 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; the word at `m+0x18` points at 4 random bytes less `0x2b4` (placed); stub 4's answer points at 256 random bytes less `0` (placed); stub 5's answer points at 128 random bytes less `0` (placed); the word at `p2+0` points at 8 random bytes less `0` (placed); the word at `p2+0x14` points at 4 random bytes less `0` (placed); the word at `p2+0x30` points at 4 random bytes less `0` (placed); the word at `c1+0x14` points at 4 random bytes less `0` (placed); the word at `p6+0` points at 24 random bytes less `0` (placed); stub 6 answered from [0, 1] (90%); the stubbed vtable laid at `p1+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p7+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_add_air_patrol_order.py") else {
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
            // `p[0]` is the type's word at `+0x2b4`; `p[2]` the order the
            // slot `+0x8c` stub answers, its vbtable `p[3]` (the base's
            // offset `d` at `+4`), its two arrays' data `p[4]`/`p[5]`; `p[7]`
            // the home `objects[0][0]` (`c[1]+0x14` -> `p[6]`). The order is
            // carried as a copy of `p[2]` and written back at the link.
            // Stub 6's (`is_active`) answer is the row's `a[12]`.
            const KEY: i32 = 0x63637;
            let d = rd(&p[3], 4) as usize;
            let raw_action = a[4];
            let active = a[12];
            let mut cx = (m, c, p, &mut log);
            crate::orders::add_air_patrol_steps(
                &mut cx,
                crate::Pos::new(a[0], a[1]),
                a[4] != 0,
                |(_, _, p, _)| p[0][0] & 0x20 != 0,
                |(m, ..)| crate::Pos::new(rd(m, 0x10) ^ KEY, rd(m, 0x14) ^ KEY),
                |(_, c, _, log), to, angle, action| {
                    assert_eq!(action, raw_action != 0);
                    let cell = |v: i32| rd(&c[0], 4 * (v >> 4) as usize);
                    log.push(format!(
                        "s0:m+0x0,{},{},{},2,0,2,{raw_action},-1,-1,-1,0",
                        cell(to.x),
                        cell(to.y),
                        angle.0
                    ));
                },
                |(m, ..)| {
                    wr(m, 0x68, rd(m, 0x68) & !0x0400_0000);
                    wr(m, 0xc0, 0);
                },
                |(.., log)| log.push("s1:m+0x0,0".into()),
                |(.., log)| log.push("s2:m+0x0".into()),
                |(.., log)| log.push("s3:m+0x0".into()),
                |(_, _, p, log)| {
                    log.push("s4:m+0x0,17".into());
                    log.push("s5:p1+0x0".into());
                    p[2].clone()
                },
                |(_, _, p, log)| {
                    if a[2] < 0 || a[3] < 0 {
                        return None;
                    }
                    log.push("s6:p7+0x0".into());
                    (active != 0)
                        .then(|| crate::Pos::new(rd(&p[7], 0x10) ^ KEY, rd(&p[7], 0x14) ^ KEY))
                },
                |(_, _, p, log), o, at| {
                    // `SimpleArray::increase_size` to one, for the x array
                    // (`+0x4`: count `+0x8`, size `+0xc`, growth `+0x10`)
                    // and the y (`+0x20`).
                    for base in [0x4usize, 0x20] {
                        let size = rd(o, base + 8);
                        let need = 1i32.wrapping_sub(size);
                        if size < 1 && need > 0 {
                            let g = i32::from(i16::from_le_bytes([o[base + 12], o[base + 13]]));
                            let local = if g >= 0 { g } else { size };
                            let by = if need <= local { local } else { need };
                            log.push(format!("s7:p2+{base:#x},{by}"));
                        }
                        if base == 0x4 {
                            let n = rd(o, 0x8).max(1);
                            wr(o, 0x8, n);
                        }
                    }
                    wr(o, 0x8, 1);
                    wr(o, 0x24, 1);
                    wr(&mut p[4], 0, at.x);
                    wr(&mut p[5], 0, at.y);
                    wr(o, 0x44, a[2]);
                    wr(o, 0x48, a[3]);
                    wr(o, 0x3c, 0);
                    wr(o, 0x58, 0);
                    wr(o, 0x4c, 0x640);
                },
                |o| &mut o[d + 4],
                |(_, _, p, log), o| {
                    log.push(format!("s8:m+0xcc,p2+{d:#x}"));
                    p[2] = o;
                },
            );
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "add_air_patrol_order — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "add_air_patrol_order — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "add_air_patrol_order — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "add_air_patrol_order — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Unit::add_air_patrol_order@005e4350: {n} rows agree");
}
