//! `UnitData::is_corner@0060a040` under the emulator against the port (item 2621; tools/sweep_gen.py, obj with windows).

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

/// `UnitData::is_corner@0060a040`, run by `tools/emu/sweep_unit_data_is_corner.py` under unicorn
/// (tools/emu): 256 random bytes of object, 3 argument(s), vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x005de270 (2 args) stubbed; the word at `m+0xf4` points at 16 random bytes less `0` (placed); stub 0 answered from [0, 1, 1, 1, 1] (90%); stub 1 answered from [0, 1, 1, 1, 1] (90%); stub 2 answered from [0, 0, 0, 5] (90%); stub 2 answers 4 values in call order, the last from [0], printed after the answers; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_is_corner.py") else {
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
            let _ = (&c, &a[5]);
let (s0, s1, seq) = (a[3], a[4], &a[6..10]);
let mut r = 0;
let mut cl: Vec<String> = Vec::new();
if s0 != 0 && s1 != 0 {
    let n = i32::from(m[0xb5] as i8).max(0) as usize;
    let mut k = 0;
    for i in 0..n {
        let g = rd(&p[0], 4 * i);
        if g == 0 {
            continue;
        }
        let ans = seq[k.min(3)];
        k += 1;
        cl.push(format!("{g},{},{}", a[0], a[1]));
        if ans != 0 {
            r = ans;
            break;
        }
    }
}
log.push(if cl.is_empty() { "-".into() } else { cl.join("/") });
i64::from(r)
        };
        assert_eq!(got, want_eax, "is_corner — eax, row `{line}`");
        assert_eq!(log, want_log, "is_corner — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_corner — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_corner — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_corner — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::is_corner@0060a040: {n} rows agree");
}
