//! `CityData::is_seen@00739560` under the emulator against the port (item 2309; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `CityData::is_seen@00739560`, run by `tools/emu/sweep_obj.py` on its row `city_data_is_seen` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 96 random bytes of object, 1 argument(s), vtable slot +0x3c (0 args) stubbed; vtable slot +0x48 (2 args) stubbed; the word at `0xc0ab84+i8(m,0x5e)*0x1c` points at 4 random bytes less `i16(m,8)*4` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); stub 1 answered from [0, 1] (90%); stub 0 answers the address of 16 random bytes, a `p` window after the places; the stubbed vtable laid at `p1+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p2+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("city_data_is_seen") else {
        return;
    };
    let obj = super::Obj {
        name: "is_seen",
        full: "CityData::is_seen@00739560",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // A live city (+4 bit 1) with its object (+8, a short) and owner (+0x5e,
            // a byte) both non-negative asks the object (objects[who][o]) for its
            // record (slot +0x3c), and the record's slot +0x48(a0, 0) answers.
            let o = i16::from_le_bytes([m[8], m[9]]);
            let who = m[0x5e] as i8;
            if m[4] & 1 != 0 && o >= 0 && who >= 0 {
                log.push("p1+0x0".into());
                log.push(format!("p2+0x0,{},0", a[0]));
                i64::from(a[2])
            } else {
                log.push("-".into());
                log.push("-".into());
                0
            }
        }
    });
}
