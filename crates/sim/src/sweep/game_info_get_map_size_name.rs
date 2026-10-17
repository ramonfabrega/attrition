//! `GameInfo::get_map_size_name@005d46f0` under the emulator against the port (item 3004; tools/sweep_gen.py, obj with windows).

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

/// `GameInfo::get_map_size_name@005d46f0`, run by `tools/emu/sweep_game_info_get_map_size_name.py` under unicorn
/// (tools/emu): 20 random bytes of object, 0 argument(s), `p` windows in order: none; 4 random bytes at 0x00c8cd00; 1 random bytes at 0x00cc2311; callee 0x00a1eeb0 (1 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `edx` an integer in -2..9, after `ecx` on the row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_info_get_map_size_name.py") else {
        return;
    };
    let mut n = 0;
    let mut this_addr = None;
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
            // An empty `String` is built in place (every field zeroed but the byte at
            // `+0xb`, the module id `c[1]`), then `String::operator=` is handed the
            // size name's offset in the localized table `c[0]` points at by `edx`
            // (unsigned, 0..=6, else the last name); the answer is `this`.
            m[..0xb].fill(0);
            m[0xb] = c[1][0];
            m[0xc..0x14].fill(0);
            let off: i32 = match a[1] as u32 {
                0 => 0x133a8,
                1 => 0x133bc,
                2 => 0x133d0,
                3 => 0x133e4,
                4 => 0x133f8,
                5 => 0x1340c,
                6 => 0x13420,
                _ => 0x13434,
            };
            log.push(format!("m+0x0,{}", rd(&c[0], 0).wrapping_add(off)));
            *this_addr.get_or_insert(want_eax)
        };
        assert_eq!(got, want_eax, "get_map_size_name — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "get_map_size_name — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_map_size_name — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_map_size_name — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "get_map_size_name — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GameInfo::get_map_size_name@005d46f0: {n} rows agree");
}
