//! `Unit::set_attack@005fce70` under the emulator against the port (item 2368; tools/sweep_gen.py, obj with windows).

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

/// `Unit::set_attack@005fce70`, run by `tools/emu/sweep_unit_set_attack.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 2 argument(s), the global cell 0x00c0618c points at 64 random bytes; the global cell 0x00c06214 points at 1808 random bytes; vtable slot +0x34 (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0x1c (0 args) stubbed; callee 0x00902540 (1 args) stubbed; callee 0x005d8bc0 (0 args) stubbed; the word at `m+0xf4` points at 16 random bytes less `0` (placed); the word at `p0+0x0` points at 160 random bytes less `0` (placed); the word at `p0+0x4` points at 160 random bytes less `0` (placed); the word at `m+0x18` points at 512 random bytes less `0` (placed); the word at `c1+0x70c` points at 256 random bytes less `0` (placed); the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `p5+0x0` points at 64 random bytes less `0` (placed); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 5 answers 2 values in call order, printed after the answers; stub 4 answers 50..53 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_set_attack.py") else {
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
            // `a`: `o`, `who`, then the stubs' answers — `get_gpiece`,
            // `is_active`, `is_on_map`, `+0x1c`, `get_type`,
            // `set_all_pivots` — and `set_all_pivots`'s two in call order
            // (`--seq 5:2`). The figures `p1`, `p2` hang off `m+0xf4` →
            // `p0`; `m+0x18` → `p3` the type (`+0x1fc` max range);
            // `graphic_pieces` is `c1` (`+0x700` the count, `+0x70c` → `p4`
            // the rows, `0x20` apart); the target is `p6`.
            let mark = usize::try_from(m[0xb5] as i8).unwrap_or(0);
            let target = (a[0] >= 0 && a[1] >= 0).then_some(());
            let mut cx = (m, p, &mut log);
            let eax: i32 = crate::fight::set_attack_steps(
                &mut cx,
                mark,
                target,
                |(_, p, _), g, _| {
                    p[1 + g][0x8e..0x90].copy_from_slice(&(a[0] as i16).to_le_bytes());
                    p[1 + g][0x9f] = a[1] as u8;
                },
                |(_, p, log)| {
                    log.push("s0:m+0x0".into());
                    log.push(format!("s4:c1+0x0,{}", a[2]));
                    let k = a[6] - 0x32;
                    k < rd(&c[1], 0x700) && rd(&p[4], 0x14 + 0x20 * k as usize) != 0
                },
                |(_, p, _)| rd(&p[3], 0x1fc) != 0,
                |(_, _, log), ()| {
                    log.push("s1:p6+0x0".into());
                    if a[3] != 0 {
                        log.push("s2:p6+0x0".into());
                        if a[4] != 0 {
                            return true;
                        }
                    }
                    log.push("s3:p6+0x0".into());
                    a[5] != 0
                },
                |(_, _, log), g, ()| {
                    log.push(format!("s5:p{}+0x0", 1 + g));
                    a[8 + g.min(1)]
                },
            );
            // `--record-order` prints `-` for a row that called nothing.
            if log.is_empty() {
                log.push("-".into());
            }
            i64::from(eax)
        };
        assert_eq!(got, want_eax, "set_attack — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "set_attack — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_attack — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_attack — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "set_attack — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2000, "only {n} rows");
    eprintln!("Unit::set_attack@005fce70: {n} rows agree");
}
