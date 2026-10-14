//! `GuyData::get_speed@005de410` under the emulator against the port (item 2341; tools/sweep_gen.py, obj with windows).

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

/// `GuyData::get_speed@005de410`, run by `tools/emu/sweep_guy_data_get_speed.py` under unicorn
/// (tools/emu): 192 random bytes of object, 0 argument(s), vtable slot +0x17c (3 args) stubbed; callee 0x0060b030 (0 args) stubbed; vtable slot +0x2c (0 args) stubbed; the word at `0xc0aec0+i8(m,0xa1)*0x1c` points at 4 random bytes less `i16(m,0x8c)*4` (placed); the word at `p0+0` points at 224 random bytes less `0` (placed); the word at `p1+0x18` points at 4 random bytes less `0x304` (placed); the word at `p1+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p3+4` points at 16 random bytes less `0` (placed); the word at `p4+8` points at 4 random bytes less `0` (placed, null a third of the rows); stub 2 answered from [0, 1] (90%); stub 1 answers the address of 64 random bytes, a `p` window after the places; the stubbed vtable laid at `p1+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_guy_data_get_speed.py") else {
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
            // `p[0]` the units row's slot, `p[1]` the unit (the stubbed
            // vtable at `+0`), `p[2]` its type's `squad_size` (`+0x304`),
            // `p[3]` the list's head node (`+0xdc`), `p[4]` its next node,
            // `p[5]` that node's order, `p[6]` the order `get_order`
            // answers (stub 1). The rewind is the engine's iterator, held
            // here as the unit's bytes `+0xd4`, `+0xcc`, `+0xd0`.
            let guy_num = i32::from(m[0xa2] as i8);
            let mut cx = (m, p, &mut log);
            let v = crate::orders::guy_speed_steps(
                &mut cx,
                guy_num,
                |(m, _, log)| {
                    log.push(format!("s0:p1+0x0,{},{},1", rd(m, 0xc), rd(m, 0x10)));
                    a[0]
                },
                |(_, p, _)| rd(&p[2], 0),
                |(_, p, _)| rd(&p[1], 0xdc) != 0,
                |(_, p, _)| {
                    let next = rd(&p[3], 4);
                    let order = rd(&p[4], 8);
                    let byte = p[4][0xc];
                    wr(&mut p[1], 0xd4, next);
                    wr(&mut p[1], 0xcc, order);
                    p[1][0xd0] = byte;
                    order != 0
                },
                |(_, _, log)| {
                    log.push("s1:p1+0x0".into());
                    log.push("s2:p6+0x0".into());
                    a[2] != 0
                },
            );
            i64::from(v)
        };
        assert_eq!(got, want_eax, "get_speed — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "get_speed — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_speed — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_speed — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_speed — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("GuyData::get_speed@005de410: {n} rows agree");
}
