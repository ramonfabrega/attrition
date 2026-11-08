//! `Unit::update_local_seen@0060e410` under the emulator against the port (item 3030; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::update_local_seen@0060e410`, run by `tools/emu/sweep_obj.py` on its row `unit_update_local_seen` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 72 random bytes of object, 0 argument(s), `p` windows in order: p0 the place; the global cell 0x00cae5fc points at 512 random bytes; the global cell 0x00c06188 points at 32 random bytes; 16 random bytes at 0x00cbe330 (each word in 0..24); 32 random bytes at 0x00cb7e90 (each byte in -4..4); 32 random bytes at 0x00cbb0e0 (each byte in -4..4); callee 0x006b4bb0 (4 args) stubbed; the word at `m+0x18` points at 576 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 0 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); the stack 256 bytes under the call is zeroed before each row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_update_local_seen") else {
        return;
    };
    let obj = super::Obj {
        name: "update_local_seen",
        full: "Unit::update_local_seen@0060e410",
        cells: 5,
        calls: true,
        eax: false,
        min_rows: 600,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        let le = |v: i32| {
            v.to_le_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        };
        let d3 = |i: i32| rd(&c[0], 4 * i as usize);
        let seen = i32::from(m[0x40] as i8);
        let (mut t_call, mut t_x, mut t_y): (Vec<String>, Vec<String>, Vec<String>) =
            (Vec::new(), Vec::new(), Vec::new());
        if seen != 0 {
            let radius = rd(&c[2], 4 * rd(&p[0], 0x234) as usize);
            let gx = d3((rd(m, 0x10) ^ 0x63637) >> 7);
            let gy = d3((rd(m, 0x14) ^ 0x63637) >> 7);
            let (w, h) = (rd(&c[1], 0xc), rd(&c[1], 0x10));
            for (&bx, &by) in c[3].iter().zip(c[4].iter()).take(radius.max(0) as usize) {
                let x = i32::from(bx as i8) + gx;
                let y = i32::from(by as i8) + gy;
                if x >= 0 && y >= 0 && x < w && y < h {
                    t_call.push(format!("{x},sp-0x10,sp-0x14,{seen},0"));
                    t_x.push(le(x));
                    t_y.push(le(y));
                }
            }
        }
        for t in [t_call, t_x, t_y] {
            log.push(if t.is_empty() {
                "-".to_string()
            } else {
                t.join("/")
            });
        }
        0
    });
}
