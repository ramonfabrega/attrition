//! `Unit::add_patrol_order@005e4560` under the emulator against the port (item 3166; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::add_patrol_order@005e4560`, run by `tools/emu/sweep_obj.py` on its row `unit_add_patrol_order` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 10 argument(s), `p` windows in order: p0..p4 the place(s), in flag order, p5 stub 3's answer window; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0xa4 (0 args) stubbed; callee 0x00434630 (1 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; stub 4's answer points at 128 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+0x14` points at 16 random bytes less `0` (placed); the word at `p0+0x30` points at 16 random bytes less `0` (placed); the word at `m+0xdc` points at 16 random bytes less `0` (placed); stub 3 answers the address of 16 random bytes, a `p` window after the places; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p5+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_add_patrol_order") else {
        return;
    };
    let obj = super::Obj {
        name: "add_patrol_order",
        full: "Unit::add_patrol_order@005e4560",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 600,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // p0 the patrol order (get_obj's slot +0xa4 answer), p1 the record `*order` points at (its +4 the
        // displacement), p2/p3 the order's two coordinate arrays, p4 the word `m+0xdc` points at.
        let rdw =
            |w: &[u8], o: usize| i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"));
        let q = a[8];
        let new = q == 2;
        let first = q == 0;
        let (mut close, mut clears, mut updates) = ("-".to_string(), Vec::new(), Vec::new());
        if new {
            let f = rdw(m, 0x68) & 0xfbff_ffffu32 as i32;
            wr(m, 0x68, f);
            wr(m, 0xc0, 0);
            close = "m+0x0,0".to_string();
            clears.push("m+0x0");
            updates.push("m+0x0");
        }
        wr(&mut p[0], 0x44, a[6]);
        wr(&mut p[0], 0x48, a[7]);
        wr(&mut p[0], 0x4c, a[4]);
        wr(&mut p[0], 0x50, a[5]);
        let mut incs = Vec::new();
        for b in [0x4usize, 0x20] {
            let len = rdw(&p[0], b + 8);
            if len < 2 {
                let need = 2 - len;
                let sh = i16::from_le_bytes([p[0][b + 12], p[0][b + 13]]) as i32;
                let slot = if sh >= 0 { sh } else { len };
                let arg = if need > slot {
                    need
                } else if sh >= 0 {
                    sh
                } else {
                    len
                };
                incs.push(format!("p0+{b:#x},{arg}"));
            }
        }
        wr(&mut p[0], 8, 2);
        wr(&mut p[0], 0x24, 2);
        let k = |v: i32| v.wrapping_mul(0x30).wrapping_add(0x18);
        wr(&mut p[2], 0, k(a[0]));
        wr(&mut p[3], 0, k(a[1]));
        wr(&mut p[2], 4, k(a[2]));
        wr(&mut p[3], 4, k(a[3]));
        wr(&mut p[0], 0x3c, 0);
        let d = rdw(&p[1], 4) as usize;
        p[0][d + 4] |= 4;
        let add = format!("m+0xcc,p0+{d:#x}");
        if first {
            clears.push("m+0x0");
            let v = rdw(&p[4], 0);
            wr(m, 0xdc, v);
        }
        updates.push("m+0x0");
        let j = |v: Vec<&str>| {
            if v.is_empty() {
                "-".to_string()
            } else {
                v.join("/")
            }
        };
        log.push(close);
        log.push(j(clears));
        log.push(j(updates));
        log.push(if incs.is_empty() {
            "-".to_string()
        } else {
            incs.join("/")
        });
        log.push(add);
        0
    });
}
