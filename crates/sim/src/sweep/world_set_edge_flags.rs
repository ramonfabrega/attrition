//! `World::set_edge_flags@006b0240` under the emulator against the port (item 3064; tools/sweep_gen.py, obj with windows).

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

/// `World::set_edge_flags@006b0240`, run by `tools/emu/sweep_world_set_edge_flags.py` under unicorn
/// (tools/emu): 512 random bytes of object, 0 argument(s), `p` windows in order: p0..p1 the place(s), in flag order; callee 0x0089c1a0 (0 args) stubbed; callee 0x008a85e0 (0 args) stubbed; callee 0x006b4e50 (5 args) stubbed; the word at `m+0x134` points at 1024 random bytes less `0` (placed); the word at `m+0x138` points at 1024 random bytes less `0` (placed); stub 2 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 2 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 2 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); the global cell 0x00c06188 holds `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_set_edge_flags.py") else {
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
            // `m` is the world (`this` and the global): `+0`/`+4` its width and height in tiles, `p[0]` (`+0x134`) the
            // tile records (0x1c each, bit 0x20 of the first byte), `p[1]` (`+0x138`) the 16-bit flag table `+0x18`
            // wide. After the mountains (stub 0) and cliffs (stub 1, `ecx` the constant 0xe88eb0) have their edge
            // flags set, each marked tile asks stub 2 (`has_gather_access`, handed the sub-tile's x and y by
            // pointer) about its 16 sub-tiles and sets bit 0x8000 of the flag table entry for each it answers non-zero;
            // the calls' x and y are printed after the record.
            let (w, h, w2) = (rd(m, 0) as usize, rd(m, 4) as usize, rd(m, 0x18) as usize);
            let mut tokens: Vec<String> = vec!["s0:m+0x0".to_string(), "s1:15240880".to_string()];
            let (mut xs, mut ys): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
            for ty in 0..h {
                for tx in 0..w {
                    if p[0][(w * ty + tx) * 0x1c] & 0x20 == 0 {
                        continue;
                    }
                    for k in 0..16usize {
                        let (x, y) = ((k & 3) + tx * 4, (k >> 2) + ty * 4);
                        tokens.push("s2:m+0x0,sp-0x18,sp-0x10,-1,0,1".to_string());
                        xs.push(format!("{:08x}", (x as u32).swap_bytes()));
                        ys.push(format!("{:08x}", (y as u32).swap_bytes()));
                        if a[2] != 0 {
                            let at = (w2 * y + x) * 2;
                            let v = u16::from_le_bytes([p[1][at], p[1][at + 1]]) | 0x8000;
                            p[1][at..at + 2].copy_from_slice(&v.to_le_bytes());
                        }
                    }
                }
            }
            for list in [xs, ys] {
                tokens.push(if list.is_empty() {
                    "-".to_string()
                } else {
                    list.join("/")
                });
            }
            log.extend(tokens);
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "set_edge_flags — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_edge_flags — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "set_edge_flags — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "set_edge_flags — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("World::set_edge_flags@006b0240: {n} rows agree");
}
