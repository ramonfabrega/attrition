//! `World::new_coll_block@0046d250` under the emulator against the port (item 3049; tools/sweep_gen.py, obj with windows).

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

/// `World::new_coll_block@0046d250`, run by `tools/emu/sweep_world_new_coll_block.py` under unicorn
/// (tools/emu): 16 random bytes of object, 2 argument(s), `p` windows in order: p0..p1 pointed argument(s), p2 the place, p3 stub 0's answer window; the global cell 0x00c06188 points at 320 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x00681fa0 (0 args) stubbed; callee 0x0055e0b2 (3 args) stubbed; the word at `c0+0x134` points at 448 random bytes less `0` (placed); stub 0 answers the address of 112 random bytes, a `p` window after the places; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 is `__cdecl`: the caller pops its arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_new_coll_block.py") else {
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
            let _ = (&a, &m, &c, &p);
            // A collision block (the allocator stub, answering `p3`) is initialised (+0 = 0x300, +4 = 0x60,
            // +8 = 1; the 0x60 bytes at +0xc cleared by the CRT memset, a stub here, whose call is the log),
            // then stored in the world tile (`c0` the world: width +0, tile array +0x134, 0x1c a tile) at
            // (x, y) = the words the two arguments point at, at tile offset +0x18; the tile's word is returned.
            // (The allocator's null answer — block 0 stored — is not drawn: the stub answers a window.)
            let (x, y) = (rd(&p[0], 0), rd(&p[1], 0));
            let at = ((rd(&c[0], 0) * y + x) * 0x1c + 0x18) as usize;
            wr(&mut p[3], 0, 0x300);
            wr(&mut p[3], 4, 0x60);
            wr(&mut p[3], 8, 1);
            wr(&mut p[2], at, a[2]);
            log.push("m+0x0".to_string());
            log.push("m+0x0,p3+0xc,0,96".to_string());
            i64::from(rd(&p[2], at))
        };
        assert_eq!(got, want_eax, "new_coll_block — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "new_coll_block — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "new_coll_block — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "new_coll_block — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "new_coll_block — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("World::new_coll_block@0046d250: {n} rows agree");
}
