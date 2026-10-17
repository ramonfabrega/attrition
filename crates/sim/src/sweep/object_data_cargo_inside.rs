//! `ObjectData::cargo_inside@0064dcd0` under the emulator against the port (item 2910; tools/sweep_gen.py, obj with windows).

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

/// `ObjectData::cargo_inside@0064dcd0`, run by `tools/emu/sweep_object_data_cargo_inside.py` under unicorn
/// (tools/emu): 16 random bytes of object, 0 argument(s), vtable slot +0xbc (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `p0+8` points at 64 random bytes less `0` (placed); the word at `p0+12` points at 64 random bytes less `0` (placed); the word at `0xc0aba0` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 64 random bytes less `0` (placed); the word at `p5+4` points at 64 random bytes less `0` (placed); the word at `p5+8` points at 64 random bytes less `0` (placed); the word at `p5+12` points at 64 random bytes less `0` (placed); the word at `0xc0abbc` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 64 random bytes less `0` (placed); the word at `p10+4` points at 64 random bytes less `0` (placed); the word at `p10+8` points at 64 random bytes less `0` (placed); the word at `p10+12` points at 64 random bytes less `0` (placed); the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 144 random bytes less `0` (placed); the word at `p15+4` points at 144 random bytes less `0` (placed); the word at `p15+8` points at 144 random bytes less `0` (placed); the word at `p15+12` points at 144 random bytes less `0` (placed); the word at `0xc0aedc` points at 16 random bytes less `0` (placed); the word at `p20+0` points at 144 random bytes less `0` (placed); the word at `p20+4` points at 144 random bytes less `0` (placed); the word at `p20+8` points at 144 random bytes less `0` (placed); the word at `p20+12` points at 144 random bytes less `0` (placed); the word at `0xc0aef8` points at 16 random bytes less `0` (placed); the word at `p25+0` points at 144 random bytes less `0` (placed); the word at `p25+4` points at 144 random bytes less `0` (placed); the word at `p25+8` points at 144 random bytes less `0` (placed); the word at `p25+12` points at 144 random bytes less `0` (placed); the word at `p16+0x18` points at 736 random bytes less `0` (placed); the word at `p17+0x18` points at 736 random bytes less `0` (placed); the word at `p18+0x18` points at 736 random bytes less `0` (placed); the word at `p19+0x18` points at 736 random bytes less `0` (placed); the word at `p21+0x18` points at 736 random bytes less `0` (placed); the word at `p22+0x18` points at 736 random bytes less `0` (placed); the word at `p23+0x18` points at 736 random bytes less `0` (placed); the word at `p24+0x18` points at 736 random bytes less `0` (placed); the word at `p26+0x18` points at 736 random bytes less `0` (placed); the word at `p27+0x18` points at 736 random bytes less `0` (placed); the word at `p28+0x18` points at 736 random bytes less `0` (placed); the word at `p29+0x18` points at 736 random bytes less `0` (placed); stub 0 answered from [0, 1, 1, 1, 5] (90%); stub 1 answered from [0, 1, 1] (90%); stub 1 answers 5 values in call order, printed after the answers; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_cargo_inside.py") else {
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
            let _ = &c;
            // Slot 0xbc (answer `a[0]`) says whether `this` carries cargo; the chain of links starting at the
            // object (kind byte +9, index short +0xa) is then followed: each step names the next object by the
            // current one's kind (signed byte +0x3e) and index (short +0x28), asks it slot 8 (the n-th of the
            // row's answers `a[2 + n]`) and, when that is true and the unit at the same place has a negative
            // word at +0x8e, adds the word at +0x2d8 of the unit's type record (+0x18); the chain ends when a
            // link is negative. Windows: objects p[1 + 5k + j], units p[16 + 5k + j], their records p[30 + 4k + j].
            let sh = |b: &[u8], o: usize| i32::from(i16::from_le_bytes([b[o], b[o + 1]]));
            let mut calls: Vec<String> = Vec::new();
            let mut sum = 0i32;
            if a[0] != 0 {
                let (mut k, mut j) = (m[9] as usize, sh(m, 10) as usize);
                if sh(&p[1 + 5 * k + j], 0x28) >= 0 {
                    loop {
                        let o = &p[1 + 5 * k + j];
                        let (nk, nj) = (o[0x3e] as i8 as usize, sh(o, 0x28) as usize);
                        calls.push(format!("p{}+0x0", 1 + 5 * nk + nj));
                        if a[2 + calls.len() - 1] != 0 && sh(&p[16 + 5 * nk + nj], 0x8e) < 0 {
                            sum = sum.wrapping_add(rd(&p[30 + 4 * nk + nj], 0x2d8));
                        }
                        (k, j) = (nk, nj);
                        if sh(&p[1 + 5 * k + j], 0x28) < 0 {
                            break;
                        }
                    }
                }
            }
            log.push(if calls.is_empty() {
                "-".into()
            } else {
                calls.join("/")
            });
            i64::from(sum)
        };
        assert_eq!(got, want_eax, "cargo_inside — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "cargo_inside — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "cargo_inside — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "cargo_inside — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "cargo_inside — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectData::cargo_inside@0064dcd0: {n} rows agree");
}
