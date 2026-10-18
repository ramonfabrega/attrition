//! `Build::find_gather_tiles@00623350` under the emulator against the port (item 3238; tools/sweep_gen.py, obj with windows).

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

/// `Build::find_gather_tiles@00623350`, run by `tools/emu/sweep_build_find_gather_tiles.py` under unicorn
/// (tools/emu): 176 random bytes of object, 0 argument(s), `p` windows in order: p0..p2 the place(s), in flag order; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; vtable slot +0x90 (0 args) stubbed; vtable slot +0x94 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; callee 0x0063bdc0 (4 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x0063c430 (4 args) stubbed; the word at `m+0x18` points at 576 random bytes less `0` (placed); the word at `m+0xa8` points at 128 random bytes less `0` (placed); the word at `c0+0x138` points at 1024 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 1, 0] (90%); stub 1 answered from [0, 0, 0, 0, 1] (90%); stub 2 answered from [0, 0, 0, 0, 1] (90%); stub 4 answers 30 values in call order, printed after the answers; stub 5 answers 0..127 every row; stub 4 answers 0..65534 every row; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 writes 6 (4 bytes) at `m+0x9c` each call; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_find_gather_tiles.py") else {
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
            {
                // Windows: m the Build, c0 `GameAccess::world` (stride at +0x18), p0 the type (sizes at
                // +0x234/+0x238), p1 the MiningList's tcoords (x, y words), p2 the world's u16 tile masks.
                // Answers: a[0] `is_gather`, a[1] `is_flat`, a[2] the university test, a[3] the
                // `find_gather_tcoords` stub (it appends the list's tail: it writes the count 6 and the
                // tiles are already in p1), a[4] unused, a[5] `max_gatherers`, a[6..] the shuffle's
                // `Random::get(0, 0xffff)` answers in call order.
                // The crate's own `extend_and_shuffle_gather_list` runs the append and the shuffle
                // (`Sim::find_gather_tiles` calls it); the gate, the `0x1000` marking and `gather_max`
                // are the sweep's, since the crate method needs a whole `Sim` world for `gather_tcoords`.
                use crate::Pos;
                use crate::gather::extend_and_shuffle_gather_list;
                use crate::place::tile_corner_of;
                const K: i32 = 0x63637;
                let mut toks = ["-".to_string(), "-".to_string()];
                if a[0] != 0 && a[1] == 0 && a[2] == 0 {
                    let before = rd(m, 0x9c) as usize;
                    let tile = |i: usize| Pos::new(rd(&p[1], 8 * i), rd(&p[1], 8 * i + 4));
                    let mut list: Vec<Pos> = (0..before).map(tile).collect();
                    let found: Vec<Pos> = (before..6).map(tile).collect();
                    let (x, y) = (rd(m, 0x10) ^ K, rd(m, 0x14) ^ K);
                    let who = i32::from(m[9]);
                    toks[0] = format!("p0+0x0,{x},{y},m+0x98,{who}");
                    let mut next = 6;
                    extend_and_shuffle_gather_list(&mut list, found, |n| {
                        if n <= 1 {
                            return 0;
                        }
                        next += 1;
                        a[next - 1] % n
                    });
                    let stride = rd(&c[0], 0x18);
                    for (i, t) in list.iter().enumerate() {
                        wr(&mut p[1], 8 * i, t.x);
                        wr(&mut p[1], 8 * i + 4, t.y);
                        let o = 2 * (t.y * stride + t.x) as usize;
                        let v = u16::from_le_bytes([p[2][o], p[2][o + 1]]) | 0x1000;
                        p[2][o..o + 2].copy_from_slice(&v.to_le_bytes());
                    }
                    wr(m, 0x9c, list.len() as i32);
                    let (xs, ys) = (rd(&p[0], 0x234), rd(&p[0], 0x238));
                    let corner = tile_corner_of(xs, ys, Pos::new(x, y));
                    let ty = i32::from(i16::from_le_bytes([m[0xa], m[0xb]]));
                    toks[1] = format!("p0+0x0,{ty},{who},{},{}", corner.x, corner.y);
                    m[0x80] = a[5] as u8;
                }
                log.extend(toks);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "find_gather_tiles — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_gather_tiles — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_gather_tiles — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "find_gather_tiles — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Build::find_gather_tiles@00623350: {n} rows agree");
}
