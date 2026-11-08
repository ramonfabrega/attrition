//! `UnitData::has_damage@00609b50` under the emulator against the port (item 2709; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::has_damage@00609b50`, run by `tools/emu/sweep_obj.py` on its row `unit_data_has_damage` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 152 random bytes of object, 1 argument(s), vtable slot +0xe8 (0 args) stubbed; the word at `0xc0aec0+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 160 random bytes less `0` (placed); the word at `p0+4` points at 160 random bytes less `0` (placed); the word at `p0+8` points at 160 random bytes less `0` (placed); the word at `p0+12` points at 160 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 160 random bytes less `0` (placed); the word at `p5+4` points at 160 random bytes less `0` (placed); the word at `p5+8` points at 160 random bytes less `0` (placed); the word at `p5+12` points at 160 random bytes less `0` (placed); slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; stub 0 answers 1..5 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; figures p1..p4 link at +0x8e (2 bytes) below their own index or to -1, -2; figures p1..p4 link at +0x90 (2 bytes) above their own index or to -1; figures p6..p9 link at +0x8e (2 bytes) below their own index or to -1, -2; figures p6..p9 link at +0x90 (2 bytes) above their own index or to -1; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_has_damage") else {
        return;
    };
    let obj = super::Obj {
        name: "has_damage",
        full: "UnitData::has_damage@00609b50",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            let _ = &c;
            let r16 = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
            // Slot +0xe8 the base `UnitData::is_captain` (the flag after the answer) reads `o_up < 0` inline;
            // else the stub (answering 1..5) is asked, `ecx` the unit it stands on.
            let based = a[2] != 0;
            let mut calls: Vec<String> = Vec::new();
            let mut cur: Option<usize> = None;
            let mut param = a[0];
            let r = loop {
                let w: &[u8] = match cur {
                    None => m,
                    Some(k) => &p[k],
                };
                let who = usize::from(w[9]);
                if param == 0 {
                    let up = r16(w, 0x8e);
                    let cap = if based {
                        i32::from(up < 0)
                    } else {
                        calls.push(match cur {
                            None => "m+0x0".to_string(),
                            Some(k) => format!("p{k}+0x0"),
                        });
                        a[1]
                    };
                    if cap == 0 {
                        cur = Some(5 * who + 1 + usize::try_from(up).expect("a column"));
                        continue;
                    }
                }
                if w[8] & 1 == 0 || rd(w, 0x24) != 0 || w[0x3b] != 0 {
                    break 1;
                }
                let down = r16(w, 0x90);
                if down < 0 {
                    break 0;
                }
                param = 1;
                cur = Some(5 * who + 1 + usize::try_from(down).expect("a column"));
            };
            log.push(if calls.is_empty() {
                "-".to_string()
            } else {
                calls.join("/")
            });
            r
        }
    });
}
