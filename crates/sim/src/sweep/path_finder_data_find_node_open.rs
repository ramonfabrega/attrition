//! `PathFinderData::find_node_open@006882b0` under the emulator against the port (item 2774; tools/sweep_gen.py, obj with windows).

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

/// `PathFinderData::find_node_open@006882b0`, run by `tools/emu/sweep_path_finder_data_find_node_open.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), argument 0 names 32 random bytes; callee 0x00479b90 (1 args) stubbed; the word at `m+0` points at 32 random bytes less `0` (placed); the word at `m+4` points at 32 random bytes less `0` (placed); the word at `p2+0` points at 32 random bytes less `0` (placed); the word at `p2+0x10` points at 32 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_path_finder_data_find_node_open.py") else {
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
            // `PathFinderData::find_node_open(node)`: seeks the node's metric (`+0x18`, `p[0]`) in the
            // refs tree (`this+4`, `p[2]`; `0x479b90`, `ecx` the tree). A hit copies the tree's current data
            // record (`p[3]`, pointed to by the word at `p[2]+0`) into the open list (`this+0`, `p[1]`):
            // its address to `+0x10`, its `+8` to `+0x14`, its `+0xc` to `+0`, its `+0x10` to `+4`; the
            // answer is that record's `+0xc`, or 0 when the tree's current node (`p[4]`, `p[2]+0x10`)
            // is flagged removed (`+0x15`).
            log.push(format!("p2+0x0,{}", rd(&p[0], 0x18)));
            let mut eax = 0;
            if a[1] != 0 {
                let (addr, f8, fc, f10) =
                    (rd(&p[2], 0), rd(&p[3], 8), rd(&p[3], 0xc), rd(&p[3], 0x10));
                wr(&mut p[1], 0x10, addr);
                wr(&mut p[1], 0x14, f8);
                wr(&mut p[1], 0, fc);
                wr(&mut p[1], 4, f10);
                if p[4][0x15] == 0 {
                    eax = fc;
                }
            }
            i64::from(eax)
        };
        assert_eq!(got, want_eax, "find_node_open — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_node_open — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_node_open — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_node_open — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_node_open — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("PathFinderData::find_node_open@006882b0: {n} rows agree");
}
