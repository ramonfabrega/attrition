//! `GoodTypeOut::say_gather_level@006700d0` under the emulator against the port (item 2783; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `GoodTypeOut::say_gather_level@006700d0`, run by `tools/emu/sweep_obj.py` on its row `good_type_out_say_gather_level` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 5 argument(s), the global cell 0x00c06378 points at 32 random bytes; 4 random bytes at 0x00c8cd00; argument 0 names 16 random bytes; callee 0x00a1d440 (1 args) stubbed; callee 0x00a1d180 (1 args) stubbed; callee 0x00670130 (5 args) stubbed; the word at `c0+0x10` points at 32 random bytes less `0x17c0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("good_type_out_say_gather_level") else {
        return;
    };
    let obj = super::Obj {
        name: "say_gather_level",
        full: "GoodTypeOut::say_gather_level@006700d0",
        cells: 2,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `GoodTypeOut::say_gather_level(text, a1, ..)`, `text` the string at `a[0]` (`p[0]`): appends
        // (`String::operator+=`, `0xa1d440`) the string at `*(G+0x10) + 0x17c0` for `G` the record of cell
        // `0xc06378` (`c[0]`, placed so the sum is the start of a window, `p[1]`), then the integer
        // `[this+4] + 1` (`0xa1d180`), then the string at `[0xc8cd00] + 0x7db4` (`c[1]`), and calls
        // `say_level` (`0x670130`) with `text`, `a1`, two stale stack words (a 0 and the return address
        // `0x67010d`) and 0.
        log.push(format!(
            "p0+0x0,p1+0x0/p0+0x0,{}",
            rd(&c[1], 0).wrapping_add(0x7db4)
        ));
        log.push(format!("p0+0x0,{}", rd(m, 4).wrapping_add(1)));
        log.push(format!("p0+0x0,p0+0x0,{},0,6750477,0", a[1]));
        0i64
    });
}
