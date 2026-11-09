//! `SubObjectData::is_seen@00661f10` under the emulator against the port (item 2633; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `SubObjectData::is_seen@00661f10`, run by `tools/emu/sweep_obj.py` on its row `sub_object_data_is_seen` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 2 argument(s), the global cell 0x00c061d0 points at 16 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; callee 0x006b53f0 (3 args) stubbed; stub 0 answered from [0, 0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 0 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("sub_object_data_is_seen") else {
        return;
    };
    let obj = super::Obj {
        name: "is_seen",
        full: "SubObjectData::is_seen@00661f10",
        cells: 2,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        if a[0] == i32::from(m[9]) {
            log.extend(["-".to_string(), "-".to_string(), "-".to_string()]);
            1
        } else {
            let d = |v: i32| rd(&c[1], (((v ^ 0x63637) >> 7) as usize) * 4);
            let h = |v: i32| {
                v.to_le_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            };
            log.push(format!("c0+0x0,sp-0x18,sp-0x14,{}", a[0]));
            log.push(h(d(rd(m, 0x10))));
            log.push(h(d(rd(m, 0x14))));
            i64::from(a[2] != 0)
        }
    });
}
