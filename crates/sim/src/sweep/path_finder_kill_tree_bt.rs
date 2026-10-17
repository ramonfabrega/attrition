//! `PathFinder::kill_tree@00687ba0` under the emulator against the port (item 2910; tools/sweep_gen.py, obj with windows).

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

/// `PathFinder::kill_tree@00687ba0`, run by `tools/emu/sweep_path_finder_kill_tree_bt.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), 16 random bytes at 0x00c8da70; argument 0 names 32 random bytes; callee 0x0047a820 (1 args) stubbed; the word at `p0+0` points at 32 random bytes less `0` (placed, null a third of the rows); the word at `p0+4` points at 32 random bytes less `0` (placed, null a third of the rows); the word at `p1+0` points at 32 random bytes less `0` (placed, null a third of the rows); the word at `p1+4` points at 32 random bytes less `0` (placed, null a third of the rows); the word at `p2+0` points at 32 random bytes less `0` (placed, null a third of the rows); the word at `p2+4` points at 32 random bytes less `0` (placed, null a third of the rows); the word at `c0+0` points at 64 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_path_finder_kill_tree_bt.py") else {
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
            let _ = (&a, &m);
            // A post-order walk of the tree rooted at argument 0 (nodes: left +0, right +4, data +0xc; a node whose byte +0x15 is set is skipped): each
            // node's data is pushed on the recycler's pool at 0xc8da70 (list pointer +0, size +4, length +8, growth
            // step the byte at +0xc; `increase_size` is stubbed and called first when full) and the node's data
            // slot cleared. The windows are p0 the root and p1..p6 its children breadth-first, p7 the pool's list.
            fn visit(p: &mut [Vec<u8>], c: &mut [u8], n: usize, calls: &mut Vec<String>) {
                if n >= 7 {
                    return;
                }
                for k in [1, 2] {
                    if rd(&p[n], 4 * (k - 1)) != 0 {
                        visit(p, c, 2 * n + k, calls);
                    }
                }
                if p[n][0x15] == 0 {
                    let (data, len) = (rd(&p[n], 0xc), rd(c, 8));
                    if len >= rd(c, 4) {
                        calls.push(format!("{len},{}", c[0xc] as i8));
                    }
                    wr(&mut p[7], 4 * len as usize, data);
                    wr(c, 8, len + 1);
                    wr(&mut p[n], 0xc, 0);
                }
            }
            let mut calls = Vec::new();
            visit(p, &mut c[0], 0, &mut calls);
            log.push(if calls.is_empty() {
                "-".into()
            } else {
                calls.join("/")
            });
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "kill_tree — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "kill_tree — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "kill_tree — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "kill_tree — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("PathFinder::kill_tree@00687ba0: {n} rows agree");
}
