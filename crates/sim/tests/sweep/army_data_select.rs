//! `ArmyData::select@006f8f20` under the emulator against the port (item 2877; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ArmyData::select@006f8f20`, run by `tools/emu/sweep_obj.py` on its row `army_data_select` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 152 random bytes of object, 1 argument(s), the global cell 0x00c06210 points at 680 random bytes; the global cell 0x00e8d444 points at 2544 random bytes; callee 0x00717270 (1 args) stubbed; callee 0x006f9df0 (1 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0x18 (3 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `0xc0aba0` points at 16 random bytes less `0` (placed); the word at `0xc0abbc` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed); the word at `p0+12` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p1+12` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 16 random bytes less `0` (placed); the word at `p2+4` points at 16 random bytes less `0` (placed); the word at `p2+8` points at 16 random bytes less `0` (placed); the word at `p2+12` points at 16 random bytes less `0` (placed); stub 1 answered from [-1, 0, 1, 2, 3] (90%); stub 2 answered from [0, 1, 1] (90%); stub 3 answered from [0, 1, 1] (90%); the stubbed vtable laid at `p3+0`; stub 1 answers 4 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 3 answers 4 values in call order, printed after the answers; stub 1 answers -1..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `c1+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("army_data_select") else {
        return;
    };
    let obj = super::Obj {
        name: "select",
        full: "ArmyData::select@006f8f20",
        cells: 2,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            let n = rd(m, 0x10);
            let who = i32::from(i16::from_le_bytes([m[0x94], m[0x95]]));
            let (mut gets, mut sels) = (Vec::new(), Vec::new());
            let (mut used, mut on) = (0usize, 0usize);
            for i in 0..n.max(0) as usize {
                gets.push(format!("m+0x0,{i}"));
                let unit = a[6 + i.min(3)];
                if unit < 0 {
                    continue;
                }
                if p[(3 + 4 * who + unit) as usize][8] & 1 == 0 {
                    continue;
                }
                used += 1;
                if a[10 + (used - 1).min(3)] == 0 {
                    continue;
                }
                on += 1;
                if a[14 + (on - 1).min(3)] == 0 {
                    continue;
                }
                sels.push(format!("c1+0x0,{unit},{who},0"));
            }
            log.push("c1+0x0,-1".into());
            log.push(if gets.is_empty() {
                "-".into()
            } else {
                gets.join("/")
            });
            log.push(if sels.is_empty() {
                "-".into()
            } else {
                sels.join("/")
            });
            0
        }
    });
}
