//! `ObjectData::get_inside@00651a80` under the emulator against the port (item 3024; tools/sweep_gen.py, obj with windows).

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

/// `ObjectData::get_inside@00651a80`, run by `tools/emu/sweep_object_data_get_inside.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p20 the place(s), in flag order; argument 0 names 4 random bytes (null a third of the rows); vtable slot +0x18 (0 args) stubbed; the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `0xc0aec0+0x1c` points at 16 random bytes less `0` (placed); the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `0xc0ab84+0x1c` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 192 random bytes less `0` (placed); the word at `p1+4` points at 192 random bytes less `0` (placed); the word at `p1+8` points at 192 random bytes less `0` (placed); the word at `p1+12` points at 192 random bytes less `0` (placed); the word at `p2+0` points at 192 random bytes less `0` (placed); the word at `p2+4` points at 192 random bytes less `0` (placed); the word at `p2+8` points at 192 random bytes less `0` (placed); the word at `p2+12` points at 192 random bytes less `0` (placed); the word at `p3+0` points at 16 random bytes less `0` (placed); the word at `p3+4` points at 16 random bytes less `0` (placed); the word at `p3+8` points at 16 random bytes less `0` (placed); the word at `p3+12` points at 16 random bytes less `0` (placed); the word at `p4+0` points at 16 random bytes less `0` (placed); the word at `p4+4` points at 16 random bytes less `0` (placed); the word at `p4+8` points at 16 random bytes less `0` (placed); the word at `p4+12` points at 16 random bytes less `0` (placed); stub 0 answered from [0, 1, 1, 1] (90%); stub 0 answers 8 values in call order, the last from [0], printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_get_inside.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `this+9` the owner, `this+0xa` its unit index; p[5 + 4 * owner + index] the unit
            // (its link short `+0x82` into the container, the container's player byte `+0xb4`),
            // p[13 + 4 * owner + index] the object whose slot 0x18 (`a[2..10]`, one answer a
            // call, 0 past the eighth) says the container is itself inside another. `a[0]` the
            // out pointer (0: null), written the player of the last container.
            let out = a[0] != 0;
            if out {
                wr(&mut p[0], 0, -1);
            }
            let mut calls: Vec<String> = Vec::new();
            let mut turn = 0usize;
            let sx16 = |u: &[u8]| i32::from(i16::from_le_bytes([u[0x82], u[0x83]]));
            let sx8 = |u: &[u8]| i32::from(u[0xb4] as i8);
            let mut ask = |calls: &mut Vec<String>, win: Option<usize>| {
                calls.push(win.map_or("m+0x0".to_string(), |w| format!("p{w}+0x0")));
                let v = if turn < 8 { a[2 + turn] } else { 0 };
                turn += 1;
                v != 0
            };
            let got = 'g: {
                if !ask(&mut calls, None) {
                    break 'g -1;
                }
                let owner = m[9] as usize;
                let index = i16::from_le_bytes([m[0xa], m[0xb]]) as usize;
                let unit = &p[5 + 4 * owner + index];
                let mut link = sx16(unit);
                let mut who = sx8(unit);
                if link < 0 {
                    break 'g -1;
                }
                loop {
                    if !ask(&mut calls, Some(13 + 4 * who as usize + link as usize)) {
                        break;
                    }
                    let unit = &p[5 + 4 * who as usize + link as usize];
                    let next = sx16(unit);
                    if next < 0 {
                        break;
                    }
                    who = sx8(unit);
                    link = next;
                }
                if out {
                    wr(&mut p[0], 0, who);
                }
                link
            };
            log.push(calls.join("/"));
            i64::from(got)
        };
        assert_eq!(got, want_eax, "get_inside — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "get_inside — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_inside — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_inside — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_inside — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 500, "only {n} rows");
    eprintln!("ObjectData::get_inside@00651a80: {n} rows agree");
}
