//! `ObjectData::is_in_range@006486b0` under the emulator against the port (item 3182; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ObjectData::is_in_range@006486b0`, run by `tools/emu/sweep_obj.py` on its row `object_data_is_in_range_at` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 512 random bytes of object, 7 argument(s), `p` windows in order: p0 argument 6, p1..p14 the place(s), in flag order; the global cell 0x00c0ab84 points at 16 random bytes; the global cell 0x00c0aec0 points at 16 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; the global cell 0x00c061d0 points at 512 random bytes; argument 6 names 4 random bytes (null a third of the rows); vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x006488f0 (4 args) stubbed; vtable slot +0x12c (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (0 args) stubbed; the word at `c0+0` points at 64 random bytes less `0` (placed); the word at `c0+4` points at 64 random bytes less `0` (placed); the word at `c0+8` points at 64 random bytes less `0` (placed); the word at `c0+12` points at 64 random bytes less `0` (placed); the word at `c1+0` points at 256 random bytes less `0` (placed); the word at `c1+4` points at 256 random bytes less `0` (placed); the word at `c1+8` points at 256 random bytes less `0` (placed); the word at `c1+12` points at 256 random bytes less `0` (placed); the word at `p5+0x18` points at 768 random bytes less `0` (placed); the word at `p6+0x18` points at 768 random bytes less `0` (placed); the word at `p7+0x18` points at 768 random bytes less `0` (placed); the word at `p8+0x18` points at 768 random bytes less `0` (placed); the word at `c3+0x138` points at 17408 random bytes less `0` (placed); the word at `m+0x18` points at 768 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0xbc laid as 0x0046ce30 or its stub, a flag after the answers; stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [1, 1, 1, 0] (90%); stub 3 answered from [1, 2, 3] (90%); stub 4 answered from [1, 2, 3] (90%); stub 5 answered from [1, 1, 0] (90%); stub 6 answered from [1, 0, 0] (90%); stub 7 answered from [1, 0, 0] (90%); stub 2 answered from [101, 102, 103, 245, 246, 247, 101, 102, 103, 245, 246, 247, 101, 102, 103, 245, 246, 247, 53, 54, 55, 135, 136, 137, 185, 186, 187, 197, 198, 199, 245, 246, 247, 327, 328, 329, 377, 378, 379, 389, 390, 391, 437, 438, 439, 519, 520, 521, 569, 570, 571, 581, 582, 583, 53, 54, 55, 135, 136, 137, 185, 186, 187, 197, 198, 199, 245, 246, 247, 327, 328, 329, 377, 378, 379, 389, 390, 391, 437, 438, 439, 519, 520, 521, 569, 570, 571, 581, 582, 583] (90%); stub 5 answers 2 values in call order, printed after the answers; stub 3 answers 2 values in call order, printed after the answers; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("object_data_is_in_range_at") else {
        return;
    };
    let obj = super::Obj {
        name: "is_in_range",
        full: "ObjectData::is_in_range@006486b0",
        cells: 4,
        calls: true,
        eax: true,
        min_rows: 1200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // a: 0 object index, 1 player (0), 2 / 3 the coordinates, 5 the margin flag, 6 the out
        // pointer (0 when null); stub answers a[7] slot +8, a[8] slot +0xbc, a[9] attack_dist, a[11]
        // slot +0x130, a[13] slot +0xb8, a[14] the type record's slot +0x60; slot +0x18's two calls
        // a[15] a[16], slot +0x12c's a[17] a[18]; a[19] / a[20] 1 when `ObjectData::is` /
        // `UnitData::is_on_map` stand in slots +0xb8 / +0xbc. p0 the out word, p1..p4 the objects,
        // p5..p8 the units, p9..p12 their types, p13 the tiles, p14 this's type record; c2 the
        // div_3_table, c3 the world.
        let rdw =
            |w: &[u8], o: usize| i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"));
        let idx = a[0] as usize;
        let d3 = |v: i32| rdw(&c[2], (v as usize) * 4);
        let mut t: Vec<String> = Vec::new();
        let r: i64 = 'body: {
            if p[1 + idx][8] & 1 == 0 {
                break 'body 0;
            }
            if a[7] != 0 {
                let on = if a[19 + 1] != 0 {
                    (u16::from_le_bytes([p[5 + idx][0x82], p[5 + idx][0x83]]) >> 15) as i32
                } else {
                    a[8]
                };
                if on == 0 {
                    break 'body 0;
                }
            }
            let tile = d3(a[3] >> 6)
                .wrapping_mul(rdw(&c[3], 0x18))
                .wrapping_add(d3(a[2] >> 6));
            if p[13][tile as usize * 2] & 0x30 == 0x30 {
                break 'body 0;
            }
            t.push(format!(
                "m+0x0,{},{},{},{}",
                a[0],
                a[1],
                d3(a[2] >> 4).wrapping_mul(0x30).wrapping_add(0x18),
                d3(a[3] >> 4).wrapping_mul(0x30).wrapping_add(0x18)
            ));
            let dist = a[9];
            if a[6] != 0 {
                p[0][0..4].copy_from_slice(&dist.to_le_bytes());
            }
            let margin: i32 = if a[5] != 0 { 0x90 } else { 0 };
            if rdw(&p[14], 0x1fc) == 0 {
                let v = if a[19] != 0 { a[14] } else { a[13] };
                if v == 0 {
                    break 'body (dist <= 0x66) as i64;
                }
                break 'body (dist <= 0xf6) as i64;
            }
            if dist < a[17].wrapping_mul(0xc0).wrapping_sub(6) {
                let (mut mine, mut theirs) = (0, 0);
                if a[15] != 0 {
                    let s = i16::from_le_bytes([m[0xa], m[0xb]]) as usize;
                    mine = rdw(&p[9 + s], 0x244);
                }
                if a[16] != 0 {
                    theirs = rdw(&p[9 + idx], 0x244);
                }
                if mine.wrapping_add(dist).wrapping_add(theirs)
                    < a[18].wrapping_mul(0xc0).wrapping_sub(6)
                {
                    break 'body 0;
                }
            }
            (margin.wrapping_add(dist) <= a[11].wrapping_mul(0xc0).wrapping_add(6)) as i64
        };
        log.push(if t.is_empty() {
            "-".to_string()
        } else {
            t.join("/")
        });
        r
    });
}
