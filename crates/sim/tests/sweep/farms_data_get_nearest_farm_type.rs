//! `FarmsData::get_nearest_farm_type@008d73a0` under the emulator against the port (item 2893; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `FarmsData::get_nearest_farm_type@008d73a0`, run by `tools/emu/sweep_obj.py` on its row `farms_data_get_nearest_farm_type` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 48 random bytes of object, 2 argument(s), 4 random bytes at 0x00c0ad70; callee 0x00659ca0 (12 args) stubbed; vtable slot +0x3c (0 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `0xc0aba0` points at 16 random bytes less `0` (placed); the word at `0xc0abbc` points at 16 random bytes less `0` (placed); the word at `m+0x24` points at 576 random bytes less `0` (placed); the word at `p0+0` points at 24 random bytes less `0` (placed); the word at `p0+4` points at 24 random bytes less `0` (placed); the word at `p0+8` points at 24 random bytes less `0` (placed); the word at `p0+12` points at 24 random bytes less `0` (placed); the word at `p1+0` points at 24 random bytes less `0` (placed); the word at `p1+4` points at 24 random bytes less `0` (placed); the word at `p1+8` points at 24 random bytes less `0` (placed); the word at `p1+12` points at 24 random bytes less `0` (placed); the word at `p2+0` points at 24 random bytes less `0` (placed); the word at `p2+4` points at 24 random bytes less `0` (placed); the word at `p2+8` points at 24 random bytes less `0` (placed); the word at `p2+12` points at 24 random bytes less `0` (placed); stub 0 answered from [-1, 0, 1, 2, 3] (90%); stub 1 answers the address of 128 random bytes, a `p` window after the places; the stubbed vtable laid at `p4+0`; stub 0 answers -1..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("farms_data_get_nearest_farm_type") else {
        return;
    };
    let obj = super::Obj {
        name: "get_nearest_farm_type",
        full: "FarmsData::get_nearest_farm_type@008d73a0",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            let w = |b: &[u8], o: usize| i32::from_le_bytes(b[o..o + 4].try_into().unwrap());
            let o1 = &p[4 + 4 * a[1] as usize + a[0] as usize];
            log.push(format!(
                "12626800,{},{},0,-1,1152,0,1,417,0,3,{},{}",
                w(o1, 0x10) ^ 0x63637,
                w(o1, 0x14) ^ 0x63637,
                a[0],
                a[1]
            ));
            if a[2] < 0 {
                log.push("-".into());
                -1
            } else {
                let q = rd(&c[0], 0) as usize;
                let k = 4 + 4 * q + a[2] as usize;
                let idx = i32::from(i16::from_le_bytes([p[16][0x78], p[16][0x79]]));
                if idx < 0 {
                    log.push(format!("p{k}+0x0"));
                    -1
                } else {
                    log.push(format!("p{k}+0x0/p{k}+0x0"));
                    if idx < rd(m, 0x18) {
                        i64::from(p[3][idx as usize * 0xc0 + 0xbd] & 1)
                    } else {
                        0
                    }
                }
            }
        }
    });
}
