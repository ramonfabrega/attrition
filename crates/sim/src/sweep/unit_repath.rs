//! `Unit::repath@005e29b0` under the emulator against the port (item 2341; tools/sweep_gen.py, obj with windows).

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

/// `Unit::repath@005e29b0`, run by `tools/emu/sweep_unit_repath.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 0 argument(s), the global cell 0x00e85f20 points at 10240 random bytes; vtable slot +0x10 (0 args) stubbed; vtable slot +0x10c (0 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; callee 0x007123f0 (1 args) stubbed; the word at `m+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p0+0x4` points at 16 random bytes less `0` (placed); the word at `p1+0x8` points at 16 random bytes less `0` (placed); stub 0 answered from [1, 2, 3, 4, 18, 19, 21, 0, 7, 26] (90%); stub 1 answers the address of 32 random bytes, a `p` window after the places; the stubbed vtable laid at `p2+0`; stub 0 answers 6 values in call order, the last from [0, 7, 26], printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_repath.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `a` is the four stubs' answers then `get_type`'s six (`--seq
            // 0:6`, the last held off the family by the script's hand
            // edit). The head is `m+0xdc` → `p0`, its node `p0+4` → `p1`,
            // whose `+8` is the order `p2` (the stubbed vtable at `+0`,
            // flags at `+4`); `get_group_move_order` answers `p3`, the id
            // its `+0xc`; `groups` is `c0`, stride `0x9d4`.
            let mut turn = 0usize;
            let mut cx = (m, p, &mut log);
            crate::orders::repath_steps(
                &mut cx,
                |(m, _, _)| rd(m, 0xdc) != 0,
                |(_, _, log)| {
                    log.push("s0:p2+0x0".into());
                    let v = a[4 + turn.min(5)];
                    turn += 1;
                    v
                },
                |(_, p, _)| p[2][4] & 4 != 0,
                |(m, _, _)| {
                    let g = i16::from_le_bytes([m[0x80], m[0x81]]);
                    (g >= 0).then_some(i32::from(g))
                },
                |(_, p, log)| {
                    log.push("s1:p2+0x0".into());
                    rd(&p[3], 0xc)
                },
                |(_, _, log), g, id| log.push(format!("s3:c0+{:#x},{id}", g * 0x9d4)),
                |(_, _, log)| log.push("s2:m+0x0,0".into()),
            );
            let (m, p, log) = cx;
            // `--record-order` prints `-` for a row that called nothing.
            if log.is_empty() {
                log.push("-".into());
            }
            // Every head read writes the engine's iterator cache.
            if rd(m, 0xdc) != 0 {
                let node = rd(&p[0], 4);
                wr(m, 0xd4, node);
                wr(m, 0xcc, rd(&p[1], 8));
                m[0xd0] = p[1][0xc];
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "repath — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "repath — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "repath — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "repath — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Unit::repath@005e29b0: {n} rows agree");
}
