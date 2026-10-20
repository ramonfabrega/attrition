//! `SpellTypeData::find_target@00674920` under the emulator against the port (item 3387; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `SpellTypeData::find_target@00674920`, run by `tools/emu/sweep_spell_type_data_find_target.py` under unicorn
/// (tools/emu): 512 random bytes of object, 3 argument(s), `p` windows in order: none; the global cell 0x00c06210 points at 64 random bytes; callee 0x007cba20 (9 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_data_find_target.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
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
                log: &mut log,
            };
            i64::from(sim::spell_find_view::spell_find_target(
                &mut sw, flags, a[2],
            ))
        };
        assert_eq!(got, want_eax, "find_target — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_target — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_target — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_target — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "find_target — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2000, "only {n} rows");
    eprintln!("SpellTypeData::find_target@00674920: {n} rows agree");
}
