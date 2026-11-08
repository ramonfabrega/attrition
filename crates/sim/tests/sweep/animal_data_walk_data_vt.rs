//! `AnimalData::walk_data@005d7ea0` under the emulator against the port (item 2900; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `AnimalData::walk_data@005d7ea0`, run by `tools/emu/sweep_obj.py` on its row `animal_data_walk_data_vt` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): an empty body: 384 random bytes of object and 1 argument(s), every byte held after the return and `esp` checked (parked 1889).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("animal_data_walk_data_vt") else {
        return;
    };
    let obj = super::Obj {
        name: "walk_data",
        full: "AnimalData::walk_data@005d7ea0",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // Object::walk_data(this, walker), then the walker's slot 4 handed the string at the global + 0x22ccc. The kept arm
        // (this's slot 0x78 is Object::must_walk, `a[5]`) tests a flag byte at [ebp+0xb] -- the top byte of the walker
        // pointer when `input != 0`, else set from this+0x18, this+8 bit 0, the u16 at this+0x32 and a non-negative i16 at this+0x8e -- and calls the walker's
        // slot 0 with (&flag, ebp+0xc); the other arm calls this's slot 0x78 and tests that call's whole `eax` (the listing
        // jumps to the `test eax` past the byte load; the decompile's `>> 0x18` is not it). A set flag hands the walker
        // (this+0x150, this+0x155).
        let _ = (&c, &p);
        log.push("s0:m+0x0,p0+0x0".into());
        log.push("s2:p0+0x0,2640".into());
        let mut flag = ((a[0] as u32) >> 24) as u8;
        let r16 = |w: &[u8], o: usize| u16::from_le_bytes([w[o], w[o + 1]]);
        if a[5] != 0 {
            if rd(&p[0], 4) == 0 {
                flag = u8::from(
                    rd(m, 0x18) != 0
                        || m[8] & 1 != 0
                        || r16(m, 0x32) != 0
                        || (r16(m, 0x8e) as i16) >= 0,
                );
            }
            log.push("s1:p0+0x0,sp-0x1,sp+0x0".into());
        } else {
            log.push("s3:m+0x0,p0+0x0".into());
            flag = u8::from(a[4] != 0);
        }
        if flag != 0 {
            log.push("s1:p0+0x0,m+0x150,m+0x155".into());
        }
        0
    });
}
