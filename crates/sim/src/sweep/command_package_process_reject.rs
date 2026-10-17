//! `CommandPackage::process_reject@00946e90` under the emulator against the port (item 2959; tools/sweep_gen.py, obj with windows).

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

/// `CommandPackage::process_reject@00946e90`, run by `tools/emu/sweep_command_package_process_reject.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p32 sparse table 0 (8 rows × 4 slice(s), row-major); the global cell 0x00c06378 points at 32 random bytes; the global cell 0x00c061ec points at 1536 random bytes; 8 random bytes at 0x00eb13a8; argument 0 names 16 random bytes; the import cell 0x00ac543c holds a stub (2 args); callee 0x004b8350 (7 args) stubbed; callee 0x00a3ab00 (10 args) stubbed; callee 0x006d03c0 (2 args) stubbed; stub 0 answered from [0, 4194304] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 0 is `__cdecl`: the caller pops its arguments; 8 rows 0x6eec apart at `@0xe3a390`, slices +0x692c:0x4, +0x6988:0x4, +0x69e4:0x4, +0x6a40:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_package_process_reject.py") else {
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
            // unless the detail was negative, then 1), then `Leader::action_respond` on the leader `who` (0x6eec
            // bytes apart at 0xe3a390) with `whom`, and 0 where that leader's word at 0x692c + whom*0x5c is 1, else 2.
            let file = if a[1] == 0 {
                0xaf9e18
            } else {
                a[1].wrapping_add(1)
            };
            log.push(format!(
                "0,{file},1819,p0+0x1,11509136,p0+0x1,p0+0x5,c1+0x550"
            ));
            log.push(format!(
                "15405920,{},15418236,{},{},0,0,0,0,0,2",
                rd(&c[0], 0x10).wrapping_add(0x346c),
                rd(&p[0], 5),
                rd(&c[1], 0x550)
            ));
            if rd(&c[2], 4) < 0 {
                wr(&mut c[2], 4, 1);
            }
            let (who, whom) = (rd(&p[0], 1), rd(&p[0], 5));
            let word = rd(&p[1 + who as usize * 4 + whom as usize], 0);
            let act = if word == 1 { 0 } else { 2 };
            for r in 0..8 {
                log.push(if r == who {
                    format!("r+0x0,{whom},{act}")
                } else {
                    "-".to_string()
                });
            }
            log.push("-".to_string());
            9
        };
        assert_eq!(got, want_eax, "process_reject — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "process_reject — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process_reject — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "process_reject — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "process_reject — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CommandPackage::process_reject@00946e90: {n} rows agree");
}
