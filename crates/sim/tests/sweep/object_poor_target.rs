//! `Object::poor_target@0064a270` under the emulator against the port (item 3030; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Object::poor_target@0064a270`, run by `tools/emu/sweep_obj.py` on its row `object_poor_target` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 3 argument(s), `p` windows in order: p0..p23 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; vtable slot +0x18 (0 args) stubbed; vtable slot +0x148 (1 args) stubbed; vtable slot +0x130 (0 args) stubbed; vtable slot +0x17c (3 args) stubbed; callee 0x0064c880 (2 args) stubbed; callee 0x00616e80 (0 args) stubbed; callee 0x0046f050 (0 args) stubbed; the word at `c0+0x14` points at 8 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `c0+0x30` points at 8 random bytes less `0` (placed); the word at `p3+0` points at 32 random bytes less `0` (placed); the word at `p3+4` points at 32 random bytes less `0` (placed); the word at `0xc0aec0` points at 8 random bytes less `0` (placed); the word at `p6+0` points at 256 random bytes less `0` (placed); the word at `p6+4` points at 256 random bytes less `0` (placed); the word at `0xc0aedc` points at 8 random bytes less `0` (placed); the word at `p9+0` points at 256 random bytes less `0` (placed); the word at `p9+4` points at 256 random bytes less `0` (placed); the word at `p7+0xf4` points at 8 random bytes less `0` (placed); the word at `p12+0` points at 256 random bytes less `0` (placed); the word at `p7+0x18` points at 768 random bytes less `0` (placed); the word at `p8+0xf4` points at 8 random bytes less `0` (placed); the word at `p15+0` points at 256 random bytes less `0` (placed); the word at `p8+0x18` points at 768 random bytes less `0` (placed); the word at `p10+0xf4` points at 8 random bytes less `0` (placed); the word at `p18+0` points at 256 random bytes less `0` (placed); the word at `p10+0x18` points at 768 random bytes less `0` (placed); the word at `p11+0xf4` points at 8 random bytes less `0` (placed); the word at `p21+0` points at 256 random bytes less `0` (placed); the word at `p11+0x18` points at 768 random bytes less `0` (placed); stub 1 answered from [1, 1, 0] (90%); stub 2 answered from [0, 1, 2, 3] (90%); stub 5 answered from [1, 0] (90%); stub 6 answered from [1, 1, 0] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 3 answers 2 values in call order, printed after the answers; stub 4 answers 2 values in call order, printed after the answers; stub 0 answers 0..3 every row; stub 3 answers 0..600 every row; stub 4 answers 150..450 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `m+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 12000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("object_poor_target") else {
        return;
    };
    let obj = super::Obj {
        name: "poor_target",
        full: "Object::poor_target@0064a270",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 12000,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        let k = 0x63637;
        let (idx, pl) = (a[0] as usize, a[1] as usize);
        let ri = 2 * pl + idx;
        let u = 7 + 3 * pl + idx;
        let yw = 13 + 3 * ri;
        let (mut t1, mut t2, mut t3, mut t4, mut t5) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let res: i64 = 'run: {
            if a[10] == 0 || a[11] == 0 {
                break 'run 0;
            }
            let ux = rd(&p[u], 0x10) ^ k;
            let uy = rd(&p[u], 0x14) ^ k;
            let byte = (rd(&p[yw], 0x98) >> 16) & 0xff;
            if byte & 0x40 != 0 {
                t1.push("m+0x0,-2147483648".to_string());
                if a[4] == 0 {
                    break 'run 1;
                }
                t4.push(format!("m+0x0,{idx},{pl}"));
                t2.push("m+0x0".to_string());
                break 'run i64::from(a[14] > a[5].wrapping_mul(0xc0));
            }
            t5.push(format!("p{u}+0x0"));
            if a[9] == 0 {
                break 'run 0;
            }
            t3.push(format!("p{u}+0x0,{ux},{uy},0"));
            let (b9, sh) = (
                m[9] as usize,
                i32::from(i16::from_le_bytes([m[10], m[11]])) as usize,
            );
            let ou = 7 + 3 * b9 + sh;
            t3.push(format!(
                "p{ou}+0x0,{},{},0",
                rd(&p[ou], 0x10) ^ k,
                rd(&p[ou], 0x14) ^ k
            ));
            if a[12] <= a[13] {
                break 'run 0;
            }
            let o = 1 + 3 * pl + idx;
            let dy = (rd(&p[o], 0x14) ^ k).wrapping_sub(rd(m, 0x14) ^ k);
            let dx = (rd(&p[o], 0x10) ^ k).wrapping_sub(rd(m, 0x10) ^ k);
            let fa = crate::movement::find_angle(dx, dy).0;
            let e = (rd(&p[u], 0x50).wrapping_sub(fa) as u32).wrapping_sub(0x8000_0000);
            if e < 0x2aaa_aaaa {
                break 'run 0;
            }
            if crate::combat::flanking(e) < 2 {
                break 'run 0;
            }
            t4.push(format!("m+0x0,{idx},{pl}"));
            let ri2 = 2 * b9 + sh;
            if rd(&p[14 + 3 * ri2], 0x2c8) & 0x400 != 0 {
                t2.push("m+0x0".to_string());
                i64::from(a[14] > a[5].wrapping_mul(0xc0))
            } else {
                i64::from(a[14] > 0xc0)
            }
        };
        for t in [t1, t2, t3, t4, t5] {
            log.push(if t.is_empty() {
                "-".to_string()
            } else {
                t.join("/")
            });
        }
        res
    });
}
