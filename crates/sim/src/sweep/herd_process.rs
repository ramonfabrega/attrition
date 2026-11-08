//! `Herd::process@00741760` under the emulator against the port (item 2341; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Herd::process@00741760`, run by `tools/emu/sweep_obj.py` on its row `herd_process` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), the global cell 0x00c06184 points at 16 random bytes; the global cell 0x00c06188 points at 320 random bytes; callee 0x00a39d70 (2 args) stubbed; the word at `c1+0x134` points at 1024 random bytes less `0` (placed); stub 0 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("herd_process") else {
        return;
    };
    let obj = super::Obj {
        name: "process",
        full: "Herd::process@00741760",
        cells: 2,
        calls: true,
        eax: false,
        min_rows: 1200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // a[0] is stub 0's single answer, which `--seq` replaces;
        // a[1], a[2] the two draws in call order.
        let (w, h) = (rd(&c[1], 0), rd(&c[1], 4));
        let mut turn = 0usize;
        let mut calls: Vec<String> = Vec::new();
        let walked = crate::gaia::herd_process_steps(
            rd(m, 0),
            rd(m, 4),
            w,
            h,
            |_site| {
                calls.push("c0+0x0,0,65535".to_string());
                let v = a[1 + turn.min(1)];
                turn += 1;
                v
            },
            |x, y| {
                let t = ((w * y + x) * 0x1c) as usize;
                let flags = u16::from_le_bytes([p[0][t], p[0][t + 1]]);
                (flags, p[0][t + 0x11])
            },
        );
        log.push(if calls.is_empty() {
            "-".to_string()
        } else {
            calls.join("/")
        });
        if let Some((x, y)) = walked {
            wr(m, 8, x);
            wr(m, 12, y);
        }
        0
    });
}
