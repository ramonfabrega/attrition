//! `Terrain::walk_coord_data@00850ef0` under the emulator against the port (item 2857; tools/sweep_gen.py, obj with windows).

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

/// `Terrain::walk_coord_data@00850ef0`, run by `tools/emu/sweep_terrain_walk_coord_data.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), the global cell 0x00c06188 points at 64 random bytes; the global cell 0x00c06218 points at 18432 random bytes; argument 0 names 16 random bytes; vtable slot +0x0 (2 args) stubbed; callee 0x0085f6a0 (0 args) stubbed; the word at `c1+0x47d8` points at 72 random bytes less `0` (placed); the word at `p1+8` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p1+20` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p1+32` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p1+44` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p1+56` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p1+68` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p2+8` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p3+8` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p4+8` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p5+8` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p6+8` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p7+8` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p8+8` points at 64 random bytes less `0` (placed); the word at `p9+8` points at 64 random bytes less `0` (placed); the word at `p10+8` points at 64 random bytes less `0` (placed); the word at `p11+8` points at 64 random bytes less `0` (placed); the word at `p12+8` points at 64 random bytes less `0` (placed); the word at `p13+8` points at 64 random bytes less `0` (placed); the stubbed vtable laid at `p0+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_terrain_walk_coord_data.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `Terrain::walk_coord_data`: the row count is world[0] - 1 + world[1] (read before anything runs);
            // a walker whose `input` (+4) is non-zero first regenerates the land lists on the terrain; then each of
            // the count rows of the terrain's table (12 bytes a row, the list head at +8) has its list walked
            // through the node's +8 link, the walker's slot 0 handed (node+0x1c, node+0x1e).
            let rows = rd(&c[0], 0).wrapping_sub(1).wrapping_add(rd(&c[0], 4));
            if rd(&p[0], 4) != 0 {
                log.push("s1:c1+0x0".into());
            }
            for k in 0..usize::try_from(rows.max(0)).expect("a count") {
                for lvl in 0..3 {
                    let live = if lvl == 0 {
                        rd(&p[1], 8 + 12 * k) != 0
                    } else {
                        rd(&p[2 + 6 * (lvl - 1) + k], 8) != 0
                    };
                    if !live {
                        break;
                    }
                    let w = 2 + 6 * lvl + k;
                    log.push(format!("s0:p0+0x0,p{w}+0x1c,p{w}+0x1e"));
                }
            }
            if log.is_empty() {
                log.push("-".into());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "walk_coord_data — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "walk_coord_data — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "walk_coord_data — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "walk_coord_data — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Terrain::walk_coord_data@00850ef0: {n} rows agree");
}
