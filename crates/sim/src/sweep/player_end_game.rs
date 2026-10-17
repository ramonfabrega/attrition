//! `Player::end_game@006ee110` under the emulator against the port (item 2857; tools/sweep_gen.py, obj with windows).

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

/// `Player::end_game@006ee110`, run by `tools/emu/sweep_player_end_game.py` under unicorn
/// (tools/emu): 128 random bytes of object, 0 argument(s), the global cell 0x00c06210 points at 768 random bytes; the global cell 0x00c061ec points at 2304 random bytes; the global cell 0x00c0620c points at 768 random bytes; the global cell 0x00c06200 points at 64 random bytes; the global cell 0x00c06204 points at 64 random bytes; callee 0x00844b80 (0 args) stubbed; callee 0x0072c040 (0 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_player_end_game.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &p);
            // c[0] console, c[1] game, c[2] scene, c[3] camera, c[4] options.
            // The seat byte at this+0x36 against the console's play at +0x2a0.
            let seat = i32::from(m[0x36]);
            let (mut camera, mut options) = ("-", "-");
            if seat == rd(&c[0], 0x2a0) {
                let tick = rd(&c[1], 0x844);
                wr(&mut c[2], 0x214, tick);
                camera = "c3+0x0"; // Camera::outdate
                options = "c4+0x0"; // Options::reset
                if seat == rd(&c[0], 0x2a0) {
                    c[1][0x821] |= 0x80;
                    wr(&mut c[1], 0x81c, 0);
                }
            }
            log.push(camera.into());
            log.push(options.into());
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "end_game — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "end_game — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "end_game — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "end_game — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Player::end_game@006ee110: {n} rows agree");
}
