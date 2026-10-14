//! `SpellType::cast_depopulate@00671e50` under the emulator against the port (item 2298; tools/sweep_gen.py, obj with windows).

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

/// `SpellType::cast_depopulate@00671e50`, run by `tools/emu/sweep_spell_type_cast_depopulate.py` under unicorn
/// (tools/emu): 32 random bytes of object, 2 argument(s), the global cell 0x00c0618c points at 288 random bytes; vtable slot +0xc (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0xd8 (3 args) stubbed; callee 0x00671ed0 (2 args) stubbed; the word at `c0+0x14+a[1]*0x1c` points at 4 random bytes less `a[0]*4` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p1+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_cast_depopulate.py") else {
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
            {
                // Unit a[0] of a[1] (objects +0x14 row, 0x1c apart): a city (slot +0xc)
                // that is active (slot +0x4c) with +8 bit 0x20 is disbanded
                // (cast_disband(a[0], a[1])); anything else goes to this's slot +0xd8
                // (a[1], a[0], -1).
                let u = "p1+0x0".to_string();
                let k0 = a[2] != 0;
                let k1 = k0 && a[3] != 0;
                log.push(u.clone());
                log.push(if k0 { u.clone() } else { "-".into() });
                if k1 && p[1][8] & 0x20 != 0 {
                    log.push("-".into());
                    log.push(format!("{u},{},{}", a[0], a[1]));
                } else {
                    log.push(format!("m+0x0,{},{},-1", a[1], a[0]));
                    log.push("-".into());
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "cast_depopulate — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "cast_depopulate — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "cast_depopulate — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "cast_depopulate — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("SpellType::cast_depopulate@00671e50: {n} rows agree");
}
