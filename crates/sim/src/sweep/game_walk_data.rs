//! `Game::walk_data@00589600` under the emulator against the port (item 3058; tools/sweep_gen.py, obj with windows).

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

/// `Game::walk_data@00589600`, run by `tools/emu/sweep_game_walk_data.py` under unicorn
/// (tools/emu): 32 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0; the global cell 0x00c06378 points at 32 random bytes; the global cell 0x00c061ec points at 2304 random bytes; argument 0 names 16 random bytes; vtable slot +0x4 (1 args) stubbed; callee 0x005d6570 (1 args) stubbed; vtable slot +0x0 (2 args) stubbed; the import cell 0x00ac5460 holds a stub (0 args); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 is `__cdecl`: the caller pops its arguments; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_walk_data.py") else {
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
                // `a`: 0 the DataWalk pointer, 1..=4 the stubs' answers (a[4] is `timeGetTime`). The walker's callbacks
                // write nothing here; `c1` is the game record, `c0` the string table cell.
                let (input, checksum) = (rd(&p[0], 4), rd(&p[0], 8));
                let l0 = format!("p0+0x0,{}", rd(&c[0], 0x10).wrapping_add(0xd980));
                let l1 = "c1+0xc,p0+0x0".to_string();
                let mut calls = vec!["p0+0x0,c1+0x550,c1+0x6e4".to_string()];
                let mut l3 = "-".to_string();
                if checksum == 0 {
                    calls.push("p0+0x0,c1+0x814,c1+0x81c".to_string());
                    calls.push(format!(
                        "p0+0x0,c1+0x820,c1+{:#x}",
                        0x820 + rd(&c[1], 0x818)
                    ));
                    if input != 0 {
                        wr(&mut c[1], 0x81c, 2);
                    }
                }
                if input != 0 {
                    c[1][0x821] &= 0xfe;
                    if rd(&c[1], 0x81c) == 0 {
                        wr(&mut c[1], 0x81c, 2);
                    }
                }
                if checksum == 0 {
                    calls.push("p0+0x0,c1+0x844,c1+0x848".to_string());
                    l3 = "p0+0x0".to_string();
                    wr(&mut c[1], 0x840, a[4]);
                }
                log.extend([l0, l1, calls.join("/"), l3]);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "walk_data — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "walk_data — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "walk_data — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "walk_data — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Game::walk_data@00589600: {n} rows agree");
}
