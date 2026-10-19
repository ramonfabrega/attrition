//! `Object::replace_hotunit@00643b70` under the emulator against the port (item 3337; tools/sweep_gen.py, obj with windows).

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

/// `Object::replace_hotunit@00643b70`, run by `tools/emu/sweep_object_replace_hotunit.py` under unicorn
/// (tools/emu): 12 random bytes of object, 2 argument(s), `p` windows in order: p0..p215 sparse table 0 (36 rows × 6 slice(s), row-major), p216..p225 the place(s), in flag order; the global cell 0x00c061ec points at 1536 random bytes; callee 0x00711540 (0 args) stubbed; callee 0x007152f0 (1 args) stubbed; vtable slot +0x10 (4 args) stubbed; vtable slot +0xc (4 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `0xc0aba0` points at 16 random bytes less `0` (placed); the word at `p216+0` points at 16 random bytes less `0` (placed); the word at `p216+4` points at 16 random bytes less `0` (placed); the word at `p216+8` points at 16 random bytes less `0` (placed); the word at `p216+12` points at 16 random bytes less `0` (placed); the word at `p217+0` points at 16 random bytes less `0` (placed); the word at `p217+4` points at 16 random bytes less `0` (placed); the word at `p217+8` points at 16 random bytes less `0` (placed); the word at `p217+12` points at 16 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 2 writes 0 (4 bytes) at `p1+0x0` each call; stub 2 writes 0 (4 bytes) at `p7+0x0` each call; stub 2 writes 0 (4 bytes) at `p13+0x0` each call; stub 2 writes 0 (4 bytes) at `p19+0x0` each call; stub 2 writes 0 (4 bytes) at `p25+0x0` each call; stub 2 writes 0 (4 bytes) at `p31+0x0` each call; stub 2 writes 0 (4 bytes) at `p37+0x0` each call; stub 2 writes 0 (4 bytes) at `p43+0x0` each call; stub 2 writes 0 (4 bytes) at `p49+0x0` each call; stub 2 writes 0 (4 bytes) at `p55+0x0` each call; stub 2 writes 0 (4 bytes) at `p61+0x0` each call; stub 2 writes 0 (4 bytes) at `p67+0x0` each call; stub 2 writes 0 (4 bytes) at `p73+0x0` each call; stub 2 writes 0 (4 bytes) at `p79+0x0` each call; stub 2 writes 0 (4 bytes) at `p85+0x0` each call; stub 2 writes 0 (4 bytes) at `p91+0x0` each call; stub 2 writes 0 (4 bytes) at `p97+0x0` each call; stub 2 writes 0 (4 bytes) at `p103+0x0` each call; stub 2 writes 0 (4 bytes) at `p109+0x0` each call; stub 2 writes 0 (4 bytes) at `p115+0x0` each call; stub 2 writes 0 (4 bytes) at `p121+0x0` each call; stub 2 writes 0 (4 bytes) at `p127+0x0` each call; stub 2 writes 0 (4 bytes) at `p133+0x0` each call; stub 2 writes 0 (4 bytes) at `p139+0x0` each call; stub 2 writes 0 (4 bytes) at `p145+0x0` each call; stub 2 writes 0 (4 bytes) at `p151+0x0` each call; stub 2 writes 0 (4 bytes) at `p157+0x0` each call; stub 2 writes 0 (4 bytes) at `p163+0x0` each call; stub 2 writes 0 (4 bytes) at `p169+0x0` each call; stub 2 writes 0 (4 bytes) at `p175+0x0` each call; stub 2 writes 0 (4 bytes) at `p181+0x0` each call; stub 2 writes 0 (4 bytes) at `p187+0x0` each call; stub 2 writes 0 (4 bytes) at `p193+0x0` each call; stub 2 writes 0 (4 bytes) at `p199+0x0` each call; stub 2 writes 0 (4 bytes) at `p205+0x0` each call; stub 2 writes 0 (4 bytes) at `p211+0x0` each call; 36 rows 0x9fc apart at `0xc0afe0`, slices +0x0:0x4, +0xc:0x4, +0x14:0x4, +0x30:0x4, +0x4a:0x4, +0x8cc:0x12c (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p0+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p48+0` as well as at `this`; the stubbed vtable laid at `p54+0` as well as at `this`; the stubbed vtable laid at `p60+0` as well as at `this`; the stubbed vtable laid at `p66+0` as well as at `this`; the stubbed vtable laid at `p72+0` as well as at `this`; the stubbed vtable laid at `p78+0` as well as at `this`; the stubbed vtable laid at `p84+0` as well as at `this`; the stubbed vtable laid at `p90+0` as well as at `this`; the stubbed vtable laid at `p96+0` as well as at `this`; the stubbed vtable laid at `p102+0` as well as at `this`; the stubbed vtable laid at `p108+0` as well as at `this`; the stubbed vtable laid at `p114+0` as well as at `this`; the stubbed vtable laid at `p120+0` as well as at `this`; the stubbed vtable laid at `p126+0` as well as at `this`; the stubbed vtable laid at `p132+0` as well as at `this`; the stubbed vtable laid at `p138+0` as well as at `this`; the stubbed vtable laid at `p144+0` as well as at `this`; the stubbed vtable laid at `p150+0` as well as at `this`; the stubbed vtable laid at `p156+0` as well as at `this`; the stubbed vtable laid at `p162+0` as well as at `this`; the stubbed vtable laid at `p168+0` as well as at `this`; the stubbed vtable laid at `p174+0` as well as at `this`; the stubbed vtable laid at `p180+0` as well as at `this`; the stubbed vtable laid at `p186+0` as well as at `this`; the stubbed vtable laid at `p192+0` as well as at `this`; the stubbed vtable laid at `p198+0` as well as at `this`; the stubbed vtable laid at `p204+0` as well as at `this`; the stubbed vtable laid at `p210+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 700 rows (an arm few rows reach).
/// The port, inline: the crate keeps no hotkey groups (`cast_transport` names `replace_hotunit` a seam).
/// `p[6g..6g+6]` are group `g`'s slices (`+0`, `+0xc`, `+0x14`, `+0x30`, `+0x4a`, `+0x8cc`); `p[216 + o]` owner
/// `o`'s `objects` array and `p[218 + 4o + id]` its record; `c[0]` the game (frame at `+0x550`). Stub 0 is
/// `Group::normalize`, 1 `HotKeyGroupOut::update_name`, 2 and 3 the group's vslots `+0x10` and `+0xc`; slot
/// `+0x10`'s stub zeroes every group's count (the original removes the member, so the restart ends).
fn replace_hotunit(a: &[i32], m: &mut [u8], c: &mut [Vec<u8>], p: &mut [Vec<u8>], log: &mut Vec<String>) {
    let owner = usize::from(m[9]);
    let id = i32::from(i16::from_le_bytes([m[10], m[11]]));
    if owner >= 8 {
        return;
    }
    'restart: loop {
        for g in owner * 18..owner * 18 + 18 {
            let k = 6 * g;
            wr(&mut p[k + 3], 0, 1);
            log.push(format!("s0:p{k}+0x0"));
            let b = usize::from(p[k + 4][0]);
            if owner == b {
                let rec = &p[218 + 4 * owner + usize::try_from(id).expect("small id")];
                if rec[8] & 1 != 0 {
                    let n = rd(&p[k + 1], 0);
                    let mut hit = false;
                    for i in 0..n.max(0) as usize {
                        if i32::from(i16::from_le_bytes([p[k + 5][2 * i], p[k + 5][2 * i + 1]])) == id {
                            hit = true;
                            break;
                        }
                    }
                    if hit {
                        let frame = rd(&c[0], 0x550);
                        wr(&mut p[k + 2], 0, frame);
                        log.push(format!("s1:p{k}+0x0,0"));
                        log.push(format!("s2:p{k}+0x0,{id},{owner},0,0"));
                        for r in 0..36 {
                            wr(&mut p[6 * r + 1], 0, 0);
                        }
                        log.push(format!("s3:p{k}+0x0,{},{},0,0", a[0], a[1]));
                        continue 'restart;
                    }
                }
            }
        }
        break;
    }
}

#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_replace_hotunit.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            replace_hotunit(a, m, c, p, &mut log);
            0
        };
        let _ = (got, want_eax);
        if log.is_empty() {
            log.push("-".to_string());
        }
        assert_eq!(log, want_log, "replace_hotunit — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "replace_hotunit — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "replace_hotunit — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "replace_hotunit — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 700, "only {n} rows");
    eprintln!("Object::replace_hotunit@00643b70: {n} rows agree");
}
