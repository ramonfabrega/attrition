//! `GameAccessConst::find_tcoord_z@005834b0` under the emulator against the port (item 2346; tools/sweep_gen.py, obj with windows).

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

/// `GameAccessConst::find_tcoord_z@005834b0`, run by `tools/emu/sweep_game_access_const_find_tcoord_z.py` under unicorn
/// (tools/emu): 16 random bytes of object, 2 argument(s), the global cell 0x00c06218 points at 16 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; callee 0x008544a0 (4 args) stubbed; `__cdecl`: the caller pops the arguments; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `ecx` an integer in -2147483648..2147483647, not the object, the row's last; `edx` an integer in -2147483648..2147483647, after `ecx` on the row; `ecx` and `edx` held to their draw every row, no wild word; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_access_const_find_tcoord_z.py") else {
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
            {
                // The terrain (cell 0xc06218) asked for the height into the caller's `Coord` (`ecx`, the
                // hidden return pointer) at the tile the two coordinates name — each `>> 6` through
                // `div_3_table` (cell 0xcae5fc) — with `edx` passed through; `ecx` answered.
                let (x, y) = (
                    rd(&c[1], (a[0] >> 6) as usize * 4),
                    rd(&c[1], (a[1] >> 6) as usize * 4),
                );
                log.push(format!("c0+0x0,{},{x},{y},{}", a[3], a[4]));
                i64::from(a[3])
            }
        };
        assert_eq!(got, want_eax, "find_tcoord_z — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_tcoord_z — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_tcoord_z — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_tcoord_z — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "find_tcoord_z — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GameAccessConst::find_tcoord_z@005834b0: {n} rows agree");
}
