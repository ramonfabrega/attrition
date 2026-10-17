//! `MiscAccess::world_to_screen@00583810` under the emulator against the port (item 2801; tools/sweep_gen.py, obj with windows).

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

/// `MiscAccess::world_to_screen@00583810`, run by `tools/emu/sweep_world_to_screen_a.py` under unicorn
/// (tools/emu): 16 random bytes of object, 5 argument(s), the global cell 0x00c06210 points at 320 random bytes; argument 3 names 8 random bytes; callee 0x00583840 (5 args) stubbed; stub 0 answered from [0, 1, -1, 320, 240] (90%); `__cdecl`: the caller pops the arguments; stub 0 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; `edx` the address of window `p0`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_to_screen_a.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
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
                // world_to_console (stub 0, cdecl, handed `ecx`, `edx` and the five arguments) answers; the word `edx`
                // names then gains the signed short at +0x12a of the console (the cell at 0xc06210).
                log.push(format!("m+0x0,p0+0x0,{},{},{},p0+0x0,{}", a[0], a[1], a[2], a[4]));
                let k = i32::from(i16::from_le_bytes([c[0][0x12a], c[0][0x12b]]));
                let w = rd(&p[0], 0).wrapping_add(k);
                wr(&mut p[0], 0, w);
                i64::from(a[5])
            }
        };
        assert_eq!(got, want_eax, "world_to_screen — eax, row `{line}`");
        assert_eq!(log, want_log, "world_to_screen — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "world_to_screen — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "world_to_screen — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "world_to_screen — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("MiscAccess::world_to_screen@00583810: {n} rows agree");
}
