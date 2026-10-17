//! `Game::unload_net_lib@005877d0` under the emulator against the port (item 2783; tools/sweep_gen.py, obj with windows).

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

/// `Game::unload_net_lib@005877d0`, run by `tools/emu/sweep_game_unload_net_lib.py` under unicorn
/// (tools/emu): 3072 random bytes of object, 0 argument(s), 4 random bytes at 0x00e335b8; vtable slot +0x8 (0 args) stubbed; vtable slot +0x100 (0 args) stubbed; the import cell 0x00ac50c4 holds a stub (1 args); the word at `0xe335c8` points at 32 random bytes less `0` (placed, null a third of the rows); the stubbed vtable laid at `p0+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_unload_net_lib.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `Game::unload_net_lib()`: with the net system cell `0xe335c8` non-null (the original's own
            // first slot call says which, `want_log[0]`; the cell itself is no window, so the store of 0 to it
            // is not held) it calls slot 8 and slot 0x100 of its vtable (`ecx` the record, `p[0]`); then a
            // nonzero library handle `0xe335b8` (`c[0]`) goes to `FreeLibrary` (`0xac50c4`, `ecx` still the record) and is zeroed;
            // last `this+0xbf0` is -1.
            if want_log[0] != "-" {
                log.push("p0+0x0".to_string());
                log.push("p0+0x0".to_string());
                let h = rd(&c[0], 0);
                if h != 0 {
                    log.push(format!("p0+0x0,{h}"));
                    wr(&mut c[0], 0, 0);
                } else {
                    log.push("-".to_string());
                }
                wr(m, 0xbf0, -1);
            } else {
                log.extend(["-".to_string(), "-".to_string(), "-".to_string()]);
            }
            0i64
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "unload_net_lib — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "unload_net_lib — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "unload_net_lib — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "unload_net_lib — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Game::unload_net_lib@005877d0: {n} rows agree");
}
