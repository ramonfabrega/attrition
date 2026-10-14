//! `Game::wonder_winning@005948a0` under the emulator against the port (item 2247; tools/sweep_gen.py, obj with windows).

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

/// `Game::wonder_winning@005948a0`, run by `tools/emu/sweep_game_wonder_winning.py` under unicorn
/// (tools/emu): 32 random bytes of object, 0 argument(s), the global cell 0x00c061ec points at 64 random bytes; callee 0x006ebb10 (0 args) stubbed; callee 0x006edb50 (1 args) stubbed; callee 0x006ebb90 (0 args) stubbed; the word at `0xe803e8` points at 64 random bytes less `0x58*u8(c[0],0x39)` (placed); stub 0 answered from [0, 0, 1, 2, 5, 5, 6, 10, -1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1, 2, 3, -1] (90%); stub 0 answers 8 values in call order, printed after the answers; stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 16 values in call order, printed after the answers; 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_wonder_winning.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // The leaders are `p[0..8]`, their flag words (`& 3 == 3` active);
                // the victory's points `p[8]+0x3c`, its row by the game's byte
                // `c[0]+0x39`. After the three callees' first answers come
                // `get_wonder_net`'s eight (`a[3..11]`), `is_ally`'s eight
                // (`a[11..19]`) and `get_wonder_value`'s sixteen (`a[19..35]`),
                // in call order, each's last repeated past its count.
                let active: Vec<crate::Player> = (0..8u8)
                    .filter(|&k| rd(&p[k as usize], 0) & 3 == 3)
                    .collect();
                let calls = |base: usize, len: usize| {
                    let mut k = 0;
                    move || {
                        let x = a[base + k.min(len - 1)];
                        k += 1;
                        x
                    }
                };
                let (mut net, mut ally, mut val) = (calls(3, 8), calls(11, 8), calls(19, 16));
                i64::from(crate::ai_build::wonder_lead(
                    rd(&p[8], 0x3c),
                    active,
                    |_| net(),
                    |_, _| ally() != 0,
                    |_| val(),
                ))
            }
        };
        assert_eq!(got, want_eax, "wonder_winning — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "wonder_winning — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "wonder_winning — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "wonder_winning — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Game::wonder_winning@005948a0: {n} rows agree");
}
