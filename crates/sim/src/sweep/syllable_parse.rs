//! `Syllable::parse@00935930` under the emulator against the port (item 2946; tools/sweep_gen.py, obj with windows).

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

/// `Syllable::parse@00935930`, run by `tools/emu/sweep_syllable_parse.py` under unicorn
/// (tools/emu): 8 random bytes of object, 1 argument(s), argument 0 names 16 random bytes; callee 0x00a15fc0 (1 args) stubbed; the word at `p0+0` points at 16 random bytes less `0` (placed, null a third of the rows); stub 0 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_syllable_parse.py") else {
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
            // Syllable::parse: a String record (+0 data, +6 offset, +8 length, +0xa flags) is handed to the integer converter
            // as a pointer: a wide string (flag 1) at data + 2 * offset, else, with a non-null data, at the pointer data
            // points to + 2 * offset, else null; the length is the converter's edx. The answer is the converter's.
            let data = u32::from_le_bytes(p[0][0..4].try_into().expect("four bytes"));
            let off = u16::from_le_bytes([p[0][6], p[0][7]]) as u32;
            let len = u16::from_le_bytes([p[0][8], p[0][9]]);
            let v = if p[0][10] & 1 != 0 {
                data.wrapping_add(2 * off)
            } else if data != 0 {
                (rd(&p[1], 0) as u32).wrapping_add(2 * off)
            } else {
                0
            };
            let name = if p[0][10] & 1 != 0 && data != 0 {
                format!("p1+{:#x}", 2 * off)
            } else {
                (v as i32).to_string()
            };
            log.push(format!("{name},{len},{name}"));
            a[2] as i64
        };
        assert_eq!(got, want_eax, "parse — eax, row `{line}`");
        assert_eq!(log, want_log, "parse — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "parse — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "parse — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "parse — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Syllable::parse@00935930: {n} rows agree");
}
