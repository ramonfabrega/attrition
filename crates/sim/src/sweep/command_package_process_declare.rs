//! `CommandPackage::process_declare@009472b0` under the emulator against the port (item 2941; tools/sweep_gen.py, obj with windows).

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

/// `CommandPackage::process_declare@009472b0`, run by `tools/emu/sweep_command_package_process_declare.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), the global cell 0x00c06378 points at 32 random bytes; the global cell 0x00c061ec points at 1536 random bytes; 8 random bytes at 0x00eb13a8; argument 0 names 32 random bytes; the import cell 0x00ac543c holds a stub (2 args); callee 0x004b7910 (8 args) stubbed; callee 0x00a3ab00 (10 args) stubbed; callee 0x006dab50 (4 args) stubbed; stub 0 answered from [0, 4194304] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_package_process_declare.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The log (strrchr's answer picks the file name), `Log::say` round `game_log`'s type and detail (put back
            // unless the detail was negative, then 1), then the leader action on the slot `who` of the 0x6eec-byte
            // leader table at 0xe3a390; its arguments the command's dwords.
            let file = if a[1] == 0 {
                0xaf9b30
            } else {
                a[1].wrapping_add(1)
            };
            log.push(format!(
                "0,{file},1783,p0+0x1,11508368,p0+0x1,p0+0x5,p0+0x9,c1+0x550"
            ));
            log.push(format!(
                "15405920,{},15418236,{},{},{},0,0,0,0,3",
                rd(&c[0], 0x10).wrapping_add(0x341c),
                rd(&p[0], 5),
                rd(&p[0], 9),
                rd(&c[1], 0x550)
            ));
            if rd(&c[2], 4) < 0 {
                wr(&mut c[2], 4, 1);
            }
            log.push(format!(
                "{},{},{},{},{}",
                rd(&p[0], 1).wrapping_mul(0x6eec).wrapping_add(0xe3a390),
                rd(&p[0], 5),
                rd(&p[0], 9),
                0,
                0
            ));
            13
        };
        assert_eq!(got, want_eax, "process_declare — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "process_declare — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process_declare — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "process_declare — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "process_declare — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CommandPackage::process_declare@009472b0: {n} rows agree");
}
