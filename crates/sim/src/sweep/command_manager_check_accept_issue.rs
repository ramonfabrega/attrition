//! `CommandManager::check_accept_issue@00940a70` under the emulator against the port (item 2910; tools/sweep_gen.py, obj with windows).

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

/// `CommandManager::check_accept_issue@00940a70`, run by `tools/emu/sweep_command_manager_check_accept_issue.py` under unicorn
/// (tools/emu): 86640 random bytes of object, 0 argument(s), vtable slot +0x10 (0 args) stubbed; callee 0x004b71d0 (0 args) stubbed; the word at `0xc061ec` points at 2096 random bytes less `0` (placed); the word at `0xc06210` points at 688 random bytes less `0` (placed); the word at `0xe335c8` points at 48 random bytes less `0` (placed); the word at `p2+0x28` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `0xc06180` points at 32 random bytes less `0` (placed); stub 0 answered from [0, 1, 1] (90%); stub 1 answered from [0, 1, 1] (90%); the stubbed vtable laid at `p3+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_manager_check_accept_issue.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = &c;
            // Windows: p0 the game record (0xc061ec; flag bytes 0x820, 0x821, 0x20), p1 the console (word +0x2a0
            // the local player), p2 the net system (the local-player pointer at +0x28, null in a third of the
            // rows) with its object p3, p4 the turn control (flag byte +0x10). The net system's own cell is
            // never null here: that arm is not drawn. Slot 0x10 (`a[0]`) and `is_active_connection` (`a[1]`)
            // are the two stubs; the second is handed the local player's record in the game.
            let g = &p[0];
            let (mut t0, mut t1) = ("-".to_string(), "-".to_string());
            let r = if m[0x15264] != 0 || g[0x820] & 0x10 != 0 {
                0
            } else if g[0x820] & 4 == 0 || g[0x821] & 8 != 0 {
                1
            } else {
                let lp = rd(&p[2], 0x28) != 0;
                if lp {
                    t0 = "p3+0x0".into();
                }
                if lp && a[0] != 0 {
                    0
                } else {
                    t1 = format!("p0+{:#x}", 0x44 + 0x8c * rd(&p[1], 0x2a0));
                    if a[1] == 0 {
                        0
                    } else {
                        i64::from(p[4][0x10] & 1 == 0 || g[0x20] & 0x40 != 0)
                    }
                }
            };
            log.extend([t0, t1]);
            r
        };
        assert_eq!(got, want_eax, "check_accept_issue — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "check_accept_issue — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "check_accept_issue — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "check_accept_issue — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "check_accept_issue — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CommandManager::check_accept_issue@00940a70: {n} rows agree");
}
