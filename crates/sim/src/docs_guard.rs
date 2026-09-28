//! The working agreement's paperwork rules, enforced rather than trusted.
//!
//! `docs/QUEUE.md` says of itself that it *subtracts*: a finished entry
//! leaves for the journal, the handoff is rewritten from scratch and is
//! about twenty lines. Within two days of writing that it was 409 lines,
//! its struck entries had grown paragraphs, and the handoff was seventy
//! lines — because nothing failed when they did. `no_float.rs` is the
//! precedent: a rule that is only prose is a rule a tired afternoon
//! defeats. So this module reads the documents and fails.
//!
//! Three rules, each of which was made to fail before it was landed
//! (2026-08-27):
//!
//! - **The queue deletes; it does not strike.** A finished item's story is
//!   in `docs/JOURNAL.md` under its number; the queue keeps nothing of it,
//!   not even a struck line. The file is bounded, and its handoff section
//!   is bounded harder.
//! - **`CLAUDE.md` carries rules, not findings.** A blind reader inherits
//!   it, so an address or a section number in it contaminates the reading.
//! - **Every `name@00xxxxxx` a specification cites is that function.**
//!   Added 2026-08-31 (queue item 89b, item 72's manual sweep made a
//!   check): the address is looked up in the decompile export's
//!   `INDEX.tsv` and the exported name must end with the cited one — the
//!   class where a name sits on a different function's address has cost a
//!   wrong conclusion more than once ("36 twice over"). Skips loudly on a
//!   machine without the export; `JOURNAL.md` and `docs/audit/` are not
//!   read, because both tell stories *about* wrong citations.
//! - **A section over the ceiling may not grow.** The specifications are
//!   the handoff between sessions, and the unit a session reads is the
//!   `## ` section, not the file. Every section is held under
//!   [`SECTION_CEILING`]; the ones already over it are pinned at their size
//!   in [`OVER`] and may only shrink — the way to add a correction to one is
//!   to split it, or to move its story out to the journal first. When one
//!   drops under the ceiling, delete its row. (Until 2026-08-30 the unit was
//!   the file, at 60 KB; the journal entry of that day says why it moved.)

use std::path::PathBuf;

fn docs() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs"))
}

fn read(name: &str) -> String {
    let path = docs().join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The most **open items** the queue may book, and the lines one may take.
///
/// This replaced a whole-file line count on 2026-09-07 (item 262, decided
/// with Ramon and lore). The line count was measured across 221 sessions
/// and cost **43 fitting episodes and 117 USD of list price** since
/// 08-25 — 22 of the 43 needed two or more guard runs, so the fitting loop
/// was the cost rather than the edit, and the "count the lines before
/// writing" rule of `d03c279` had not ended it.
///
/// The reason it is replaced rather than raised is **not** the money. A
/// global line count is satisfied by compressing *any* item, so its remedy
/// is a retelling of entries the author has no reason to have read, and
/// every one of those edits is a chance to drop somebody else's finding.
/// That is not hypothetical: `8b37e5f` was a post-crash rewrite under this
/// exact pressure, and items 232, 233 and 234 went out with the
/// compression unlanded and unnoticed for a day
/// (`docs/audit/queue-ledger.md`). A per-item cap localises the edit to
/// the item being added, by the person who knows what is safe to cut.
///
/// Fifteen items stood on the day this landed, the longest at seven lines.
const OPEN_ITEMS: usize = 18;
const ITEM_LINES: usize = 8;

/// The handoff section, in lines including blanks. The file says "about
/// twenty"; this is the tolerance.
const HANDOFF_LINES: usize = 32;

/// The queue's **non-item mass**, pinned by section at its size on
/// 2026-09-07 and may only shrink.
///
/// An item cap bounds items and nothing else, and the queue is not only
/// items: the preamble, the standing paragraphs inside "The queue", and the
/// maintenance notes all grew under the old line count without ever being
/// an item. So each section carries the same may-only-shrink pin the
/// specifications use ([`OVER`]) — the mechanism this project already
/// trusts — in lines rather than bytes, because lines are what a queue
/// section costs a reader. Lower a pin whenever a section comes in under
/// it; that is the only direction it moves.
///
/// "Where things stand" is absent on purpose: [`HANDOFF_LINES`] is its
/// bound and two caps on one section is the double-binding that made the
/// old fitting loop.
/// The sizes are **non-item lines**: an item's own lines are bounded by
/// [`ITEM_LINES`] and are subtracted here, so booking one costs a section
/// nothing. "The queue"'s 35 is its heading, its opening instruction, the
/// widening ledger and the `(242)` cluster.
const QUEUE_SECTIONS: &[(&str, usize)] = &[
    ("", 12),
    ("The queue", 8),
    ("How to maintain this file", 38),
];

/// Bytes, per `## ` section. The unit is the section because that is what
/// a session reads: an item names §6.4 and §4, never the file. A whole-file
/// ceiling (60 KB, 2026-08-27 to 08-30) taxed whoever added a finding to
/// any section of a large file, and what got cut under a 254-byte margin
/// was whatever the session personally needed least — once, nearly the
/// evidence (`docs/JOURNAL.md`, 2026-08-30, "what the byte pins are actually
/// doing"). Sixteen thousand is a section a session reads whole before
/// starting; the text before the first `## ` counts as a section too.
const SECTION_CEILING: usize = 16_000;

/// The sections over the ceiling on 2026-08-30, pinned at that day's size by
/// file and heading. Each may only shrink; a row whose section drops under
/// the ceiling, or whose heading is renamed, must be deleted. `JOURNAL.md`
/// is not measured: it is the append-only chronicle and is meant to grow.
/// A section that wants to grow past its pin splits — a `## ` heading is
/// the split, and it costs nothing a reader needs.
const OVER: &[(&str, &str, usize)] = &[
    // 71_928 -> 71_076 by the tenth pass, struck text no longer counted;
    // `AI.md` §15 left this table the same day — 27,421 bytes of which
    // 21,771 were struck, so it is 5,650 live bytes and under the ceiling.
    ("AI.md", "2. The production AI — read", 71_076),
    ("CITIES.md", "3. Construction", 16_280),
    // The three rows below were lowered by the tenth pass (2026-09-22)
    // when struck text stopped counting (parked 480): a section's pin is
    // its live bytes now, so each fell by what it had already struck.
    (
        "DATALAYER.md",
        "2. The loader — the tables into the sim's types",
        20_155,
    ),
    (
        "GROUPS.md",
        "6. The move — `Group::action_move_near@00704990`",
        44_686,
    ),
    // Lowered 37_601 -> 37_590 by item 405: §4.4 step 4 gains the ATTACK
    // arm's `max_range` gate and loses a gloss `docs/COLLISION.md` §5.1
    // already carries; 37_590 -> 37_584 by the tenth pass, the strike
    // item 475 paid for with a reflow.
    ("ORDERS.md", "4. The move order", 37_584),
    (
        "ORDERS.md",
        "5. Build, repair, garrison — and what a citizen does next",
        16_088,
    ),
];

/// `(heading, bytes)` for every `## ` section of a document, the preamble
/// first under an empty heading. A section's bytes run from its heading
/// line to the next `## ` line, newlines included.
fn sections(text: &str) -> Vec<(String, usize)> {
    let mut out: Vec<(String, usize)> = vec![(String::new(), 0)];
    for line in text.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            out.push((h.to_string(), 0));
        }
        out.last_mut().expect("preamble").1 += line.len() + 1;
    }
    // **Struck text is not live text** (parked 480, the tenth pass). The
    // ceiling measures what a reader has to read as a claim, and a
    // `~~…~~` span is the opposite of one: it is how this repo keeps a
    // wrong claim from being read as a live one, and the smallest honest
    // amendment there is. Item 475 owed §4.4 a 158-byte strike and paid
    // for it by reflowing an unrelated list, because the guard rationed a
    // correction like an addition. So a struck span's bytes are not
    // counted, its markers are, and a section at its pin gains exactly
    // the room it strikes.
    // A span is credited to the section its opening marker falls in;
    // `bounds[k]` is where section `k + 1` begins.
    let mut bounds: Vec<usize> = Vec::new();
    let mut pos = 0usize;
    for line in text.lines() {
        if line.starts_with("## ") {
            bounds.push(pos);
        }
        pos += line.len() + 1;
    }
    // A `~~~` is a code fence, not a marker; a span never crosses a
    // heading — an unpaired marker would otherwise strike the rest of
    // the file — so a span is credited up to its section's end at most.
    let bytes = text.as_bytes();
    let fence =
        |i: usize| (i > 0 && bytes[i - 1] == b'~') || bytes.get(i + 2).is_some_and(|&c| c == b'~');
    let mut open: Option<usize> = None;
    for (i, _) in text.match_indices("~~") {
        if fence(i) {
            continue;
        }
        match open.take() {
            None => open = Some(i + 2),
            Some(start) => {
                let sec = bounds.partition_point(|&b| b <= start);
                let end = bounds.get(sec).map_or(i, |&next| i.min(next));
                out[sec].1 -= end.saturating_sub(start);
            }
        }
    }
    out
}

/// The strike rule above, made to fail first: a section at its pin that
/// strikes a claim comes in *under* the pin, and a strike in one section
/// is never credited to another.
#[test]
fn struck_text_is_not_counted_against_a_section() {
    let live = "## A\n\nthe claim stands here\n\n## B\n\nanother\n";
    let struck = "## A\n\n~~the claim stands here~~ **struck**\n\n## B\n\nanother\n";
    let a = sections(live);
    let b = sections(struck);
    assert_eq!(a[1].0, "A");
    assert_eq!(
        b[1].1,
        a[1].1 + 4 + " **struck**".len() - "the claim stands here".len()
    );
    assert_eq!(b[2].1, a[2].1, "section B is untouched by A's strike");
    let multi = "## A\n\n~~one\nline two~~\n\n## B\n\n~~x~~\n";
    let m = sections(multi);
    assert_eq!(m[1].1, "## A\n\n~~\n\n".len() + "~~".len());
    assert_eq!(m[2].1, "## B\n\n~~~~\n".len());
}

#[test]
fn the_queue_deletes_rather_than_strikes() {
    let q = read("QUEUE.md");
    let struck: Vec<usize> = q
        .lines()
        .enumerate()
        .filter(|(_, l)| l.contains("~~"))
        .map(|(i, _)| i + 1)
        .collect();
    assert!(
        struck.is_empty(),
        "docs/QUEUE.md strikes entries at lines {struck:?}; delete them — the story is the journal's"
    );
}

/// `(number, lines)` for every open item the queue books. An item opens a
/// paragraph — `N. **The claim**` after a blank line — and runs to the next
/// blank line. The blank line is what keeps prose out: the queue wraps, so
/// a sentence ending in a number leaves a bare `307.` at the head of the
/// next line and nothing else tells the two apart. The same reading as
/// `tools/queueledger.py`'s.
fn open_items(text: &str) -> Vec<(u32, usize)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let opens = (i == 0 || lines[i - 1].trim().is_empty())
            && lines[i]
                .split_once(". ")
                .and_then(|(n, _)| n.parse::<u32>().ok())
                .is_some();
        if opens {
            let n: u32 = lines[i]
                .split_once(". ")
                .expect("checked")
                .0
                .parse()
                .expect("checked");
            let mut len = 0;
            while i + len < lines.len() && !lines[i + len].trim().is_empty() {
                len += 1;
            }
            out.push((n, len));
            i += len;
        } else {
            i += 1;
        }
    }
    out
}

/// `(number, text)` for every open item, by the same reading as [`open_items`].
fn open_item_texts(text: &str) -> Vec<(u32, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    for (n, len) in open_items(text) {
        while i < lines.len()
            && !((i == 0 || lines[i - 1].trim().is_empty())
                && lines[i].starts_with(&format!("{n}. ")))
        {
            i += 1;
        }
        out.push((n, lines[i..i + len].join("\n")));
        i += len;
    }
    out
}

/// **A capture booked on a map cites the window the disk already holds**
/// (parked 575, the twelfth pass). "Grep the disk before booking a
/// capture" was prose, and the eleventh pass — the one that wrote the
/// clause — booked 573 with "no capture on disk reaches it" while run99's
/// stanza spanned the frame and its dump sat in the Logs directory. So
/// the ledger is read: every stanza in `tools/gamelog/captures.txt` with
/// a `frame_window:` (or the older `window:`) is a window on its
/// `mapstyle:`, and an open queue item that books a capture on a map —
/// the word "capture" and the map's name — names every such window that
/// spans the item's frame, which is its first number of four digits. An
/// item that names the window and books the capture anyway is saying what
/// the disk could not answer, which is the rule; one that does not name
/// it has not looked. Made to fail first on 571 with its `run136` spelled
/// apart.
#[test]
fn a_capture_booked_on_a_map_cites_the_window_the_disk_holds() {
    let q = read("QUEUE.md");
    let path = docs().join("../tools/gamelog/captures.txt");
    let ledger =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut windows: Vec<(u32, u32, i64, i64)> = Vec::new();
    let (mut run, mut map, mut win) = (None, None, None);
    for line in ledger.lines().chain(std::iter::once("")) {
        if line.trim().is_empty() {
            if let (Some(r), Some(m), Some((lo, hi))) = (run, map, win) {
                windows.push((r, m, lo, hi));
            }
            (run, map, win) = (None, None, None);
            continue;
        }
        if let Some(v) = line.strip_prefix("run:") {
            run = v.trim().parse().ok();
        } else if let Some(v) = line.strip_prefix("mapstyle:") {
            map = v.trim().parse().ok();
        } else if let Some(v) = line
            .strip_prefix("frame_window:")
            .or_else(|| line.strip_prefix("window:"))
        {
            let mut it = v.split_whitespace().filter_map(|x| x.parse::<i64>().ok());
            if let (Some(lo), Some(hi)) = (it.next(), it.next()) {
                win = Some((lo, hi));
            }
        }
    }
    assert!(
        windows.len() >= 40,
        "captures.txt parsed {} windowed stanzas; the ledger's form changed and this \
         guard is checking nothing",
        windows.len()
    );
    let mut bad = Vec::new();
    for (n, text) in open_item_texts(&q) {
        // A golden chapter's captures are `golden_capture.sh`'s and its
        // ledger is `docs/GOLDEN.md` §14, not this file.
        if !text.contains("capture") || text.to_lowercase().contains("chapter") {
            continue;
        }
        // The third scored map first (DECISIONS 54): its item names the
        // other two only to say what it is not.
        let map = if text.contains("Great Sahara") {
            7
        } else if text.contains("East Indies") {
            18
        } else if text.contains("Great Lakes") {
            14
        } else {
            // 571 as first written: a squad, a block, a capture — and no map,
            // so nothing could check the ledger against it.
            bad.push(format!("item {n} books a capture and names no map"));
            continue;
        };
        let Some(frame) = booked_frame(&text) else {
            continue;
        };
        for (r, m, lo, hi) in &windows {
            if *m == map && (*lo..*hi).contains(&frame) && !text.contains(&format!("run{r}")) {
                bad.push(format!(
                    "item {n} books a capture at {frame} and does not cite run{r} \
                     ({lo}..{hi}), which tools/gamelog/captures.txt says spans it"
                ));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "a booking cites what the disk could not answer (CLAUDE.md): {bad:#?}"
    );
}

/// The frame an open item books: the number after its first `frame`, else
/// its first number of four digits or more — **after the item's own
/// number**. Until the eighteenth pass the guard above took the first
/// number past 999 anywhere in the item, and from item 1000 on that was
/// the item's number: every booking's frame read as `1061` or `1050`, and
/// the ledger's windows were checked against a frame nobody booked
/// (parked 1006, "the guards' own parsers want the same check").
fn booked_frame(item: &str) -> Option<i64> {
    let numbers = |t: &str| -> Vec<i64> {
        t.split(|c: char| !c.is_ascii_digit())
            .filter_map(|d| d.parse().ok())
            .collect()
    };
    let body = item.split_once(". ").map_or(item, |(_, rest)| rest);
    if let Some((_, after)) = body.split_once("frame ") {
        let digits: String = after
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == ',')
            .filter(char::is_ascii_digit)
            .collect();
        if let Ok(f) = digits.parse::<i64>() {
            return Some(f);
        }
    }
    numbers(body).into_iter().find(|&f| f >= 1000)
}

#[test]
fn an_item_s_frame_is_not_its_number() {
    assert_eq!(
        booked_frame("1061. **Great Lakes' second word: frame 4924, ours 8 draws** (1052)."),
        Some(4924)
    );
    assert_eq!(
        booked_frame("1050. **Chapter thirty-five** (1048; parked 1051): frame 2,701, the round"),
        Some(2701)
    );
    assert_eq!(
        booked_frame("571. **A squad on block 10899**, a capture"),
        Some(10899)
    );
    assert_eq!(
        booked_frame("1066. **A third map**, no number past it"),
        None
    );
}

/// The queue books a bounded number of open items, each of a bounded size.
///
/// **This is the whole budget** — there is no line count on the file any
/// more (see [`OPEN_ITEMS`]). The two failures it can report are the two
/// remedies that are actually local: an item over its lines is shortened by
/// whoever is touching it, and a queue over its items has one to finish or
/// park. `docs/PARKED.md` is where a parked one goes, and moving it there
/// is not a deletion — `tools/queueledger.py` reads both files.
#[test]
fn the_queue_caps_items_not_lines() {
    let items = open_items(&read("QUEUE.md"));
    let long: Vec<String> = items
        .iter()
        .filter(|(_, len)| *len > ITEM_LINES)
        .map(|(n, len)| format!("{n} ({len} lines)"))
        .collect();
    assert!(
        long.is_empty(),
        "docs/QUEUE.md items are over {ITEM_LINES} lines: {}. Shorten the item you are \
         touching — do not compress the rest of the file to make room (item 262)",
        long.join(", ")
    );
    assert!(
        items.len() <= OPEN_ITEMS,
        "docs/QUEUE.md books {} open items; the cap is {OPEN_ITEMS}. Finish one, or park \
         one in docs/PARKED.md — which is a move, not a deletion",
        items.len()
    );
}

/// Every pinned queue section is at or under its pin, and a section that
/// has come in under it says so rather than banking the slack.
#[test]
fn a_queue_section_may_only_shrink() {
    let q = read("QUEUE.md");
    // **The item lines do not count.** If they did, booking an ordinary
    // item would push its section past the pin and send the author off to
    // compress the rest of the file — the fitting loop item 262 removed,
    // rebuilt one level down. It was rebuilt, briefly, and the fixture
    // that was meant to prove the item cap caught it. So a section is
    // measured by what is *not* an item: prose, standing paragraphs, the
    // maintenance notes. Items are bounded by their own cap and by nothing
    // else.
    let lines: Vec<&str> = q.lines().collect();
    let mut item_line = vec![false; lines.len()];
    let mut i = 0;
    while i < lines.len() {
        let opens = (i == 0 || lines[i - 1].trim().is_empty())
            && lines[i]
                .split_once(". ")
                .and_then(|(n, _)| n.parse::<u32>().ok())
                .is_some();
        if opens {
            while i < lines.len() && !lines[i].trim().is_empty() {
                item_line[i] = true;
                i += 1;
            }
            // The blank line that separates it from the next entry belongs
            // to the item too. Without this an item still costs its
            // section one line, which is a fitting loop with a slower fuse.
            if i < lines.len() {
                item_line[i] = true;
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    let mut have: Vec<(String, usize)> = vec![(String::new(), 0)];
    for (i, line) in lines.iter().enumerate() {
        if let Some(h) = line.strip_prefix("## ") {
            have.push((h.to_string(), 0));
        }
        if !item_line[i] {
            have.last_mut().expect("preamble").1 += 1;
        }
    }
    for (heading, pin) in QUEUE_SECTIONS {
        let (_, size) = have.iter().find(|(h, _)| h == heading).unwrap_or_else(|| {
            panic!("docs/QUEUE.md has no section {heading:?}; QUEUE_SECTIONS names it")
        });
        let label = if heading.is_empty() {
            "the preamble"
        } else {
            heading
        };
        assert!(
            size <= pin,
            "docs/QUEUE.md's {label} is {size} lines against its pin of {pin}. \
             The pin only falls: shorten this section rather than another (item 262)"
        );
        assert!(
            *size + 4 > *pin,
            "docs/QUEUE.md's {label} is {size} lines, well under its pin of {pin}; \
             lower the pin in QUEUE_SECTIONS to bank the shrink"
        );
    }
}

#[test]
fn the_handoff_is_short() {
    let q = read("QUEUE.md");
    let mut in_section = false;
    let mut counted: Vec<&str> = Vec::new();
    for line in q.lines() {
        if line.starts_with("## ") {
            if in_section {
                break;
            }
            in_section = line.starts_with("## Where things stand");
            continue;
        }
        if in_section {
            counted.push(line);
        }
    }
    let n = counted.len();
    assert!(
        n > 0,
        "docs/QUEUE.md has no '## Where things stand' section"
    );
    // The span is named so trimming is not guess-and-retry (parked 374):
    // every line from the heading to the next `## `, blank lines included.
    assert!(
        n <= HANDOFF_LINES,
        "the handoff is {n} lines; the bound is {HANDOFF_LINES}. Counted: every line \
         after `## Where things stand` up to the next `## `, blanks included — \
         from {:?} to {:?}. It is rewritten from scratch, not appended to; the \
         literal phrases `Scoreboard:`, `Long captures:`, `Golden:`, `Endpoint ` \
         and `Fable backlog: N Loop items` are read by other guards and stay",
        counted.iter().find(|l| !l.trim().is_empty()).unwrap_or(&""),
        counted
            .iter()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or(&"")
    );
}

/// The steering pass's backlog is the parked file's Loop section, which no
/// session reads at boot; the queue's handoff carries its count so a Fable
/// session sees it in the same breath as the score (parked item 332, ruled
/// 2026-09-18). A count nobody checks drifts — it read "twelve" over thirteen
/// items the day this was written — so the line is parsed against the section.
#[test]
fn the_handoff_counts_the_loop_backlog() {
    let q = read("QUEUE.md");
    let line = q
        .lines()
        .find(|l| l.contains("Fable backlog:"))
        .expect("docs/QUEUE.md's handoff has no `Fable backlog: N Loop items` line");
    let after = line.split("Fable backlog:").nth(1).unwrap();
    let word = after
        .trim_start_matches([' ', '*'])
        .split_whitespace()
        .next()
        .unwrap_or("");
    const WORDS: [&str; 21] = [
        "zero",
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "eleven",
        "twelve",
        "thirteen",
        "fourteen",
        "fifteen",
        "sixteen",
        "seventeen",
        "eighteen",
        "nineteen",
        "twenty",
    ];
    let said = word
        .parse::<usize>()
        .ok()
        .or_else(|| WORDS.iter().position(|w| *w == word))
        .unwrap_or_else(|| panic!("`Fable backlog:` is followed by {word:?}, not a count"));
    let parked = read("PARKED.md");
    let mut in_loop = false;
    let mut items = Vec::new();
    for l in parked.lines() {
        if l.starts_with("## ") {
            in_loop = l.starts_with("## Loop");
            continue;
        }
        // An item opens `(N) **…`; a bare `(N) ` mid-item is a reference.
        let Some((number, _)) = l.strip_prefix('(').and_then(|l| l.split_once(") **")) else {
            continue;
        };
        if let (true, Ok(n)) = (in_loop, number.parse::<u32>()) {
            items.push(n);
        }
    }
    assert!(
        !items.is_empty(),
        "docs/PARKED.md has no `## Loop` section with `(N) **` items"
    );
    assert_eq!(
        said,
        items.len(),
        "docs/QUEUE.md's handoff says the Fable backlog is {said} Loop items; docs/PARKED.md's \
         Loop section holds {}: {items:?}. Rewrite the count with the handoff",
        items.len()
    );
}

/// **An item number is minted once, and in its file's form** (parked 461,
/// the eighth pass). Item 457 parked two real findings as `308.` and
/// `309.` — both numbers already taken, one by a landed item with a journal
/// entry, one by a live parked entry three hundred lines up the same file
/// — and nothing fired: `tools/queueledger.py` asks whether a number was
/// ever booked, not whether it is booked twice, and the counting guards
/// key on the parked file's `(N) **` form, which a queue-form `N. **`
/// entry is invisible to. So this collects every open item across the
/// queue (`N. **`, paragraph-initial), the parked file (`(N) **`,
/// line-initial) and the journal directory (`<date>-item-N.md`), and
/// fails on a number that is live twice, on a landed number standing as a
/// live entry, and on an entry written in the other file's form.
///
/// **Its first run found three**: `306.` and `307.` standing in the
/// parked file's older backlog in the queue's form, and `(423)` live in
/// the parked file a week after item 423 landed with a journal entry.
#[test]
fn an_item_number_is_minted_once_and_in_its_file_s_form() {
    use std::collections::BTreeMap;
    let queue = read("QUEUE.md");
    let parked = read("PARKED.md");
    let mut where_live: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    let mut wrong_form: Vec<String> = Vec::new();
    for (n, _) in open_items(&queue) {
        where_live
            .entry(n)
            .or_default()
            .push("docs/QUEUE.md".into());
    }
    for (i, l) in queue.lines().enumerate() {
        if let Some(rest) = l.strip_prefix('(')
            && let Some((n, _)) = rest.split_once(") **")
            && n.parse::<u32>().is_ok()
        {
            wrong_form.push(format!(
                "docs/QUEUE.md:{}: `({n}) **` is the parked file's form; the queue books `{n}. **`",
                i + 1
            ));
        }
    }
    for (i, l) in parked.lines().enumerate() {
        if let Some(rest) = l.strip_prefix('(')
            && let Some((n, _)) = rest.split_once(") **")
            && let Ok(n) = n.parse::<u32>()
        {
            where_live
                .entry(n)
                .or_default()
                .push(format!("docs/PARKED.md:{}", i + 1));
        }
        if let Some((n, rest)) = l.split_once(". ")
            && rest.starts_with("**")
            && n.parse::<u32>().is_ok()
        {
            wrong_form.push(format!(
                "docs/PARKED.md:{}: `{n}. **` is the queue's form; the parked file holds `({n}) **`",
                i + 1
            ));
        }
    }
    let mut landed: BTreeMap<u32, String> = BTreeMap::new();
    for entry in std::fs::read_dir(docs().join("journal")).expect("docs/journal") {
        let name = entry
            .expect("entry")
            .file_name()
            .to_string_lossy()
            .into_owned();
        if let Some(stem) = name.strip_suffix(".md")
            && let Some((_, n)) = stem.rsplit_once("-item-")
            && let Ok(n) = n.parse::<u32>()
        {
            landed.insert(n, format!("docs/journal/{name}"));
        }
    }
    let twice: Vec<String> = where_live
        .iter()
        .filter(|(_, at)| at.len() > 1)
        .map(|(n, at)| format!("{n} is live at {}", at.join(" and ")))
        .collect();
    let landed_and_live: Vec<String> = where_live
        .iter()
        .filter_map(|(n, at)| {
            landed.get(n).map(|j| {
                format!(
                    "{n} landed ({j}) and still stands live at {}",
                    at.join(", ")
                )
            })
        })
        .collect();
    assert!(
        twice.is_empty() && landed_and_live.is_empty() && wrong_form.is_empty(),
        "an item number is minted once, by the commander, in its file's form.\n\
         live twice: {twice:#?}\nlanded and still live: {landed_and_live:#?}\n\
         wrong form: {wrong_form:#?}\n\
         A worker never mints a number (CLAUDE.md); a landed item's parked entry is \
         closed — `(N) closed <date> by item N: …` — or deleted; the next free number is \
         one past the highest anywhere"
    );
}

/// **No `docs/RUNS.md` section heading stands twice** (parked 428, the
/// eighth pass). Two lanes appending a capture each conflicted at the
/// file's end on 2026-09-19 with their run numbers properly reserved,
/// because the append *point* is the same line for both. `.gitattributes`
/// now merges the file with the `union` driver, which keeps both appended
/// sections instead of refusing — and whose one failure mode, a hunk
/// applied twice, is a heading that appears twice. This is the check
/// behind that driver; a duplicated heading is a merge to redo by hand.
#[test]
fn no_runs_section_heading_stands_twice() {
    let runs = read("RUNS.md");
    let mut seen: std::collections::BTreeMap<&str, Vec<usize>> = Default::default();
    for (i, l) in runs.lines().enumerate() {
        if l.starts_with("## ") {
            seen.entry(l).or_default().push(i + 1);
        }
    }
    let twice: Vec<String> = seen
        .iter()
        .filter(|(_, at)| at.len() > 1)
        .map(|(h, at)| format!("{h:?} at lines {at:?}"))
        .collect();
    assert!(
        twice.is_empty(),
        "docs/RUNS.md carries a section heading twice — the union merge applied a hunk \
         on both sides, or a section was pasted twice: {twice:#?}"
    );
    let attrs = std::fs::read_to_string(docs().join("../.gitattributes")).unwrap_or_default();
    assert!(
        attrs
            .lines()
            .any(|l| l.split_whitespace().collect::<Vec<_>>() == ["docs/RUNS.md", "merge=union"]),
        ".gitattributes no longer merges docs/RUNS.md with the union driver; two lanes' \
         captures will conflict at the append point again (parked 428)"
    );
}

#[test]
fn claude_md_carries_no_findings() {
    let text = std::fs::read_to_string(docs().join("../CLAUDE.md")).expect("CLAUDE.md");
    let bad: Vec<(usize, &str)> = text
        .lines()
        .enumerate()
        .filter(|(_, l)| {
            // A function address, an offset, or a section citation is a
            // finding; a blind reader must not inherit one.
            let hex = l.match_indices("0x").any(|(i, _)| {
                l[i + 2..]
                    .chars()
                    .take_while(|c| c.is_ascii_hexdigit())
                    .count()
                    >= 4
            });
            // `§` followed by a digit is a citation; the bare sign is prose.
            let cite = l.match_indices('§').any(|(i, _)| {
                l[i + '§'.len_utf8()..]
                    .trim_start()
                    .starts_with(|c: char| c.is_ascii_digit())
            });
            hex || l.contains("@00") || cite
        })
        .map(|(i, l)| (i + 1, l))
        .collect();
    assert!(
        bad.is_empty(),
        "CLAUDE.md names findings a reader is meant to re-derive: {bad:#?}"
    );
}

/// **No conflict marker survives anywhere in the tree** (parked 420, the
/// seventh pass). One stray `=======` at `tools/gamelog/captures.txt:3387`,
/// left by merge `9ae8070` with neither `<<<<<<<` nor `>>>>>>>` beside it,
/// made `runqueue.sh` refuse the whole file with `unknown key '======='`
/// for an unknown number of days — every stanza after run107's was
/// unreachable by the capture driver, and a lane looking for a booked
/// capture would have read it as never booked (item 414 deleted it). Text
/// files under `docs/`, `tools/`, `crates/` and the root, by extension;
/// made to fail first on a fixture line in `tools/gamelog/captures.txt`.
#[test]
fn no_conflict_marker_survives_in_the_tree() {
    const TEXT: &[&str] = &[
        "md", "rs", "py", "sh", "txt", "toml", "cmd", "ini", "c", "h", "json", "yml", "yaml",
    ];
    fn walk(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
            let path = entry.expect("entry").path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if path.is_dir() {
                if !matches!(name, "target" | ".git" | "node_modules" | "__pycache__") {
                    walk(&path, out);
                }
            } else if path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| TEXT.contains(&e))
            {
                out.push(path);
            }
        }
    }
    let root = docs().join("..");
    let mut files = Vec::new();
    for top in ["docs", "tools", "crates"] {
        walk(&root.join(top), &mut files);
    }
    for entry in std::fs::read_dir(&root).expect("repo root") {
        let path = entry.expect("entry").path();
        if path.is_file()
            && path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| TEXT.contains(&e))
        {
            files.push(path);
        }
    }
    let mut bad = Vec::new();
    for path in &files {
        let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let text = String::from_utf8_lossy(&bytes);
        for (i, l) in text.lines().enumerate() {
            if l.starts_with("<<<<<<< ")
                || l == "======="
                || l.starts_with(">>>>>>> ")
                || l == "|||||||"
            {
                bad.push(format!(
                    "{}:{}: {l}",
                    path.strip_prefix(&root).unwrap_or(path).display(),
                    i + 1
                ));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "conflict markers survive in the tree; a driver that reads the file refuses it whole: {bad:#?}"
    );
}

/// `Class<T,U>::method` → `Class::method`: the export prints template
/// arguments, the documents cite without them.
fn strip_templates(name: &str) -> String {
    let mut out = String::new();
    let mut depth = 0usize;
    for c in name.chars() {
        match c {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

/// Every `name@00xxxxxx` in a document, as `(line, name, address)`. The
/// name is the run of identifier characters (plus `:` and `~`) before the
/// `@`; a citation wrapped across a line break yields the suffix that
/// survived, which the ends-with match accepts. **A break after the `@`
/// is joined** (parked 835, the sixteenth pass): `name@` ending a line
/// and `00xxxxxx` opening the next were invisible here and to
/// `tools/census.py` — 49 citations across `docs/` when the pass counted,
/// each unchecked against the index and each counted as two functions.
fn cites(text: &str) -> Vec<(usize, String, u32)> {
    let mut out = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let bytes = line.as_bytes();
        for (at, _) in line.match_indices('@') {
            let mut hex: String = line[at + 1..]
                .chars()
                .take_while(|c| c.is_ascii_hexdigit())
                .collect();
            if hex.is_empty() && line[at + 1..].trim().is_empty() {
                hex = lines
                    .get(i + 1)
                    .map(|next| {
                        next.trim_start()
                            .chars()
                            .take_while(|c| c.is_ascii_hexdigit())
                            .collect()
                    })
                    .unwrap_or_default();
            }
            if hex.len() != 8 || !hex.starts_with("00") {
                continue;
            }
            let mut start = at;
            while start > 0 {
                let c = bytes[start - 1] as char;
                if c.is_ascii_alphanumeric() || c == '_' || c == ':' || c == '~' {
                    start -= 1;
                } else {
                    break;
                }
            }
            let name = line[start..at].trim_start_matches(':').to_string();
            let addr = u32::from_str_radix(&hex, 16).expect("hex");
            out.push((i + 1, name, addr));
        }
    }
    out
}

#[test]
fn every_cited_address_names_its_function() {
    let home = std::env::var("HOME").unwrap_or_default();
    let index_path = format!("{home}/ghidra-projects/decomp/INDEX.tsv");
    let Ok(index) = std::fs::read_to_string(&index_path) else {
        eprintln!("skipping: no {index_path} (the Ghidra export is not on this machine)");
        return;
    };
    let mut funcs = std::collections::BTreeMap::new();
    let mut last = 0u32;
    for line in index.lines() {
        let mut it = line.split('\t');
        let (Some(addr), Some(name)) = (it.next(), it.next()) else {
            continue;
        };
        let Ok(addr) = u32::from_str_radix(addr, 16) else {
            continue;
        };
        funcs.insert(addr, strip_templates(name).to_lowercase());
        last = last.max(addr);
    }
    let mut failures = Vec::new();
    for entry in std::fs::read_dir(docs()).expect("docs/") {
        let path = entry.expect("entry").path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.ends_with(".md") || name == "JOURNAL.md" {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read");
        for (line, cited, addr) in cites(&text) {
            let c = cited.to_lowercase();
            match funcs.get(&addr) {
                // An empty name is a cite whose name sits behind an
                // argument list or a line wrap: the address half still
                // checks. Either side may be the more qualified — the
                // documents cite `add` for `LinkListBase<…>::add`, and
                // `OrdersMemManager::get_new_order` where the export
                // holds the bare global.
                Some(f) if c.is_empty() || f.ends_with(&c) || c.ends_with(f.as_str()) => {}
                Some(f) => failures.push(format!(
                    "docs/{name}:{line}: `{cited}@{addr:08x}` — the export names {addr:08x} `{f}`"
                )),
                // Past the last function it is data, which a function
                // index cannot check (the four `PATHDATA` tables live
                // there); inside the range an unknown address is a typo.
                None if addr > last => {}
                None => failures.push(format!(
                    "docs/{name}:{line}: `{cited}@{addr:08x}` — no function in the export starts there"
                )),
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// A section number at the head of `text`: `23`, `4.4`, `14a`, `70.6`.
fn section_number(text: &str) -> Option<String> {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    loop {
        let mut digits = String::new();
        while let Some(c) = chars.peek().copied().filter(char::is_ascii_digit) {
            digits.push(c);
            chars.next();
        }
        if digits.is_empty() {
            break;
        }
        out.push_str(&digits);
        if let Some(c) = chars.peek().copied().filter(char::is_ascii_lowercase) {
            // `14a` is a section; `4th` and `3rd` are not, and neither is
            // a number run into a word.
            let mut rest = chars.clone();
            rest.next();
            if !rest.peek().is_some_and(|n| n.is_ascii_alphanumeric()) {
                out.push(c);
                chars.next();
            }
        }
        let mut rest = chars.clone();
        if rest.next() == Some('.') && rest.peek().is_some_and(char::is_ascii_digit) {
            out.push('.');
            chars.next();
        } else {
            break;
        }
    }
    (!out.is_empty()).then_some(out)
}

/// The numbered sections a document holds: a heading that opens with a
/// number, and a paragraph that opens with one in bold (`**70.6**`), which
/// is how the longer sections number their parts.
fn sections_of(text: &str) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for line in text.lines() {
        let body = if line.starts_with('#') {
            line.trim_start_matches('#')
        } else if line.starts_with("**") {
            line
        } else {
            continue;
        };
        let body =
            body.trim_start_matches(|c: char| c.is_whitespace() || matches!(c, '~' | '*' | '§'));
        if let Some(n) = section_number(body) {
            out.insert(n);
        }
    }
    out
}

/// `(document, section)` for every `DOC §N` or `` `docs/DOC.md` §N `` on a
/// line. The name and the sign share a line or the citation is not read.
fn section_cites(line: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (at, sign) in line.match_indices('§') {
        let Some(section) = section_number(line[at + sign.len()..].trim_start()) else {
            continue;
        };
        let before = line[..at].trim_end().trim_end_matches('`');
        let before = before.strip_suffix(".md").unwrap_or(before);
        let name: String = before
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_uppercase() || *c == '_')
            .collect::<Vec<char>>()
            .into_iter()
            .rev()
            .collect();
        if name.len() >= 2 {
            out.push((name, section));
        }
    }
    out
}

/// The sections the code cites and the documents do not hold, pinned on
/// the day the guard landed: thirteen, at thirty-five sites. Each is a
/// comment pointing at a section that was renumbered, never numbered
/// (`docs/TECH.md` has no numbered section at all) or never written. A row
/// is deleted when its citation is mended; a new one fails.
const NO_SUCH_SECTION: &[(&str, &str)] = &[
    ("AI", "2.5.1"),
    ("ANIM", "4.1"),
    ("ANIM", "4.3"),
    ("ANIM", "4.4"),
    ("ARMY", "15.8"),
    ("ARMY", "15.9"),
    ("ARMY", "16.3"),
    ("CARAVAN", "3.1"),
    ("COMBAT", "12.5"),
    ("COMBAT", "7.12"),
    ("PATHFINDER", "18.6"),
    ("TECH", "13"),
    ("TECH", "7"),
];

/// **A section the code cites is one the document holds** (parked 1013,
/// the eighteenth pass). A specification's section numbers are an API the
/// code cites (`CLAUDE.md`), and nothing checked the call: `do_move`'s
/// comment named an arm under a section this crate's code did not carry,
/// and measured whole the crates cite 3,330 sections of which thirteen do
/// not exist. What a comment *says* of a section stays a reading; that the
/// section is there is this.
#[test]
fn a_section_the_code_cites_is_one_the_document_holds() {
    assert_eq!(
        section_cites("// the margin (`docs/ORDERS.md` §4.4 step 4), and COMBAT §70.6; §9 alone"),
        vec![
            ("ORDERS".to_string(), "4.4".to_string()),
            ("COMBAT".to_string(), "70.6".to_string()),
        ]
    );
    let held = sections_of(
        "## 4. The draw\n### 4.4 The arms\n**70.6** A part.\n## Coverage\n## 14a. More\n",
    );
    assert_eq!(
        held.iter().map(String::as_str).collect::<Vec<_>>(),
        vec!["14a", "4", "4.4", "70.6"]
    );

    let mut held = std::collections::BTreeMap::new();
    for entry in std::fs::read_dir(docs()).expect("docs/") {
        let path = entry.expect("entry").path();
        if let Some(stem) = path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.strip_suffix(".md"))
        {
            held.insert(
                stem.to_string(),
                sections_of(&std::fs::read_to_string(&path).expect("read")),
            );
        }
    }
    let mut missing: std::collections::BTreeMap<(String, String), String> =
        std::collections::BTreeMap::new();
    let mut stack = vec![docs().join("..").join("crates")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("crates/") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs")
                && path.file_name().is_some_and(|n| n != "docs_guard.rs")
            {
                let text = std::fs::read_to_string(&path).expect("read");
                for (i, line) in text.lines().enumerate() {
                    for (doc, section) in section_cites(line) {
                        if held.get(&doc).is_some_and(|h| !h.contains(&section)) {
                            missing
                                .entry((doc, section))
                                .or_insert_with(|| format!("{}:{}", path.display(), i + 1));
                        }
                    }
                }
            }
        }
    }
    let pinned = |d: &str, s: &str| NO_SUCH_SECTION.iter().any(|(pd, ps)| *pd == d && *ps == s);
    let new: Vec<String> = missing
        .iter()
        .filter(|((d, s), _)| !pinned(d, s))
        .map(|((d, s), at)| format!("docs/{d}.md has no §{s}, cited first at {at}"))
        .collect();
    let mended: Vec<String> = NO_SUCH_SECTION
        .iter()
        .filter(|(d, s)| !missing.contains_key(&(d.to_string(), s.to_string())))
        .map(|(d, s)| format!("{d} §{s} is cited nowhere or exists now; delete its row"))
        .collect();
    assert!(
        new.is_empty() && mended.is_empty(),
        "a section the code cites is one the document holds (parked 1013):\n{}\n{}",
        new.join("\n"),
        mended.join("\n")
    );
}

/// Every qualified function name in `text` — `Class::method`, with or
/// without a `+0x..` site or an `@address` after it — that `known` does not
/// hold. A name is qualified or it is not read: a bare `come_out` is as
/// likely a field as a function.
fn unknown_functions(text: &str, known: &std::collections::BTreeSet<String>) -> Vec<String> {
    let mut out = Vec::new();
    let b = text.as_bytes();
    for (at, _) in text.match_indices("::") {
        let ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
        let mut start = at;
        while start > 0 && ident(b[start - 1]) {
            start -= 1;
        }
        let mut end = at + 2;
        while end < b.len() && ident(b[end]) {
            end += 1;
        }
        // A class is capitalised and a method is not; `rondata::diff` and
        // `sim::collide` are this crate's paths and are neither.
        let (class, method) = (&text[start..at], &text[at + 2..end]);
        if class.is_empty()
            || method.is_empty()
            || !class.as_bytes()[0].is_ascii_uppercase()
            || !method.as_bytes()[0].is_ascii_lowercase()
            || (start > 0 && b[start - 1] == b':')
        {
            continue;
        }
        let name = format!("{class}::{method}");
        if !known.contains(&name.to_lowercase()) && !out.contains(&name) {
            out.push(name);
        }
    }
    out
}

/// **A function a booking names is one the export holds** (parked 1010,
/// the eighteenth pass). Item 976's queue line named `issue_launch_flight`
/// from a parked row's wording; the export holds no such function, and the
/// worker's first hour was finding which one was meant. The address guard
/// above checks a name against its address; a booking names a function
/// with a site and no address, so its names are checked against the index
/// whole. Made to fail first on a line that names one the index lacks.
#[test]
fn a_function_a_booking_names_is_in_the_export() {
    let fixture: std::collections::BTreeSet<String> =
        ["group::action_launch_flight", "guy::set_anim"]
            .iter()
            .map(|s| s.to_string())
            .collect();
    assert_eq!(
        unknown_functions(
            "ours `Guy::set_anim+0xf2f`, theirs `CommandManager::issue_launch_flight`, \
             pinned in `rondata::diff` and `Group::action_launch_flight@0070d830`",
            &fixture
        ),
        vec!["CommandManager::issue_launch_flight".to_string()]
    );
    let home = std::env::var("HOME").unwrap_or_default();
    let index_path = format!("{home}/ghidra-projects/decomp/INDEX.tsv");
    let Ok(index) = std::fs::read_to_string(&index_path) else {
        eprintln!("skipping: no {index_path} (the Ghidra export is not on this machine)");
        return;
    };
    let known: std::collections::BTreeSet<String> = index
        .lines()
        .filter_map(|l| l.split('\t').nth(1))
        .map(|n| strip_templates(n).to_lowercase())
        .collect();
    let queue = read("QUEUE.md");
    let mut failures = Vec::new();
    for (n, text) in open_item_texts(&queue) {
        for name in unknown_functions(&text, &known) {
            failures.push(format!(
                "item {n} names `{name}`, which the export does not hold"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "a booking names a function by its `INDEX.tsv` row (parked 1010):\n{}",
        failures.join("\n")
    );
}

/// How many of `rondata::blind::RESIDUE`'s rows say "no reference in the
/// executable" — the dead list's length, pinned so a new row is a decision.
const UNREFERENCED: usize = 27;

/// What `docs/EMULATOR.md` §4 enumerates as dead and the residue table does
/// not hold as unreferenced, each for a reason the table's own rows give.
const EMULATOR_ONLY: &[u32] = &[];

/// The documents that a dead-listed address may be cited from, pinned
/// (parked 321, ruled by the sixth pass 2026-09-19). `docs/EMULATOR.md` §4
/// lists the functions the executable never reaches — no call, no jump, no
/// embedded address — so a specification citing one by address cites code
/// the game does not run, and its claim rests on whatever else backs it. The
/// rows here are the citations standing on the day the guard landed; each
/// is owed a reconciliation (the parked file names them), a row is deleted
/// when its citation goes, and a new one fails.
///
/// **Sixteen more on 2026-09-27** (parked 952, the seventeenth pass), the
/// guard's first run against `rondata::blind::RESIDUE`: what the documents
/// cite of the twelve functions item 940's scan added to §4's fifteen, and
/// of `Caravan::process`, unreferenced once `.reloc` stopped counting
/// (parked 967). Each is a claim about the live inlined copy or about
/// nothing, and parked 970 owes the reading that says which.
const DEAD_CITED: &[(&str, u32)] = &[
    ("AI.md", 0x006b46b0),
    ("ARMY.md", 0x006ec170),
    ("CARAVAN.md", 0x0073e000),
    ("CITIES.md", 0x0062d430),
    ("COLLISION.md", 0x006b88b0),
    ("COMBAT.md", 0x00633390),
    ("COMBAT.md", 0x006b88b0),
    ("COMBAT.md", 0x006ec170),
    ("COMBAT.md", 0x0092fc50),
    ("DANGER.md", 0x006b22e0),
    ("ECONOMY.md", 0x006b88b0),
    ("ECONOMY.md", 0x006d6e80),
    ("GROUPS.md", 0x00683730),
    ("GROUPS.md", 0x006ec170),
    ("GROUPS.md", 0x007137f0),
    ("GROUPS.md", 0x00713bb0),
    ("INPUT.md", 0x005930c0),
    ("MOVEMENT.md", 0x00a469f0),
    ("ORDERS.md", 0x00622ce0),
    ("ORDERS.md", 0x00683730),
    ("ORDERS.md", 0x006ec170),
    ("ORDERS.md", 0x0073e000),
    ("PATHFINDER.md", 0x00688310),
    ("ROADS.md", 0x006b4230),
    ("RUNS.md", 0x005930c0),
    ("RUNS.md", 0x00688310),
    ("RUNS.md", 0x006d6740),
    ("RUNS.md", 0x0092fc50),
    ("TRANSPORT.md", 0x0065cfd0),
    ("TRANSPORT.md", 0x006d5230),
];

/// The rows of `rondata::blind::RESIDUE` that say "no reference in the
/// executable", read from the source: `sim` takes no dependency on the
/// data crate, and the table is the list item 940's byte scan made
/// (`Pe::references`, re-scanned against the install by that crate's own
/// test). A row is `( 0x00aa_bbbb, "reason …", )` with the reason a string
/// literal that may continue over lines.
fn unreferenced_in_the_image() -> std::collections::BTreeMap<u32, String> {
    let path = docs().join("../crates/rondata/src/blind.rs");
    let text = std::fs::read_to_string(&path).expect("crates/rondata/src/blind.rs");
    let start = text
        .find("pub const RESIDUE")
        .expect("rondata::blind::RESIDUE");
    let end = text[start..].find("\n];").expect("RESIDUE's end") + start;
    let mut dead = std::collections::BTreeMap::new();
    for row in text[start..end].split("\n    (\n").skip(1) {
        let addr = row
            .trim_start()
            .strip_prefix("0x")
            .and_then(|r| r.split(',').next())
            .and_then(|h| u32::from_str_radix(&h.replace('_', ""), 16).ok())
            .expect("a RESIDUE row opens with its address");
        let reason: String = row
            .split('"')
            .nth(1)
            .expect("a RESIDUE row carries its reason")
            .split("\\\n")
            .map(str::trim_start)
            .collect();
        if reason.contains("no reference in the executable") {
            let name = reason.split(':').next().unwrap_or("").to_string();
            dead.insert(addr, name);
        }
    }
    dead
}

/// **A dead-listed function is cited only where pinned.** ~~The dead list is
/// read from `docs/EMULATOR.md` §4's own `name@00xxxxxx` citations~~ The
/// dead list is `rondata::blind::RESIDUE`'s "no reference" rows since the
/// seventeenth pass (parked 952): item 940's scan found twelve more than
/// §4's fifteen, and a guard reading §4's prose let a document cite any of
/// the twelve. §4's own enumeration is held to the table, so the two cannot
/// drift apart.
#[test]
fn a_dead_listed_address_is_cited_only_where_pinned() {
    let dead = unreferenced_in_the_image();
    assert_eq!(
        dead.len(),
        UNREFERENCED,
        "rondata::blind::RESIDUE has {} rows that say \"no reference in the executable\", \
         not {UNREFERENCED}; re-pin, and pin what cites the new ones",
        dead.len()
    );
    // The enumeration runs from "are dead." to "which is data."; the
    // sentence after it names the *live* inlined copies, which are not dead.
    let emu = read("EMULATOR.md");
    let start = emu
        .find("are dead.**")
        .expect("docs/EMULATOR.md §4's `**15 are dead.**` bullet");
    let end = emu[start..]
        .find("which is data")
        .expect("the dead list ends at `which is data`")
        + start;
    let strays: Vec<String> = cites(&emu[start..end])
        .into_iter()
        .filter(|(_, _, addr)| !dead.contains_key(addr) && !EMULATOR_ONLY.contains(addr))
        .map(|(_, name, addr)| format!("{name}@{addr:08x}"))
        .collect();
    assert!(
        strays.is_empty(),
        "docs/EMULATOR.md §4 calls these dead and rondata::blind::RESIDUE has no \
         \"no reference\" row for them: {strays:?}"
    );
    let mut found = Vec::new();
    for entry in std::fs::read_dir(docs()).expect("docs/") {
        let path = entry.expect("entry").path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        // The census ranks the blind list, the dead among it: its citations
        // are the list itself, as §4's are.
        if !name.ends_with(".md")
            || matches!(
                name,
                "JOURNAL.md" | "EMULATOR.md" | "PARKED.md" | "CENSUS.md"
            )
        {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read");
        for (line, cited, addr) in cites(&text) {
            if dead.contains_key(&addr) {
                found.push((name.to_string(), addr, line, cited));
            }
        }
    }
    let mut new = Vec::new();
    for (name, addr, line, cited) in &found {
        if !DEAD_CITED.iter().any(|(n, a)| n == name && a == addr) {
            new.push(format!(
                "docs/{name}:{line}: `{cited}@{addr:08x}` has no reference in the executable \
                 (rondata::blind::RESIDUE) — the game never runs it; say what backs the claim, \
                 or pin the row in DEAD_CITED"
            ));
        }
    }
    let mut gone = Vec::new();
    for (name, addr) in DEAD_CITED {
        if !found.iter().any(|(n, a, _, _)| n == name && a == addr) {
            gone.push(format!(
                "docs/{name} no longer cites {addr:08x}; delete its DEAD_CITED row to bank it"
            ));
        }
    }
    assert!(
        new.is_empty() && gone.is_empty(),
        "{}\n{}",
        new.join("\n"),
        gone.join("\n")
    );
}

/// The short hex constants each specification names that no crate carries,
/// pinned by file (parked 356, ruled by the sixth pass 2026-09-19). A
/// document is written from the original and the code from the document,
/// and nothing checked the second step: `docs/COMBAT.md` carried the melee
/// reach `0xf6` for a month while every melee unit fought at `0x66`, and
/// `docs/CITIES.md` §3.6 named a cell flag `crates/sim` never wrote. The
/// count is the day's; a file may only shrink, and a file that grows names
/// a constant read but not built — build it, spell an offset `+0x..`, or
/// raise the pin on purpose with the reason beside it.
///
/// **Re-pinned 2026-09-25, the fifteenth pass (parked 802)**: the code
/// side is comment-blind now — a comment counts only for the `+0x..`
/// offset spelling — and accepts the decimal spelling of a value past a
/// byte. 72 → 104: the 35 that arrived are constants a comment alone
/// carried, and three (CITIES, TRANSPORT) were banked by the decimal, listed in `docs/audit/2026-09-25-fable-pass-15.md` and parked
/// as one ordinary item to build or to name.
///
/// **Re-pinned 2026-09-26, the sixteenth pass (parked 820)**: the decimal
/// spelling counts from `crates/sim` alone. Four landings of the tranche
/// had moved these pins on harness integers — a floor tuple, two pinned
/// widening rows, a standing count — with nothing built or unbuilt, and
/// measured whole the harness was banking **39** constants: 27 by
/// `rondata/src/diff` (pins and comparator tables) and 5 by the data layer
/// (`306`, `620`, `787`, `1023`, `7680`), none a mechanic carried. 100 →
/// 139; the arrivals are on the same ordinary item as 802's (809).
///
/// **Item 890 banked two**: ECONOMY 14 → 13 and PRODUCTION 10 → 9, the
/// Supercollider's `0x21d`, which `get_cost`'s space-race arm now carries
/// (`Sim::wonder_ramp_count`).
///
/// **Item 928 banked one**: ARMY 10 → 9, `0x55555555`, the neutral bias
/// `come_out`'s gather block passes `find_nearby_spot`, which `crate::rally`
/// now carries.
/// **Item 947 moved one, and it is not a build**: GOLDEN 4 → 3, `11520`,
/// chapter thirty-two's P1 x, which the item's unit tests in `crate::air`
/// use as a fixture point.
///
/// **Re-pinned 2026-09-28, the eighteenth pass (parked 975)**: the decimal
/// spelling counts from the simulation's production code, never from a
/// `#[cfg(test)]` item or a file `lib.rs` mounts under one. Measured
/// whole, test fixtures were banking **17** constants across nine
/// documents — AI 4, TECH 3, CITIES, ECONOMY and GOLDEN 2 each, ATTRITION,
/// COSTS, ORDERS and PRODUCTION 1 each — none a mechanism carried. 135 →
/// 152; the arrivals are listed in `docs/audit/2026-09-28-fable-pass-18.md`
/// and parked with 802's and 820's.
const UNBUILT: &[(&str, usize)] = &[
    ("AI.md", 35),
    ("ANIM.md", 4),
    ("ARMY.md", 9),
    ("ATTRITION.md", 1),
    ("CITIES.md", 9),
    ("COLLISION.md", 1),
    ("COMBAT.md", 9),
    ("COSTS.md", 5),
    ("ECONOMY.md", 15),
    ("GOLDEN.md", 5),
    ("GOODY.md", 5),
    ("GROUPS.md", 5),
    ("MERCHANT.md", 2),
    ("ORDERS.md", 12),
    ("PATHFINDER.md", 1),
    ("PRODUCTION.md", 10),
    ("ROADS.md", 1),
    ("SCOUT.md", 1),
    ("TECH.md", 16),
    ("TRANSPORT.md", 3),
    ("VISION.md", 2),
];

/// Which documents the constant check reads: the specifications, not the
/// ledgers, runbooks, format notes or chronicles, whose hex is addresses,
/// record bytes and stories.
fn is_specification(name: &str) -> bool {
    name.ends_with(".md")
        && !matches!(
            name,
            "JOURNAL.md"
                | "QUEUE.md"
                | "PARKED.md"
                | "DECISIONS.md"
                | "RUNS.md"
                | "ORACLE.md"
                | "CENSUS.md"
                | "EMULATOR.md"
                | "FORMATS.md"
                | "DEBUG_VIEWER.md"
                | "RECGAME.md"
                | "COMMANDS.md"
                | "SYNC.md"
        )
}

/// Every `0x` constant of two to four hex digits in a line that is not an
/// address suffix (`@`, `+`, `-` before it) — the shape of a flag, a reach,
/// a mask or a tuning value, as the specifications write them.
fn short_constants(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let b = text.as_bytes();
    for (at, _) in text.match_indices("0x") {
        let before = if at == 0 { b' ' } else { b[at - 1] };
        if before.is_ascii_alphanumeric() || matches!(before, b'_' | b'@' | b'+' | b'-') {
            continue;
        }
        let hex: String = text[at + 2..]
            .chars()
            .take_while(|c| c.is_ascii_hexdigit())
            .collect();
        let after = text[at + 2 + hex.len()..].chars().next().unwrap_or(' ');
        if (2..=4).contains(&hex.len()) && !(after.is_ascii_alphanumeric() || after == '_') {
            out.push(hex.to_lowercase());
        }
    }
    out
}

/// A source file less its comments, its strings and its `#[cfg(test)]`
/// items: what the simulation is built from, as `no_float` reads it.
fn production_code(raw: &str) -> String {
    crate::no_float::without_test_modules(&crate::no_float::code_only(raw))
}

/// The files `lib.rs` mounts only under `#[cfg(test)]`: `mod name;` on the
/// line after the attribute.
fn test_only_files(lib: &str) -> Vec<String> {
    let lines: Vec<&str> = lib.lines().collect();
    lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[cfg(test)]")
        .filter_map(|w| {
            w[1].trim()
                .strip_prefix("mod ")
                .or_else(|| w[1].trim().strip_prefix("pub mod "))
                .or_else(|| w[1].trim().strip_prefix("pub(crate) mod "))
                .and_then(|m| m.strip_suffix(';'))
                .map(|m| format!("{m}.rs"))
        })
        .collect()
}

/// **A test's fixture banks no constant** (parked 975). Made to fail
/// first on the reading that took the decimal from every line of `sim`.
#[test]
fn the_decimal_spelling_is_read_from_production_code() {
    let file = "pub const REACH: i32 = 640;\nfn f() -> i32 { 3 } // 11520 in a comment\n\
                #[cfg(test)]\nmod tests {\n    #[test]\n    fn t() { let x = 11520; assert_eq!(x, 11520); }\n}\n\
                pub const AFTER: i32 = 4096;\n";
    let code = production_code(file);
    assert!(code.contains("640") && code.contains("4096"));
    assert!(
        !code.contains("11520"),
        "a fixture's number read as the simulation's: {code}"
    );
    let lib = "mod a;\n#[cfg(test)]\nmod harness_tests;\n#[cfg(test)]\nmod no_float;\npub mod b;\n";
    assert_eq!(
        test_only_files(lib),
        vec!["harness_tests.rs".to_string(), "no_float.rs".to_string()]
    );
}

/// **A constant a specification names is in the code, or pinned as not.**
#[test]
fn a_constant_a_document_names_is_built_or_pinned() {
    let crates = docs().join("..").join("crates");
    let sim = crates.join("sim");
    let mut code = String::new();
    // The simulation's own source alone, for the decimal spelling: the
    // harness's numbers are frames, floors and counts, and four landings
    // of one tranche moved `UNBUILT` on them with nothing built or unbuilt
    // — 803's floor `(434, 438)` spelt `0x1b6`, 813's pinned rows `1206`
    // and `1153` spelt `0x4b6` and `0x481`, 824 undid both by closing the
    // chapter, 870 moved three files (parked 820, the sixteenth pass).
    //
    // **And never from a test** (parked 975, the eighteenth pass): item
    // 947's fixture put a plane at x `11520`, GOLDEN's `0x2d00` read as
    // built, and the pin fell 4 → 3 on a number no mechanism carries. The
    // decimal spelling is read from the simulation's production code —
    // `#[cfg(test)]` items and the files `lib.rs` mounts under one are out.
    let mut sim_code = String::new();
    let test_files =
        test_only_files(&std::fs::read_to_string(sim.join("src/lib.rs")).expect("lib.rs"));
    let mut stack = vec![crates];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("crates/") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                // Code, and of a comment only the offset spelling (`+0x7c`,
                // `-0x10`) this guard's contract names: a bare `0x..` in
                // prose counted as building the constant (parked 802, the
                // fifteenth pass), and that path would bank one nobody
                // built. Stripping comments whole was measured first: it
                // raised the pin from 72 constants to ~260, because the
                // honest offset comments are most of what the pin holds.
                let mut file_code = String::new();
                for line in std::fs::read_to_string(&path).expect("read").lines() {
                    let (code_only, comment) = line.split_once("//").unwrap_or((line, ""));
                    file_code.push_str(&code_only.to_lowercase());
                    for word in comment.split(|c: char| {
                        !(c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '_'))
                    }) {
                        if word.starts_with("+0x") || word.starts_with("-0x") {
                            file_code.push(' ');
                            file_code.push_str(&word[1..].to_lowercase());
                        }
                    }
                    file_code.push('\n');
                }
                let mounted_for_tests = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| test_files.iter().any(|t| t == n));
                if path.starts_with(&sim) && !mounted_for_tests {
                    let raw = std::fs::read_to_string(&path).expect("read");
                    sim_code.push_str(&production_code(&raw).to_lowercase());
                    sim_code.push('\n');
                }
                code.push_str(&file_code);
            }
        }
    }
    let built = |hex: &str| {
        let trimmed = hex.trim_start_matches('0');
        // The decimal spelling too, for a value past a byte: a type index
        // the document writes `0x1a1` is `417` in a table here, and the
        // hex forms alone read it as unbuilt. Below 0x100 the decimal is
        // any small number and is not accepted — and it is accepted from
        // `crates/sim` alone, never from the harness, whose integers are
        // frames and floors (parked 820).
        let decimal = u32::from_str_radix(hex, 16)
            .ok()
            .filter(|v| *v >= 0x100)
            .map(|v| v.to_string())
            .unwrap_or_default();
        // A whole token: neither byte beside it is a digit or an identifier.
        let whole = |hay: &str, f: &str| {
            f.len() > 2
                && hay.match_indices(f).any(|(i, _)| {
                    let next = hay.as_bytes().get(i + f.len()).copied().unwrap_or(b' ');
                    let prev = if i == 0 { b' ' } else { hay.as_bytes()[i - 1] };
                    !(next.is_ascii_alphanumeric() || next == b'_')
                        && !(prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'.')
                })
        };
        [
            format!("0x{hex}"),
            format!("0x{hex:0>4}"),
            format!("0x{trimmed}"),
        ]
        .iter()
        .any(|f| whole(&code, f))
            || whole(&sim_code, &decimal)
    };
    let mut per_file: Vec<(String, Vec<String>)> = Vec::new();
    for entry in std::fs::read_dir(docs()).expect("docs/") {
        let path = entry.expect("entry").path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !is_specification(name) {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read");
        let mut missing: Vec<String> = short_constants(&text)
            .into_iter()
            .filter(|h| !built(h))
            .map(|h| format!("0x{h}"))
            .collect();
        missing.sort();
        missing.dedup();
        if !missing.is_empty() {
            per_file.push((name.to_string(), missing));
        }
    }
    per_file.sort();
    let mut failures = Vec::new();
    for (name, missing) in &per_file {
        let pin = UNBUILT
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, c)| *c)
            .unwrap_or(0);
        if missing.len() > pin {
            failures.push(format!(
                "docs/{name} names {} constants no crate carries, pinned at {pin}: {}",
                missing.len(),
                missing.join(" ")
            ));
        } else if missing.len() < pin {
            failures.push(format!(
                "docs/{name} is down to {} unbuilt constants from a pin of {pin}; lower the pin in UNBUILT to bank it",
                missing.len()
            ));
        }
    }
    for (name, pin) in UNBUILT {
        if *pin > 0 && !per_file.iter().any(|(n, _)| n == name) {
            failures.push(format!(
                "docs/{name} has no unbuilt constants; delete its UNBUILT row"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn a_section_over_the_ceiling_may_not_grow() {
    let mut failures = Vec::new();
    let mut seen = Vec::new();
    for entry in std::fs::read_dir(docs()).expect("docs/") {
        let path = entry.expect("entry").path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.ends_with(".md") || name == "JOURNAL.md" {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read");
        for (heading, size) in sections(&text) {
            let pin = OVER
                .iter()
                .find(|(n, h, _)| *n == name && *h == heading)
                .map(|&(_, _, pinned)| pinned);
            if pin.is_some() {
                seen.push((name.to_string(), heading.clone()));
            }
            let bound = pin.unwrap_or(SECTION_CEILING);
            let label = if heading.is_empty() {
                "the preamble".to_string()
            } else {
                format!("`## {heading}`")
            };
            if size > bound {
                failures.push(format!(
                    "docs/{name} {label}: {size} bytes, bound {bound} — split the section, or move its story to the journal"
                ));
            } else if let Some(pinned) = pin {
                if size <= SECTION_CEILING {
                    failures.push(format!(
                        "docs/{name} {label} is under the ceiling now ({size} ≤ {SECTION_CEILING}); delete its row from OVER"
                    ));
                } else if size < pinned {
                    // Fine — but say so, so the next session lowers the pin.
                    eprintln!("docs/{name} {label} shrank to {size}; lower its pin from {pinned}");
                }
            }
        }
    }
    for (name, heading, _) in OVER {
        if !seen.iter().any(|(n, h)| n == name && h == heading) {
            failures.push(format!(
                "docs/{name} has no `## {heading}` — the pinned section was renamed or removed; fix the row"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The rules track has a slot that is never empty (DECISIONS 41). The
/// handoff carries a `Golden:` line beside `Scoreboard:` and `Long
/// captures:` — `none pinned` with the takes-chain until the first chapter
/// pins, then `w<frame>` — so the golden record cannot fall out of the
/// handoff the way the second map's capture once fell out of the queue.
#[test]
fn the_handoff_carries_the_golden_line() {
    let q = read("QUEUE.md");
    let line = q
        .lines()
        .find(|l| l.starts_with("Golden:"))
        .expect("docs/QUEUE.md's handoff has no `Golden:` line; write `Golden: none pinned · <takes-chain>` or `Golden: w<frame> of <length> · …`");
    let rest = line.trim_start_matches("Golden:").trim();
    let pinned = rest.split_whitespace().any(|w| {
        w.strip_prefix('w')
            .is_some_and(|d| !d.is_empty() && d.chars().all(|c| c.is_ascii_digit()))
    });
    // Three forms: before the first chapter pins, with an open word, and
    // with every pinned word closed (parked 638, the thirteenth pass —
    // the third form borrowed `none pinned` for an item, which reads as
    // the opposite of what it meant). `rondata::diff::floors` reads the
    // parts against the constants; this only asks that the slot is filled.
    assert!(
        pinned || rest.starts_with("none pinned") || rest.starts_with("every chapter closed"),
        "the `Golden:` line is {rest:?}; it says `none pinned`, names a word `w<frame>`, \
         or leads with `every chapter closed`"
    );
}

/// `docs/DECISIONS.md` is a ledger nobody reads whole, and an amended entry
/// does not say so at its own heading; its index does (DECISIONS 41). Every
/// `## N.` entry has an index row `- N <status> — <title>`, and no row names
/// an entry that does not exist.
#[test]
fn every_decision_has_an_index_row() {
    let d = std::fs::read_to_string(docs().join("DECISIONS.md")).expect("docs/DECISIONS.md");
    let entries: Vec<u32> = d
        .lines()
        .filter_map(|l| l.strip_prefix("## "))
        .filter_map(|h| h.split_once(". ").and_then(|(n, _)| n.parse().ok()))
        .collect();
    assert!(
        !entries.is_empty(),
        "docs/DECISIONS.md has no `## N.` entries"
    );
    let mut in_index = false;
    let mut rows = Vec::new();
    for l in d.lines() {
        if let Some(h) = l.strip_prefix("## ") {
            in_index = h.starts_with("Index");
            continue;
        }
        if !in_index {
            continue;
        }
        if let Some(r) = l.strip_prefix("- ") {
            let (n, rest) = r.split_once(' ').unwrap_or((r, ""));
            let n: u32 = n
                .parse()
                .unwrap_or_else(|_| panic!("index row {l:?} does not start with an entry number"));
            let status = rest.split(" — ").next().unwrap_or("");
            let ok = status == "standing"
                || status == "amended in place"
                || status.starts_with("amended by ")
                || status.starts_with("extended by ")
                || status.starts_with("superseded by ");
            assert!(
                ok,
                "index row {l:?}: the status is {status:?}, not one of standing / amended in place / amended by N / extended by N / superseded by N"
            );
            rows.push(n);
        }
    }
    assert!(
        !rows.is_empty(),
        "docs/DECISIONS.md has no `## Index` section with `- N` rows"
    );
    let missing: Vec<u32> = entries
        .iter()
        .copied()
        .filter(|n| !rows.contains(n))
        .collect();
    let extra: Vec<u32> = rows
        .iter()
        .copied()
        .filter(|n| !entries.contains(n))
        .collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "docs/DECISIONS.md's index and its entries disagree: entries with no row {missing:?}, rows with no entry {extra:?}"
    );
}

/// **A script with a shebang under `tools/` is executable** (parked 756,
/// the fifteenth pass). `tools/gamelog/waitrun.sh` was committed `100644`,
/// so the invocation `CLAUDE.md` spells — the bare path — exited 126 at
/// once, and a backgrounded wait on a running capture reported "completed"
/// in seconds (parked 751); four workers found it and each wrote `zsh …`
/// in front. The mode bit is git's to keep and this is what keeps it: the
/// first run failed on seven scripts.
#[test]
fn every_shell_script_under_tools_is_executable() {
    use std::os::unix::fs::PermissionsExt;
    let tools = docs().join("..").join("tools");
    let mut stack = vec![tools];
    let mut bad = Vec::new();
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("tools/") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().is_none_or(|e| e != "sh") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            if !text.starts_with("#!") {
                continue;
            }
            let mode = std::fs::metadata(&path).expect("stat").permissions().mode();
            if mode & 0o111 == 0 {
                bad.push(path.display().to_string());
            }
        }
    }
    bad.sort();
    assert!(
        bad.is_empty(),
        "scripts with a shebang and no execute bit (`chmod +x`, and git keeps it): {bad:#?}"
    );
}

/// **A journal carries no tool-call markup outside a fence** (parked 841,
/// the sixteenth pass): 837's arrived with two stray tags (`</content>`,
/// `</invoke>`) that no guard caught, and the commander removed them at
/// the merge. Inside a code fence anything goes; outside one, the
/// harness's own tag names are a transcript leaking into the record.
#[test]
fn a_journal_carries_no_tool_call_markup() {
    const TAGS: &[&str] = &[
        "<invoke",
        "</invoke>",
        "<parameter",
        "</parameter>",
        "<function_calls>",
        "</function_calls>",
        "<content>",
        "</content>",
        "<antml",
    ];
    let mut found = Vec::new();
    for entry in std::fs::read_dir(docs().join("journal")).expect("docs/journal/") {
        let path = entry.expect("entry").path();
        if !path.extension().is_some_and(|e| e == "md") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read");
        let mut fenced = false;
        for (i, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("```") {
                fenced = !fenced;
                continue;
            }
            if fenced {
                continue;
            }
            if let Some(tag) = TAGS.iter().find(|t| line.contains(*t)) {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                found.push(format!("{name}:{}: {tag}", i + 1));
            }
        }
    }
    assert!(
        found.is_empty(),
        "tool-call markup outside a fence in docs/journal/ — a transcript leaked into \
         the record; delete it:\n{}",
        found.join("\n")
    );
}
