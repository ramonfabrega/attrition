//! `Group::set_up_insert@0070e520` under the emulator against the port (item 3112; tools/sweep_gen.py, obj with windows).

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

/// `Group::set_up_insert@0070e520`, run by `tools/emu/sweep_group_set_up_insert.py` under unicorn
/// (tools/emu): 80 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p8 the place(s), in flag order; argument 0 names 32 random bytes; callee 0x0070ccb0 (1 args) stubbed; callee 0x0072f900 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; the word at `0xc0aec0` points at 4 random bytes less `0` (placed); the word at `p1+0` points at 256 random bytes less `0` (placed); the word at `p2+0xdc` points at 32 random bytes less `0` (placed); the word at `p3+0` points at 32 random bytes less `0` (placed); the word at `p4+0` points at 32 random bytes less `0` (placed); the word at `p3+8` points at 32 random bytes less `0` (placed); the word at `p4+8` points at 32 random bytes less `0` (placed); the word at `p5+8` points at 32 random bytes less `0` (placed); stub 0 answered from [-1, 0, 0, 0] (90%); the word at `p5+0` laid as the address of `p3` every row; stub 0 answers -1..0 every row; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 300 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_set_up_insert.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `a`: 0 the list's address, 1 `find_leader`'s answer (-1 or 0), 2 `copy_order`'s answer, 3 `add`'s.
                // Windows: p2 the unit record, p3..p5 the ring's three nodes (`+0` next, `+8` order, `+0xc` flag byte),
                // p6..p8 their orders (flags at +4). The ring is laid by the alias, so the port walks it by index: the
                // head (`+0xdc`) is node 0, and the cursor (`+0xd4`) holds the node's own address, read back from the
                // window that points at it.
                let rd = |w: &[u8], o: usize| i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"));
                let wr = |w: &mut [u8], o: usize, v: i32| w[o..o + 4].copy_from_slice(&v.to_le_bytes());
                let mut calls: Vec<String> = Vec::new();
                if a[1] >= 0 && rd(&p[2], 0xd8) != 0 {
                    let addr = [rd(&p[2], 0xdc), rd(&p[3], 0), rd(&p[4], 0)];
                    let mut k = 0usize;
                    let step = |p: &mut [Vec<u8>], k: usize| {
                        wr(&mut p[2], 0xd4, addr[k]);
                        let o = rd(&p[3 + k], 8);
                        wr(&mut p[2], 0xcc, o);
                        let f = p[3 + k][0xc];
                        p[2][0xd0] = f;
                    };
                    if rd(&p[2], 0xdc) != 0 {
                        step(p, 0);
                    }
                    loop {
                        if rd(&p[2], 0xcc) != 0 && p[6 + k][4] & 4 != 0 {
                            calls.push(format!("p0+0x4,{}", a[2]));
                        }
                        if rd(&p[2], 0xdc) != 0 {
                            k = (k + 1) % 3;
                            step(p, k);
                        }
                        if rd(&p[2], 0xd4) == rd(&p[2], 0xdc) {
                            break;
                        }
                    }
                }
                log.push(if calls.is_empty() { "-".into() } else { calls.join("/") });
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "set_up_insert — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_up_insert — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_up_insert — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_up_insert — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("Group::set_up_insert@0070e520: {n} rows agree");
}
