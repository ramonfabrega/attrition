//! `TechType::add_preq_ai@0066c350` under the emulator against the port (item 2946; tools/sweep_gen.py, obj with windows).

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

struct Walk<'a> {
    count: i32,
    keep: bool,
    seq1: &'a [i32],
    seq2: &'a [i32],
    n1: usize,
    n2: usize,
}

/// `add_preq_ai`'s loop over `this`'s prerequisites: `vftable` is the base class (three of them), any other record is asked slot +0x4.
fn add_preq_ai(w: &mut Walk, p: &mut [Vec<u8>], vftable: bool, p1: i32, p2: i32, p3: i32) {
    let mut i = 0;
    loop {
        let n = if vftable { 3 } else { w.count };
        if n <= i {
            return;
        }
        let idx = w.seq1[w.n1.min(w.seq1.len() - 1)];
        w.n1 += 1;
        i += 1;
        if idx < 0 {
            continue;
        }
        let idx = idx as usize;
        let ty = rd(&p[1 + idx], 4);
        let add = if w.keep {
            if !(0x220..0x275).contains(&ty) {
                continue;
            }
            true
        } else {
            let v = w.seq2[w.n2.min(w.seq2.len() - 1)];
            w.n2 += 1;
            v != 0
        };
        if !add {
            continue;
        }
        let at = 11 + idx;
        let off = 0x1cc + 2 * p1 as usize;
        let v = i16::from_le_bytes([p[at][off], p[at][off + 1]]).wrapping_add(p2 as i16);
        p[at][off..off + 2].copy_from_slice(&v.to_le_bytes());
        if p3 != 0 {
            add_preq_ai(w, p, false, p1, p2, p3 - 1);
        }
    }
}

/// `TechType::add_preq_ai@0066c350`, run by `tools/emu/sweep_tech_type_add_preq_ai.py` under unicorn
/// (tools/emu): 64 random bytes of object, 3 argument(s), vtable slot +0x4 (0 args) stubbed; callee 0x00668700 (2 args) stubbed; vtable slot +0x28 (0 args) stubbed; the word at `0xe85ddc` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `p0+16` points at 32 random bytes less `0` (placed); the word at `p0+20` points at 32 random bytes less `0` (placed); the word at `p0+24` points at 32 random bytes less `0` (placed); the word at `p0+28` points at 32 random bytes less `0` (placed); the word at `0xc061b0` points at 64 random bytes less `0` (placed); the word at `p9+0x10` points at 32 random bytes less `0` (placed); the word at `p10+0` points at 512 random bytes less `0` (placed); the word at `p10+4` points at 512 random bytes less `0` (placed); the word at `p10+8` points at 512 random bytes less `0` (placed); the word at `p10+12` points at 512 random bytes less `0` (placed); the word at `p10+16` points at 512 random bytes less `0` (placed); the word at `p10+20` points at 512 random bytes less `0` (placed); the word at `p10+24` points at 512 random bytes less `0` (placed); the word at `p10+28` points at 512 random bytes less `0` (placed); slot +0x28 laid as 0x004706c0 or its stub, a flag after the answers; stub 2 answered from [0, 1, 1] (90%); the word at `m+0` laid as the image vtable at 0xb4456c (the stub slots copied there) half the rows; stub 1 answers 16 values in call order, printed after the answers; stub 2 answers 16 values in call order, printed after the answers; stub 0 answers 0..4 every row; stub 1 answers -1..7 every row; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_tech_type_add_preq_ai.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
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
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            let mut w = Walk {
                count: a[3],
                keep: a[38] != 0,
                seq1: &a[6..22],
                seq2: &a[22..38],
                n1: 0,
                n2: 0,
            };
            add_preq_ai(&mut w, p, rd(m, 0) == 0xb4456cu32 as i32, a[0], a[1], a[2]);
            0
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add_preq_ai — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "add_preq_ai — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "add_preq_ai — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TechType::add_preq_ai@0066c350: {n} rows agree");
}
