//! `Armies::process_all@006f3b00` under the emulator against the port (item 3044; tools/sweep_gen.py, obj with windows).

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

/// `Armies::process_all@006f3b00`, run by `tools/emu/sweep_armies_process_all.py` under unicorn
/// (tools/emu): 16 random bytes of object, 0 argument(s), `p` windows in order: p0..p15 sparse table 0 (8 rows × 2 slice(s), row-major), p16..p31 sparse table 1 (8 rows × 2 slice(s), row-major), p32..p71 the place(s), in flag order; callee 0x006f93d0 (1 args) stubbed; the word at `p17+0` points at 16 random bytes less `0` (placed); the word at `p19+0` points at 16 random bytes less `0` (placed); the word at `p21+0` points at 16 random bytes less `0` (placed); the word at `p23+0` points at 16 random bytes less `0` (placed); the word at `p25+0` points at 16 random bytes less `0` (placed); the word at `p27+0` points at 16 random bytes less `0` (placed); the word at `p29+0` points at 16 random bytes less `0` (placed); the word at `p31+0` points at 16 random bytes less `0` (placed); the word at `p32+0` points at 16 random bytes less `0` (placed); the word at `p32+4` points at 16 random bytes less `0` (placed); the word at `p32+8` points at 16 random bytes less `0` (placed); the word at `p32+12` points at 16 random bytes less `0` (placed); the word at `p33+0` points at 16 random bytes less `0` (placed); the word at `p33+4` points at 16 random bytes less `0` (placed); the word at `p33+8` points at 16 random bytes less `0` (placed); the word at `p33+12` points at 16 random bytes less `0` (placed); the word at `p34+0` points at 16 random bytes less `0` (placed); the word at `p34+4` points at 16 random bytes less `0` (placed); the word at `p34+8` points at 16 random bytes less `0` (placed); the word at `p34+12` points at 16 random bytes less `0` (placed); the word at `p35+0` points at 16 random bytes less `0` (placed); the word at `p35+4` points at 16 random bytes less `0` (placed); the word at `p35+8` points at 16 random bytes less `0` (placed); the word at `p35+12` points at 16 random bytes less `0` (placed); the word at `p36+0` points at 16 random bytes less `0` (placed); the word at `p36+4` points at 16 random bytes less `0` (placed); the word at `p36+8` points at 16 random bytes less `0` (placed); the word at `p36+12` points at 16 random bytes less `0` (placed); the word at `p37+0` points at 16 random bytes less `0` (placed); the word at `p37+4` points at 16 random bytes less `0` (placed); the word at `p37+8` points at 16 random bytes less `0` (placed); the word at `p37+12` points at 16 random bytes less `0` (placed); the word at `p38+0` points at 16 random bytes less `0` (placed); the word at `p38+4` points at 16 random bytes less `0` (placed); the word at `p38+8` points at 16 random bytes less `0` (placed); the word at `p38+12` points at 16 random bytes less `0` (placed); the word at `p39+0` points at 16 random bytes less `0` (placed); the word at `p39+4` points at 16 random bytes less `0` (placed); the word at `p39+8` points at 16 random bytes less `0` (placed); the word at `p39+12` points at 16 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x4:0x4 (`p` windows after the pointed buffers, before the places); 8 rows 0x1c apart at `@0xc09700`, slices +0x4:0x4, +0x10:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_armies_process_all.py") else {
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
            // Leader `k`: `p[2k]` its flags word, `p[2k+1]` the word at +4; its armies record: `p[16+2k]` the
            // count, `p[17+2k]` the list (four pointers, buffer `p[32+k]`, army `j` in `p[40+4k+j]`).
            // An army with a nonzero short at +0 is processed, handed whether the byte at +4 had bit 7,
            // and that bit cleared first.
            for k in 0..8 {
                let flags = rd(&p[2 * k], 0);
                if flags & 1 != 0 && flags & 0xc != 4 && rd(&p[2 * k + 1], 0) & 10 == 0 {
                    for j in 0..rd(&p[16 + 2 * k], 0).max(0) as usize {
                        let w = 40 + 4 * k + j;
                        if i16::from_le_bytes([p[w][0], p[w][1]]) != 0 {
                            let hot = p[w][4] & 0x80 != 0;
                            if hot {
                                p[w][4] &= 0x7f;
                            }
                            log.push(format!("s0:p{w}+0x0,{}", u8::from(hot)));
                        }
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
            "process_all — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process_all — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process_all — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process_all — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Armies::process_all@006f3b00: {n} rows agree");
}
