//! `SpellType::pay_cast_costs@00676c40` under the emulator against the port (item 2281; tools/sweep_gen.py, obj with windows).

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

/// `SpellType::pay_cast_costs@00676c40`, run by `tools/emu/sweep_spell_type_pay_cast_costs.py` under unicorn
/// (tools/emu): 472 random bytes of object, 2 argument(s), the global cell 0x00c061ec points at 2096 random bytes; the global cell 0x00c0618c points at 224 random bytes; vtable slot +0x84 (4 args) stubbed; vtable slot +0xd4 (5 args) stubbed; vtable slot +0x18 (0 args) stubbed; the word at `c1+0x14+a[1]*0x1c` points at 4 random bytes less `a[0]*4` (placed); the word at `p0+0` points at 4 random bytes less `0` (placed); the word at `0xc0aec0+a[1]*0x1c` points at 4 random bytes less `a[0]*4` (placed); the word at `p2+0` points at 4 random bytes less `0x96` (placed); stub 0 answered from [0, 10, 1, 5] (90%); stub 2 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p1+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_pay_cast_costs.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Slot +0x84 asked (player, -1, 0, 1): 0 answers 1. Else, unless it
                // answered 10 with the game's `0x821` bit 8 set, slot +0xd4 is told
                // (player, index, -1, 1, 0), the object `objects[player][index]` asked
                // its slot +0x18, and set, the unit's `+0x96` short takes `+0x1d0`.
                let (idx, pl) = (a[0], a[1]);
                log.push(format!("m+0x0,{pl},-1,0,1"));
                if a[2] == 0 {
                    log.push("-".into());
                    log.push("-".into());
                    1
                } else if a[2] == 10 && c[0][0x821] & 8 != 0 {
                    log.push("-".into());
                    log.push("-".into());
                    0
                } else {
                    log.push(format!("m+0x0,{pl},{idx},-1,1,0"));
                    log.push("p1+0x0".into());
                    if a[4] != 0 {
                        let v = i16::from_le_bytes([p[3][0], p[3][1]])
                            .wrapping_add(i16::from_le_bytes([m[0x1d0], m[0x1d1]]));
                        p[3][0..2].copy_from_slice(&v.to_le_bytes());
                    }
                    0
                }
            }
        };
        assert_eq!(got, want_eax, "pay_cast_costs — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "pay_cast_costs — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "pay_cast_costs — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "pay_cast_costs — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "pay_cast_costs — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("SpellType::pay_cast_costs@00676c40: {n} rows agree");
}
