//! `Build::process@0061edf0` under the emulator against the port (item 3403; tools/sweep_gen.py, obj with windows).

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

/// `Build::process@0061edf0`, run by `tools/emu/sweep_build_process_city.py` under unicorn
/// (tools/emu): 136 random bytes of object, 0 argument(s), `p` windows in order: p0..p55 sparse table 0 (8 rows × 7 slice(s), row-major), p56..p57 sparse table 1 (1 rows × 2 slice(s), row-major), p58 sparse table 2 (1 rows × 1 slice(s), row-major), p59 sparse table 3 (1 rows × 1 slice(s), row-major), p60..p62 sparse table 4 (1 rows × 3 slice(s), row-major), p63..p66 sparse table 5 (1 rows × 4 slice(s), row-major), p67..p74 sparse table 6 (8 rows × 1 slice(s), row-major), p75..p76 sparse table 7 (1 rows × 2 slice(s), row-major), p77..p99 the place(s), in flag order, p100 stub 28's answer window; callee 0x00640450 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x1b4 (1 args) stubbed; vtable slot +0x90 (0 args) stubbed; vtable slot +0x94 (0 args) stubbed; vtable slot +0x180 (0 args) stubbed; vtable slot +0x70 (0 args) stubbed; vtable slot +0x164 (0 args) stubbed; vtable slot +0x11c (1 args) stubbed; vtable slot +0x168 (3 args) stubbed; callee 0x006da000 (0 args) stubbed; callee 0x0062d1d0 (0 args) stubbed; callee 0x0065ca80 (13 args) stubbed; callee 0x006276a0 (2 args) stubbed; callee 0x00471360 (0 args) stubbed; callee 0x006ec000 (0 args) stubbed; callee 0x005e4ff0 (4 args) stubbed; callee 0x00646b00 (2 args) stubbed; callee 0x00738e90 (0 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x00739340 (0 args) stubbed; callee 0x00738410 (0 args) stubbed; callee 0x0065c6b0 (13 args) stubbed; callee 0x00713e80 (1 args) stubbed; callee 0x00714350 (4 args) stubbed; callee 0x0070ec30 (0 args) stubbed; callee 0x00a1ee30 (0 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; callee 0x0046e620 (0 args) stubbed; callee 0x008c2cc0 (1 args) stubbed; callee 0x007e9b70 (9 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; callee 0x0063c580 (6 args) stubbed; the word at `0xc09970+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p77+0` points at 104 random bytes less `0` (placed); the word at `p77+4` points at 104 random bytes less `0` (placed); the word at `p77+8` points at 104 random bytes less `0` (placed); the word at `0xc09970+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p81+0` points at 104 random bytes less `0` (placed); the word at `p81+4` points at 104 random bytes less `0` (placed); the word at `p81+8` points at 104 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p85+0` points at 16 random bytes less `0` (placed); the word at `p85+4` points at 16 random bytes less `0` (placed); the word at `p85+8` points at 16 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p89+0` points at 16 random bytes less `0` (placed); the word at `p89+4` points at 16 random bytes less `0` (placed); the word at `p89+8` points at 16 random bytes less `0` (placed); the word at `0xc0aec0-0x1c` points at 12 random bytes less `0` (placed); the word at `p93+0` points at 16 random bytes less `0` (placed); the word at `p93+4` points at 16 random bytes less `0` (placed); the word at `p93+8` points at 16 random bytes less `0` (placed); the word at `m+0x18` points at 496 random bytes less `0` (placed); the word at `p6+0` points at 232 random bytes less `0` (placed); the word at `p13+0` points at 232 random bytes less `0` (placed); stub 1 answered from [1, 1, 1, 0] (90%); stub 6 answered from [0, 1, 1, 1, 2] (90%); stub 11 answered from [0, 1, 1, 1, 1] (90%); stub 13 answered from [0, 0, 0, 1] (90%); stub 14 answered from [0, 1, 1, 1, 1] (90%); stub 17 answered from [-1, 0, 1, 2] (90%); stub 19 answered from [0, 1] (90%); stub 22 answered from [-1, -1, 0, 5, -3] (90%); stub 28 answers the address of 32 random bytes, a `p` window after the places; the word at `m+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 12 answers 2 values in call order, printed after the answers; stub 3 answers 0..0 every row; stub 4 answers 0..0 every row; stub 5 answers 0..0 every row; stub 8 answers 0..20 every row; stub 10 answers 0..3 every row; stub 15 answers 0..3 every row; stub 20 answers 0..7 every row; stub 21 answers 0..20 every row; stub 12 answers -2..2 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 27 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 28 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 29 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 30 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 31 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 32 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 4096 bytes under the call is zeroed before each row; 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x8:0x4, +0x3f8:0x4, +0x940:0x4, +0x5a98:0x4, +0x6928:0x4, +0x6eb8:0x10 (`p` windows after the pointed buffers, before the places); 1 rows 0x560 apart at `0xc061ec`, slices +0x24:0x4, +0x550:0x4 (`p` windows after the pointed buffers, before the places); 1 rows 0x2a0 apart at `0xc06210`, slices +0x298:0x4 (`p` windows after the pointed buffers, before the places); 1 rows 0x10 apart at `0xc06208`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 1 rows 0xa04 apart at `0xc8cd00`, slices +0x9d8:0x4, +0x9ec:0x4, +0xa00:0x4 (`p` windows after the pointed buffers, before the places); 1 rows 0xd40 apart at `0xc061f0`, slices +0x664:0x4, +0xc10:0x4, +0xc9c:0x4, +0xd2c:0x8 (`p` windows after the pointed buffers, before the places); 8 rows 0x20 apart at `0xc061b4`, slices +0x1c:0x4 (`p` windows after the pointed buffers, before the places); 1 rows 0x210 apart at `0xc0618c`, slices +0x0:0x4, +0x200:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p97+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_process_city.py") else {
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
            // hand: port
            // `Build::process@0061edf0` has no crate method: the crate carries its parts scattered
            // (`Sim::process_building` the head, `process_building_combat` the tower, `process_building_tail`
            // this slice's city block, `nations::{kremlin_spy, persian_market_caravan, terra_cotta_army}`,
            // `gather::gather_region_building`, `roads::regen_roads_due`), each reading a `Sim` and none
            // the `Build` record, so the slice is inline. Window map: m the building; leader row r's slices
            // p(7r+s): 0 flags, 1 `+8`, 2 `+0x3f8`, 3 `+0x940`, 4 queued word, 5 team colour, 6 the stamps and
            // the pointer `+0x6eb8` (p98, p99 the record behind it); p56 team style, p57 frame, p58 console who,
            // p59 message window, p60..p62 strings, p63 turk, p64 heal rate, p65 speed, p66 reassimilation +
            // timer, p67.. leader options; p75 objects base, p76 objects `+0x200`; cities (player K) array
            // p77+4K, records p78+4K+i; units (player K) array p85+4K, records p86+4K+i; p97 the type; p100 the
            // bubble. **Domain** (asserted): a finished building, the first reminder's gate shut, no attack,
            // ejection, launch or rebuild flags, the leader's other fields zero.
            //
            // The shadow `sh` is the four stack slots under the frame's `esp`, which `ObjectsData::find`'s
            // three uninitialised arguments (`sub esp, 0xc`) read: whatever the last call to touch each left.
            const IMAGE: i32 = 0xb4_2174;
            const KEY: i32 = 0x6_3637;
            fn emit(
                log: &mut Vec<String>,
                sh: &mut [String; 5],
                rec: Option<usize>,
                ecx: &str,
                args: &[String],
                ret: i32,
            ) {
                if let Some(s) = rec {
                    let mut t = format!("s{s}:{ecx}");
                    for a in args {
                        t.push(',');
                        t.push_str(a);
                    }
                    log.push(t);
                }
                let n = args.len();
                for (k, v) in args.iter().enumerate() {
                    if n - k <= 4 {
                        sh[n - k] = v.clone();
                    }
                }
                if n + 1 <= 4 {
                    sh[n + 1] = ret.to_string();
                }
            }
            let s = |v: i32| v.to_string();
            let short = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
            let frame = rd(&p[57], 0);
            let style = i32::from(p[56][0]);
            let console = rd(&p[58], 0);
            let image = rd(m, 0) == IMAGE;
            let who = i32::from(m[9]);
            assert!(frame >= 0 && who < 2, "frame, who");
            let w = who as usize;
            let lead = |r: usize| format!("p{}+0x0", 7 * r);
            let mut sh: [String; 5] = std::array::from_fn(|_| "0".to_string());
            emit(&mut log, &mut sh, Some(0), "m+0x0", &[], 0x61_ee16);
            let active = if image {
                m[8] & 4 != 0
            } else {
                emit(&mut log, &mut sh, Some(1), "m+0x0", &[], 0x61_ee2f);
                a[1] != 0
            };
            if active {
                if short(m, 0x38) != 0 {
                    let v = (short(m, 0x38) - 1) as i16;
                    m[0x38..0x3a].copy_from_slice(&v.to_le_bytes());
                }
                assert_eq!(
                    short(m, 0x60) & 0xfeff,
                    0,
                    "+0x60's low half is 0 or the roads flag"
                );
                assert_eq!(rd(&p[97], 0x1e8), 0, "no attack");
                if !image {
                    emit(&mut log, &mut sh, Some(5), "m+0x0", &[], 0x61_efa9);
                    assert_eq!(a[5], 0);
                }
                emit(&mut log, &mut sh, Some(2), "m+0x0", &[s(0)], 0x61_f3c5);
                emit(&mut log, &mut sh, Some(3), "p97+0x0", &[], 0x61_f3d0);
                assert_eq!(a[3], 0);
                assert!(
                    ![0x21a, 0x1b6, 0x1b4, 0x211].contains(&rd(&p[97], 4)),
                    "wonder arms excluded"
                );
                let phase = short(m, 0xa);
                let x = frame + phase;
                assert_ne!(
                    x % 256,
                    0,
                    "the first reminder's gate is shut in this slice"
                );
                let (xk, yk) = (rd(m, 0x10) ^ KEY, rd(m, 0x14) ^ KEY);
                let mut seqn = 0usize;
                let mut finished = false;
                if m[8] & 0x20 != 0 && short(m, 0x72) >= 0 {
                    let idx = short(m, 0x72) as usize;
                    assert!(idx < 3, "city index");
                    let ci = 78 + 4 * w + idx;
                    let city = format!("p{ci}+0x0");
                    let cflags =
                        |p: &[Vec<u8>]| i32::from(u16::from_le_bytes([p[ci][4], p[ci][5]]));
                    // The 200-frame decay of `CITY_ATTACKING` / `CITY_UNDER_ATTACK` and the counter.
                    if x % 200 == 0 {
                        let f = cflags(p);
                        let f = if f & 4 != 0 { f & !4 } else { f & !2 };
                        p[ci][4..6].copy_from_slice(&(f as u16).to_le_bytes());
                        if p[ci][0x61] != 0 {
                            p[ci][0x61] -= 1;
                        }
                    }
                    // The team alarm: a leader whose target is this player.
                    if cflags(p) & 0x10 != 0 && style == 2 {
                        for r in 0..8usize {
                            if rd(&p[7 * r], 0) & 3 == 3 {
                                emit(&mut log, &mut sh, Some(10), &lead(r), &[], 0x61_fba0);
                                if a[10] == who && m[0x62] & (1 << r) == 0 {
                                    m[0x62] |= 1 << r;
                                    emit(&mut log, &mut sh, Some(7), "m+0x0", &[], 0x61_fbd8);
                                    break;
                                }
                            }
                        }
                    }
                    // The third reminder.
                    let pw = |r: usize, k: usize| 7 * r + k;
                    if x % 16 == 0
                        && who == console
                        && rd(&p[pw(w, 2)], 0) == 1
                        && (rd(&p[98 + w], 0xe0) ^ 0x6_9587) >= 3
                        && frame > 1000
                        && rd(&p[pw(w, 3)], 0) >= 0xf
                    {
                        let stamp = rd(&p[pw(w, 6)], 4);
                        if stamp == 0 || frame - stamp > 5000 {
                            let human = if image {
                                m[0x1c] != 0
                            } else {
                                emit(&mut log, &mut sh, Some(6), "m+0x0", &[], 0x61_fc84);
                                a[6] != 0
                            };
                            if human {
                                wr(&mut p[pw(w, 6)], 4, frame);
                                emit(&mut log, &mut sh, Some(26), "sp-0x50", &[], 0x61_fcaf);
                                emit(
                                    &mut log,
                                    &mut sh,
                                    Some(27),
                                    "sp-0x50",
                                    &["p62+0x0".to_string()],
                                    0x61_fcc9,
                                );
                                emit(&mut log, &mut sh, Some(28), "sp-0x50", &[], 0x61_fcce);
                                emit(
                                    &mut log,
                                    &mut sh,
                                    Some(27),
                                    "p100+0x4",
                                    &["sp-0x50".to_string()],
                                    0x61_fce3,
                                );
                                p[100][0x1a] = who as u8;
                                let team = i32::from(p[pw(w, 5)][0]);
                                emit(
                                    &mut log,
                                    &mut sh,
                                    Some(29),
                                    &(0xe3_9ce0 + team * 0x64).to_string(),
                                    &["sp-0x3c".to_string()],
                                    0x61_fd29,
                                );
                                let neon = a[29];
                                emit(
                                    &mut log,
                                    &mut sh,
                                    Some(30),
                                    "p59+0x0",
                                    &[
                                        s(xk),
                                        s(yk),
                                        s(0),
                                        s(neon),
                                        s(-1),
                                        s(3072),
                                        s(0),
                                        "p100+0x0".to_string(),
                                        s(0),
                                    ],
                                    0x61_fd39,
                                );
                                emit(&mut log, &mut sh, Some(31), "sp-0x50", &[], 0x61_fd48);
                            } else if stamp != 0 {
                                wr(&mut p[pw(w, 6)], 4, frame - 900);
                            }
                        }
                    }
                    // The capture re-test, every 64 frames.
                    let mut edi = -1;
                    if x % 64 == 0 {
                        emit(&mut log, &mut sh, Some(11), "m+0x0", &[], 0x61_fd9c);
                        if a[11] != 0 {
                            let args = [
                                s(xk),
                                s(yk),
                                s(3),
                                s(who),
                                s(0xc00),
                                s(0),
                                s(0xd),
                                s(0),
                                s(0),
                                s(0),
                                s(0),
                                s(0),
                                s(yk),
                            ];
                            emit(&mut log, &mut sh, Some(12), "p75+0x0", &args, 0x61_fdd9);
                            edi = a[33 + seqn.min(1)];
                            seqn += 1;
                            let obj = rd(&p[76], 0);
                            if edi >= 0 && obj >= 0 {
                                emit(
                                    &mut log,
                                    &mut sh,
                                    Some(13),
                                    "m+0x0",
                                    &[s(edi), s(obj)],
                                    0x61_fdf8,
                                );
                                if a[13] != 0 {
                                    finished = true;
                                }
                            }
                        }
                        if !finished && rd(&p[pw(w, 0)], 0) & 4 == 0 && edi < 0 {
                            emit(&mut log, &mut sh, Some(14), "m+0x0", &[], 0x61_fe26);
                            if a[14] != 0 {
                                emit(&mut log, &mut sh, Some(8), "m+0x0", &[s(0)], 0x61_fe3a);
                                let half = a[8] / 2;
                                if rd(m, 0x24) > half && cflags(p) & 2 == 0 {
                                    emit(&mut log, &mut sh, Some(15), &lead(w), &[], 0x61_fe7e);
                                    if a[15] > 1 {
                                        let args = [
                                            s(xk),
                                            s(yk),
                                            s(1),
                                            s(who),
                                            s(0x1e00),
                                            s(0x200),
                                            s(1),
                                            s(0x32),
                                            s(0),
                                            s(0xc),
                                            s(0),
                                            s(0),
                                            s(yk),
                                        ];
                                        emit(
                                            &mut log,
                                            &mut sh,
                                            Some(12),
                                            "p75+0x0",
                                            &args,
                                            0x61_febb,
                                        );
                                        let edx = a[33 + seqn.min(1)];
                                        if edx >= 0 {
                                            let obj = rd(&p[76], 0);
                                            let unit = format!(
                                                "p{}+0x0",
                                                if obj < 0 { 94 } else { 86 + 4 * obj as usize }
                                                    + edx as usize
                                            );
                                            emit(
                                                &mut log,
                                                &mut sh,
                                                Some(16),
                                                &unit,
                                                &[s(phase), s(who), s(2), s(0)],
                                                0x61_fef2,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !finished {
                        // Assimilation of a city that is not the player's own.
                        let owner = i32::from(p[ci][0x5f] as i8);
                        if owner != who {
                            emit(
                                &mut log,
                                &mut sh,
                                Some(17),
                                "m+0x0",
                                &[s(0), s(0x164)],
                                0x61_ff27,
                            );
                            if a[17] >= 0 {
                                let v =
                                    rd(&p[ci], 0x24).wrapping_add(1i32.wrapping_sub(rd(&p[65], 0)));
                                wr(&mut p[ci], 0x24, v);
                            }
                            let mut t = frame.wrapping_sub(rd(&p[ci], 0x24));
                            emit(&mut log, &mut sh, Some(19), &lead(w), &[s(8)], 0x61_ff9c);
                            if a[19] != 0 {
                                t = rd(&p[63], 0).wrapping_add(100).wrapping_mul(t) / 100;
                            }
                            if who == i32::from(p[ci][0x60] as i8) {
                                t = rd(&p[66], 0).wrapping_add(100).wrapping_mul(t) / 100;
                            }
                            if t >= rd(&p[66], 4) {
                                emit(&mut log, &mut sh, Some(18), &city, &[], 0x61_fff8);
                            }
                        }
                        if cflags(p) & 2 == 0 {
                            // The city heal.
                            let owner = i32::from(p[ci][0x5f] as i8);
                            if rd(m, 0x24) != 0 && owner == who && x % rd(&p[64], 0) == 0 {
                                emit(&mut log, &mut sh, Some(20), &city, &[], 0x62_0056);
                                let level = a[20];
                                emit(
                                    &mut log,
                                    &mut sh,
                                    Some(9),
                                    "m+0x0",
                                    &[s(level), s(0), s(1)],
                                    0x62_0065,
                                );
                            }
                            // The alarm for a city with no enemy near.
                            if cflags(p) & 0x40 != 0 {
                                let lf = rd(&p[pw(w, 0)], 0);
                                let mut period = if lf & 4 == 0 { 60 } else { 200 };
                                if lf & 4 != 0 && rd(&p[67 + w], 0) & 0x20 == 0 {
                                    period = 0;
                                }
                                if period != 0 && (frame + 100 + phase) % period == 0 {
                                    // `get_radius` is called with eight arguments already pushed: its return address is not in the shadow.
                                    log.push(format!("s21:{city}"));
                                    let rad = a[21].wrapping_mul(3).wrapping_shl(6);
                                    let (cx, cy) = (rd(&p[ci], 0xc), rd(&p[ci], 0x10));
                                    let args = [
                                        s(cx),
                                        s(cy),
                                        s(6),
                                        s(who),
                                        s(rad),
                                        s(phase),
                                        s(0x11),
                                        s(who),
                                        s(0),
                                        sh[4].clone(),
                                        sh[3].clone(),
                                        sh[2].clone(),
                                        s(1),
                                    ];
                                    emit(&mut log, &mut sh, Some(22), &city, &args, 0x62_010c);
                                    if a[22] < 0 {
                                        emit(
                                            &mut log,
                                            &mut sh,
                                            Some(23),
                                            "sp-0xa24",
                                            &[s(-1)],
                                            0x62_012e,
                                        );
                                        emit(
                                            &mut log,
                                            &mut sh,
                                            Some(23),
                                            "sp-0xa24",
                                            &[s(-1)],
                                            0x62_0149,
                                        );
                                        emit(
                                            &mut log,
                                            &mut sh,
                                            Some(24),
                                            "sp-0xa24",
                                            &[s(phase), s(who), s(0), s(0)],
                                            0x62_015c,
                                        );
                                        emit(
                                            &mut log,
                                            &mut sh,
                                            Some(25),
                                            "sp-0xa24",
                                            &[],
                                            0x62_0167,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                // The deferred road replan, below `is_active` — and not reached by a capture's return.
                if !finished && short(m, 0x60) & 0x100 != 0 && (frame + short(m, 0xa)) % 16 == 0 {
                    m[0x61] &= 0xfe;
                    emit(
                        &mut log,
                        &mut sh,
                        Some(32),
                        "p97+0x0",
                        &[s(xk), s(yk), s(short(m, 0xa)), s(who), s(1), s(3)],
                        0x62_01c1,
                    );
                }
            }
            0i64
            // end hand
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "process — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Build::process@0061edf0: {n} rows agree");
}
