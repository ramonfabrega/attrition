//! `Type::pay_cost@006681f0` under the emulator against the port (item 2837; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Type::pay_cost@006681f0`, run by `tools/emu/sweep_obj.py` on its row `type_pay_cost` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 5 argument(s), the global cell 0x00c061ec points at 2112 random bytes; argument 4 names 24 random bytes (null a third of the rows); callee 0x006e33a0 (2 args) stubbed; vtable slot +0x78 (7 args) stubbed; the word at `p2+0` points at 24 random bytes less `0` (placed); the word at `p4+0` points at 24 random bytes less `0` (placed); the word at `p6+0` points at 24 random bytes less `0` (placed); the word at `p8+0` points at 24 random bytes less `0` (placed); the word at `p10+0` points at 24 random bytes less `0` (placed); the word at `p12+0` points at 24 random bytes less `0` (placed); the word at `p14+0` points at 24 random bytes less `0` (placed); the word at `p16+0` points at 24 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1, 2, 5, 100, -7, 3, 250] (90%); stub 0 answers 6 values in call order, printed after the answers; stub 1 answers 6 values in call order, printed after the answers; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x468:0x18, +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("type_pay_cost") else {
        return;
    };
    let obj = super::Obj {
        name: "pay_cost",
        full: "Type::pay_cost@006681f0",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // Each good `g` of six: the out array (`a[4]`, window p0, when non-null) is zeroed, then,
        // where the leader (`a[0]`) has it available (`type_avail` answers `a[7..13]`), the cost slot
        // `+0x78` (`a[13..19]`, in call order) is taken from the bucket (`p[17 + who]`, XOR 0x8221,
        // floored at 0), from the escrow (`p[1 + 2·who]`, `+0x468`, floored at 0) where `a[3]` is set,
        // else the escrow is emptied when the bucket cannot cover the cost (item 2837).
        let who = a[0] as usize;
        let mut asked = 13;
        for g in 0..6usize {
            if a[4] != 0 {
                wr(&mut p[0], 4 * g, 0);
            }
            if a[7 + g] == 0 {
                continue;
            }
            log.push(format!(
                "s1:m+0x0,{g},{},{},{},{},1,-1",
                a[0], a[1], a[2], 0
            ));
            let cost = a[asked];
            asked += 1;
            if a[4] != 0 {
                wr(&mut p[0], 4 * g, cost);
            }
            let bucket = rd(&p[17 + who], 4 * g) ^ 0x8221;
            let escrow = rd(&p[1 + 2 * who], 4 * g);
            if a[3] == 0 && bucket.wrapping_sub(escrow) < cost {
                wr(&mut p[1 + 2 * who], 4 * g, 0);
            }
            wr(
                &mut p[17 + who],
                4 * g,
                bucket.wrapping_sub(cost).max(0) ^ 0x8221,
            );
            if a[3] != 0 {
                let escrow = rd(&p[1 + 2 * who], 4 * g);
                wr(&mut p[1 + 2 * who], 4 * g, escrow.wrapping_sub(cost).max(0));
            }
        }
        if log.is_empty() {
            log.push("-".into());
        }
        0
    });
}
