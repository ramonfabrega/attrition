//! `ResourceDivvyPool::done_adding_goods@0068a700` under the emulator against the port (item 2893; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ResourceDivvyPool::done_adding_goods@0068a700`, run by `tools/emu/sweep_obj.py` on its row `resource_divvy_pool_done_adding_goods` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 104 random bytes of object, 0 argument(s), callee 0x00a3a3c0 (1 args) stubbed; the word at `m+0x8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `m+0x30` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `m+0x58` points at 16 random bytes less `0` (placed, null a third of the rows); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("resource_divvy_pool_done_adding_goods") else {
        return;
    };
    let obj = super::Obj {
        name: "done_adding_goods",
        full: "ResourceDivvyPool::done_adding_goods@0068a700",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            log.push(format!(
                "m+0x0,{}/m+0x28,{}/m+0x50,{}",
                rd(m, 0x10),
                rd(m, 0x38),
                rd(m, 0x60)
            ));
            for (k, (size, ptr)) in [(0x4usize, 0x8usize), (0x2c, 0x30), (0x54, 0x58)]
                .into_iter()
                .enumerate()
            {
                let n = rd(m, size);
                if rd(m, ptr) != 0 && n > 0 {
                    for b in p[k][..n as usize].iter_mut() {
                        *b = 0;
                    }
                }
            }
            0
        }
    });
}
