//! `Armies::emergency@006f3250` under the emulator against the port (item 2852; tools/sweep_gen.py, obj with windows).

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

/// `Armies::emergency@006f3250`, run by `tools/emu/sweep_armies_emergency.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), callee 0x006f93d0 (1 args) stubbed; the word at `p3+0` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 16 random bytes less `0` (placed); the word at `p6+0` points at 64 random bytes less `0` (placed); the word at `p6+4` points at 64 random bytes less `0` (placed); the word at `p6+8` points at 64 random bytes less `0` (placed); the word at `p6+12` points at 64 random bytes less `0` (placed); the word at `p7+0` points at 64 random bytes less `0` (placed); the word at `p7+4` points at 64 random bytes less `0` (placed); the word at `p7+8` points at 64 random bytes less `0` (placed); the word at `p7+12` points at 64 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x8 (`p` windows after the pointed buffers, before the places); 2 rows 0x1c apart at `@0xc09700`, slices +0x4:0x4, +0x10:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_armies_emergency.py") else {
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
            // Owner `a[0]`'s leader (`p[a[0]]`, two flag words) is live (bit 0), not a type with `& 0xc == 4`, and has none
            // of `& 0xa` in the second word: each of its armies (the count `p[2 + 2·who]`, the army records
            // `p[8 + 4·who + i]`) that is occupied (the short at +0 not zero) has its words at +0x30 and +0x34
            // set to -1 and `Army::process` told 1 (item 2852).
            let who = a[0] as usize;
            let (f0, f1) = (rd(&p[who], 0), rd(&p[who], 4));
            if f0 & 1 != 0 && f0 & 0xc != 4 && f1 & 0xa == 0 {
                for i in 0..rd(&p[2 + 2 * who], 0).max(0) as usize {
                    let k = 8 + 4 * who + i;
                    if p[k][0] != 0 || p[k][1] != 0 {
                        wr(&mut p[k], 0x30, -1);
                        wr(&mut p[k], 0x34, -1);
                        log.push(format!("s0:p{k}+0x0,1"));
                    }
                }
            }
            if log.is_empty() {
                log.push("-".into());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "emergency — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "emergency — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "emergency — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "emergency — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Armies::emergency@006f3250: {n} rows agree");
}
