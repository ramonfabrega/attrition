//! `SpellTypeData::find_target@00674920` under the emulator against the port (item 3387; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `SpellTypeData::find_target@00674920`, run by `tools/emu/sweep_obj.py` on its row `spell_type_data_find_target` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 512 random bytes of object, 3 argument(s), `p` windows in order: none; the global cell 0x00c06210 points at 64 random bytes; callee 0x007cba20 (9 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("spell_type_data_find_target") else {
        return;
    };
    let obj = super::Obj {
        name: "find_target",
        full: "SpellTypeData::find_target@00674920",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 2000,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        struct Sw<'a> {
            answer: i32,
            log: &'a mut Vec<String>,
        }
        impl sim::spell_find_view::SpellConsole for Sw<'_> {
            fn find_target(&mut self, param3: i32, search: i32, flags: i32) -> i32 {
                // `FILTER_ALL`, two zeros, and three dead stack slots (zero here).
                self.log
                    .push(format!("c0+0x0,{param3},{search},{flags},0,0,0,0,0,0"));
                self.answer
            }
        }
        let flags = rd(m, 0x1c8) as u32;
        let mut sw = Sw {
            answer: a[3],
            log: &mut *log,
        };
        i64::from(sim::spell_find_view::spell_find_target(
            &mut sw, flags, a[2],
        ))
    });
}
