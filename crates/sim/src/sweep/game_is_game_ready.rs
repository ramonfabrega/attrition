//! `Game::is_game_ready@00583af0` under the emulator against the port (item 2336; tools/sweep_gen.py, obj with windows).

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

/// `Game::is_game_ready@00583af0`, run by `tools/emu/sweep_game_is_game_ready.py` under unicorn
/// (tools/emu): 2096 random bytes of object, 0 argument(s), the global cell 0x00c06210 points at 676 random bytes; the global cell 0x00c061ec points at 1264 random bytes; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_is_game_ready.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
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
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Flag 0x10 of the semaphore byte (+0x820) clear, players (+0x69c) and sides (+0x6a8)
                // not 1 → whether the console's player (+0x2a0 of the record cell 0xc06210 names)
                // has its cheat byte (+0xcb of a 0x8c-byte player row of the game, cell 0xc061ec) zero.
                if m[0x820] & 0x10 == 0 && rd(m, 0x69c) != 1 && rd(m, 0x6a8) != 1 {
                    let play = rd(&c[0], 0x2a0) as usize;
                    i64::from(c[1][play * 0x8c + 0xcb] == 0)
                } else {
                    0
                }
            }
        };
        assert_eq!(
            got & 0xff,
            want_eax & 0xff,
            "is_game_ready — al, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_game_ready — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "is_game_ready — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "is_game_ready — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Game::is_game_ready@00583af0: {n} rows agree");
}
