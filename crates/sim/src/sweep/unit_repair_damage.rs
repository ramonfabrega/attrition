//! `Unit::repair_damage@0060de10` under the emulator against the port (item 3400; tools/sweep_gen.py, obj with windows).

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

// **The port is inline: the crate has no `Unit::repair_damage`** (`heal.rs`'s `civilian_heal`
// is one caller's tail and `Sim::repair_building` is `Build::repair_damage`'s). The windows (see
// the py's docstring): `m` this; `c0` objects, `c1` game, `c2` gamec, `c3` constants, `c4` console,
// `c5` the select groups, `c6` the groups, `c7` `ignore_always`; `p0, p1` the leaders' rows (the
// `+0x68fc` word), `p2..p37` the hotkey groups (owner * 18 + i), `p38 + 5w` the units array of
// player w and `p39 + 5w + i` its records, `p48 + 5w` the objects array and `p49 + 5w + i` its
// records, `p58` this's type, `p59` the `+0xa8` slot's answer. Row layout: `a[0..3]` the
// arguments, `a[3 + i]` stub i's answer, `a[24..27]` the outargs (get_inside's `param_3`,
// find_nearby_spot's two), `a[27 + k]` the k-th member answer, `a[47 + k]` the k-th `hits`
// answer, `a[50]` whether slot 0x11c is the image's `UnitData::hits`.
const S_IS_ALLY: usize = 0;
const S_RANDOM: usize = 1;
const S_UNDERLAYS: usize = 2;
const S_FIND_FREE: usize = 7;
const S_GET_INSIDE: usize = 8;
const S_UBER: usize = 13;
const S_SLOT_A8: usize = 15;

/// The harness's token for a wild integer: near the stack's top it prints as `sp-0x..`.
fn tok(v: i32) -> String {
    const TOP: i64 = 0x7fff_ff00;
    let u = i64::from(v as u32);
    if (TOP - 0x1000..TOP).contains(&u) {
        format!("sp-{:#x}", TOP - u)
    } else if (TOP..TOP + 0x40).contains(&u) {
        format!("sp+{:#x}", u - TOP)
    } else {
        v.to_string()
    }
}

fn ri16(m: &[u8], off: usize) -> i32 {
    i32::from(i16::from_le_bytes([m[off], m[off + 1]]))
}

fn wr16(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 2].copy_from_slice(&(v as i16).to_le_bytes());
}

/// `cvtdq2ps` of one integer, as bits (round to nearest even), without a float.
fn int_to_f32_bits(v: i32) -> i32 {
    if v == 0 {
        return 0;
    }
    let sign = u32::from(v < 0) << 31;
    let mag = v.unsigned_abs();
    let mut e = 31 - mag.leading_zeros();
    let mut mant = if e <= 23 {
        mag << (23 - e)
    } else {
        let sh = e - 23;
        let mut q = mag >> sh;
        let rem = mag & ((1u32 << sh) - 1);
        let half = 1u32 << (sh - 1);
        if rem > half || (rem == half && q & 1 == 1) {
            q += 1;
        }
        if q == 1 << 24 {
            q >>= 1;
            e += 1;
        }
        q
    };
    mant &= 0x7f_ffff;
    (sign | ((127 + e) << 23) | mant) as i32
}

fn repair_damage_port(
    a: &[i32],
    m: &mut [u8],
    c: &[Vec<u8>],
    p: &mut [Vec<u8>],
    log: &mut Vec<String>,
) {
    const K: i32 = 0x63637;
    let (mut param_1, param_2) = (a[0], a[1]);
    let param_3 = a[2];
    let owner = m[9] as usize;
    // The CTW heal bonus: the leader's byte (`+0x68ff`) scaled by the constants' `+0xaec`.
    if param_3 != 0 && c[1][0x822] & 2 != 0 && c[2][0x822] & 2 != 0 {
        let b = i32::from(p[owner][3]);
        if b != 0 {
            let t = b.wrapping_mul(100).wrapping_add(rd(&c[3], 0xaec));
            param_1 = (t / 100).wrapping_mul(param_1);
        }
    }
    let esi = param_1.wrapping_sub(rd(m, 0x24));
    let mut hits_k = 0usize;
    let mut hits = |m: &[u8], log: &mut Vec<String>| -> i32 {
        if a[50] != 0 {
            rd(m, 0x20)
        } else {
            log.push("s14:m+0x0,0".to_string());
            hits_k += 1;
            a[47 + hits_k - 1]
        }
    };
    // The sparkle, drawn on the console player's own and allies' screens.
    if ri16(m, 0x82) == -1 && param_2 != 0 {
        let who = rd(&c[4], 0x298);
        let mut show = owner as i32 == who;
        if !show {
            log.push(format!(
                "s{S_IS_ALLY}:{},{}",
                0xe3a390 + owner as i32 * 0x6eec,
                tok(who)
            ));
            show = a[3 + S_IS_ALLY] != 0;
        }
        if show {
            let x = rd(m, 0xc) ^ K;
            let size = rd(&p[58], 0x234).wrapping_mul(0x60);
            log.push(format!("s{S_RANDOM}:15427964,0,360"));
            log.push(format!(
                "s{S_UNDERLAYS}:13,{},{size},1000,1,0,-1,-1",
                tok(int_to_f32_bits(x))
            ));
        }
    }
    // Damage already on the figure shrinks by the heal.
    let damage = rd(m, 0x24);
    if damage != 0 {
        let h1 = hits(m, log);
        let mut d = damage;
        if d > h1 {
            d = hits(m, log);
        }
        d = if param_1 < d {
            d.wrapping_sub(param_1)
        } else {
            0
        };
        wr(m, 0x24, d);
        m[0x3b] = 0;
    }
    if esi <= 0 {
        return;
    }
    let o_down = ri16(m, 0x90);
    if o_down >= 0 {
        let rec = 49 + 5 * owner + o_down as usize;
        log.push(format!("s{S_SLOT_A8}:p{rec}+0x0"));
        if p[59][8] & 1 != 0 {
            log.push(format!("s{S_SLOT_A8}:p{rec}+0x0"));
            log.push(format!("s16:p59+0x0,{},{},0", tok(esi), tok(param_2)));
            return;
        }
    }
    // A squad with room to grow a figure: below the type's size, and either inside something or
    // not a hotunit.
    let size = rd(&p[58], 0x308);
    log.push("s13:m+0x0,0".to_string());
    if a[3 + S_UBER] >= size {
        return;
    }
    if ri16(m, 0x82) < 0 && rd(m, 0x68) & 4 != 0 {
        return;
    }
    if o_down < 0 && c[7][0] == 0 {
        // `Error::report`: the strings, the report, the closes. The two argument slots it
        // is handed unwritten hold the sparkle's leftovers.
        let sparkled = log
            .iter()
            .any(|l| l.starts_with(&format!("s{S_UNDERLAYS}:")));
        let (g0, g1) = if sparkled { (1, 0) } else { (0, 0) };
        log.push("s3:sp-0x58,11376956".to_string());
        log.push("s4:sp-0x44,11377008".to_string());
        log.push(format!("s5:13161384,sp-0x44,sp-0x44,{g0},{g1},c7+0x0,1"));
        log.push("s6:sp-0x44".to_string());
        log.push("s6:sp-0x58".to_string());
    }
    let slot_addr = if owner == 0 { 0x15c } else { 0x160 };
    log.push(format!(
        "s{S_FIND_FREE}:c0+0x0,{owner},0,2000,c0+{slot_addr:#x},{o_down}"
    ));
    let idx = a[3 + S_FIND_FREE];
    if idx < 0 {
        return;
    }
    let idx = idx as usize;
    let nw = 39 + 5 * owner + idx;
    let ob = 49 + 5 * owner + idx;
    let (lo, hi) = (ri16(&p[nw], 0x8e), ri16(&p[nw], 0x90));
    log.push(format!(
        "s17:p{nw}+0x0,{owner},{},{idx},{},{}",
        tok(rd(&p[58], 4)),
        tok(rd(m, 0x10) ^ K),
        tok(rd(m, 0x14) ^ K)
    ));
    wr16(&mut p[nw], 0x90, hi);
    wr16(&mut p[nw], 0x8e, lo);
    let h = hits(m, log);
    wr(&mut p[nw], 0x24, h.wrapping_div(size));
    p[nw][0xb1] = m[0xb1];
    let masks = rd(m, 0x68);
    wr(&mut p[nw], 0x68, masks);
    log.push(format!("s{S_GET_INSIDE}:m+0x0,sp-0x4"));
    let inside = a[3 + S_GET_INSIDE];
    if inside < 0 {
        log.push(format!(
            "s9:{},{},{},sp-0x30,sp-0x4,0,-1,0,1431655765,3,{},{},0,0,-1,0,-1",
            tok(rd(&p[nw], 0x18)),
            tok(rd(&p[ob], 0x10) ^ K),
            tok(rd(&p[ob], 0x14) ^ K),
            ri16(&p[nw], 0xa),
            p[nw][9]
        ));
        log.push(format!(
            "s10:p{nw}+0x0,{},{},1,0,2,0,p{}+{:#x},-1,-1",
            tok(a[25]),
            tok(a[26]),
            38 + 5 * owner,
            4 * idx
        ));
    } else {
        log.push(format!("s11:p{nw}+0x0,{},{}", tok(inside), tok(a[24])));
    }
    // The selection, the unit's own group and the hotkey groups let go of the old figure.
    let idx16 = ri16(m, 0xa);
    let mut mem_k = 0usize;
    let mut drop = |log: &mut Vec<String>, who: &str, slot: &str, tail: &str| {
        log.push(format!("s12:{who},{idx16},{owner},1"));
        let yes = a[27 + mem_k] != 0;
        mem_k += 1;
        if yes {
            log.push(format!("s18:{who},{idx16},{owner},0,0"));
            log.push(format!("{slot}:{who},{idx16},{owner},{tail}"));
        }
    };
    let play = rd(&c[4], 0x2a0) as usize;
    drop(log, &format!("c5+{:#x}", play * 0x9f0), "s19", "0");
    let group = ri16(m, 0x80);
    if group >= 0 {
        drop(
            log,
            &format!("c6+{:#x}", group as usize * 0x9d4),
            "s20",
            "0,0",
        );
    }
    for i in 0..18 {
        drop(log, &format!("p{}+0x0", 2 + owner * 18 + i), "s20", "0,0");
    }
    log.push(format!("s16:p{nw}+0x0,{},{},0", tok(esi), tok(param_2)));
}

/// `Unit::repair_damage@0060de10`, run by `tools/emu/sweep_unit_repair_damage.py` under unicorn
/// (tools/emu): 256 random bytes of object, 3 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p37 sparse table 1 (36 rows × 1 slice(s), row-major), p38..p58 the place(s), in flag order, p59 stub 15's answer window; the global cell 0x00c0618c points at 384 random bytes; the global cell 0x00c061ec points at 2304 random bytes; the global cell 0x00c061e8 points at 2304 random bytes; the global cell 0x00c061f0 points at 2816 random bytes; the global cell 0x00c06210 points at 688 random bytes; the global cell 0x00e8d444 points at 5088 random bytes; the global cell 0x00e85f20 points at 7260 random bytes; 4 random bytes at 0x00ee108e; callee 0x006edb50 (1 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x008e1410 (7 args) stubbed; callee 0x00a1edd0 (1 args) stubbed; callee 0x00a1d660 (1 args) stubbed; callee 0x00a2e550 (6 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; callee 0x0065ad60 (5 args) stubbed; callee 0x00651a80 (1 args) stubbed; callee 0x0061de70 (16 args) stubbed; callee 0x00616ed0 (9 args) stubbed; callee 0x00647e90 (2 args) stubbed; callee 0x0070f8f0 (3 args) stubbed; callee 0x0060a760 (1 args) stubbed; vtable slot +0x11c (1 args) stubbed; vtable slot +0xa8 (0 args) stubbed; vtable slot +0x168 (3 args) stubbed; vtable slot +0x8c (5 args) stubbed; vtable slot +0x10 (4 args) stubbed; vtable slot +0x18 (3 args) stubbed; vtable slot +0xc (4 args) stubbed; the word at `0xc0aec0+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p38+0` points at 256 random bytes less `0` (placed); the word at `p38+4` points at 256 random bytes less `0` (placed); the word at `p38+8` points at 256 random bytes less `0` (placed); the word at `p38+12` points at 256 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p43+0` points at 256 random bytes less `0` (placed); the word at `p43+4` points at 256 random bytes less `0` (placed); the word at `p43+8` points at 256 random bytes less `0` (placed); the word at `p43+12` points at 256 random bytes less `0` (placed); the word at `c0+0x14+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p48+0` points at 256 random bytes less `0` (placed); the word at `p48+4` points at 256 random bytes less `0` (placed); the word at `p48+8` points at 256 random bytes less `0` (placed); the word at `p48+12` points at 256 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p53+0` points at 256 random bytes less `0` (placed); the word at `p53+4` points at 256 random bytes less `0` (placed); the word at `p53+8` points at 256 random bytes less `0` (placed); the word at `p53+12` points at 256 random bytes less `0` (placed); the word at `m+0x18` points at 784 random bytes less `0` (placed); slot +0x11c laid as 0x00610890 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 15 answers the address of 64 random bytes, a `p` window after the places; stub 12 answers 20 values in call order, printed after the answers; stub 14 answers 3 values in call order, printed after the answers; stub 12 answers 0..1 every row; stub 5 answers 0..0 every row; stub 7 answers -1..3 every row; stub 8 answers -2..3 every row; stub 13 answers -3..120 every row; stub 14 answers -5..1000 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 8 writes a value in -300..300 through its argument 0, printed after the answers (`o` in a place); stub 9 writes a value in -300..300 through its argument 2, printed after the answers (`o` in a place); stub 9 writes a value in -300..300 through its argument 3, printed after the answers (`o` in a place); stub 2 is `__cdecl`: the caller pops its arguments; the stack 1024 bytes under the call is zeroed before each row; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x68fc:0x4 (`p` windows after the pointed buffers, before the places); 36 rows 0x9fc apart at `0xc0afe0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p40+0` as well as at `this`; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p44+0` as well as at `this`; the stubbed vtable laid at `p45+0` as well as at `this`; the stubbed vtable laid at `p46+0` as well as at `this`; the stubbed vtable laid at `p47+0` as well as at `this`; the stubbed vtable laid at `p49+0` as well as at `this`; the stubbed vtable laid at `p50+0` as well as at `this`; the stubbed vtable laid at `p51+0` as well as at `this`; the stubbed vtable laid at `p52+0` as well as at `this`; the stubbed vtable laid at `p54+0` as well as at `this`; the stubbed vtable laid at `p55+0` as well as at `this`; the stubbed vtable laid at `p56+0` as well as at `this`; the stubbed vtable laid at `p57+0` as well as at `this`; the stubbed vtable laid at `p59+0` as well as at `this`; the stubbed vtable laid at `c5+0` as well as at `this`; the stubbed vtable laid at `c5+0x9f0` as well as at `this`; the stubbed vtable laid at `c6+0` as well as at `this`; the stubbed vtable laid at `c6+0x9d4` as well as at `this`; the stubbed vtable laid at `c6+0x13a8` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p40+0` as well as at `this`; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p44+0` as well as at `this`; the stubbed vtable laid at `p45+0` as well as at `this`; the stubbed vtable laid at `p46+0` as well as at `this`; the stubbed vtable laid at `p47+0` as well as at `this`; the stubbed vtable laid at `p49+0` as well as at `this`; the stubbed vtable laid at `p50+0` as well as at `this`; the stubbed vtable laid at `p51+0` as well as at `this`; the stubbed vtable laid at `p52+0` as well as at `this`; the stubbed vtable laid at `p54+0` as well as at `this`; the stubbed vtable laid at `p55+0` as well as at `this`; the stubbed vtable laid at `p56+0` as well as at `this`; the stubbed vtable laid at `p57+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_repair_damage.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(8);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            repair_damage_port(a, m, c, p, &mut log);
            if log.is_empty() {
                log.push("-".to_string());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "repair_damage — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "repair_damage — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "repair_damage — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[9 + i],
                "repair_damage — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Unit::repair_damage@0060de10: {n} rows agree");
}
