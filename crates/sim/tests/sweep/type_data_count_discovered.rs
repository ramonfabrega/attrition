//! `TypeData::count_discovered@00663f80` under the emulator against the port (item 3024; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `TypeData::count_discovered@00663f80`, run by `tools/emu/sweep_obj.py` on its row `type_data_count_discovered` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), `p` windows in order: p0..p15 sparse table 0 (8 rows × 2 slice(s), row-major), p16..p18 the place(s), in flag order; the global cell 0x00e85ddc points at 2304 random bytes; vtable slot +0xc (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; callee 0x006db810 (1 args) stubbed; the word at `c0+0xc8` points at 64 random bytes less `0` (placed); the word at `c0+0xcc` points at 64 random bytes less `0` (placed); the word at `c0+0xd0` points at 64 random bytes less `0` (placed); slot +0xc laid as 0x004707e0 or its stub, a flag after the answers; slot +0x14 laid as 0x004707c0 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1] (90%); stub 1 answered from [0, 1, 1] (90%); stub 2 answered from [0, 1, 1] (90%); the stubbed vtable laid at `p16+0`; stub 2 answers 16 values in call order, printed after the answers; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 8 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x1, +0x6c18:0x8 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("type_data_count_discovered") else {
        return;
    };
    let obj = super::Obj {
        name: "count_discovered",
        full: "TypeData::count_discovered@00663f80",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 400,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // Eight leaders (stride 0x6eec; slice p[2i] their flag byte, p[2i+1] the tech bits). `this+4`
        // the type: -1 counts every leader, -2 none, below 0x32 counts. Else the type record
        // `table[t]` (p[16 + t - 0x32]): `is_unit_type` (slot 0xc, inline = a[19]) then
        // `is_build_type` (slot 0x14, a[20]); a unit or building type counts when the leader
        // `has_preq` it (a[3..19], one answer a call), anything else on the tech bit.
        let t = rd(m, 4);
        let mut count = 0;
        let mut turn = 0;
        let mut calls: Vec<String> = Vec::new();
        for i in 0..8 {
            if p[2 * i][0] & 1 == 0 {
                continue;
            }
            if t == -2 {
                continue;
            }
            if t == -1 || t < 0x32 {
                count += 1;
                continue;
            }
            let rec = &p[16 + (t - 0x32) as usize];
            let v = rd(rec, 4);
            let mut preq = || {
                calls.push(format!("p{}+0x0,{t}", 2 * i));
                let ans = a[3 + turn] != 0;
                turn += 1;
                ans
            };
            let unit = if a[19] != 0 {
                (0x32..0x19e).contains(&v)
            } else {
                a[0] != 0
            };
            if unit && !preq() {
                continue;
            }
            let build = if a[20] != 0 {
                (0x19e..0x21f).contains(&v)
            } else {
                a[1] != 0
            };
            if build {
                if preq() {
                    count += 1;
                }
            } else if p[2 * i + 1][(t >> 3) as usize] & (1 << (t & 7)) != 0 {
                count += 1;
            }
        }
        log.push(if calls.is_empty() {
            "-".into()
        } else {
            calls.join("/")
        });
        count
    });
}
