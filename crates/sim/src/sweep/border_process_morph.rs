//! `Border::process_morph@0087fd00` under the emulator against the port (item 2852; tools/sweep_gen.py, obj with windows).

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

/// `Border::process_morph@0087fd00`, run by `tools/emu/sweep_border_process_morph.py` under unicorn
/// (tools/emu): 208 random bytes of object, 0 argument(s), callee 0x00880e50 (0 args) stubbed; callee 0x004a12a0 (0 args) stubbed; the word at `m+0x4c` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 80 random bytes less `0` (placed); the word at `p0+4` points at 80 random bytes less `0` (placed); the word at `p0+8` points at 80 random bytes less `0` (placed); the word at `p0+12` points at 80 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 0 answers 4 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_border_process_morph.py") else {
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
            // The morph renders `p[0]` (a pointer word each, `m+0x40` of them): each is advanced (the answers `a[2..]` in call
            // order, 0 = still running); one that finished raises the border's rebuild flag (`m+0xc8` bit 0) when its
            // record (the window it was laid as, `p[1 + origin]`) has bit 2 at +0x4c, is pushed back to its recycler,
            // and is replaced in the array by the last pointer, which is cleared, the count falling by one; the
            // slot is not looked at again (item 2852).
            let origin: Vec<usize> = (0..4).collect();
            let mut next = 2;
            let mut i = 0;
            while (i as i32) < rd(m, 0x40) {
                let w = origin[i];
                next += 1;
                if a[next - 1] != 0 {
                    if p[1 + w][0x4c] & 4 != 0 {
                        m[0xc8] |= 1;
                    }
                    log.push(format!("s0:p{}+0x0", 1 + w));
                    log.push(format!("s1:p{}+0x0", 1 + w));
                    let n = rd(m, 0x40) as usize;
                    let last = rd(&p[0], 4 * (n - 1));
                    wr(&mut p[0], 4 * i, last);
                    wr(&mut p[0], 4 * (n - 1), 0);
                    wr(m, 0x40, n as i32 - 1);
                } else {
                    log.push(format!("s0:p{}+0x0", 1 + w));
                }
                i += 1;
            }
            if log.is_empty() {
                log.push("-".into());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "process_morph — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process_morph — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "process_morph — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "process_morph — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Border::process_morph@0087fd00: {n} rows agree");
}
