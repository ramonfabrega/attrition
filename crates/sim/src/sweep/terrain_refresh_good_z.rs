//! `Terrain::refresh_good_z@00850b60` under the emulator against the port (item 3044; tools/sweep_gen.py, obj with windows).

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

/// `Terrain::refresh_good_z@00850b60`, run by `tools/emu/sweep_terrain_refresh_good_z.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0..p7 the place(s), in flag order; 4 random bytes at 0x00c0a0e4; 4 random bytes at 0x00c0a104; 4 random bytes at 0x00c06218; callee 0x00866560 (4 args) stubbed; the word at `0xc0a0f0` points at 16 random bytes less `0` (placed); the word at `0xc0a110` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p1+0` points at 32 random bytes less `0` (placed); the word at `p1+4` points at 32 random bytes less `0` (placed); the word at `p1+8` points at 32 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 writes a value in 0..262143 through its argument 0, printed after the answers (`o` in a place); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_terrain_refresh_good_z.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The goods (`c0` the count, list `p[0]`, record `j` in `p[2+j]`) then the items (`c1`, `p[1]`,
            // `p[5+j]`): each live record (bit 0 of `+8`) is handed to the terrain's `find_data_z` (the
            // cell `c2`; `a[2]` the z it writes through its out pointer) with its position words
            // `+0x10`, `+0x14` unmasked (^ 0x63637), and its `+0xc` becomes the answer masked again.
            let terrain = rd(&c[2], 0);
            for (count, first) in [(rd(&c[0], 0), 2usize), (rd(&c[1], 0), 5)] {
                for j in 0..count.max(0) as usize {
                    let r = &mut p[first + j];
                    if r[8] & 1 != 0 {
                        let x = rd(r, 0x10) ^ 0x63637;
                        let y = rd(r, 0x14) ^ 0x63637;
                        log.push(format!("s0:{terrain},sp-0x10,{x},{y},1"));
                        wr(r, 0xc, a[2] ^ 0x63637);
                    }
                }
            }
            if log.is_empty() {
                log.push("-".to_string());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "refresh_good_z — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "refresh_good_z — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "refresh_good_z — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "refresh_good_z — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Terrain::refresh_good_z@00850b60: {n} rows agree");
}
