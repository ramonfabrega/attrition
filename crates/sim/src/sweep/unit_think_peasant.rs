//! `Unit::think_peasant@005f5760` under the emulator against the port (item 3250; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

use crate::{Pos, Sim, Tuning, Unit, World};

fn rd16(m: &[u8], off: usize) -> i16 {
    i16::from_le_bytes([m[off], m[off + 1]])
}

/// The recorded call tokens, `find_gather_spot`'s two stale stack arguments dropped (they are
/// leftovers of earlier frames, not arguments the function passes: `find_gather_spot` takes one).
fn norm(want: &[&str]) -> Vec<String> {
    want.iter()
        .map(|t| {
            if t.starts_with("s4:") {
                t.splitn(3, ',').take(2).collect::<Vec<_>>().join(",")
            } else {
                (*t).to_string()
            }
        })
        .collect()
}

/// `Unit::think_peasant@005f5760` as the listing reads (`5f5760`-`5f5a48`), the callees answered by the
/// row. Inline: the crate's `Sim::think_peasant` runs its own searches.
fn think_peasant_port(
    a: &[i32],
    m: &mut [u8],
    c: &[Vec<u8>],
    p: &[Vec<u8>],
    log: &mut Vec<String>,
) -> i64 {
    let forced = a[0];
    let owner = usize::from(m[9]);
    let flags = rd(m, 0x68);
    let ai = flags & 0x40000 != 0;
    let idle = i32::from(m[0xb0]);
    let ty = rd(&p[8], 4);
    let scholar = ty == 0x34 || ty == 0x35;
    let seq = &a[9..14];
    let mut turn = 0usize;
    let mut calls: Vec<String> = Vec::new();
    // The stance slot: one call, one answer in turn, the last repeated.
    let mut stance = |calls: &mut Vec<String>| -> i32 {
        calls.push("s0:m+0x0".to_string());
        let v = seq[turn.min(4)];
        turn += 1;
        v
    };
    let out = |calls: Vec<String>, d: [Option<i32>; 2], r: i64, log: &mut Vec<String>| -> i64 {
        log.extend(if calls.is_empty() {
            vec!["-".to_string()]
        } else {
            calls
        });
        for x in d {
            log.push(x.map_or("-".to_string(), hex4));
        }
        r
    };
    if forced == 0 {
        let t = if ai {
            1
        } else {
            match rd(&c[0], owner * 32 + 8) {
                1 => 7,
                2 => 0xc,
                3 => 0x11,
                4 => 0x20,
                5 => 0x3e,
                _ => 2,
            }
        };
        if idle < t {
            return out(calls, [None, None], 0, log);
        }
        if idle != t && (idle - 2) % 5 != 0 {
            return out(calls, [None, None], 0, log);
        }
    }
    // The colonist arm.
    if ai && (ty == 0x32 || ty == 0x33) {
        calls.push("s2:m+0x0,1".to_string());
        if a[3] != 0 {
            return out(calls, [None, None], 1, log);
        }
    }
    // The build arm.
    if !scholar {
        let ask = flags & 0x400 != 0 || stance(&mut calls) == 2 || stance(&mut calls) == 1;
        if ask {
            calls.push("s3:m+0x0".to_string());
            if a[4] != 0 {
                return out(calls, [None, None], 1, log);
            }
        }
    }
    // The gather arm: stance 1, or (a second call) a stance of 0.
    let first = stance(&mut calls);
    let gather = first == 1 || stance(&mut calls) == 0;
    if gather {
        let range = if ai {
            -1
        } else {
            rd(&c[3], 0x28).wrapping_mul(3 * 64)
        };
        calls.push(format!("s4:m+0x0,{range}"));
        if a[5] != 0 {
            // The select list's `deselect`, when it holds more than one.
            if rd(&p[9], 0xc) > 1 {
                calls.push(format!(
                    "s1:p9+0x0,{},{},0,0",
                    i32::from(rd16(m, 0xa)),
                    owner
                ));
            }
            if !ai {
                m[0x80] = 0xff;
                m[0x81] = 0xff;
            }
            return out(calls, [None, None], 1, log);
        }
    }
    // The tail.
    if scholar {
        return out(calls, [None, None], 0, log);
    }
    if !ai {
        let ask = flags & 0x400 != 0 || stance(&mut calls) == 2 || stance(&mut calls) == 1;
        if ask {
            calls.push("s5:m+0x0".to_string());
            if a[6] != 0 {
                return out(calls, [None, None], 1, log);
            }
        }
    }
    wr(m, 0x68, rd(m, 0x68) & !0x400);
    if !ai {
        return out(calls, [None, None], 0, log);
    }
    // The AI's: the region of the unit's tile, through `div_3_table`.
    let d0 = rd(&c[2], 4 * ((rd(m, 0x10) ^ 0x63637) >> 6) as usize);
    let d1 = rd(&c[2], 4 * ((rd(m, 0x14) ^ 0x63637) >> 6) as usize);
    calls.push("s7:12621800,sp-0x10,sp-0x4".to_string());
    let region = a[8];
    let counts = &p[2 * owner];
    if rd16(counts, 2 * region as usize) == 0 {
        let sites = &p[10 + owner];
        let found =
            (0..10).find(|i| rd(sites, 8 + 24 * i) != 0 && rd(sites, 0xc + 24 * i) == region);
        let scout = match found {
            None => true,
            Some(_) => m[0xb0] >= 7,
        };
        if scout {
            calls.push("s6:m+0x0,0".to_string());
            return out(calls, [Some(d0), Some(d1)], i64::from(a[7]), log);
        }
    }
    calls.push("s5:m+0x0".to_string());
    let r = i64::from(a[6] != 0);
    out(calls, [Some(d0), Some(d1)], r, log)
}

fn hex4(v: i32) -> String {
    v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `Unit::think_peasant@005f5760`, run by `tools/emu/sweep_unit_think_peasant.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0..p7 sparse table 0 (4 rows × 2 slice(s), row-major), p8..p13 the place(s), in flag order; the global cell 0x00c061b4 points at 256 random bytes; the global cell 0x00c06210 points at 1024 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; the global cell 0x00c061f0 points at 64 random bytes; vtable slot +0xf8 (0 args) stubbed; vtable slot +0x10 (4 args) stubbed; callee 0x005f40d0 (1 args) stubbed; callee 0x00603e20 (0 args) stubbed; callee 0x005f5170 (3 args) stubbed; callee 0x00604320 (0 args) stubbed; callee 0x005f6010 (1 args) stubbed; callee 0x006b52e0 (2 args) stubbed; the word at `m+0x18` points at 64 random bytes less `0` (placed); the word at `0xe8d444` points at 256 random bytes less `0` (placed); the word at `p1+0` points at 256 random bytes less `0` (placed); the word at `p3+0` points at 256 random bytes less `0` (placed); the word at `p5+0` points at 256 random bytes less `0` (placed); the word at `p7+0` points at 256 random bytes less `0` (placed); stub 0 answered from [0, 0, 1, 2, 3] (90%); stub 2 answered from [0, 0, 0, 1] (90%); stub 3 answered from [0, 0, 0, 1] (90%); stub 4 answered from [0, 0, 1] (90%); stub 5 answered from [0, 0, 1] (90%); stub 6 answered from [0, 1, 5] (90%); stub 0 answers 5 values in call order, printed after the answers; stub 7 answers 0..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 7 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 7 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); 4 rows 0x6eec apart at `@0xe3a390`, slices +0x125e:0x20, +0x6e44:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p9+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 6000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_think_peasant.py") else {
        return;
    };
    let mut n = 0;
    let mut gate_parted = 0;
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
        let (m0, c0) = (m.clone(), wins[0].clone());
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Row: a[0] `forced`, a[1..9] the eight stubs' answers (0 the stance slot, answered from the
            // sequence instead, 1 `deselect`, 2 `think_civilian_transport`, 3 `find_build_spot`, 4
            // `find_gather_spot`, 5 `find_repair_spot`, 6 `think_scout`, 7 `get_tregion`), a[9..14] the
            // stance slot's five answers in call order. Windows: m the unit; c0 the leader options (`+8 +
            // 32 owner`), c1 the console, c2 the `div_3_table`, c3 the constants (`+0x28`
            // `unit_gather_respond_range`); p`2w` leader `w`'s region counts (`+0x125e`), p`10+w` its sites
            // (`+0x6e44` points here, `10 x 0x18` from `+8`), p8 the type record, p9 the select list.
            //
            // The port is INLINE: the crate's `Sim::think_peasant` calls the crate's own `find_*_spot` and
            // `think_scout`, which cannot be answered from a row. What the crate method does answer is the
            // idle gate, and that is driven below (`crate_gate`).
            think_peasant_port(a, m, c, p, &mut log)
        };
        // The crate's gate: a `Sim` laid from the row, its unit a plain worker (type 0x30, stance 3) with
        // `was_builder` set, asked `Sim::think_peasant` once. The original reaches its first stance call
        // exactly when the gate passed, and the tail clears `was_builder` for any non-scholar that
        // searched and found nothing; on an empty world the crate does the same, so
        // `!was_builder` after the call is "the gate passed".
        {
            let gate_passed_orig = want_log.iter().any(|t| t.starts_with('s'));
            let (owner, flags0, idle) = (usize::from(m0[9]), rd(&m0, 0x68), m0[0xb0]);
            let ai = flags0 & 0x40000 != 0;
            let mut sim = Sim::new(Tuning::RON, World::new(8, 8), 4);
            sim.nation[owner].human = !ai;
            sim.leader_options[owner].peasants_wait = rd(&c0, owner * 32 + 8);
            let mut un = Unit::new(owner as u8, 0, Pos::new(0, 0), 20);
            un.idle = idle;
            un.stance = 3;
            un.type_index = 0x30;
            un.was_builder = true;
            let u = sim.add_unit(un);
            let r = sim.think_peasant(u, a[0] != 0);
            let passed = !sim.units[u].was_builder;
            if passed != gate_passed_orig {
                gate_parted += 1;
            }
            let _ = r;
        }
        assert_eq!(got, want_eax, "think_peasant — eax, row `{line}`");
        assert_eq!(
            log,
            norm(&want_log),
            "think_peasant — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "think_peasant — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "think_peasant — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[5 + i],
                "think_peasant — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert_eq!(
        gate_parted, 0,
        "the crate's gate parted from the original on {gate_parted} rows"
    );
    assert!(n >= 6000, "only {n} rows");
    eprintln!("Unit::think_peasant@005f5760: {n} rows agree");
}
