//! `MiscAccess::coord_vis@00583aa0` under the emulator against the port (item 2336; tools/sweep_gen.py, obj with windows).

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

/// `MiscAccess::coord_vis@00583aa0`, run by `tools/emu/sweep_misc_access_coord_vis_4.py` under unicorn
/// (tools/emu): 16 random bytes of object, 2 argument(s), the global cell 0x00c06218 points at 16 random bytes; callee 0x00866560 (4 args) stubbed; callee 0x00583840 (5 args) stubbed; stub 1 answered from [0, 1, -1, 256, 305419776] (90%); `__cdecl`: the caller pops the arguments; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 writes a value in -100000..100000 through its argument 0, printed after the answers (`o` in a place); stub 1 is `__cdecl`: the caller pops its arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_misc_access_coord_vis_4.py") else {
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
                // The terrain (the record cell 0xc06218 names) finds the coordinate's height into a
                // local (stub 0, `o` the height it writes), and `world_to_console` (stub 1, `__cdecl`)
                // is handed the coordinate, that height and -1, -1, its `ecx` a second local: on
                // screen when it answers 0.
                log.push(format!("c0+0x0,sp-0x14,{},{},1", a[0], a[1]));
                log.push(format!("sp-0x18,{},{},{},-1,-1", a[0], a[1], a[4]));
                i64::from(a[3] == 0)
            }
        };
        assert_eq!(got & 0xff, want_eax & 0xff, "coord_vis — al, row `{line}`");
        assert_eq!(
            log, want_log,
            "coord_vis — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "coord_vis — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "coord_vis — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "coord_vis — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("MiscAccess::coord_vis@00583aa0: {n} rows agree");
}
