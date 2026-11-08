//! `add_type_bucket@00669160` under the emulator against the port (item 2346; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `add_type_bucket@00669160`, run by `tools/emu/sweep_obj.py` on its row `add_type_bucket` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), 3944 random bytes at 0x00ed89ce; 8 random bytes at 0x00ed88c8; `ecx` an integer in 0..492, not the object, the row's last; `edx` an integer in 0..7, after `ecx` on the row; `ecx` and `edx` held to their draw every row, no wild word; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("add_type_bucket") else {
        return;
    };
    let obj = super::Obj {
        name: "add_type_bucket",
        full: "add_type_bucket@00669160",
        cells: 2,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // A byte of the bucket table (`type_buckets`, 0xed89ce, rows 0x1ed apart) at row `edx`,
            // column `ecx`, counted up; a bucket that becomes 1 is a new type, and the unique-type
            // count (0xed88cb) is counted up too.
            let k = a[1] as usize * 0x1ed + a[0] as usize;
            c[0][k] = c[0][k].wrapping_add(1);
            if c[0][k] == 1 {
                c[1][3] = c[1][3].wrapping_add(1);
            }
            0
        }
    });
}
