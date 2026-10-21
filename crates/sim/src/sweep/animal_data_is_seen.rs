//! `AnimalData::is_seen@005d8160` under the emulator against the port (item 3411; tools/sweep_gen.py, obj with windows).

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

/// `AnimalData::is_seen@005d8160`, run by `tools/emu/sweep_animal_data_is_seen.py` under unicorn
/// (tools/emu): 352 random bytes of object, 2 argument(s), `p` windows in order: p0..p9 sparse table 0 (10 rows × 1 slice(s), row-major), p10..p27 the place(s), in flag order, p28 stub 3's answer window; the global cell 0x00c061d0 points at 16 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c061e8 points at 64 random bytes; callee 0x006e1370 (1 args) stubbed; callee 0x006b53f0 (3 args) stubbed; callee 0x006b54f0 (3 args) stubbed; vtable slot +0x3c (0 args) stubbed; vtable slot +0x48 (2 args) stubbed; the word at `0xc0a250` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 24 random bytes less `0` (placed); the word at `p10+4` points at 24 random bytes less `0` (placed); the word at `p10+8` points at 24 random bytes less `0` (placed); the word at `p10+12` points at 24 random bytes less `0` (placed); the word at `0xc0a0f0` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 36 random bytes less `0` (placed); the word at `p15+4` points at 36 random bytes less `0` (placed); the word at `p15+8` points at 36 random bytes less `0` (placed); the word at `p15+12` points at 36 random bytes less `0` (placed); the word at `p16+0x18` points at 8 random bytes less `0` (placed); the word at `p17+0x18` points at 8 random bytes less `0` (placed); the word at `p18+0x18` points at 8 random bytes less `0` (placed); the word at `p19+0x18` points at 8 random bytes less `0` (placed); the word at `0xc0ab84+u8(m,9)*0x1c` points at 4 random bytes less `i16(m,0xa)*4` (placed); the word at `p24+0` points at 24 random bytes less `0` (placed); the word at `0xc0ab84+i16(m,0x152)*0x1c` points at 4 random bytes less `i16(m,0x150)*4` (placed); the word at `p26+0` points at 8 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1, 2, -3] (90%); stub 2 answered from [0, 1, 2, -3] (90%); stub 4 answered from [0, 1, 2, -3] (90%); stub 3 answers the address of 16 random bytes, a `p` window after the places; the stubbed vtable laid at `p27+0`; stub 0 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 2 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 2 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); 10 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x1 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p28+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_animal_data_is_seen.py") else {
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
            {
                // `AnimalData::is_seen(player, _)`: a revealed map (gamec +0x30 == 3) sees
                // everything. An animal in a herd (short +0x86 != -1) is seen by a player
                // whose bit is in its goods record's mask (+0x20, one byte), else by
                // `was_seen` when a tribe bonus (9, then 4) says so and the goods type
                // (+0x18 -> +4) is neither 6 nor 0x1f, else by `was_really_seen`. A herdless
                // one asks its object's record (slot +0x3c, then +0x48(player, 0)) when it
                // has one (shorts +0x150 / +0x152), else `was_seen` as a boolean.
                let player = a[0];
                let d = |v: i32| rd(&c[1], (((v ^ 0x63637) >> 7) as usize) * 4);
                let h = |v: i32| {
                    v.to_le_bytes()
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                };
                // The slots of the row's log: the tribe-bonus asks, was_seen, was_really_seen,
                // slot +0x3c, slot +0x48, then the two coordinates each of the two seen calls.
                let mut l: [String; 9] = std::array::from_fn(|_| "-".to_string());
                let (answer_seen, answer_really) = (a[3], a[4]);
                let seen_call = |l: &mut [String; 9], really: bool| {
                    let (x, y) = (d(rd(&p[25], 0x10)), d(rd(&p[25], 0x14)));
                    l[if really { 2 } else { 1 }] = format!("c0+0x0,sp-0x14,sp-0x8,{player}");
                    l[if really { 7 } else { 5 }] = h(x);
                    l[if really { 8 } else { 6 }] = h(y);
                };
                let eax: i64 = if c[2][0x30] == 3 {
                    1
                } else if i16::from_le_bytes([m[0x86], m[0x87]]) == -1 {
                    if player >= 0 && i16::from_le_bytes([m[0x150], m[0x151]]) >= 0 {
                        l[3] = "p27+0x0".into();
                        l[4] = format!("p28+0x0,{player},0");
                        i64::from(a[6])
                    } else {
                        seen_call(&mut l, false);
                        i64::from(answer_seen != 0)
                    }
                } else {
                    let herd = i16::from_le_bytes([m[0x86], m[0x87]]) as usize;
                    let good = rd(&p[11 + herd], 0x14) as usize;
                    let rec = &p[16 + good];
                    let kind = rd(&p[20 + good], 4);
                    if (1u32.wrapping_shl(player as u32 & 31) as u8) & rec[0x20] != 0 {
                        1
                    } else {
                        let mut bonus = false;
                        if kind != 6 && kind != 0x1f && player >= 0 {
                            let mut calls = vec![format!("p{player}+0x0,9")];
                            bonus = a[7] != 0;
                            if !bonus {
                                calls.push(format!("p{player}+0x0,4"));
                                bonus = a[8] != 0;
                            }
                            l[0] = calls.join("/");
                        }
                        if bonus {
                            seen_call(&mut l, false);
                            i64::from(answer_seen)
                        } else {
                            seen_call(&mut l, true);
                            i64::from(answer_really)
                        }
                    }
                };
                log.extend(l);
                eax
            }
        };
        assert_eq!(got, want_eax, "is_seen — eax, row `{line}`");
        assert_eq!(log, want_log, "is_seen — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_seen — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_seen — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "is_seen — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("AnimalData::is_seen@005d8160: {n} rows agree");
}
