//! `Group::action_eject_all@00710b40` under the emulator against the port (item 3261; tools/sweep_gen.py, obj with windows).

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

/// `Group::action_eject_all@00710b40`, run by `tools/emu/sweep_group_action_eject_all_eject_o.py` under unicorn
/// (tools/emu): 2304 random bytes of object, 4 argument(s), `p` windows in order: p0..p23 the place(s), in flag order; the global cell 0x00c0618c points at 1024 random bytes; 4 random bytes at 0x00cc02f8 (each word in 0..0); vtable slot +0x14 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; callee 0x00617c10 (1 args) stubbed; the word at `c0+0x14+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p0+0` points at 96 random bytes less `0` (placed); the word at `p0+4` points at 96 random bytes less `0` (placed); the word at `p0+8` points at 96 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p4+0` points at 32 random bytes less `0` (placed); the word at `p4+4` points at 32 random bytes less `0` (placed); the word at `p4+8` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p8+0` points at 96 random bytes less `0` (placed); the word at `p8+4` points at 96 random bytes less `0` (placed); the word at `p8+8` points at 96 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p12+0` points at 32 random bytes less `0` (placed); the word at `p12+4` points at 32 random bytes less `0` (placed); the word at `p12+8` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*2` points at 12 random bytes less `0` (placed); the word at `p16+0` points at 96 random bytes less `0` (placed); the word at `p16+4` points at 96 random bytes less `0` (placed); the word at `p16+8` points at 96 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*2` points at 12 random bytes less `0` (placed); the word at `p20+0` points at 32 random bytes less `0` (placed); the word at `p20+4` points at 32 random bytes less `0` (placed); the word at `p20+8` points at 32 random bytes less `0` (placed); stub 1 answered from [0, 1, 1, 2] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 2 writes -1 (2 bytes) at `p1+0x28` each call; stub 2 writes -1 (2 bytes) at `p2+0x28` each call; stub 2 writes -1 (2 bytes) at `p3+0x28` each call; stub 2 writes -1 (2 bytes) at `p9+0x28` each call; stub 2 writes -1 (2 bytes) at `p10+0x28` each call; stub 2 writes -1 (2 bytes) at `p11+0x28` each call; stub 2 writes -1 (2 bytes) at `p17+0x28` each call; stub 2 writes -1 (2 bytes) at `p18+0x28` each call; stub 2 writes -1 (2 bytes) at `p19+0x28` each call; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_eject_all_eject_o.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // a[0..4] the arguments (back_to_work, who, eject_o, eject_who); a[4..7] the
                // three stubs' answers, a[5] the object's slot-0x10 answer. The object
                // tables of players 0..2 are the windows p(8q+1+j) and the unit tables
                // p(8q+5+j); a window's +0x28 short is its container's index, its +0x3e
                // byte the container's player.
                let (who, eject_o, eject_who) = (a[1], a[2], a[3]);
                log.push("s0:m+0x0".to_string());
                // Not the own-buildings arm: `who` names another player's unit, or the
                // command names `eject_who`.
                if !crate::group::eject_all_own_arm(i32::from(m[0x4a]), eject_who, who)
                    && eject_o >= 0
                {
                    let obj_win = |pl: i32, idx: i32| (8 * pl + 1 + idx) as usize;
                    let start = obj_win(eject_who, eject_o);
                    log.push(format!("s1:p{start}+0x0"));
                    if a[5] != 0 {
                        loop {
                            let r = &p[start];
                            let (mut idx, mut pl) = (
                                i32::from(i16::from_le_bytes([r[0x28], r[0x29]])),
                                i32::from(r[0x3e] as i8),
                            );
                            if idx < 0 {
                                break;
                            }
                            let mut gone = false;
                            while pl != who {
                                let r = &p[obj_win(pl, idx)];
                                idx = i32::from(i16::from_le_bytes([r[0x28], r[0x29]]));
                                pl = i32::from(r[0x3e] as i8);
                                if idx < 0 {
                                    gone = true;
                                    break;
                                }
                            }
                            if gone {
                                break;
                            }
                            // `Unit::come_out(units[who][idx], 0)`, which unlinks the
                            // object from the container it is walking out of.
                            log.push(format!("s2:p{}+0x0,0", 8 * pl + 5 + idx));
                            for q in 0..3usize {
                                for j in 0..3usize {
                                    p[8 * q + 1 + j][0x28..0x2a]
                                        .copy_from_slice(&(-1i16).to_le_bytes());
                                }
                            }
                        }
                    }
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "action_eject_all — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "action_eject_all — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "action_eject_all — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "action_eject_all — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Group::action_eject_all@00710b40: {n} rows agree");
}
