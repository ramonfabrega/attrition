//! The golden record's staged input, interpreted into the simulation.
//!
//! `docs/DECISIONS.md` entry 41 §3 and `docs/INPUT.md` §11. The rules track
//! runs on a game staged from `rontrace.cmd` — one line per staged frame,
//! handed to `ConsoleWin::parse_cmd` at `Game::do_frame`'s entry — and this
//! is the side of that channel the harness owns: the script read back, its
//! lines parsed into the fixed cheat set entry 41 lists, and each one
//! applied to a `sim::Sim` through the entry point the original's own case
//! calls.
//!
//! **A cheat is a modelled input here, and that is deliberate.**
//! `docs/INPUT.md` §8 argued the other way — capture ground truth without
//! cheats, because a recording of a cheat-staged run cannot be replayed —
//! and entry 41 reverses it for this track: the set is small, listed, and
//! every member is a state poke with a named target. **No console command
//! issues an order at all** (`docs/ORACLE.md`, "The channel's vocabulary"),
//! so nothing here reaches `sim::orders`; the orders come from the `.rcx`
//! through [`crate::input`], and the two halves never overlap.
//!
//! What this module is not: `ConsoleWin::parse_cmd`. It re-derives the
//! argument grammar of the cases it implements and refuses everything else
//! by name, the way [`crate::input::Applied`] refuses a command kind.

use std::collections::BTreeMap;

use crate::diff::Built;
use crate::load::Loaded;
use sim::Pos;

/// One line of a `rontrace.cmd` script.
///
/// `<sim-frame> <text>`, `#` comments, a `!` prefix selecting the
/// console-only half of `run_cmd`'s two switches. Lines run in file order
/// and **a frame below its predecessor is clamped** — `rontrace.dll`'s own
/// rule, reproduced by [`Script::parse`] so the harness stages the lines on
/// the frames the original staged them on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Staged {
    pub frame: i64,
    /// `!`-prefixed: `from_chat = 0`, the console-only half.
    pub console: bool,
    /// The line with its frame and any `!` removed.
    pub text: String,
}

/// The cheats the golden record is allowed to use (`docs/DECISIONS.md` 41
/// §3), as parsed. Anything else is [`Cheat::Unmapped`] and is reported
/// rather than guessed at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cheat {
    /// `ai [on | off]`, console-only (table index 12). A bare `ai`
    /// toggles.
    Ai(Option<bool>),
    /// `age [age] [who]` — `Leader::set_age` and **nothing else**; the four
    /// epochs stay where they were (`docs/RUNS.md` run101–run105).
    Age { who: i32, level: i32 },
    /// `military`/`civic`/`commerce`/`science` — one epoch, by category.
    Epoch { who: i32, cat: i32, level: i32 },
    /// `library [level] [who]` — all four epochs **and** the age.
    Library { who: i32, level: i32 },
    /// `ally`/`peace`/`war [who | all]` → `Leader::set_diplo(console->who,
    /// target, 2 | 1 | 0)`. `None` for the target is the **bare** form,
    /// which prints the table and changes nothing.
    Diplo { level: i32, target: Option<Target> },
    /// `human`/`computer [who]`.
    Control { who: i32, human: bool },
    /// `add`/`insert [#] typename [who=RED] [x,y]`.
    Add {
        num: i32,
        name: String,
        who: i32,
        at: Option<Pos>,
    },
    /// `tech [who] [tech | all] [on | off]`.
    Tech { who: i32, name: String, on: bool },
    /// A line the channel has and this interpreter does not model, or one
    /// whose arguments did not parse. The string is the command word.
    Unmapped(String),
}

/// `ally`/`peace`/`war`'s argument.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Who(i32),
    All,
}

/// What the interpreter did on one frame.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Applied {
    /// Lines whose effect reached the simulation.
    pub ran: usize,
    /// Units created by `add`, squad members included.
    pub units: usize,
    /// Buildings created by `add`.
    pub buildings: usize,
    /// Lines carried and not acted on, by command word and reason — the
    /// honest half, as [`crate::input::Applied::skipped`] is for commands.
    pub skipped: BTreeMap<(String, &'static str), usize>,
}

impl Applied {
    fn skip(&mut self, word: &str, why: &'static str) {
        *self.skipped.entry((word.to_string(), why)).or_default() += 1;
    }

    pub fn merge(&mut self, other: &Applied) {
        self.ran += other.ran;
        self.units += other.units;
        self.buildings += other.buildings;
        for (k, n) in &other.skipped {
            *self.skipped.entry(k.clone()).or_default() += n;
        }
    }

    pub fn skipped_total(&self) -> usize {
        self.skipped.values().sum()
    }
}

/// The staged script, consumed in frame order.
#[derive(Clone, Debug, Default)]
pub struct Script {
    lines: Vec<Staged>,
    next: usize,
}

impl Script {
    /// Parses a `rontrace.cmd`. Unreadable lines are dropped the way the
    /// tracer drops them; the frame is clamped upward to its predecessor's.
    pub fn parse(text: &str) -> Script {
        let mut lines: Vec<Staged> = Vec::new();
        for raw in text.lines() {
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let Some((head, rest)) = line.split_once(char::is_whitespace) else {
                continue;
            };
            let Ok(frame) = head.trim().parse::<i64>() else {
                continue;
            };
            let rest = rest.trim();
            let (console, text) = match rest.strip_prefix('!') {
                Some(t) => (true, t.trim()),
                None => (false, rest),
            };
            if text.is_empty() {
                continue;
            }
            let frame = frame.max(lines.last().map_or(0, |l| l.frame));
            lines.push(Staged {
                frame,
                console,
                text: text.to_string(),
            });
        }
        Script { lines, next: 0 }
    }

    pub fn read(path: &std::path::Path) -> std::io::Result<Script> {
        Ok(Script::parse(&std::fs::read_to_string(path)?))
    }

    pub fn lines(&self) -> &[Staged] {
        &self.lines
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// The last frame any line is staged on.
    pub fn last_frame(&self) -> i64 {
        self.lines.last().map_or(0, |l| l.frame)
    }

    /// [`Script::apply`] with the staged draws kept in the fold.
    ///
    /// `Sim::tick` clears `phase_marks` at its head, so a draw a cheat
    /// spends before the call would move the stream and leave no mark —
    /// and `mark_sites` would then attribute it to nothing and drop it.
    /// This marks into a fresh list and hands it to
    /// [`sim::Sim::staged_marks`], which the next `tick` puts at the head
    /// of the frame's own. Item 364 measured the difference: chapter one's
    /// `add hoplite` spends three `Guy::init_real+0x52` draws and without
    /// this they were the frame's first three and invisible.
    pub fn stage(&mut self, frame: i64, built: &mut Built, loaded: &Loaded) -> Applied {
        built.sim.phase_marks.clear();
        let done = self.apply(frame, built, loaded);
        built.sim.staged_marks = std::mem::take(&mut built.sim.phase_marks);
        done
    }

    /// Runs every line staged on `frame`, in file order.
    ///
    /// Call **before** the tick that is frame `frame`: `rontrace.dll` hands
    /// each line to `parse_cmd` at `Game::do_frame`'s entry, before phase 1
    /// (`docs/ORACLE.md`, "The cheat channel").
    ///
    /// SEAM: `ai` is the one member of the set that does not act where it is
    /// typed — `run_cmd` case 0xc calls
    /// `CommandManager::issue_cheat_ai_toggle`, so the flag is flipped by
    /// `CommandPackage::process_cheat_ai_toggle` when the turn pump walks
    /// the package. Here it is flipped at the line's own frame. No capture
    /// can separate the two: the control run (`docs/RUNS.md` run104) has
    /// frame 0 identical either way and parts from frame 1's draws.
    pub fn apply(&mut self, frame: i64, built: &mut Built, loaded: &Loaded) -> Applied {
        let mut done = Applied::default();
        while self.next < self.lines.len() && self.lines[self.next].frame < frame {
            let word = command_word(&self.lines[self.next].text);
            done.skip(&word, "the frame was stepped past");
            self.next += 1;
        }
        while self.next < self.lines.len() && self.lines[self.next].frame == frame {
            let line = self.lines[self.next].clone();
            self.next += 1;
            run(&line, built, loaded, &mut done);
        }
        done
    }
}

fn command_word(text: &str) -> String {
    text.split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
}

/// `ConsoleWin::parse_who@007e3ad0(arg, default)`, as far as a script uses
/// it — and the **bare-number rule is the whole of why this is a function
/// rather than a `parse`**.
///
/// The head computes `param_2 = (default < 0) ? 0 : 1`, a `who=` prefix
/// sets it to 1 and is stripped, and the digit arm at the tail is reached
/// only when it is 1: `if (param_2 == 0) goto <return the default>`
/// stands immediately above it. So a bare number is a player number only
/// where the caller passed a real default, and the caller that passes
/// `-1` — `age`'s first token, `add`'s `who=` slot — reads it as
/// something else. `age 3` is player 0's third age, not player 3's; `age
/// 3 1` is player 1's.
///
/// SEAM: the original also matches the eight player names, the eight
/// colour names and `gaia`. A script here says the number.
fn parse_who(tok: &str, allow_bare: bool) -> Option<i32> {
    let (t, tagged) = match tok.strip_prefix("who=") {
        Some(t) => (t, true),
        None => (tok, false),
    };
    if !(allow_bare || tagged) {
        return None;
    }
    match t.parse::<i32>() {
        Ok(n) if (0..8).contains(&n) => Some(n),
        _ => None,
    }
}

/// `ConsoleWin::parse_coord@007e5390` in the mode this install runs in.
///
/// The function has three arms: a leading `c` (or a bare number under
/// `coord_mode == 3`) is a raw internal coordinate, a leading `t` (or a
/// bare number under `coord_mode == 2`) is **`n × 0xc0 + 0x60`** — a tile,
/// centred — and anything else is `n × 0x300 + 0x180`, a world cell.
///
/// **This install reads a bare number as a tile**, and it is measured
/// rather than read: `docs/RUNS.md` run101–run105 staged `add hoplite
/// who=0 4,40` and the dump puts the squad's head at `(888, 7800)`, which
/// is `(4 × 192 + 96, 40 × 192 + 96)` plus `find_nearby_spot`'s offset.
/// The world arm would have asked for `(3456, 31104)`, eighteen tiles
/// away. That corrects `docs/ORACLE.md`'s stored `arg × 768 + half a
/// footprint` for the staged channel (`docs/INPUT.md` §11).
fn parse_coord(tok: &str) -> Option<i32> {
    let t = tok.trim();
    if let Some(n) = t.strip_prefix(['c', 'C']) {
        return n.parse::<i32>().ok();
    }
    let n = t
        .strip_prefix(['t', 'T'])
        .unwrap_or(t)
        .parse::<i32>()
        .ok()?;
    Some(n * 0xc0 + 0x60)
}

fn parse_pos(tok: &str) -> Option<Pos> {
    let (x, y) = tok.split_once(',')?;
    Some(Pos::new(parse_coord(x)?, parse_coord(y)?))
}

/// The console's own player — `MiscAccess::console->who`, the seat the
/// staged game is played from. Every capture so far is player 0's.
const CONSOLE_WHO: i32 = 0;

fn parse(text: &str) -> Cheat {
    let mut tok = text.split_whitespace();
    let word = tok.next().unwrap_or("").to_ascii_lowercase();
    let rest: Vec<&str> = tok.collect();
    // **The age family's token walk**, which is `run_cmd`'s own: the first
    // token is `parse_who(·, −1)` — so only a `who=` form is a player
    // there — then the level, then one more `parse_who(·, console->who)`,
    // where a bare number *is* a player. `age 8 who=1` and `age who=1 8`
    // both read; `age 3` is player 0's.
    let level_and_who = |rest: &[&str]| -> Option<(i32, i32)> {
        let mut it = rest.iter();
        let mut who = CONSOLE_WHO;
        let mut head = it.next()?;
        if let Some(w) = parse_who(head, false) {
            who = w;
            head = it.next()?;
        }
        let level = head.parse::<i32>().ok()?;
        if let Some(w) = it.next().and_then(|t| parse_who(t, true)) {
            who = w;
        }
        Some((level, who))
    };
    let who_of = |rest: &[&str]| -> i32 {
        rest.iter()
            .find_map(|t| parse_who(t, false))
            .unwrap_or(CONSOLE_WHO)
    };
    match word.as_str() {
        "ai" => Cheat::Ai(
            match rest.first().map(|s| s.to_ascii_lowercase()).as_deref() {
                Some("on") => Some(true),
                Some("off") => Some(false),
                None => None,
                _ => return Cheat::Unmapped(word),
            },
        ),
        "age" => match level_and_who(&rest) {
            Some((level, who)) => Cheat::Age { who, level },
            None => Cheat::Unmapped(word),
        },
        "library" => match level_and_who(&rest) {
            Some((level, who)) => Cheat::Library { who, level },
            None => Cheat::Unmapped(word),
        },
        "military" | "civic" | "commerce" | "science" => {
            let cat = match word.as_str() {
                "military" => 0,
                "civic" => 1,
                "commerce" => 2,
                _ => 3,
            };
            match level_and_who(&rest) {
                Some((level, who)) => Cheat::Epoch { who, cat, level },
                None => Cheat::Unmapped(word),
            }
        }
        "ally" | "peace" | "war" => {
            let level = match word.as_str() {
                "ally" => 2,
                "peace" => 1,
                _ => 0,
            };
            let target = match rest.first() {
                None => None,
                Some(t) if t.eq_ignore_ascii_case("all") => Some(Target::All),
                Some(t) => match parse_who(t, true) {
                    Some(w) => Some(Target::Who(w)),
                    None => return Cheat::Unmapped(word),
                },
            };
            Cheat::Diplo { level, target }
        }
        "human" | "computer" => Cheat::Control {
            who: rest
                .first()
                .and_then(|t| parse_who(t, true))
                .unwrap_or(CONSOLE_WHO),
            human: word == "human",
        },
        "add" | "insert" => {
            // `[#] typename [who=RED] [x,y]` — the leading count is a count
            // only when it parses as one (`run_cmd` case 0x4e reads
            // `String::number` and falls back to 1), and it is capped at
            // 300.
            let mut it = rest.iter().peekable();
            let num = match it.peek().and_then(|t| t.parse::<i32>().ok()) {
                Some(n) => {
                    it.next();
                    n.min(300)
                }
                None => 1,
            };
            let Some(name) = it.next() else {
                return Cheat::Unmapped(word);
            };
            let tail: Vec<&str> = it.copied().collect();
            Cheat::Add {
                num,
                name: (*name).to_string(),
                who: who_of(&tail),
                at: tail.iter().find_map(|t| parse_pos(t)),
            }
        }
        "tech" => {
            let who = rest
                .first()
                .and_then(|t| parse_who(t, true))
                .unwrap_or(CONSOLE_WHO);
            let named = rest.iter().find(|t| {
                !t.starts_with("who=")
                    && !t.eq_ignore_ascii_case("on")
                    && !t.eq_ignore_ascii_case("off")
                    && t.parse::<i32>().is_err()
            });
            match named {
                Some(name) => Cheat::Tech {
                    who,
                    name: (*name).to_string(),
                    on: !rest.iter().any(|t| t.eq_ignore_ascii_case("off")),
                },
                None => Cheat::Unmapped(word),
            }
        }
        _ => Cheat::Unmapped(word),
    }
}

fn run(line: &Staged, built: &mut Built, loaded: &Loaded, done: &mut Applied) {
    let word = command_word(&line.text);
    // `run_cmd`'s two switches are disjoint: 56 console-only commands and
    // 45 chat-reachable ones (`docs/ORACLE.md`). A script that puts a chat
    // command behind `!`, or the reverse, reaches a case that is not there.
    let console_only = matches!(word.as_str(), "ai" | "quit" | "go" | "break" | "restart");
    if line.console != console_only {
        done.skip(&word, "the wrong half of run_cmd's two switches");
        return;
    }
    match parse(&line.text) {
        Cheat::Ai(state) => {
            built.sim.ai_off = state.map_or(!built.sim.ai_off, |on| !on);
            done.ran += 1;
        }
        Cheat::Age { who, level } => {
            built.sim.set_leader_age(who as sim::Player, level);
            done.ran += 1;
        }
        Cheat::Library { who, level } => {
            for cat in 0..4 {
                built.sim.set_leader_epoch(who as sim::Player, cat, level);
            }
            built.sim.set_leader_age(who as sim::Player, level);
            done.ran += 1;
        }
        Cheat::Epoch { who, cat, level } => {
            built.sim.set_leader_epoch(who as sim::Player, cat, level);
            done.ran += 1;
        }
        Cheat::Diplo { level, target } => {
            // The **bare** form prints the diplomacy table and changes
            // nothing: `run_cmd` case 0x2c–0x2e bails to the printer when
            // the line has no further token. Chapter one's `604 war` is
            // that form, and the two players are already at war from the
            // lobby (`docs/ARMY.md` §16.3).
            let Some(target) = target else {
                done.skip(&word, "the bare form prints the table and changes nothing");
                return;
            };
            let me = CONSOLE_WHO as sim::Player;
            let live: Vec<sim::Player> = (0..built.sim.players.len() as sim::Player)
                .filter(|&w| w != me)
                .collect();
            let whom: Vec<sim::Player> = match target {
                Target::All => live,
                Target::Who(w) => vec![w as sim::Player],
            };
            for w in whom {
                built.sim.set_diplo(me, w, level);
            }
            done.ran += 1;
        }
        Cheat::Control { who, human } => {
            if let Some(n) = built.sim.nation.get_mut(who as usize) {
                n.human = human;
                done.ran += 1;
            } else {
                done.skip(&word, "no such leader");
            }
        }
        Cheat::Tech { who, name, on } => {
            if !on {
                done.skip(&word, "`tech … off` is not modelled");
                return;
            }
            match loaded.tech_named(&name) {
                Some(t) => {
                    built.sim.gain_tech(who as sim::Player, loaded.tech_tree[t]);
                    done.ran += 1;
                }
                None => done.skip(&word, "no technology of that name"),
            }
        }
        Cheat::Add { num, name, who, at } => add(&word, num, &name, who, at, built, loaded, done),
        Cheat::Unmapped(w) => done.skip(&w, "not in the interpreter's cheat set"),
    }
}

/// `run_cmd` case 0x4d/0x4e's loop, and the two entry points under it.
///
/// A **unit** type takes `UnitType::find_nearby_spot(x, y, 0, 0xc00, 0,
/// 0x55555555, FILTER_NOT_ME, −1, −1, …)` and then
/// `Objects::init_unit(who, type, spot, −1, −1, −1)` — which is itself a
/// squad loop, `uber_size` units threaded `o_up`/`o_down`
/// ([`sim::Sim::init_unit`]). A **building** type takes
/// `Objects::init_build` once and **breaks out of the count loop**, so a
/// leading count places one building and not `num` of them.
#[expect(
    clippy::too_many_arguments,
    reason = "the original's own argument list"
)]
fn add(
    word: &str,
    num: i32,
    name: &str,
    who: i32,
    at: Option<Pos>,
    built: &mut Built,
    loaded: &Loaded,
    done: &mut Applied,
) {
    let Some(at) = at else {
        // `no_mouse = 1`, so `parse_cmd` never refreshes `mouse_coord_x/y`
        // and an omitted coordinate silently reuses a stale cursor
        // (`docs/ORACLE.md`). There is no cursor here and guessing one
        // would be worse than refusing.
        done.skip(word, "no x,y and the channel has no cursor");
        return;
    };
    if !(0..built.sim.players.len() as i32).contains(&who) {
        done.skip(word, "no such leader");
        return;
    }
    if let Some(ty) = named_unit(loaded, name) {
        for _ in 0..num.max(0) {
            let Some(spot) =
                built
                    .sim
                    .find_nearby_spot_type(ty, at, 0, 0xc00, 0, sim::movement::Angle::INITIAL)
            else {
                done.skip(word, "find_nearby_spot found no room");
                return;
            };
            let head = built.sim.init_unit(who as sim::Player, ty, spot);
            // The squad's members, as `Sim::init_unit` threads them.
            let mut n = 1;
            let mut at_unit = head;
            while let Some(next) = built.sim.units[at_unit].o_down {
                n += 1;
                at_unit = next;
            }
            done.units += n;
        }
        done.ran += 1;
        return;
    }
    if let Some(ty) = named_build(loaded, name) {
        // One, whatever the count says: `init_build`'s arm breaks.
        match built.sim.place_building(who as sim::Player, ty, at) {
            Ok(_) => {
                done.buildings += 1;
                done.ran += 1;
            }
            Err(_) => done.skip(word, "the site is not placeable"),
        }
        return;
    }
    done.skip(word, "no unit or building of that name");
}

/// `ConsoleWin::parse_type`'s matching, as far as a script needs it.
///
/// The function underscores-to-spaces the typed token
/// (`String::replace(L'_', L' ')`) and then walks each category's name
/// table — the 402 unit types first, then the 543 building types — asking
/// `String::ignore(typed, name, len)` and, on a miss, the same against the
/// name with its spaces purged. That is `parse_who`'s shape exactly, and it
/// is a **prefix** match rather than an equality: `docs/RUNS.md`
/// run101–run105 staged `add hoplite` and got the type whose name is
/// `Hoplites`.
///
/// So: exact first, then prefix, each case-insensitively and each also
/// against the space-purged name. Units before buildings, because the
/// original asks in that order and a name that matches both is the unit.
fn type_named(names: &[String], name: &str) -> Option<usize> {
    let want = name.replace('_', " ").to_ascii_lowercase();
    let purged = want.replace(' ', "");
    let hit = |f: &dyn Fn(&str, &str) -> bool| {
        names.iter().position(|n| {
            let lower = n.to_ascii_lowercase();
            f(&lower, &want) || f(&lower.replace(' ', ""), &purged)
        })
    };
    hit(&|n: &str, w: &str| n == w).or_else(|| hit(&|n: &str, w: &str| n.starts_with(w)))
}

fn named_unit(loaded: &Loaded, name: &str) -> Option<usize> {
    type_named(&loaded.unit_type_names, name).or_else(|| type_named(&loaded.unit_names, name))
}

fn named_build(loaded: &Loaded, name: &str) -> Option<usize> {
    type_named(&loaded.build_type_names, name).or_else(|| type_named(&loaded.build_names, name))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The script format, including the two rules a reader gets wrong:
    /// `#` is a comment *anywhere* on the line, and a frame below its
    /// predecessor's is clamped up rather than reordered
    /// (`docs/ORACLE.md`, "The cheat channel").
    #[test]
    fn the_script_format_is_frame_then_line_with_clamped_frames() {
        let s = Script::parse(
            "# a header\n\
             \n\
             0 !ai off\n\
             600 age who=0 8   # a trailing comment\n\
             599 war\n\
             610 add hoplite who=0 4,40\n\
             nonsense\n\
             700\n\
             800 !\n",
        );
        assert_eq!(
            s.lines(),
            &[
                Staged {
                    frame: 0,
                    console: true,
                    text: "ai off".into()
                },
                Staged {
                    frame: 600,
                    console: false,
                    text: "age who=0 8".into()
                },
                Staged {
                    frame: 600,
                    console: false,
                    text: "war".into()
                },
                Staged {
                    frame: 610,
                    console: false,
                    text: "add hoplite who=0 4,40".into()
                },
            ]
        );
        assert_eq!(s.last_frame(), 610);
    }

    /// Chapter one's own lines, parsed into the cheat set.
    #[test]
    fn chapter_one_parses_into_the_cheat_set() {
        assert_eq!(parse("ai off"), Cheat::Ai(Some(false)));
        assert_eq!(parse("ai on"), Cheat::Ai(Some(true)));
        assert_eq!(parse("ai"), Cheat::Ai(None));
        assert_eq!(parse("age who=0 8"), Cheat::Age { who: 0, level: 8 });
        // The original tries `parse_who` on the token before the number
        // and again on the one after it, so either order reads.
        assert_eq!(parse("age 8 who=1"), Cheat::Age { who: 1, level: 8 });
        // The second slot passes a real default, so a bare number reads
        // as a player there and only there.
        assert_eq!(parse("age 8 1"), Cheat::Age { who: 1, level: 8 });
        assert_eq!(
            parse("age 3"),
            Cheat::Age { who: 0, level: 3 },
            "a bare first token is the level: `parse_who`'s default is -1 here"
        );
        assert_eq!(
            parse("war"),
            Cheat::Diplo {
                level: 0,
                target: None
            },
            "the bare form prints the table"
        );
        assert_eq!(
            parse("war who=1"),
            Cheat::Diplo {
                level: 0,
                target: Some(Target::Who(1))
            }
        );
        assert_eq!(
            parse("peace all"),
            Cheat::Diplo {
                level: 1,
                target: Some(Target::All)
            }
        );
        assert_eq!(
            parse("ally 1"),
            Cheat::Diplo {
                level: 2,
                target: Some(Target::Who(1))
            },
            "`ally`'s default is console->who, so a bare number is a player"
        );
        assert_eq!(
            parse("add hoplite who=0 4,40"),
            Cheat::Add {
                num: 1,
                name: "hoplite".into(),
                who: 0,
                // `parse_coord`'s tile arm: `n × 0xc0 + 0x60`.
                at: Some(Pos::new(4 * 0xc0 + 0x60, 40 * 0xc0 + 0x60)),
            }
        );
        // The leading count, and its cap at 300.
        assert_eq!(
            parse("add 7 hoplite who=1 5,40"),
            Cheat::Add {
                num: 7,
                name: "hoplite".into(),
                who: 1,
                at: Some(Pos::new(5 * 0xc0 + 0x60, 40 * 0xc0 + 0x60)),
            }
        );
        let Cheat::Add { num, .. } = parse("add 5000 hoplite 1,1") else {
            panic!("not an add")
        };
        assert_eq!(num, 300);
        assert_eq!(
            parse("library 4 who=1"),
            Cheat::Library { who: 1, level: 4 }
        );
        assert_eq!(
            parse("science 6"),
            Cheat::Epoch {
                who: 0,
                cat: 3,
                level: 6
            }
        );
        assert_eq!(parse("finish"), Cheat::Unmapped("finish".into()));
    }

    /// `parse_coord`'s three arms. The bare number is a **tile** on this
    /// install, which is measured rather than read (`docs/RUNS.md`
    /// run101–run105, `docs/INPUT.md` §11) — the world arm would put
    /// `4,40` eighteen tiles away.
    #[test]
    fn a_bare_coordinate_is_a_tile_centre() {
        assert_eq!(parse_coord("4"), Some(864));
        assert_eq!(parse_coord("t4"), Some(864));
        assert_eq!(parse_coord("c864"), Some(864));
        assert_eq!(parse_coord("x"), None);
        assert_eq!(parse_pos("4,40"), Some(Pos::new(864, 7776)));
        assert_eq!(parse_pos("4"), None);
    }

    /// A line on the wrong side of `run_cmd`'s two disjoint switches
    /// reaches a case that is not there (`docs/ORACLE.md`, "The channel's
    /// vocabulary"), and the interpreter refuses it rather than running
    /// it anyway.
    #[test]
    fn a_line_on_the_wrong_half_of_the_switch_is_refused() {
        let s = Script::parse("0 ai off\n1 !age who=0 8\n");
        assert_eq!(s.lines().len(), 2);
        assert!(!s.lines()[0].console, "a chat line");
        assert!(s.lines()[1].console, "a console line");
    }
}
