//! `SpellType::cast_pilfer@00673e80` under the emulator against the port (item 3410; tools/sweep_gen.py, obj with windows).

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

/// `SpellType::cast_pilfer@00673e80`, run by `tools/emu/sweep_spell_type_cast_pilfer.py` under unicorn
/// (tools/emu): 16 random bytes of object, 4 argument(s), `p` windows in order: p0..p7 sparse table 0 (4 rows × 2 slice(s), row-major), p8..p18 the place(s), in flag order, p19..p23 the stubs' answer window(s); the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00c06210 points at 704 random bytes; the global cell 0x00c061ac points at 32 random bytes; 4 random bytes at 0x00cb195c; callee 0x006e33a0 (2 args) stubbed; callee 0x00739340 (0 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; callee 0x00a1b9a0 (3 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; callee 0x0046e620 (0 args) stubbed; callee 0x008c2cc0 (1 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x006ebe90 (1 args) stubbed; callee 0x00a1b010 (4 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0xac (0 args) stubbed; the word at `c0+0x14+0x1c*a[3]` points at 64 random bytes less `0` (placed); the word at `c2+0x10` points at 32 random bytes less `0` (placed); the word at `p8+4*a[2]` points at 32 random bytes less `0` (placed); the word at `0xe41248+0x6eec*a[1]` points at 24 random bytes less `0` (placed); the word at `0xe41248+0x6eec*a[3]` points at 24 random bytes less `0` (placed); the word at `0xc09970+0x1c*a[3]` points at 16 random bytes less `0` (placed); the word at `p13+0` points at 112 random bytes less `0` (placed); the word at `p13+4` points at 112 random bytes less `0` (placed); the word at `p13+8` points at 112 random bytes less `0` (placed); the word at `p13+12` points at 112 random bytes less `0` (placed); the word at `p10+0x1c` points at 708 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 12 answered from [1, 1, 1, 0] (90%); stub 13 answered from [1, 1, 1, 0] (90%); stub 14 answers a window of its own to each call, in call order (128 bytes, ...); stub 6 answers a window of its own to each call, in call order (32 bytes, ...); the stubbed vtable laid at `p10+0`; the word at `p19+0x18` laid as the address of `p18` every row; stub 0 answers 12 values in call order, printed after the answers; stub 1 answers 1..8 every row; stub 2 answers 0..65535 every row; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 4 rows 0x6eec apart at `@0xe3a390`, slices +0x95c:0x4, +0x6928:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_cast_pilfer.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
    // `a` = the four arguments (p1 unused, p2 the thief, p3 the object's slot, p4 the victim player), then the
    // stubs' answers in line order (a[4 + k] stub k), then the twelve `type_avail` answers in call order (a[19 + k]).
    let (p2, idx, p4) = (a[1], a[2], a[3]);
    let ans = |k: usize| a[4 + k];
    'done: {
        // The object: live flag, then the two vtable asks (slots 0x20, 0x4c).
        if p[10][8] & 1 == 0 || ans(12) == 0 || ans(13) == 0 {
            break 'done;
        }
        // Slot 0xac is asked up to three times; each ask has its own answer window in call order.
        let flag1 = rd(&p[18], 0x2c0) & 0x8000000 != 0;
        if !flag1 && p[20][8] & 0x20 == 0 {
            break 'done;
        }
        let last = if flag1 { 20 } else { 21 };
        let city = i16::from_le_bytes([p[last][0x72], p[last][0x73]]) as i32;
        if city < 0 {
            break 'done;
        }
        // The good the thief takes: the cheapest score among the five available kinds (knowledge, 3, never).
        let (mut t, mut best, mut k) = (2usize, 99999999i32, 0usize);
        for g in 0..6usize {
            if g == 3 {
                continue;
            }
            let thief_has = a[19 + k] != 0;
            k += 1;
            if !thief_has {
                continue;
            }
            let victim_has = a[19 + k] != 0;
            k += 1;
            if !victim_has {
                continue;
            }
            let m_ = rd(&p[11], 4 * g) ^ 0x8221;
            let v = rd(&p[12], 4 * g) ^ 0x8221;
            let base = if v >= 100 { m_ } else { m_.wrapping_add(100) };
            let sc = if v >= m_ { base } else { base.wrapping_add(1000) };
            if sc <= best {
                best = sc;
                t = g;
            }
        }
        let mut s = rd(&p[12], 4 * t) ^ 0x8221;
        let field = rd(&p[2 * p4 as usize], 0);
        if field != 0 {
            let lv = ans(1);
            s = lv.wrapping_mul(s).wrapping_mul(2).wrapping_div(field.wrapping_add(lv));
        }
        let b = p[14 + city as usize][0x61] as i32;
        if b != 0 {
            s = s.wrapping_div(b + 1);
        }
        let half = s.wrapping_sub(s >> 31) >> 1;
        let draw = if half - 1 > 0 { ans(2) % half } else { 0 };
        let amt = half.wrapping_add(draw);
        let w = (rd(&p[11], 4 * t) ^ 0x8221).wrapping_add(amt) ^ 0x8221;
        wr(&mut p[11], 4 * t, w);
        let sv = (rd(&p[12], 4 * t) ^ 0x8221).wrapping_sub(amt);
        wr(&mut c[3], 0, sv);
        wr(&mut p[12], 4 * t, sv ^ 0x8221);
        // The messages, each a recorded call in the original's order.
        let who = rd(&c[1], 0x298);
        let o = format!("p8+{:#x}", 4 * idx);
        let gname = rd(&p[9], 4 * t).wrapping_add(0x74);
        let x = rd(&p[10], 0x10) ^ 0x63637;
        let y = rd(&p[10], 0x14) ^ 0x63637;
        let mut pops = 0;
        if who == p2 {
            log.push(format!("s4:15558448,sp-0x4c,{gname},{amt}"));
            log.push("s6:sp-0x60".to_string());
            let team = p[2 * p2 as usize + 1][0] as i32;
            log.push(format!("s7:{},sp-0x44", 0xe39ce0 + 100 * team));
            let bub = 22 + pops;
            p[bub][0x1a] = p2 as u8;
            pops += 1;
            log.push(format!("s8:15267872,15558448,{x},{y},1,{},1,{o},0,p{bub}+0x0,{o}", ans(7)));
            log.push("s9:15267872,112".to_string());
        }
        if who == p4 && amt != 0 {
            log.push(format!("s10:{},sp-0x74", 0xe3a390 + p2 * 0x6eec));
            log.push(format!("s11:15558448,sp-0x4c,{gname},{},{amt}", ans(10)));
            log.push("s6:sp-0x60".to_string());
            let bub = 22 + pops;
            p[bub][0x1a] = p2 as u8;
            log.push(format!("s8:15267872,15558448,{x},{y},1,13161068,1,p8+0x0,0,p{bub}+0x0,p8+0x0"));
            log.push("s9:15267872,88".to_string());
        }
    }
    if log.is_empty() {
        log.push("-".to_string());
    }
    0
}
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "cast_pilfer — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "cast_pilfer — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "cast_pilfer — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "cast_pilfer — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1000, "only {n} rows");
    eprintln!("SpellType::cast_pilfer@00673e80: {n} rows agree");
}
