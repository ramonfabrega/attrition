//! `ObjectData::num_aircraft_limit@006454a0` under the emulator against the port (item 2281; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ObjectData::num_aircraft_limit@006454a0`, run by `tools/emu/sweep_obj.py` on its row `object_data_num_aircraft_limit` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 0 argument(s), the global cell 0x00c061e4 points at 176 random bytes; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `m+0x18` points at 8 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 0 answers 3 values in call order, printed after the answers; stub 1 answers 3 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("object_data_num_aircraft_limit") else {
        return;
    };
    let obj = super::Obj {
        name: "num_aircraft_limit",
        full: "ObjectData::num_aircraft_limit@006454a0",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // A carrier (0x15f) and an airbase (0x1bf) answer their constants; else
            // whether it is 0x208. With slot +0xb8 laid as `ObjectData::is` the
            // type's slot +0x60 is asked the same pushed pair (item 2281).
            let base = a[8] != 0;
            let (mut own, mut typ) = (Vec::new(), Vec::new());
            let mut ask = |arg: i32, n: usize| -> i32 {
                if base {
                    typ.push(format!("p0+0x0,{arg},1"));
                    a[5 + n]
                } else {
                    own.push(format!("m+0x0,{arg},1"));
                    a[2 + n]
                }
            };
            let r = if ask(0x15f, 0) != 0 {
                rd(&c[0], 0xa4)
            } else if ask(0x1bf, 1) != 0 {
                rd(&c[0], 0xa8)
            } else {
                i32::from(ask(0x208, 2) != 0)
            };
            for calls in [own, typ] {
                log.push(if calls.is_empty() {
                    "-".into()
                } else {
                    calls.join("/")
                });
            }
            i64::from(r)
        }
    });
}
