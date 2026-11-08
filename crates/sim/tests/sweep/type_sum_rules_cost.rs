//! `Type::sum_rules_cost@006680e0` under the emulator against the port (item 2927; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Type::sum_rules_cost@006680e0`, run by `tools/emu/sweep_obj.py` on its row `type_sum_rules_cost` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 0 argument(s), the global cell 0x00c061f0 points at 1024 random bytes; vtable slot +0xc (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0x48 (0 args) stubbed; vtable slot +0x28 (0 args) stubbed; slot +0xc laid as 0x004707e0 or its stub, a flag after the answers; slot +0x14 laid as 0x004707c0 or its stub, a flag after the answers; slot +0x48 laid as 0x00470590 or its stub, a flag after the answers; slot +0x28 laid as 0x004706c0 or its stub, a flag after the answers; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("type_sum_rules_cost") else {
        return;
    };
    let obj = super::Obj {
        name: "sum_rules_cost",
        full: "Type::sum_rules_cost@006680e0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // The cost words +0x18..+0x2c summed; the first class test that holds picks the factor of the constants record.
            let sum = (0..6).fold(0i32, |s, k| s.wrapping_add(rd(m, 0x18 + 4 * k)));
            let t = rd(m, 4);
            let k = &c[0];
            let kf = |o: usize| i32::from_le_bytes(k[o..o + 4].try_into().expect("four bytes"));
            // a[0..4] the slots 0xc, 0x14, 0x48, 0x28 as the stub answers; a[4..8] the flags: laid as the class's own test.
            let class = |i: usize, lo: i32, hi: i32| {
                if a[4 + i] != 0 {
                    t > lo && t < hi
                } else {
                    a[i] != 0
                }
            };
            let f = if class(0, 0x31, 0x19e) {
                kf(0x354)
            } else if class(1, 0x19d, 0x21f) {
                kf(0x358)
            } else if class(2, 0x274, 0x2ac) {
                kf(0x360)
            } else {
                kf(0x35c)
            };
            f.wrapping_mul(sum) as i64
        }
    });
}
