//! `Map::make@0068bc90` under the emulator against the port (item 3402; tools/sweep_gen.py, obj with windows).

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

// hand: port
/// No crate method is this function's counterpart: the crate has no map generator (a match
/// starts from a loaded scenario, `Sim::new`); the port is inline, over the windows the sweep
/// lays: c0 the world record, c1 the game random, c2 the string table's record, c3 the game
/// record, c4 the current tileset, c5 the conquest game, c6 the map-size list, c7/c8/c9 the
/// cells holding the file-mapping base, the localized-string base and the map-style list, c10
/// the game log's (type, detail); p0 the conquest style, p1/p2 the world's two coordinate
/// arrays. A call is logged as the sweep's recorder prints it: `s<stub>:<ecx>,<args>`, and the
/// last token is the game log's (type, detail) as the log call sees it, one per `Log::say`.
struct Out {
    log: Vec<String>,
    says: Vec<String>,
}

const L40: &str = "sp-0x54"; // the String local at ebp-0x40
const L54: &str = "sp-0x68"; // the String local at ebp-0x54
const L2C: &str = "sp-0x40"; // the long-lived String local at ebp-0x2c
const LOG: i32 = 0x00eb1360;
const SPLASH: i32 = 0x00e80314;
const EMPTY: i32 = 0x00eb437c;

impl Out {
    /// `String(char*)`, `GameLog::say_checksum(1, s, line)`, `String::close`.
    fn chk(&mut self, loc: &str, text: i32, line: i32) {
        self.log.push(format!("s7:{loc},{text}"));
        self.log.push(format!("s8:{LOG},1,{loc},{line}"));
        self.log.push(format!("s3:{loc}"));
    }
    /// The splash subtitle: `String(const String*)`, `operator=`, `refresh`, `close`.
    fn splash(&mut self, loc: &str, locstr: i32, off: i32) {
        self.log
            .push(format!("s0:{loc},{}", locstr.wrapping_add(off)));
        self.log.push(format!("s1:{SPLASH},{loc}"));
        self.log.push(format!("s2:{SPLASH}"));
        self.log.push(format!("s3:{loc}"));
    }
    /// `Log::say` of the local at `ebp-0x2c` under `GLOG_MAPMAKE` (0x1b) and `detail`; the log
    /// type is put back, the detail only when it was not negative.
    fn say(&mut self, gl: &mut [u8], detail: i32) {
        let (ty, old) = (rd(gl, 0), rd(gl, 4));
        self.log
            .push(format!("s23:{LOG},{L2C},{EMPTY},0,0,0,0,0,0,0,0"));
        let mut seen = 0x1bi32.to_le_bytes().to_vec();
        seen.extend(detail.to_le_bytes());
        self.says
            .push(seen.iter().map(|b| format!("{b:02x}")).collect());
        wr(gl, 0, ty);
        wr(gl, 4, if old >= 0 { old } else { detail });
    }
}

fn make_port(a: &[i32], m: &mut [u8], c: &mut [Vec<u8>], p: &[Vec<u8>], o: &mut Out) {
    const W: usize = 0;
    const RND: usize = 1;
    const STR: usize = 2;
    const GAME: usize = 3;
    const TS: usize = 4;
    const CQ: usize = 5;
    const SZ: usize = 6;
    const GL: usize = 10;
    let (n, seed, flag) = (a[0], a[1], a[2]);
    let splash = flag != 0;
    let ans = |k: usize| a[3 + k];
    let strs = rd(&c[STR], 0x10);
    let loc = rd(&c[8], 0);
    let ptr = |off: i32| strs.wrapping_add(off);
    wr(m, 0x110, n);
    if seed >= 0 {
        wr(&mut c[W], 0x7c, seed);
        wr(&mut c[RND], 0, seed);
    }
    if splash {
        o.splash(L40, loc, 0xccd8);
    }
    let ty = rd(m, 0x10);
    o.log.push(format!("s1:m+0x114,{}", ptr(0x170fc)));
    o.log.push(format!(
        "s4:m+0x114,{}",
        rd(&c[7], 0).wrapping_add(ty.wrapping_mul(20))
    ));
    o.log.push(format!("s5:m+0x0,{n},{flag}"));
    if rd(m, 0x44) < 0 {
        o.log.push("s6:c1+0x0,0,65535".to_string());
        let mut e = ans(6) & i32::MIN.wrapping_add(3);
        if e < 0 {
            e = (e.wrapping_sub(1) | -4).wrapping_add(1);
        }
        wr(m, 0x44, e);
    }
    o.chk(L40, 0xadde48, 0x1e7d);
    if splash {
        o.splash(L40, loc, 0xccb0);
    }
    o.log.push(format!("s9:m+0x0,{n}"));
    o.chk(L40, 0xadde40, 0x1e7f);
    o.log.push(format!("s10:{L40}"));
    o.log.push(format!("s11:{L40},{L40}"));
    for k in 0..6 {
        let v = rd(m, 0x4c + 4 * k);
        wr(&mut c[W], 0x38 + 4 * k, v);
    }
    if ty != 0x17 {
        o.log.push("s12:c0+0x0".to_string());
        o.chk(L40, 0xadde58, 0x1e9b);
        if splash {
            o.splash(L40, loc, 0xccec);
        }
        o.log.push(format!("s13:{L40}"));
        o.chk(L40, 0xadde50, 0x1ea3);
        o.log.push(format!("s10:{L40}"));
        o.log.push(format!("s11:{L40},{L40}"));
        o.chk(L40, 0xadde68, 0x1ea6);
        if splash {
            o.splash(L40, loc, 0xcd00);
        }
        o.log.push("s14:m+0x140".to_string());
        o.chk(L40, 0xadde60, 0x1eae);
        if c[GAME][0x821] & 2 == 0 {
            o.log.push(format!("s15:m+0x140,{flag},1"));
            o.chk(L40, 0xadde78, 0x1eb3);
            o.log.push(format!("s16:{L40}"));
            o.chk(L40, 0xadde70, 0x1eb5);
            o.log.push(format!("s17:{L40},{L40}"));
        }
        o.chk(L40, 0xadde88, 0x1eb9);
        for (off, s) in [(0x618, 18), (0x620, 19), (0x62c, 20)] {
            if rd(&c[TS], off) >= 0 {
                o.log.push(format!("s{s}:{L40}"));
            }
        }
        o.log.push(format!("s21:{L40}"));
        o.chk(L40, 0xadde80, 0x1ebe);
        if splash {
            o.splash(L40, loc, 0xcd14);
        }
        o.chk(L40, 0xadde98, 0x1ec5);
        o.chk(L40, 0xadde90, 0x1ec8);
        // The map logs: each one `operator=` of the format, a `parse`, `operator=` of its
        // answer, `close` of the parse's string, then the `Log::say`.
        let style = rd(&c[W], 0x30);
        if style >= 0 {
            o.log.push(format!("s1:{L2C},{}", ptr(0x17ebc)));
            let at = rd(&c[9], 0).wrapping_add(style.wrapping_mul(0x58));
            o.log.push(format!("s22:{L2C},{L54},{at}"));
            o.log.push(format!("s1:{L2C},{}", ans(22)));
            o.log.push(format!("s3:{L54}"));
            o.say(&mut c[GL], 5);
        }
        let size = i32::from(c[GAME][0x26]);
        o.log.push(format!("s1:{L2C},{}", ptr(0x17ed0)));
        o.log
            .push(format!("s22:{L2C},{L54},c{SZ}+{:#x}", size * 0x58));
        o.log.push(format!("s1:{L2C},{}", ans(22)));
        o.log.push(format!("s3:{L54}"));
        o.say(&mut c[GL], 5);
        let (w, h) = (rd(&c[W], 0), rd(&c[W], 4));
        o.log.push(format!("s1:{L2C},{}", ptr(0x17ee4)));
        o.log.push(format!("s24:{L2C},{L54},{w},{h}"));
        o.log.push(format!("s1:{L2C},{}", ans(24)));
        o.log.push(format!("s3:{L54}"));
        o.say(&mut c[GL], 5);
        o.log.push(format!("s1:{L2C},{}", ptr(0x17ef8)));
        let sz = rd(&c[SZ], 0x144);
        let pct = (sz / 2).wrapping_add(w.wrapping_mul(100)) / sz;
        o.log.push(format!("s25:{L2C},{L54},{}", pct.max(1)));
        o.log.push(format!("s1:{L2C},{}", ans(25)));
        o.log.push(format!("s3:{L54}"));
        o.say(&mut c[GL], 5);
        o.log.push(format!("s1:{L2C},{}", ptr(0x17f0c)));
        o.log.push(format!("s25:{L2C},{L54},{}", rd(&c[W], 8)));
        o.log.push(format!("s1:{L2C},{}", ans(25)));
        o.log.push(format!("s3:{L54}"));
        o.say(&mut c[GL], 5);
        if c[GAME][0x822] & 2 == 0 {
            for i in 0..n.max(0) {
                o.log.push(format!("s1:{L2C},{}", ptr(0x17f20)));
                let (x, y) = (rd(&p[1], 4 * i as usize), rd(&p[2], 4 * i as usize));
                let dx = x.wrapping_sub(w / 2).wrapping_abs();
                let dy = y.wrapping_sub(h / 2).wrapping_abs();
                let dist = crate::world::vector_dist(dx, dy);
                o.log.push(format!("s26:{L2C},{L54},{i},{x},{y},{dist}"));
                o.log.push(format!("s1:{L2C},{}", ans(26)));
                o.log.push(format!("s3:{L54}"));
                o.say(&mut c[GL], 6);
            }
        }
        if splash {
            o.splash(L40, loc, 0xcd28);
        }
        let no_rare = rd(&c[CQ], 0x1480) == 0 || rd(&p[0], 0x68) == 0;
        if c[GAME][0x821] & 2 == 0 && (c[GAME][0x822] & 2 == 0 || no_rare) {
            o.log.push(format!("s27:m+0x0,{n}"));
        }
        o.chk(L40, 0xaddea8, 0x1ef7);
        let mut res = L40;
        if splash {
            o.splash(L54, loc, 0xcd3c);
            res = L54;
        }
        o.log.push(format!("s28:{res},{res}"));
        o.log.push(format!("s10:{res}"));
        o.log.push(format!("s11:{res},{res}"));
        o.chk(L54, 0xaddea0, 0x1f01);
        res = L54;
        if splash {
            o.splash(L40, loc, 0xcd50);
            res = L40;
        }
        o.log.push(format!("s29:{res}"));
        o.chk(L54, 0xaddebc, 0x1f09);
        let sea = rd(m, 0x48);
        if sea >= 0 {
            wr(&mut c[W], 0x34, sea);
        }
        o.log.push(format!("s3:{L2C}"));
    }
    o.log.push(if o.says.is_empty() {
        "-".to_string()
    } else {
        o.says.join("/")
    });
}
// end hand

/// `Map::make@0068bc90`, run by `tools/emu/sweep_map_make.py` under unicorn
/// (tools/emu): an empty body: 384 random bytes of object and 3 argument(s), every byte held after the return and `esp` checked (parked 1889). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_map_make.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(11);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // hand: port
            let mut out = Out {
                log: Vec::new(),
                says: Vec::new(),
            };
            make_port(a, m, c, p, &mut out);
            log.extend(out.log);
            want_eax
            // end hand
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "make — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "make — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "make — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[12 + i], "make — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 600, "only {n} rows");
    eprintln!("Map::make@0068bc90: {n} rows agree");
}
