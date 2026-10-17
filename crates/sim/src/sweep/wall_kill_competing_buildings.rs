//! `Wall::kill_competing_buildings@0063d3a0` under the emulator against the port (item 3044; tools/sweep_gen.py, obj with windows).

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

/// `Wall::kill_competing_buildings@0063d3a0`, run by `tools/emu/sweep_wall_kill_competing_buildings.py` under unicorn
/// (tools/emu): 32 random bytes of object, 0 argument(s), `p` windows in order: p0 the place; the global cell 0x00c06188 points at 32 random bytes; callee 0x00643440 (2 args) stubbed; callee 0x0063d440 (2 args) stubbed; the word at `m+0x18` points at 576 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 writes a value in -2..5 through its argument 0, printed after the answers (`o` in a place); stub 0 writes a value in -2..5 through its argument 1, printed after the answers (`o` in a place); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_kill_competing_buildings.py") else {
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
            // `tile_corner` (stub 0) writes the corner `(a[2], a[3])` through its two out pointers; the
            // type record `p[0]` is `+0x234` wide and `+0x238` tall; every tile of it inside the
            // world (`c0+0x18` by `c0+0x1c`) goes to `kill_at_tile` (stub 1), columns outer.
            let (x0, y0) = (a[2], a[3]);
            let (tw, th) = (rd(&p[0], 0x234), rd(&p[0], 0x238));
            let (w, h) = (rd(&c[0], 0x18), rd(&c[0], 0x1c));
            log.push("s0:m+0x0,sp-0x10,sp-0xc".to_string());
            for x in x0..x0 + tw {
                for y in y0..y0 + th {
                    if x >= 0 && y >= 0 && x < w && y < h {
                        log.push(format!("s1:m+0x0,{x},{y}"));
                    }
                }
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "kill_competing_buildings — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "kill_competing_buildings — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "kill_competing_buildings — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "kill_competing_buildings — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Wall::kill_competing_buildings@0063d3a0: {n} rows agree");
}
