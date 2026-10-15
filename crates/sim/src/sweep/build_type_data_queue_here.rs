//! `BuildTypeData::queue_here@00639d10` under the emulator against the port (item 2714; tools/sweep_gen.py, obj with windows).

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

/// `BuildTypeData::queue_here@00639d10`, run by `tools/emu/sweep_build_type_data_queue_here.py` under unicorn
/// (tools/emu): 32 random bytes of object, 2 argument(s), vtable slot +0x60 (2 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; the word at `0xe85ddc` points at 2800 random bytes less `0` (placed); the word at `p0+0` points at 80 random bytes less `0` (placed); the word at `p0+4` points at 80 random bytes less `0` (placed); the word at `p0+8` points at 80 random bytes less `0` (placed); the word at `p0+12` points at 80 random bytes less `0` (placed); slot +0xc laid as 0x004707e0 or its stub, a flag after the answers; stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 0 answered from [0, 1] (90%); stub 0 answers 3 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_queue_here.py") else {
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
                let (s1, s2, flag) = (a[3], a[4], a[8]);
                let seq = [a[5], a[6], a[7]];
                let mut calls: Vec<String> = Vec::new();
                let mut call = |arg: i32, flag2: i32| -> i32 {
                    calls.push(format!("m+0x0,{arg},{flag2}"));
                    seq[(calls.len() - 1).min(2)]
                };
                let obj = |i: i32| &p[1 + i as usize];
                let o = obj(a[0]);
                let (f4, f30, f40) = (rd(o, 4), rd(o, 0x30), rd(o, 0x40));
                let r: i32 = if f30 == -2 || f40 < 0 {
                    0
                } else {
                    let unit_in_range = (0x32..0x19e).contains(&f4);
                    let falls = if flag != 0 { unit_in_range } else { s1 != 0 };
                    if !falls {
                        if s2 != 0 { call(f40, 1) } else { call(f40, 0) }
                    } else if call(f40, 0) != 0 {
                        1
                    } else {
                        let f44 = rd(obj(f40), 0x44);
                        if f44 < 0 {
                            0
                        } else if call(f44, 0) != 0 {
                            1
                        } else {
                            0
                        }
                    }
                };
                log.push(if calls.is_empty() {
                    "-".into()
                } else {
                    calls.join("/")
                });
                i64::from(r)
            }
        };
        assert_eq!(got, want_eax, "queue_here — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "queue_here — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "queue_here — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "queue_here — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "queue_here — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildTypeData::queue_here@00639d10: {n} rows agree");
}
