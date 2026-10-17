//! `CommandPackage::process_sell@009468a0` under the emulator against the port (item 3014; tools/sweep_gen.py, obj with windows).

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

/// `CommandPackage::process_sell@009468a0`, run by `tools/emu/sweep_command_package_process_sell.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0; the global cell 0x00c06378 points at 32 random bytes; the global cell 0x00c061ec points at 1536 random bytes; 8 random bytes at 0x00eb13a8; argument 0 names 16 random bytes; the import cell 0x00ac543c holds a stub (2 args); callee 0x004b7910 (8 args) stubbed; callee 0x00a3ab00 (10 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x006db810 (1 args) stubbed; callee 0x006d5410 (0 args) stubbed; callee 0x006d52c0 (0 args) stubbed; callee 0x006cfaf0 (0 args) stubbed; callee 0x006cfc60 (1 args) stubbed; stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 6 answered from [0, 0, 0, 1] (90%); stub 8 answered from [0, 0, 0, 0] (90%); stub 8 answers 5 values in call order, the last from [1, 2], printed after the answers; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; the stack 256 bytes under the call is zeroed before each row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_package_process_sell.py") else {
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
            // The stubs' answers are `a[1]` strrchr, `a[2]` the logger, `a[3]` the say, `a[4..8]` the four tests and `tell`, `a[9]` do_sell's plain one, `a[10..15]` its sequence.
            // The log (strrchr's answer picks the file name; the tags are the address of `who`), `Log::say` round
            // `game_log`'s type and detail (put back unless the detail was negative, then 1), then on the leader
            // record `leaders + who * 0x6eec`: out at once unless `has_tribe_bonus(4)` or `has_preq(685)`; with a
            // market, a nuke embargo tells and stops, else `do_sell(good)` up to 99999 times (`flags` bit 1), else
            // 5 times (bit 0), else once, until one answers nonzero.
            let file = if a[1] == 0 {
                11510360
            } else {
                a[1].wrapping_add(1)
            };
            log.push(format!(
                "0,{file},1864,p0+0x1,11510208,p0+0x1,p0+0x5,p0+0x9,c1+0x550"
            ));
            let (who, good, flags) = (rd(&p[0], 1), rd(&p[0], 5), rd(&p[0], 9));
            log.push(format!(
                "15405920,{},15418236,{},{},0,0,0,0,0,2",
                rd(&c[0], 0x10).wrapping_add(0x34d0),
                good,
                rd(&c[1], 0x550),
            ));
            if rd(&c[2], 4) < 0 {
                wr(&mut c[2], 4, 1);
            }
            let leader = 14918544i32.wrapping_add(who.wrapping_mul(28396));
            let bonus = format!("{leader},4");
            let (mut preq, mut market) = ("-".to_string(), "-".to_string());
            let (mut nuke, mut tell, mut sell) =
                ("-".to_string(), "-".to_string(), "-".to_string());
            if a[4] == 0 {
                preq = format!("{leader},685");
            }
            if a[4] != 0 || a[5] != 0 {
                market = leader.to_string();
                if a[6] != 0 {
                    nuke = leader.to_string();
                    if a[7] != 0 {
                        tell = leader.to_string();
                    } else {
                        let count: i64 = if flags & 2 != 0 {
                            99999
                        } else {
                            i64::from(flags & 1) * 4 + 1
                        };
                        let mut calls: Vec<String> = Vec::new();
                        for k in 0..count {
                            calls.push(format!("{leader},{good}"));
                            if a[10 + (k as usize).min(4)] != 0 {
                                break;
                            }
                        }
                        sell = calls.join("/");
                    }
                }
            }
            log.extend([bonus, preq, market, nuke, tell, sell]);
            13
        };
        assert_eq!(got, want_eax, "process_sell — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "process_sell — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process_sell — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "process_sell — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "process_sell — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CommandPackage::process_sell@009468a0: {n} rows agree");
}
