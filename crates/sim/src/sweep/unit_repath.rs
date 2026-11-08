//! `Unit::repath@005e29b0` under the emulator against the port (item 2341; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::repath@005e29b0`, run by `tools/emu/sweep_obj.py` on its row `unit_repath` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 1024 random bytes of object, 0 argument(s), the global cell 0x00e85f20 points at 10240 random bytes; vtable slot +0x10 (0 args) stubbed; vtable slot +0x10c (0 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; callee 0x007123f0 (1 args) stubbed; the word at `m+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p0+0x4` points at 16 random bytes less `0` (placed); the word at `p1+0x8` points at 16 random bytes less `0` (placed); stub 0 answered from [1, 2, 3, 4, 18, 19, 21, 0, 7, 26] (90%); stub 1 answers the address of 32 random bytes, a `p` window after the places; the stubbed vtable laid at `p2+0`; stub 0 answers 6 values in call order, the last from [0, 7, 26], printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_repath") else {
        return;
    };
    let obj = super::Obj {
        name: "repath",
        full: "Unit::repath@005e29b0",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 400,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `a` is the four stubs' answers then `get_type`'s six (`--seq
        // 0:6:0,7,26`, the last a terminator off the family, item 2368). The head is `m+0xdc` → `p0`, its node `p0+4` → `p1`,
        // whose `+8` is the order `p2` (the stubbed vtable at `+0`,
        // flags at `+4`); `get_group_move_order` answers `p3`, the id
        // its `+0xc`; `groups` is `c0`, stride `0x9d4`.
        let mut turn = 0usize;
        let mut cx = (m, p, &mut *log);
        crate::orders::repath_steps(
            &mut cx,
            |(m, _, _)| rd(m, 0xdc) != 0,
            |(_, _, log)| {
                log.push("s0:p2+0x0".into());
                let v = a[4 + turn.min(5)];
                turn += 1;
                v
            },
            |(_, p, _)| p[2][4] & 4 != 0,
            |(m, _, _)| {
                let g = i16::from_le_bytes([m[0x80], m[0x81]]);
                (g >= 0).then_some(i32::from(g))
            },
            |(_, p, log)| {
                log.push("s1:p2+0x0".into());
                rd(&p[3], 0xc)
            },
            |(_, _, log), g, id| log.push(format!("s3:c0+{:#x},{id}", g * 0x9d4)),
            |(_, _, log)| log.push("s2:m+0x0,0".into()),
        );
        let (m, p, log) = cx;
        // `--record-order` prints `-` for a row that called nothing.
        if log.is_empty() {
            log.push("-".into());
        }
        // Every head read writes the engine's iterator cache.
        if rd(m, 0xdc) != 0 {
            let node = rd(&p[0], 4);
            wr(m, 0xd4, node);
            wr(m, 0xcc, rd(&p[1], 8));
            m[0xd0] = p[1][0xc];
        }
        0
    });
}
