//! `UnitData::transport_type@0046f790` under the emulator against the port (item 2281; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::transport_type@0046f790`, run by `tools/emu/sweep_obj.py` on its row `unit_data_transport_type` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 0 argument(s), vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `m+0x18` points at 8 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 1 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_transport_type") else {
        return;
    };
    let obj = super::Obj {
        name: "transport_type",
        full: "UnitData::transport_type@0046f790",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // `is(0x45, 0)` (scout) answers 1; a citizen kind (0x32..0x35), `is(0x3b,
            // 0)`, or the kinds 0x3d, 0x3e, 0x190 answer 3 (civilian); else 2. With
            // slot +0xb8 laid as `ObjectData::is` the type's slot +0x60 is asked
            // the same pushed pair (item 2281).
            let base = a[6] != 0;
            let ty = rd(&p[0], 4);
            let (mut own, mut typ) = (Vec::new(), Vec::new());
            let mut ask = |arg: i32, n: usize| -> i32 {
                if base {
                    typ.push(format!("p0+0x0,{arg},0"));
                    a[4 + n]
                } else {
                    own.push(format!("m+0x0,{arg},0"));
                    a[2 + n]
                }
            };
            let r = if ask(0x45, 0) != 0 {
                1
            } else if (0x32..=0x35).contains(&ty)
                || ask(0x3b, 1) != 0
                || matches!(ty, 0x3d | 0x3e | 0x190)
            {
                3
            } else {
                2
            };
            for calls in [own, typ] {
                log.push(if calls.is_empty() {
                    "-".into()
                } else {
                    calls.join("/")
                });
            }
            r
        }
    });
}
