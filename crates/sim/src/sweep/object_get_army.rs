//! `Object::get_army@00649d70` under the emulator against the port (item 2743; tools/sweep_gen.py, obj with windows).

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

/// `Object::get_army@00649d70`, run by `tools/emu/sweep_object_get_army.py` under unicorn
/// (tools/emu): 32 random bytes of object, 0 argument(s), vtable slot +0x18 (0 args) stubbed; callee 0x0070f8f0 (3 args) stubbed; callee 0x006f9a50 (1 args) stubbed; the word at `0xe85f20` points at 10064 random bytes less `0` (placed); the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `0xc0aedc` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 144 random bytes less `0` (placed); the word at `p1+4` points at 144 random bytes less `0` (placed); the word at `p1+8` points at 144 random bytes less `0` (placed); the word at `p1+12` points at 144 random bytes less `0` (placed); the word at `p2+0` points at 144 random bytes less `0` (placed); the word at `p2+4` points at 144 random bytes less `0` (placed); the word at `p2+8` points at 144 random bytes less `0` (placed); the word at `p2+12` points at 144 random bytes less `0` (placed); the word at `0xc09710` points at 16 random bytes less `0` (placed); the word at `0xc0972c` points at 16 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_get_army.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `Object::get_army`: an object that answers its slot 0x18 zero has none; else the object this one
            // names in the per-player table (`who` = byte +9, index = short +0xa) has a record index at +0x80;
            // that record of the 0x9d4-byte array is asked (`0x70f8f0`, handed the index, `who` and 1), and its
            // word at +8 indexes the player's army table, whose entry is asked (`0x6f9a50`, handed the record
            // index) — the answer is that army index when the second callee says yes.
            let (mut t1, mut t2) = ("-".to_string(), "-".to_string());
            let r: i32 = (|| {
                if a[0] == 0 {
                    return -1;
                }
                let who = m[9] as usize;
                let idx = (rd(m, 8) >> 16) as i16 as usize;
                let b = i32::from(rd(&p[3 + 4 * who + idx], 0x80) as i16);
                if b < 0 {
                    return -1;
                }
                let off = b as usize * 0x9d4;
                t1 = format!("p0+0x{off:x},{idx},{who},1");
                if a[1] == 0 {
                    return -1;
                }
                let esi = rd(&p[0], off + 8);
                if esi < 0 {
                    return -1;
                }
                t2 = format!("{},{b}", rd(&p[11 + who], 4 * esi as usize));
                if a[2] != 0 { esi } else { -1 }
            })();
            log.push(t1);
            log.push(t2);
            i64::from(r)
        };
        assert_eq!(got, want_eax, "get_army — eax, row `{line}`");
        assert_eq!(log, want_log, "get_army — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_army — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_army — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_army — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Object::get_army@00649d70: {n} rows agree");
}
