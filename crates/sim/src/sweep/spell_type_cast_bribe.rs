//! `SpellType::cast_bribe@00672d00` under the emulator against the port (item 3410; tools/sweep_gen.py, obj with windows).

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

/// `SpellType::cast_bribe@00672d00`, run by `tools/emu/sweep_spell_type_cast_bribe.py` under unicorn
/// (tools/emu): 472 random bytes of object, 4 argument(s), `p` windows in order: p0..p19 sparse table 0 (4 rows × 5 slice(s), row-major), p20..p23 sparse table 1 (4 rows × 1 slice(s), row-major), p24..p38 the place(s), in flag order, p39..p44 the stubs' answer window(s); the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00c06210 points at 672 random bytes; the global cell 0x00c06208 points at 1440 random bytes; the global cell 0x00c061ec points at 48 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00cae5fc points at 128 random bytes; 4 random bytes at 0x00c8cd00 (each word in 0..3); callee 0x00676ff0 (2 args) stubbed; callee 0x00646b00 (2 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; callee 0x00a1ee30 (0 args) stubbed; callee 0x00a1cd60 (2 args) stubbed; callee 0x00a1b260 (3 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x007e9ab0 (7 args) stubbed; callee 0x00a1d590 (1 args) stubbed; callee 0x006ebaa0 (1 args) stubbed; callee 0x006edb50 (1 args) stubbed; callee 0x00594880 (0 args) stubbed; callee 0x006daa70 (1 args) stubbed; callee 0x008c2cc0 (1 args) stubbed; callee 0x00733380 (3 args) stubbed; callee 0x00595260 (0 args) stubbed; callee 0x00642fc0 (2 args) stubbed; callee 0x0063d230 (1 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x10c (1 args) stubbed; vtable slot +0x94 (1 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x150 (3 args) stubbed; vtable slot +0x1a8 (3 args) stubbed; the word at `c0+0x14+0x1c*a[3]` points at 16 random bytes less `0` (placed); the word at `p24+4*a[2]` points at 64 random bytes less `0` (placed); the word at `c0+0x14+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `p26+0` points at 64 random bytes less `0` (placed); the word at `p26+4` points at 64 random bytes less `0` (placed); the word at `p26+8` points at 64 random bytes less `0` (placed); the word at `p26+12` points at 64 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*a[3]` points at 16 random bytes less `0` (placed); the word at `p31+4*a[2]` points at 32 random bytes less `0` (placed); the word at `0xc09970+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `p33+0` points at 160 random bytes less `0` (placed); the word at `p33+4` points at 160 random bytes less `0` (placed); the word at `p33+8` points at 160 random bytes less `0` (placed); the word at `p33+12` points at 160 random bytes less `0` (placed); the word at `c4+0x134` points at 1024 random bytes less `0` (placed); stub 1 answered from [-1, 0, 1, 5] (90%); stub 11 answered from [0, 1] (90%); stub 12 answered from [0, 1] (90%); stub 13 answered from [0, 1] (90%); stub 17 answered from [0, 3, 4, 5, 9] (90%); stub 20 answered from [0, 1] (90%); stub 24 answered from [0, 1, 5] (90%); stub 15 answers the address of 16 random bytes, a `p` window after the places; stub 21 answers a window of its own to each call, in call order (128 bytes, ...); the stubbed vtable laid at `p25+0`; the word at `p39+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 20 answers 2 values in call order, printed after the answers; stub 23 answers 0..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 512 bytes under the call is zeroed before each row; 4 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x8:0x4, +0x74:0x10, +0x8f4:0x4, +0x6928:0x4 (`p` windows after the pointed buffers, before the places); 4 rows 0x6eec apart at `0xc061e0`, slices +0x74:0x10 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p40+0` as well as at `this`; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p43+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_cast_bribe.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(7);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
    // `a` = the four arguments (p1 the caster's slot, p2 the caster's player, p3 the target's slot, p4 its owner), then
    // the stubs' answers (a[4 + k] stub k, in line order), then slot 0x18's two answers in call order (a[31], a[32]).
    // `c` = the cells: c0 objects, c1 console, c2 message window, c3 game, c4 world, c5 div_3_table, c6 the string
    // base. `p` = p0..p19 the leaders' slices (row r: flags, team, diplomacy, bribe counter, colour), p20..p23
    // leadersc's diplomacy, p24 the victim's object array, p25 the target, p26 the caster's object array, p27..p30 its
    // objects, p31 the victim's unit array, p32 the target unit, p33 the caster's city array, p34..p37 the cities,
    // p38 the world's owner table, p39..p43 slot 0xac's answers, p44 the neon colour.
    let (p1, p2, p3, p4) = (a[0], a[1], a[2], a[3]);
    let (u2, u3, u4) = (p2 as usize, p3 as usize, p4 as usize);
    let ans = |k: usize| a[4 + k];
    let s18 = [a[31], a[32]];
    let who = rd(&c[1], 0x298);
    let base = rd(&c[6], 0);
    let msg = 15558448i32;
    let white = 13161068i32;
    let red = 13161032i32;
    let row = format!("p{}+0x0", u2 * 5);
    let o = rd(&p[25], 0);
    let _ = o;
    let xo = rd(&p[25], 0x10) ^ 0x63637;
    let yo = rd(&p[25], 0x14) ^ 0x63637;
    let unpay = |log: &mut Vec<String>| log.push(format!("s0:m+0x0,{p1},{p2}"));
    // The refusal's message: copied into a stack String (its last word the unpay call's return address), shown to
    // the console's player.
    let refuse = |log: &mut Vec<String>, off: i32, ret: i32| {
        unpay(log);
        if p2 == who {
            log.push(format!("s10:sp-0xac,{}", base.wrapping_add(off)));
            log.push(format!("s9:c2+0x0,0,0,0,0,{ret},-1,{red}"));
            log.push("s2:c2+0x0,64".to_string());
        }
    };
    'done: {
        let team = rd(&p[u2 * 5 + 1], 0);
        let diplo = rd(&p[u2 * 5 + 2], 4 * u4);
        let mut cond1 = p4 == team;
        if !cond1 && diplo == 2 {
            cond1 = rd(&p[20 + u4], 4 * team as usize) == 2;
        }
        let ts = c[3][0x24];
        let flags4 = p[u4 * 5][0];
        if cond1 && (ts == 0 || ts == 8 || ts == 0xb || flags4 & 2 != 0) {
            unpay(&mut log);
            break 'done;
        }
        log.push("s20:p25+0x0".to_string());
        if s18[0] != 0 {
            log.push("s1:p25+0x0,0,357".to_string());
            if ans(1) >= 0 {
                unpay(&mut log);
                break 'done;
            }
        }
        log.push("s20:p25+0x0".to_string());
        if s18[1] != 0 {
            // A unit.
            if who == p2 {
                log.push("s2:p25+0x0,353".to_string());
                log.push(format!("s3:{msg},{}", base.wrapping_add(0x109f0)));
                log.push("s4:sp-0x48".to_string());
                log.push("s22:p32+0x0,sp-0x48".to_string());
                log.push(format!("s5:{msg},sp-0x5c,sp-0x48"));
                log.push(format!("s3:{msg},{}", ans(5)));
                log.push("s7:sp-0x5c".to_string());
                log.push(format!("s8:c2+0x0,{msg},{xo},{yo},0,{white},1,p24+0x0,0,0,p24+0x0"));
                log.push("s7:sp-0x48".to_string());
            } else if who == p4 {
                log.push("s2:p25+0x0,358".to_string());
                log.push(format!("s3:{msg},{}", base.wrapping_add(0x10a04)));
                log.push("s4:sp-0x5c".to_string());
                log.push("s22:p32+0x0,sp-0x5c".to_string());
                log.push(format!("s14:{row},sp-0x84"));
                log.push(format!("s6:{msg},sp-0x70,{},sp-0x5c", ans(14)));
                log.push(format!("s3:{msg},{}", ans(6)));
                log.push("s7:sp-0x70".to_string());
                log.push("s7:sp-0x84".to_string());
                let col = p[u2 * 5 + 4][0] as i32;
                log.push(format!("s15:{},sp-0x40", 0xe39ce0 + 100 * col));
                let neon = p[44][..12].to_vec();
                c[2][0x580..0x58c].copy_from_slice(&neon);
                log.push(format!("s8:c2+0x0,{msg},{xo},{yo},1,{white},-1,p24+0x0,0,0,p24+0x0"));
                log.push("s7:sp-0x5c".to_string());
            }
            let cnt = rd(&p[u2 * 5 + 3], 0).wrapping_add(1);
            wr(&mut p[u2 * 5 + 3], 0, cnt);
            log.push(format!("s23:p32+0x0,{p2}"));
            break 'done;
        }
        if rd(&p[25], 8) & 0x20 != 0 {
            // A city.
            log.push(format!("s12:{row},{p4}"));
            if ans(12) == 0 || flags4 & 2 != 0 {
                refuse(&mut log, 0x10a68, 6764253);
                break 'done;
            }
            log.push(format!("s13:{row}"));
            if ans(13) == 0 {
                refuse(&mut log, 0x10a68, 6764253);
                break 'done;
            }
            log.push("s21:p25+0x0".to_string());
            let city = i16::from_le_bytes([p[39][0x72], p[39][0x73]]) as i32;
            if city < 0 {
                unpay(&mut log);
                break 'done;
            }
            if p2 == who {
                log.push("s2:p25+0x0,20".to_string());
                log.push(format!("s3:{msg},{}", base.wrapping_add(0x10a7c)));
                log.push(format!("s5:{msg},sp-0x84,p{}+0x90", 34 + city));
                log.push(format!("s3:{msg},{}", ans(5)));
                log.push("s7:sp-0x84".to_string());
                let q = &p[27 + p1 as usize];
                let (x, y) = (rd(q, 0x10) ^ 0x63637, rd(q, 0x14) ^ 0x63637);
                log.push(format!("s8:c2+0x0,{msg},{x},{y},0,{white},-1,{y},0,0,{y}"));
            }
            log.push("s21:p25+0x0".to_string());
            let city2 = i16::from_le_bytes([p[40][0x72], p[40][0x73]]) as i32;
            log.push(format!("s16:12622176,{p2},{city2},{p4}"));
            break 'done;
        }
        // A building: the tile's owner must be the caster.
        let w = rd(&c[4], 0);
        let d3 = |v: i32| rd(&c[5], 4 * (v >> 8) as usize);
        let idx = w.wrapping_mul(d3(yo)).wrapping_add(d3(xo));
        let owner = p[38][(idx as usize) * 28 + 0xf] as i8 as i32;
        if owner != p2 {
            refuse(&mut log, 0x10a18, 6763814);
            break 'done;
        }
        log.push(format!("s11:{row},{p4}"));
        if ans(11) == 0 {
            log.push(format!("s12:{row},{p4}"));
            let mut ok = ans(12) != 0 && flags4 & 2 == 0;
            if ok {
                log.push(format!("s13:{row}"));
                ok = ans(13) != 0;
            }
            if !ok {
                refuse(&mut log, 0x10a2c, 6762833);
                break 'done;
            }
        }
        if who == p2 {
            log.push(format!("s2:{row},353"));
            log.push(format!("s3:{msg},{}", base.wrapping_add(0x10a40)));
            log.push("s4:sp-0x5c".to_string());
            log.push("s22:p25+0x0,sp-0x5c".to_string());
            log.push(format!("s5:{msg},sp-0x84,sp-0x5c"));
            log.push(format!("s3:{msg},{}", ans(5)));
            log.push("s7:sp-0x84".to_string());
            log.push(format!("s8:c2+0x0,{msg},{xo},{yo},0,{white},1,p24+0x0,0,0,p24+0x0"));
            log.push("s7:sp-0x5c".to_string());
        } else if who == p4 {
            log.push(format!("s2:{row},358"));
            log.push(format!("s3:{msg},{}", base.wrapping_add(0x10a54)));
            log.push("s4:sp-0x5c".to_string());
            log.push("s22:p25+0x0,sp-0x5c".to_string());
            log.push(format!("s14:{row},sp-0x70"));
            log.push(format!("s6:{msg},sp-0x84,{},sp-0x5c", ans(14)));
            log.push(format!("s3:{msg},{}", ans(6)));
            log.push("s7:sp-0x84".to_string());
            log.push("s7:sp-0x70".to_string());
            let col = p[u2 * 5 + 4][0] as i32;
            log.push(format!("s15:{},sp-0x40", 0xe39ce0 + 100 * col));
            let neon = p[44][..12].to_vec();
            c[2][0x580..0x58c].copy_from_slice(&neon);
            log.push(format!("s8:c2+0x0,{msg},{xo},{yo},1,{white},-1,p24+0x0,0,0,p24+0x0"));
            log.push("s7:sp-0x5c".to_string());
        }
        // The building changes hands: asked what it is, then handed over.
        log.push("s21:p25+0x0".to_string());
        let u = if p[39][..4] == 0xb42174i32.to_le_bytes() {
            (p[39][8] & 4) as i32
        } else {
            log.push("s24:p39+0x0".to_string());
            ans(24)
        };
        log.push("s21:p25+0x0".to_string());
        log.push(format!("s23:p40+0x0,{p2}"));
        let q = (ans(23) as usize) & 3;
        log.push("s21:p25+0x0".to_string());
        log.push("s25:p41+0x0,0,-1,0".to_string());
        if u == 0 {
            log.push("s17:c3+0x0".to_string());
            if ans(17) > 3 {
                log.push(format!("s21:p{}+0x0", 27 + q));
                log.push("s18:p42+0x0,1,3".to_string());
                log.push(format!("s21:p{}+0x0", 27 + q));
                log.push("s19:p43+0x0,1".to_string());
                break 'done;
            }
        }
        log.push(format!("s21:p{}+0x0", 27 + q));
        log.push("s26:p42+0x0,0,1,0".to_string());
        log.push(format!("s21:p{}+0x0", 27 + q));
        log.push("s18:p43+0x0,1,2".to_string());
    }
    if log.is_empty() {
        log.push("-".to_string());
    }
    0
}
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "cast_bribe — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "cast_bribe — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "cast_bribe — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[8 + i], "cast_bribe — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("SpellType::cast_bribe@00672d00: {n} rows agree");
}
