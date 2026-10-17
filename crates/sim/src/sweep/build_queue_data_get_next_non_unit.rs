//! `BuildQueueData::get_next_non_unit@00630b10` under the emulator against the port (item 2837; tools/sweep_gen.py, obj with windows).

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

/// `BuildQueueData::get_next_non_unit@00630b10`, run by `tools/emu/sweep_build_queue_data_get_next_non_unit.py` under unicorn
/// (tools/emu): 16 random bytes of object, 2 argument(s), vtable slot +0xc (0 args) stubbed; callee 0x006e0c80 (1 args) stubbed; the word at `0xe85ddc` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 8 random bytes less `0` (placed); the word at `p0+4` points at 8 random bytes less `0` (placed); the word at `p0+8` points at 8 random bytes less `0` (placed); the word at `p0+12` points at 8 random bytes less `0` (placed); the word at `m+4` points at 200 random bytes less `0` (placed); slot +0xc laid as 0x004707e0 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [1, 1, 1, 0] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_queue_data_get_next_non_unit.py") else {
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
            // The first queue entry from the second up to `min(a[0], size)` (size the word at `m+0`,
            // entry `k` at `+0x14·k` of the queue behind `m+4`, window p5) whose type (the short at
            // `+4` indexes the table of record pointers, records p1..p4) is not a unit type of ids
            // `0x32..0x19d` (slot +0xc laid as `TypeData::is_unit_type` is inlined, a[4] set; else
            // its stub answers a[2]) or whose owner does not have the tech (callee `0x6e0c80`, a[3]);
            // -1 when every entry holds (item 2837).
            let limit = a[0].min(rd(m, 0));
            let mut found = -1;
            for k in 1..limit {
                let at = 4 + 0x14 * k as usize;
                let idx = i16::from_le_bytes([p[5][at], p[5][at + 1]]) as usize;
                let id = rd(&p[1 + idx], 4);
                if a[4] != 0 {
                    if !(0x32..0x19e).contains(&id) {
                        found = k;
                        break;
                    }
                } else {
                    log.push(format!("s0:p{}+0x0", 1 + idx));
                    if a[2] == 0 {
                        found = k;
                        break;
                    }
                }
                if a[3] == 0 {
                    found = k;
                    break;
                }
            }
            if log.is_empty() {
                log.push("-".into());
            }
            i64::from(found)
        };
        assert_eq!(got, want_eax, "get_next_non_unit — eax, row `{line}`");
        assert_eq!(log, want_log, "get_next_non_unit — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_next_non_unit — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_next_non_unit — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_next_non_unit — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildQueueData::get_next_non_unit@00630b10: {n} rows agree");
}
