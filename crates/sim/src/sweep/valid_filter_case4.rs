//! `Search::valid_filter@0067dbb0` under the emulator against the port (item 2801; tools/sweep_gen.py, obj with windows).

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

/// `Search::valid_filter@0067dbb0`, run by `tools/emu/sweep_valid_filter_case4.py` under unicorn
/// (tools/emu): 16 random bytes of object, 4 argument(s), callee 0x0046ffa0 (0 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed); the word at `p0+12` points at 16 random bytes less `0` (placed); the word at `0xc0ab84+0x1c` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 16 random bytes less `0` (placed); the word at `p5+4` points at 16 random bytes less `0` (placed); the word at `p5+8` points at 16 random bytes less `0` (placed); the word at `p5+12` points at 16 random bytes less `0` (placed); the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 144 random bytes less `0` (placed); the word at `p10+4` points at 144 random bytes less `0` (placed); the word at `p10+8` points at 144 random bytes less `0` (placed); the word at `p10+12` points at 144 random bytes less `0` (placed); the word at `0xc0aec0+0x1c` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 144 random bytes less `0` (placed); the word at `p15+4` points at 144 random bytes less `0` (placed); the word at `p15+8` points at 144 random bytes less `0` (placed); the word at `p15+12` points at 144 random bytes less `0` (placed); the word at `p11+0x18` points at 544 random bytes less `0` (placed); the word at `p12+0x18` points at 544 random bytes less `0` (placed); the word at `p13+0x18` points at 544 random bytes less `0` (placed); the word at `p14+0x18` points at 544 random bytes less `0` (placed); the word at `p16+0x18` points at 544 random bytes less `0` (placed); the word at `p17+0x18` points at 544 random bytes less `0` (placed); the word at `p18+0x18` points at 544 random bytes less `0` (placed); the word at `p19+0x18` points at 544 random bytes less `0` (placed); stub 0 answered from [0, 1, 1, 1] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `ecx` an integer in 0..1, not the object, the row's last; `ecx` and `edx` held to their draw every row, no wild word; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_valid_filter_case4.py") else {
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
            {
                // Arm 4: 0 for the same slot of the same player; 0 when `0x46ffa0` (stub 0: first on
                // `objects[p2][p1]`, then on `objects[who][p0]`) answers zero; 0 when the two type-table objects
                // (+0x80 word) share a word other than -1; else whether their type records (+0x18) agree at +0x218.
                let who = a[7] as usize;
                let (i0, i1, i2) = (a[0] as usize, a[1] as usize, a[2] as usize);
                if a[0] == a[1] && who == i2 {
                    log.push("-".into());
                    0
                } else {
                    let w1 = 5 * i2 + 1 + i1;
                    let w0 = 5 * who + 1 + i0;
                    if a[5] == 0 {
                        log.push(format!("p{w1}+0x0"));
                        0
                    } else if a[6] == 0 {
                        log.push(format!("p{w1}+0x0/p{w0}+0x0"));
                        0
                    } else {
                        log.push(format!("p{w1}+0x0/p{w0}+0x0"));
                        let t2 = |w: usize, i: usize| &p[11 + 5 * w + i];
                        let ty = |w: usize, i: usize| &p[20 + 4 * w + i];
                        let word = |o: &[u8]| u16::from_le_bytes([o[0x80], o[0x81]]);
                        let (x, y) = (t2(i2, i1), t2(who, i0));
                        if word(x) == word(y) && word(y) != 0xffff {
                            0
                        } else {
                            i64::from(rd(ty(i2, i1), 0x218) == rd(ty(who, i0), 0x218))
                        }
                    }
                }
            }
        };
        assert_eq!(got, want_eax, "valid_filter — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "valid_filter — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "valid_filter — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "valid_filter — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "valid_filter — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Search::valid_filter@0067dbb0: {n} rows agree");
}
