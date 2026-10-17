//! `TurnControl::walk_data@00956c50` under the emulator against the port (item 2783; tools/sweep_gen.py, obj with windows).

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

/// `TurnControl::walk_data@00956c50`, run by `tools/emu/sweep_turn_control_walk_data.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), the global cell 0x00c06378 points at 32 random bytes; the global cell 0x00c06180 points at 512 random bytes; argument 0 names 16 random bytes; vtable slot +0x0 (2 args) stubbed; vtable slot +0x4 (1 args) stubbed; the stubbed vtable laid at `p0+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_turn_control_walk_data.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `TurnControl::walk_data(walker)`, on the turn-control record `T` the cell `0xc06180` names
            // (`c[1]`; `this` is unread): slot 4 of the walker is handed `*(G+0x10) + 0x22268` for `G` the
            // record of cell `0xc06378` (`c[0]`); slot 0 walks `(T+4, T+0xc)`, then `(T+0x10, T+0x10+[T+8])`,
            // then — after a walker with a nonzero input word at `+4` stores 2 at `T+0xc` — `(T+0x14, T+0x169)`.
            let size = rd(&c[1], 8);
            log.push(format!(
                "p0+0x0,c1+0x4,c1+0xc/p0+0x0,c1+0x10,c1+{:#x}/p0+0x0,c1+0x14,c1+0x169",
                0x10 + size
            ));
            log.push(format!("p0+0x0,{}", rd(&c[0], 0x10).wrapping_add(0x22268)));
            if rd(&p[0], 4) != 0 {
                wr(&mut c[1], 0xc, 2);
            }
            0i64
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "walk_data — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "walk_data — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "walk_data — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "walk_data — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TurnControl::walk_data@00956c50: {n} rows agree");
}
