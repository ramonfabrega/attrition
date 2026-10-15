//! `Wall::start@0063e810` under the emulator against the port (item 2400; tools/sweep_gen.py, obj with windows).

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

/// `Wall::start@0063e810`, run by `tools/emu/sweep_wall_start.py` under unicorn
/// (tools/emu): 128 random bytes of object, 1 argument(s), the global cell 0x00c06188 points at 512 random bytes; the global cell 0x00c061ec points at 1536 random bytes; callee 0x0063d3a0 (0 args) stubbed; callee 0x00643440 (2 args) stubbed; callee 0x00850c40 (5 args) stubbed; callee 0x00642fc0 (2 args) stubbed; callee 0x0063ce70 (1 args) stubbed; callee 0x0063d230 (1 args) stubbed; the word at `m+0x18` points at 576 random bytes less `0` (placed); the word at `c0+0x134` points at 2048 random bytes less `0` (placed); the word at `c0+0x160` points at 1024 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 1 writes a value in -3..9 through its argument 0, printed after the answers (`o` in a place); stub 1 writes a value in -3..9 through its argument 1, printed after the answers (`o` in a place); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
/// `MiscAccess::terrain` (`[0xc06218]`), the image's own word: the `ecx` of `Terrain::object_placed`.
const TERRAIN: i32 = 12_628_208;

#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_start.py") else {
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
            // The calls, in the order `start` makes them (the sim's
            // `Sim::start_building` holds each in its own module; only the
            // footprint walk is a helper). `a[0]` is the unused stack
            // argument, `a[1..7]` the six stubs' answers, `a[7]`/`a[8]`
            // what `tile_corner` wrote through its two out-pointers.
            m[8] |= 2;
            let frame = rd(&c[1], 0x550);
            wr(m, 0x5c, frame);
            let (x, y) = (a[7], a[8]);
            let (xs, ys) = (rd(&p[0], 0x234), rd(&p[0], 0x238));
            log.push("s0:m+0x0".into());
            log.push("s1:m+0x0,sp-0x24,sp-0x20".into());
            log.push(format!("s2:{TERRAIN},{x},{y},{xs},{ys},1"));
            log.push("s3:m+0x0,1,3".into());
            let who = m[9];
            let bit = (1u32 << (who & 31)) as u8;
            let (w0, stride) = (rd(&c[0], 0), rd(&c[0], 0xc));
            let steps = crate::city::wall_start_seen_steps(
                (x, y),
                xs,
                ys,
                rd(&c[0], 0x18),
                rd(&c[0], 0x1c),
            );
            for (hx, hy) in steps {
                // `+0x14` of the cell's `WData` (28 bytes, `(y >> 2) * w0 +
                // (x >> 2)`) and the half-cell byte of `seen2`.
                let wd = ((hy >> 1) * w0 + (hx >> 1)) as usize * 0x1c + 0x14;
                p[1][wd] |= bit;
                let si = (stride * hy + hx) as usize;
                p[2][si] |= bit;
            }
            log.push("s4:m+0x0,0".into());
            log.push("s5:m+0x0,1".into());
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "start — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "start — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "start — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "start — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Wall::start@0063e810: {n} rows agree");
}
