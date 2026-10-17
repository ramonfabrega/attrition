//! `Search::valid_search@0067daa0` under the emulator against the port (item 3024; tools/sweep_gen.py, obj with windows).

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

/// `Search::valid_search@0067daa0`, run by `tools/emu/sweep_search_valid_search.py` under unicorn
/// (tools/emu): 16 random bytes of object, 2 argument(s), `p` windows in order: p0..p3 sparse table 0 (4 rows × 1 slice(s), row-major); callee 0x006edb50 (1 args) stubbed; callee 0x006ebaa0 (1 args) stubbed; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 4 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x1 (`p` windows after the pointed buffers, before the places); `ecx` an integer in 0..3, not the object, the row's last; `ecx` and `edx` held to their draw every row, no wild word; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_search_valid_search.py") else {
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
            let _ = (&a, &m, &c, &p);
            // `this` (`a[4]`) a player index, `a[0]` the mode, `a[1]` the other player; the
            // answers `a[2]` of `is_ally` (0x6edb50) and `a[3]` of `is_enemy` (0x6ebaa0), each
            // handed the leader row `p{this}` and the other player. A negative other: valid.
            let (mode, other, this) = (a[0], a[1], a[4]);
            let mut ally: Vec<String> = Vec::new();
            let mut enemy: Vec<String> = Vec::new();
            let mut is_ally = || {
                ally.push(format!("p{this}+0x0,{other}"));
                a[2] != 0
            };
            let mut is_enemy = || {
                enemy.push(format!("p{this}+0x0,{other}"));
                a[3] != 0
            };
            let ok = if other < 0 {
                true
            } else {
                match mode {
                    1 => other == this,
                    2 => !(is_ally() || is_enemy()),
                    3 => is_enemy(),
                    4 => is_ally(),
                    5 => !is_ally(),
                    6 => other != this,
                    7 => !is_enemy(),
                    _ => true,
                }
            };
            let tok = |v: &Vec<String>| {
                if v.is_empty() {
                    "-".to_string()
                } else {
                    v.join("/")
                }
            };
            log.push(tok(&ally));
            log.push(tok(&enemy));
            i64::from(ok)
        };
        assert_eq!(got, want_eax, "valid_search — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "valid_search — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "valid_search — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "valid_search — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "valid_search — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 500, "only {n} rows");
    eprintln!("Search::valid_search@0067daa0: {n} rows agree");
}
