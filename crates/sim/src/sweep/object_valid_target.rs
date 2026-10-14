//! `Object::valid_target@00648ba0` under the emulator against the port (item 2287; tools/sweep_gen.py, obj with windows).

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

/// `Object::valid_target@00648ba0`, run by `tools/emu/sweep_object_valid_target.py` under unicorn
/// (tools/emu): 32 random bytes of object, 3 argument(s), argument 2 names 4 random bytes (null a third of the rows); vtable slot +0x138 (2 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x0062d1d0 (0 args) stubbed; callee 0x006276a0 (2 args) stubbed; callee 0x0046ffa0 (0 args) stubbed; vtable slot +0x148 (1 args) stubbed; callee 0x00616e80 (0 args) stubbed; callee 0x0060b030 (0 args) stubbed; vtable slot +0xcc (0 args) stubbed; the word at `0xc0618c` points at 104 random bytes less `0` (placed); the word at `p1+0x14+0x1c*a[1]` points at 8 random bytes less `0` (placed); the word at `p2+4*a[0]` points at 16 random bytes less `0` (placed); the word at `m+0x18` points at 488 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*m[9]` points at 16 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [1, 1, 1, 0] (90%); stub 2 answered from [1, 1, 1, 0] (90%); stub 4 answered from [1, 1, 1, 0] (90%); stub 5 answered from [0, 1] (90%); stub 6 answered from [1, 1, 1, 0] (90%); stub 7 answered from [1, 1, 1, 0] (90%); stub 8 answered from [10, 10, 10, 3] (90%); stub 9 answers the address of 16 random bytes, a `p` window after the places; stub 10 answers the address of 32 random bytes, a `p` window after the places; stub 7 answers 2 values in call order, printed after the answers; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_valid_target.py") else {
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
            // `a[0]` `o`, `a[1]` `who`, `a[2]` the out-pointer (`p[0]`, or 0);
            // the stubs' answers from `a[3]`: `valid_target_const` 3, the
            // target's `+0x20` 4 and `+0x4c` 5, `check_capture_eligible` 7,
            // `check_capture` 8, `is_map_unit` 9, `order_type` 11; `has`
            // (`+0x148`) answers `a[14]` then `a[15]`. The target is `p[3]`
            // (`flags` `+8`), the asker's type `p[4]` (`+0x1e4`), the order's
            // record `p[7]` (`+8` o, `+0xc` who, `+0x1c` mandatory).
            let mut out = rd(&p[0], 0);
            let (p3, p4, p7) = (p[3].clone(), p[4].clone(), p[7].clone());
            let ok = crate::fight::valid_target_steps(
                &mut (),
                (a[2] != 0).then_some(&mut out),
                |_| a[3] != 0,
                |_| p3[8] & 1 != 0 && a[4] != 0 && a[5] != 0 && a[7] != 0,
                |_| rd(&p4, 0x1e4) & 0x800_0000 != 0,
                |_| a[8] != 0,
                |_| {
                    a[9] != 0
                        && a[14] != 0
                        && a[15] != 0
                        && a[11] == 10
                        && rd(&p7, 8) == a[0]
                        && rd(&p7, 0xc) == a[1]
                        && p7[0x1c] != 0
                },
            );
            wr(&mut p[0], 0, out);
            i64::from(ok)
        };
        assert_eq!(got, want_eax, "valid_target — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "valid_target — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "valid_target — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "valid_target — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Object::valid_target@00648ba0: {n} rows agree");
}
