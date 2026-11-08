//! `UnitData::los@006100c0` under the emulator against the port (item 2774; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::los@006100c0`, run by `tools/emu/sweep_obj.py` on its row `unit_data_los` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 96 random bytes of object, 0 argument(s), vtable slot +0x10c (0 args) stubbed; callee 0x00646b00 (2 args) stubbed; the word at `0xc061e0` points at 64 random bytes less `0x59c0` (placed); the word at `0xc061e4` points at 336 random bytes less `0xb60` (placed); the word at `m+0x18` points at 720 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); the stubbed vtable laid at `p2+0`; stub 1 answers -1..1 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_los") else {
        return;
    };
    let obj = super::Obj {
        name: "los",
        full: "UnitData::los@006100c0",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `UnitData::los`: the unit's own sight (`+0x3c`, a signed byte) plus two bonuses. For a player
        // (`+9`, held to 0; `p[0]`) with the short at `+0x59d2` set, and a type (`p[2]`) whose `+0x2c8`
        // has `0x420`, a general `0x16a` (`has_general`, `0x646b00`; negative = none) adds the
        // constant at `+0xb64` (`p[1]`). With the short at `+0x59c8` set and the type's slot `0x10c`
        // answering 0, a general `0x165` adds the constant at `+0xca8`. The log holds each
        // `has_general`'s arguments (`0`, then the general).
        let (s0, s1) = (a[0], a[1]);
        let mut v = i32::from(m[0x3c] as i8);
        let mut calls: Vec<String> = Vec::new();
        if i16::from_le_bytes([p[0][0x12], p[0][0x13]]) != 0 && rd(&p[2], 0x2c8) & 0x420 != 0 {
            calls.push("m+0x0,0,362".to_string());
            if s1 >= 0 {
                v = v.wrapping_add(rd(&p[1], 4));
            }
        }
        if i16::from_le_bytes([p[0][8], p[0][9]]) != 0 && s0 == 0 {
            calls.push(format!("m+0x0,{s0},357"));
            if s1 >= 0 {
                v = v.wrapping_add(rd(&p[1], 0x148));
            }
        }
        log.push(if calls.is_empty() {
            "-".to_string()
        } else {
            calls.join("/")
        });
        i64::from(v)
    });
}
