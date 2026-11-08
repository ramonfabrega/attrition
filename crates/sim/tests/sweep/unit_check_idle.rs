//! `Unit::check_idle@006032c0` under the emulator against the port (item 3182; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::check_idle@006032c0`, run by `tools/emu/sweep_obj.py` on its row `unit_check_idle` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0 the place; the global cell 0x00c061ec points at 1536 random bytes; the global cell 0x00c061f0 points at 2304 random bytes; the global cell 0x00c06214 points at 16 random bytes; 56792 random bytes at 0x00e3a390; callee 0x00605400 (3 args) stubbed; callee 0x008e4670 (2 args) stubbed; callee 0x0060a6a0 (0 args) stubbed; callee 0x0060b030 (0 args) stubbed; callee 0x00909330 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (0 args) stubbed; the word at `m+0x18` points at 704 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 2 answered from [1, 0, 0] (90%); stub 3 answered from [1, 0, 0] (90%); stub 5 answered from [1, 0, 0] (90%); stub 6 answered from [1, 0, 0] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_check_idle") else {
        return;
    };
    let obj = super::Obj {
        name: "check_idle",
        full: "Unit::check_idle@006032c0",
        cells: 4,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // m `this`; c0 the game record (frame at +0x550), c1 the constants (marine entrench at +0x874),
        // c2 the graphic pieces, c3 the leaders table; p0 the type record `m+0x18`. a[2] is_cloaked,
        // a[3] get_order, a[5] the slot +0xb8 stub's answer, a[6] the type record's slot +0x60, a[7] 1
        // when the image's `ObjectData::is` stands in slot +0xb8. Recorded: set_angle, add_entrench,
        // emit_cloak_particles, in that order.
        let rdw =
            |w: &[u8], o: usize| i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"));
        let wrw = |w: &mut [u8], o: usize, v: i32| w[o..o + 4].copy_from_slice(&v.to_le_bytes());
        let mut t: Vec<Vec<String>> = vec![Vec::new(); 3];
        let pl = m[9] as i32;
        let sh = i16::from_le_bytes([m[0xa], m[0xb]]) as i32;
        let dl = m[0xb0];
        let mut bump = false;
        let mut ecx = "m+0x0";
        if dl == 0xff {
            m[0xb0] = 1;
        } else {
            bump = true;
            if dl >= 2 {
                let x = (sh.wrapping_add(rdw(&c[0], 0x550)) as u32 & 0x8000_000f) as i32;
                let x = if x < 0 {
                    (x.wrapping_sub(1) | -16).wrapping_add(1)
                } else {
                    x
                };
                if x != 0 {
                    bump = false;
                }
                ecx = "0";
            }
        }
        if bump {
            m[0xb0] = dl.wrapping_add(1);
            if m[0xb0] == 4 && rdw(m, 0x68) & 0x2000000 != 0 {
                t[0].push(format!("m+0x0,{},{ecx},0", rdw(m, 0x5c)));
            }
        }
        if rdw(m, 0x6c) & 0x800 != 0
            && m[0xb0] as i32 >= rdw(&c[1], 0x874)
            && rdw(m, 0x68) & 0x2000000 == 0
        {
            t[1].push(format!("{},{pl},{sh}", m[0xb0]));
            wrw(m, 0x68, rdw(m, 0x68) | 0x2000000);
            wrw(m, 0x6c, rdw(m, 0x6c) | 0x20000);
            wrw(m, 0x5c, rdw(m, 0x50));
        }
        let x68 = rdw(m, 0x68);
        let ty = rdw(&p[0], 0x2b4);
        let emit = if x68 & 0x11000 == 0x1000 {
            if m[0xb0] < 10 {
                false
            } else {
                wrw(m, 0x68, x68 & !0x1000);
                a[2] != 0
            }
        } else if m[0xb0] != 1 {
            false
        } else {
            let to_d9 = x68 & 0x800 != 0
                || ty & 0x4000 != 0
                || rdw(m, 0x6c) & 0x8000 != 0
                || (ty & 0x40000 != 0 && a[3] == 0);
            to_d9 && ty & 0x40000 != 0 && rdw(m, 0x68) & 0x1000 == 0
        };
        if emit {
            t[2].push(format!("c2+0x0,{pl},{sh}"));
        }
        if m[8] & 8 == 0 {
            m[8] |= 0x18;
            let is = if a[7] != 0 { a[6] } else { a[5] };
            let ty4 = rdw(&p[0], 4);
            if is != 0 || ty4 == 0x3d || ty4 == 0x3e || ty4 == 400 {
                let o = pl as usize * 0x6eec;
                let v = rdw(&c[3], o) | 0x2000000;
                wrw(&mut c[3], o, v);
            }
        }
        for v in t {
            log.push(if v.is_empty() {
                "-".to_string()
            } else {
                v.join("/")
            });
        }
        0
    });
}
