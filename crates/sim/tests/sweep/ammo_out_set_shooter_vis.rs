//! `AmmoOut::set_shooter_vis@00679200` under the emulator against the port (item 2362; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `AmmoOut::set_shooter_vis@00679200`, run by `tools/emu/sweep_obj.py` on its row `ammo_out_set_shooter_vis` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 128 random bytes of object, 2 argument(s), the global cell 0x00c06210 points at 768 random bytes; vtable slot +0x48 (2 args) stubbed; the word at `0xc0ab84+a[1]*0x1c` points at 4 random bytes less `a[0]*4` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the stubbed vtable laid at `p1+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("ammo_out_set_shooter_vis") else {
        return;
    };
    let obj = super::Obj {
        name: "set_shooter_vis",
        full: "AmmoOut::set_shooter_vis@00679200",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The object (the objects table, image 0xc0ab84 + 0x1c × a[1], its array's entry a[0])
            // is asked its slot +0x48 `(console->who, 0)` (cell 0xc06210, +0x298); the answer's low
            // byte is the ammo's `shooter_vis` (+0x70).
            log.push(format!("p1+0x0,{},0", rd(&c[0], 0x298)));
            m[0x70] = a[2] as u8;
            0
        }
    });
}
