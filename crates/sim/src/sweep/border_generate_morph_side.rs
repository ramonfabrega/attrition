//! `Border::generate_morph_side@00880050` under the emulator against the port (item 2818; tools/sweep_gen.py, obj with windows).

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

/// `Border::generate_morph_side@00880050`, run by `tools/emu/sweep_border_generate_morph_side.py` under unicorn
/// (tools/emu): 8 random bytes of object, 1 argument(s), argument 0 names 16 random bytes; callee 0x004a1050 (0 args) stubbed; callee 0x00881b30 (2 args) stubbed; callee 0x004a1000 (0 args) stubbed; stub 0 answers the address of 32 random bytes, a `p` window after the places; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_border_generate_morph_side.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // A strip is popped (stub 0), takes the side's `who_bits` byte, and is filled by walk_strip (stub 1,
                // `ecx` the borders object + 0x164, handed the side and the strip's vertex list at +0xc); fewer than
                // two vertices and the strip is pushed back (stub 2) and the answer is null.
                p[1][9] = p[0][0xa];
                log.push("m+0x0".to_string());
                log.push("12659308,p0+0x0,p1+0xc".to_string());
                if rd(&p[1], 0x10) >= 2 {
                    log.push("-".to_string());
                    i64::from(a[1])
                } else {
                    log.push("p1+0x0".to_string());
                    0
                }
            }
        };
        assert_eq!(got, want_eax, "generate_morph_side — eax, row `{line}`");
        assert_eq!(log, want_log, "generate_morph_side — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "generate_morph_side — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "generate_morph_side — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "generate_morph_side — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Border::generate_morph_side@00880050: {n} rows agree");
}
