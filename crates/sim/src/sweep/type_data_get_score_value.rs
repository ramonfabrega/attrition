//! `TypeData::get_score_value@006673b0` under the emulator against the port (item 3024; tools/sweep_gen.py, obj with windows).

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

/// `TypeData::get_score_value@006673b0`, run by `tools/emu/sweep_type_data_get_score_value.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0..p27 the place(s), in flag order; the global cell 0x00e85ddc points at 2816 random bytes; the global cell 0x00c0a274 points at 2816 random bytes; the global cell 0x00c061e4 points at 944 random bytes; vtable slot +0xc (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0x28 (0 args) stubbed; vtable slot +0x48 (0 args) stubbed; the word at `c0+4*0x31` points at 48 random bytes less `0` (placed); the word at `c0+4*0x32` points at 48 random bytes less `0` (placed); the word at `c0+4*0x42` points at 48 random bytes less `0` (placed); the word at `c0+4*0x43` points at 48 random bytes less `0` (placed); the word at `c0+4*0x44` points at 48 random bytes less `0` (placed); the word at `c0+4*0x19d` points at 48 random bytes less `0` (placed); the word at `c0+4*0x19e` points at 48 random bytes less `0` (placed); the word at `c0+4*0x21e` points at 48 random bytes less `0` (placed); the word at `c0+4*0x21f` points at 48 random bytes less `0` (placed); the word at `c0+4*0x220` points at 48 random bytes less `0` (placed); the word at `c0+4*0x274` points at 48 random bytes less `0` (placed); the word at `c0+4*0x275` points at 48 random bytes less `0` (placed); the word at `c0+4*0x2ab` points at 48 random bytes less `0` (placed); the word at `c0+4*0x2ac` points at 48 random bytes less `0` (placed); the word at `c1+4*0x31` points at 752 random bytes less `0` (placed); the word at `c1+4*0x32` points at 752 random bytes less `0` (placed); the word at `c1+4*0x42` points at 752 random bytes less `0` (placed); the word at `c1+4*0x43` points at 752 random bytes less `0` (placed); the word at `c1+4*0x44` points at 752 random bytes less `0` (placed); the word at `c1+4*0x19d` points at 752 random bytes less `0` (placed); the word at `c1+4*0x19e` points at 752 random bytes less `0` (placed); the word at `c1+4*0x21e` points at 752 random bytes less `0` (placed); the word at `c1+4*0x21f` points at 752 random bytes less `0` (placed); the word at `c1+4*0x220` points at 752 random bytes less `0` (placed); the word at `c1+4*0x274` points at 752 random bytes less `0` (placed); the word at `c1+4*0x275` points at 752 random bytes less `0` (placed); the word at `c1+4*0x2ab` points at 752 random bytes less `0` (placed); the word at `c1+4*0x2ac` points at 752 random bytes less `0` (placed); slot +0xc laid as 0x004707e0 or its stub, a flag after the answers; slot +0x14 laid as 0x004707c0 or its stub, a flag after the answers; slot +0x28 laid as 0x004706c0 or its stub, a flag after the answers; slot +0x48 laid as 0x00470590 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_data_get_score_value.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `this+4` the type, drawn from the 14 the places hold (p[k] the type record of the
            // k-th, p[14 + k] its unit-type record); `a[0]` the argument, `a[1..5]` the stubs'
            // answers, `a[5..9]` the flags (nonzero: the base `is_*` inlined on the type's range).
            const TYPES: [i32; 14] = [
                0x31, 0x32, 0x42, 0x43, 0x44, 0x19d, 0x19e, 0x21e, 0x21f, 0x220, 0x274, 0x275,
                0x2ab, 0x2ac,
            ];
            let at = |t: i32| TYPES.iter().position(|&x| x == t).expect("a placed type");
            let (param, t) = (a[0], rd(m, 4));
            let t3 = if param == 0 && matches!(t, 0x42..=0x44) {
                0x32
            } else {
                t
            };
            let rec = &p[at(t3)];
            let sum = (0x18..=0x2c)
                .step_by(4)
                .fold(0i32, |s, o| s.wrapping_add(rd(rec, o)));
            let k = |o: usize| rd(&c[2], o);
            let is = |flag: i32, ans: i32, lo: i32, hi: i32| {
                if flag != 0 {
                    (lo..hi).contains(&t)
                } else {
                    ans != 0
                }
            };
            let r = |x: i32| x.wrapping_add((x >> 31) & 0xff) >> 8;
            let v = if is(a[5], a[1], 0x32, 0x19e) {
                let cost = k(0x354).wrapping_mul(sum);
                if param == 0 {
                    cost
                } else {
                    let x = r(k(0x3a4).wrapping_mul(cost));
                    r(x.wrapping_mul(rd(&p[14 + at(t3)], 0x2e0)))
                }
            } else if is(a[6], a[2], 0x19e, 0x21f) {
                k(0x358).wrapping_mul(sum)
            } else if is(a[7], a[3], 0x220, 0x275) {
                k(0x35c).wrapping_mul(sum)
            } else if is(a[8], a[4], 0x275, 0x2ac) {
                k(0x360).wrapping_mul(sum)
            } else {
                k(0x35c).wrapping_mul(sum)
            };
            i64::from(v)
        };
        assert_eq!(got, want_eax, "get_score_value — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_score_value — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_score_value — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "get_score_value — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 500, "only {n} rows");
    eprintln!("TypeData::get_score_value@006673b0: {n} rows agree");
}
