//! `TypeData::get_limiting_resource@00667730` under the emulator against the port (item 2309; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `TypeData::get_limiting_resource@00667730`, run by `tools/emu/sweep_obj.py` on its row `type_data_get_limiting_resource` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 3 argument(s), callee 0x006e33a0 (2 args) stubbed; vtable slot +0x78 (7 args) stubbed; the word at `p0+0` points at 24 random bytes less `0` (placed); the word at `p1+0` points at 24 random bytes less `0` (placed); the word at `p2+0` points at 24 random bytes less `0` (placed); the word at `p3+0` points at 24 random bytes less `0` (placed); the word at `p4+0` points at 24 random bytes less `0` (placed); the word at `p5+0` points at 24 random bytes less `0` (placed); the word at `p6+0` points at 24 random bytes less `0` (placed); the word at `p7+0` points at 24 random bytes less `0` (placed); stub 0 answered from [0, 1, 1] (90%); stub 1 answered from [0, 1, 2, 3, 4, 5, 100] (90%); stub 0 answers 6 values in call order, printed after the answers; stub 1 answers 6 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 8 rows 0x6eec apart at `0xc061e0`, slices +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("type_data_get_limiting_resource") else {
        return;
    };
    let obj = super::Obj {
        name: "get_limiting_resource",
        full: "TypeData::get_limiting_resource@00667730",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Over the six goods: one leader a0 has (type_avail(good, 1) on its
            // row) whose need (this's slot +0x78: good, a0, a1, a2, 0, 1, -1) is
            // more than its stock (the row's +0x6eb8 list, each word xored with
            // 0x8221) is the limiting resource; a second answers -1.
            let row = a[0] as usize;
            let (mut avail, mut need) = (5usize, 11usize);
            let mut lim = -1i64;
            let (mut ta, mut sl) = (Vec::new(), Vec::new());
            let mut v = None;
            for good in 0..6 {
                ta.push(format!("r+0x0,{good},1"));
                let ok = a[avail] != 0;
                avail = (avail + 1).min(10);
                if !ok {
                    continue;
                }
                sl.push(format!("m+0x0,{good},{},{},{},0,1,-1", a[0], a[1], a[2]));
                let n = a[need];
                need = (need + 1).min(16);
                if (rd(&p[8 + row], 4 * good) ^ 0x8221) < n {
                    if lim != -1 {
                        v = Some(-1);
                        break;
                    }
                    lim = good as i64;
                }
            }
            let mut rows = vec!["-".to_string(); 9];
            rows[row] = ta.join("/");
            log.extend(rows);
            log.push(if sl.is_empty() {
                "-".into()
            } else {
                sl.join("/")
            });
            v.unwrap_or(lim)
        }
    });
}
