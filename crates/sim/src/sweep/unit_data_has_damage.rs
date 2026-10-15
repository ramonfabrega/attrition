//! `UnitData::has_damage@00609b50` under the emulator against the port (item 2709; tools/sweep_gen.py, obj with windows).

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

/// `UnitData::has_damage@00609b50`, run by `tools/emu/sweep_unit_data_has_damage.py` under unicorn
/// (tools/emu): 152 random bytes of object, 1 argument(s), vtable slot +0xe8 (0 args) stubbed; the word at `0xc0aec0+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 160 random bytes less `0` (placed); the word at `p0+4` points at 160 random bytes less `0` (placed); the word at `p0+8` points at 160 random bytes less `0` (placed); the word at `p0+12` points at 160 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 160 random bytes less `0` (placed); the word at `p5+4` points at 160 random bytes less `0` (placed); the word at `p5+8` points at 160 random bytes less `0` (placed); the word at `p5+12` points at 160 random bytes less `0` (placed); slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; stub 0 answers 1..5 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; figures p1..p4 link at +0x8e (2 bytes) below their own index or to -1, -2; figures p1..p4 link at +0x90 (2 bytes) above their own index or to -1; figures p6..p9 link at +0x8e (2 bytes) below their own index or to -1, -2; figures p6..p9 link at +0x90 (2 bytes) above their own index or to -1; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_has_damage.py") else {
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
    let _ = &c;
    let r16 = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
    // Slot +0xe8 the base `UnitData::is_captain` (the flag after the answer) reads `o_up < 0` inline;
    // else the stub (answering 1..5) is asked, `ecx` the unit it stands on.
    let based = a[2] != 0;
    let mut calls: Vec<String> = Vec::new();
    let mut cur: Option<usize> = None;
    let mut param = a[0];
    let r = loop {
        let w: &[u8] = match cur {
            None => m,
            Some(k) => &p[k],
        };
        let who = usize::from(w[9]);
        if param == 0 {
            let up = r16(w, 0x8e);
            let cap = if based {
                i32::from(up < 0)
            } else {
                calls.push(match cur {
                    None => "m+0x0".to_string(),
                    Some(k) => format!("p{k}+0x0"),
                });
                a[1]
            };
            if cap == 0 {
                cur = Some(5 * who + 1 + usize::try_from(up).expect("a column"));
                continue;
            }
        }
        if w[8] & 1 == 0 || rd(w, 0x24) != 0 || w[0x3b] != 0 {
            break 1;
        }
        let down = r16(w, 0x90);
        if down < 0 {
            break 0;
        }
        param = 1;
        cur = Some(5 * who + 1 + usize::try_from(down).expect("a column"));
    };
    log.push(if calls.is_empty() { "-".to_string() } else { calls.join("/") });
    r
}
        };
        assert_eq!(got, want_eax, "has_damage — eax, row `{line}`");
        assert_eq!(log, want_log, "has_damage — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "has_damage — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "has_damage — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "has_damage — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::has_damage@00609b50: {n} rows agree");
}
