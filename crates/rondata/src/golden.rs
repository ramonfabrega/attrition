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
//! every member is a state poke with a named target. ~~No console command
//! issues an order at all~~ — **one does**: `bird` hands gaia's new bird an
//! air patrol (`docs/ORACLE.md`, "The channel's vocabulary"), and that
//! order is the bird's own patrol point in `sim::gaia`, never
//! `sim::orders`. The players' orders come from the `.rcx` through
//! [`crate::input`], and the two halves never overlap.
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
    /// `bird`, table case `0x52`: a Wild Bird for owner 9 at the console's
    /// cursor, on an air patrol of the same point. It takes no argument.
    Bird,
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
    /// Issuer lines run on an earlier frame, each with the frame whose
    /// tick first sees its command (see [`Script::apply`]).
    pending: Vec<(i64, Staged)>,
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
        Script {
            lines,
            next: 0,
            pending: Vec::new(),
        }
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
    ///
    /// **An issuer line (`@`) acts one frame later than it is written.**
    /// `rontrace.dll` calls the issuer at the same point it runs a cheat,
    /// but the issuer only appends to the local `CommandPackage`; the turn
    /// pump walks the package after that frame's `do_frame` and before the
    /// next one's (`docs/GOLDEN.md` §17; the lab's live probe measured
    /// `process_group` on frame 21 for a call on 20). So a line on frame
    /// `f` is run here before the tick of `f + 1`, ahead of any cheat line
    /// staged on `f + 1`, as the pump runs ahead of `do_frame`'s entry.
    pub fn apply(&mut self, frame: i64, built: &mut Built, loaded: &Loaded) -> Applied {
        let mut done = Applied::default();
        let (due, later): (Vec<_>, Vec<_>) = std::mem::take(&mut self.pending)
            .into_iter()
            .partition(|(f, _)| *f <= frame);
        self.pending = later;
        for (f, line) in due {
            if f < frame {
                done.skip(&command_word(&line.text), "the frame was stepped past");
            } else {
                issue(&line, built, &mut done);
            }
        }
        while self.next < self.lines.len() && self.lines[self.next].frame < frame {
            let word = command_word(&self.lines[self.next].text);
            done.skip(&word, "the frame was stepped past");
            self.next += 1;
        }
        while self.next < self.lines.len() && self.lines[self.next].frame == frame {
            let line = self.lines[self.next].clone();
            self.next += 1;
            if line.text.starts_with('@') {
                self.pending.push((frame + 1, line));
                continue;
            }
            run(&line, built, loaded, &mut done);
        }
        done
    }
}

/// An issuer line, parsed the way `tools/trace/tracer.c`'s `issue_line`
/// reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Issued {
    /// `@move <who> <x> <y> <o> [<o> …]`: internal coordinates, object ids,
    /// at most 32 of them.
    Move {
        who: i32,
        to: Pos,
        objects: Vec<i16>,
    },
    /// `@patrol <who> <x> <y> <o> [<o> …]`: the same fields, through
    /// `CommandManager::issue_patrol@00941800` (item 693).
    Patrol {
        who: i32,
        to: Pos,
        objects: Vec<i16>,
    },
    /// `@guard <who> <ox> <whom> <o> [<o> …]`: the charge's object id and
    /// owner in place of the point, through
    /// `CommandManager::issue_guard@00941ed0` (item 696).
    Guard {
        who: i32,
        ox: i32,
        whom: i32,
        objects: Vec<i16>,
    },
    /// `@follow <who> <ox> <whom> <o> [<o> …]`: the leader's object id and
    /// owner, through `CommandManager::issue_follow@00941e70` (item 714).
    Follow {
        who: i32,
        ox: i32,
        whom: i32,
        objects: Vec<i16>,
    },
    /// `@garrison <who> <ox> <whom> <o> [<o> …]`: the building's object id
    /// and owner, through `CommandManager::issue_garrison@00941a70` (item
    /// 718).
    Garrison {
        who: i32,
        ox: i32,
        whom: i32,
        objects: Vec<i16>,
    },
    /// `@eject <who> <b> [<b> …]`: the player's own buildings, through
    /// `CommandManager::issue_eject_all@00941ca0` with the Eject button's
    /// `back_to_work 0, who −1, eject_o −1, eject_who −1` (item 718).
    Eject { who: i32, buildings: Vec<i16> },
    /// `@form <who> <form> <rotate> <o> [<o> …]`: a formation index and a
    /// rotation, through `CommandManager::issue_form@00941580` with
    /// `QUEUE_NEW`, a formation button's bytes (item 723).
    Form {
        who: i32,
        form: i32,
        rotate: i32,
        objects: Vec<i16>,
    },
    /// `@attack <who> <ox> <whom> <o> [<o> …]`: the target's object id and
    /// owner, through `CommandManager::issue_attack@009415e0` with `ignore
    /// 0` and `QUEUE_NEW`, a right-click on an enemy (item 731).
    Attack {
        who: i32,
        ox: i32,
        whom: i32,
        objects: Vec<i16>,
    },
    /// `@amove <who> <x> <y> <o> [<o> …]`: `@move`'s fields through the
    /// same `issue_move_to@00941720`, with `ATTACK_TO` for `MOVE_TO` — a
    /// ctrl+right-click on the ground, the attack-move (item 731).
    AttackMove {
        who: i32,
        to: Pos,
        objects: Vec<i16>,
    },
    /// `@explore <who> <x> <y> <o> [<o> …]`: `@move`'s fields through the
    /// same `issue_move_to@00941720`, with `EXPLORE_TO` (3) for `MOVE_TO` —
    /// the Explore button's pick on the ground
    /// (`Options::picked_spot@00721c40:905`), item 738.
    Explore {
        who: i32,
        to: Pos,
        objects: Vec<i16>,
    },
    /// `@flee <who> <x> <y> <o> [<o> …]`: `@move`'s fields with `FLEE_TO`
    /// (4). The Flee button's pick (`Options::picked_spot@00721c40:946`)
    /// issues the same move to a friendly building's point and then a
    /// `QUEUE_LAST` garrison; the DLL issues the move alone, on the ground
    /// (item 738, `docs/GOLDEN.md` §24).
    Flee {
        who: i32,
        to: Pos,
        objects: Vec<i16>,
    },
    /// `@flight <who> <ox> <whom> <o> [<o> …]`: an aircraft's flight to its
    /// side's own base or carrier, through
    /// `CommandManager::issue_flight@00941d40` with `MOVE_TO` — a
    /// right-click on one's own Airbase (`Console::execute_at_cursor@
    /// 007c6630:2849`), item 746, `docs/GOLDEN.md` §25.
    Flight {
        who: i32,
        ox: i32,
        whom: i32,
        objects: Vec<i16>,
    },
    /// `@strike <who> <ox> <whom> <o> [<o> …]`: the same issuer with
    /// `ATTACK`, a right-click on an enemy (`:2835`), item 746.
    Strike {
        who: i32,
        ox: i32,
        whom: i32,
        objects: Vec<i16>,
    },
    /// `@build <who> <x> <y> <type> <o> [<o> …]`: a building's
    /// `TypeIndex` dropped at the point, through
    /// `CommandManager::issue_build@00941c30` with the point twice and
    /// `QUEUE_NEW` — an unmodified drop with no drag
    /// (`Options::picked_spot@00721c40:531`), item 779, `docs/GOLDEN.md`
    /// §26.
    Build {
        who: i32,
        to: Pos,
        build: i32,
        objects: Vec<i16>,
    },
    /// `@spell <who> <type> <ox> <whom> <x> <y> <o>…` — `issue_spell@
    /// 00941b80(group, type, ox, whom, x, y)`: a craft's `TypeIndex`, the
    /// object picked and the pick's point (item 790, `docs/GOLDEN.md` §27).
    Spell {
        who: i32,
        spell: i32,
        ox: i32,
        whom: i32,
        at: Pos,
        objects: Vec<i16>,
    },
    /// `@settransport <who> <flag> <o>…` — `issue_set_transport@00941910(
    /// group, flag)`: the transport button's toggle (`Options::
    /// do_transport@0071c500`), item 803, `docs/GOLDEN.md` §28.
    SetTransport {
        who: i32,
        flag: i32,
        objects: Vec<i16>,
    },
    /// `@repair <who> <ox> <whom> <o>…` — `issue_swarm_around@009416b0(
    /// group, ox, whom, QUEUE_NEW, REPAIR)`: an unmodified right-click on
    /// a damaged friendly building (`Console::execute_at_cursor@007c6630`),
    /// item 813, `docs/GOLDEN.md` §29.
    Repair {
        who: i32,
        ox: i32,
        whom: i32,
        objects: Vec<i16>,
    },
    /// `@buildmask <who> <mask> <b>…` — `issue_buildmask@00941f80(group,
    /// mask, 1)` on a group of the player's own buildings: the repeat
    /// button's 0x80 (`Options::set_air_repeat@0071c740`), item 867,
    /// `docs/GOLDEN.md` §32. The wire's `set` is always 1 and
    /// `Group::action_buildmask@006fc9a0` reads neither: it toggles.
    Buildmask {
        who: i32,
        mask: i32,
        buildings: Vec<i16>,
    },
    /// `@queueup <who> <type> <num> <b>…` — `issue_queue_up@00941be0(group,
    /// type, num)` on a group of the player's own buildings: a unit's
    /// button (`GroupOut::issue_queue_up@00708c90`), item 877,
    /// `docs/GOLDEN.md` §33. `type` is the original's `TypeIndex`.
    QueueUp {
        who: i32,
        ty: i32,
        num: i32,
        buildings: Vec<i16>,
    },
    /// `@unqueue <who> <p> <b>…` — `CommandManager::issue_unqueue@
    /// 00942c40(b, p)` once per building, as `Options::exec@007188c0`'s
    /// option 0xa6 loops a selection: the cancel (item 884,
    /// `docs/GOLDEN.md` §34). `p` is a slot, or −1 the last, −5 five, −10
    /// all. The command carries no `group`.
    Unqueue {
        who: i32,
        p: i32,
        buildings: Vec<i16>,
    },
    /// `@gatherpoint <who> <x> <y> <action> <b>…` —
    /// `CommandManager::issue_gather_point@00941b20(group, x, y, action,
    /// add_to_end 0)` on a group of the player's own buildings: the rally
    /// point (item 928, `docs/GOLDEN.md` §39). `action` is 0 for the ground,
    /// 1 a friendly object's point, 2 an enemy's; `−1, −1, 0` is the Clear
    /// button's (`Options::do_clear_gather@0071ce70`).
    GatherPoint {
        who: i32,
        at: Pos,
        action: i32,
        buildings: Vec<i16>,
    },
}

/// `None` for a line the DLL refuses as unparsed (its refusal 5): not `@`,
/// not `move`, `patrol`, `guard`, `follow`, `garrison`, `eject`, `form`,
/// `attack`, `amove`, `explore`, `flee`, `flight`, `strike`, `build`,
/// `spell`, `settransport`, `repair`, `buildmask`, `queueup`, `unqueue` or
/// `gatherpoint`, a `who` outside `0..8`, fewer than three numbers (one for
/// `eject`, two for `settransport`, `buildmask` and `unqueue`, four for
/// `build`, `queueup` and `gatherpoint`, six for `spell`), or no object.
pub fn parse_issuer(text: &str) -> Option<Issued> {
    let mut tok = text.strip_prefix('@')?.split_whitespace();
    let verb = tok.next()?;
    if !matches!(
        verb,
        "move"
            | "patrol"
            | "guard"
            | "follow"
            | "garrison"
            | "eject"
            | "form"
            | "attack"
            | "amove"
            | "explore"
            | "flee"
            | "flight"
            | "strike"
            | "build"
            | "spell"
            | "settransport"
            | "repair"
            | "buildmask"
            | "queueup"
            | "unqueue"
            | "gatherpoint"
    ) {
        return None;
    }
    let nums: Vec<i32> = tok.map_while(|t| t.parse::<i32>().ok()).collect();
    if verb == "eject" {
        let [who, ref buildings @ ..] = nums[..] else {
            return None;
        };
        let buildings: Vec<i16> = buildings
            .iter()
            .take(32)
            .map(|&o| i16::try_from(o).ok())
            .collect::<Option<_>>()?;
        if buildings.is_empty() || !(0..8).contains(&who) {
            return None;
        }
        return Some(Issued::Eject { who, buildings });
    }
    // `@buildmask`'s one number is the mask, and its objects are buildings.
    if verb == "buildmask" {
        let [who, mask, ref buildings @ ..] = nums[..] else {
            return None;
        };
        let buildings: Vec<i16> = buildings
            .iter()
            .take(32)
            .map(|&o| i16::try_from(o).ok())
            .collect::<Option<_>>()?;
        if buildings.is_empty() || !(0..8).contains(&who) {
            return None;
        }
        return Some(Issued::Buildmask {
            who,
            mask,
            buildings,
        });
    }
    // `@unqueue`'s one number is the selector, and its objects are
    // buildings, each its own command.
    if verb == "unqueue" {
        let [who, p, ref buildings @ ..] = nums[..] else {
            return None;
        };
        let buildings: Vec<i16> = buildings
            .iter()
            .take(32)
            .map(|&o| i16::try_from(o).ok())
            .collect::<Option<_>>()?;
        if buildings.is_empty() || !(0..8).contains(&who) {
            return None;
        }
        return Some(Issued::Unqueue { who, p, buildings });
    }
    // `@gatherpoint`'s three numbers are the point and the action, and its
    // objects are buildings.
    if verb == "gatherpoint" {
        let [who, x, y, action, ref buildings @ ..] = nums[..] else {
            return None;
        };
        let buildings: Vec<i16> = buildings
            .iter()
            .take(32)
            .map(|&o| i16::try_from(o).ok())
            .collect::<Option<_>>()?;
        if buildings.is_empty() || !(0..8).contains(&who) {
            return None;
        }
        return Some(Issued::GatherPoint {
            who,
            at: Pos::new(x, y),
            action,
            buildings,
        });
    }
    // `@queueup`'s two numbers are the type and the count, and its objects
    // are buildings.
    if verb == "queueup" {
        let [who, ty, num, ref buildings @ ..] = nums[..] else {
            return None;
        };
        let buildings: Vec<i16> = buildings
            .iter()
            .take(32)
            .map(|&o| i16::try_from(o).ok())
            .collect::<Option<_>>()?;
        if buildings.is_empty() || !(0..8).contains(&who) {
            return None;
        }
        return Some(Issued::QueueUp {
            who,
            ty,
            num,
            buildings,
        });
    }
    // `@settransport`'s one number is the flag.
    if verb == "settransport" {
        let [who, flag, ref objects @ ..] = nums[..] else {
            return None;
        };
        let objects: Vec<i16> = objects
            .iter()
            .take(32)
            .map(|&o| i16::try_from(o).ok())
            .collect::<Option<_>>()?;
        if objects.is_empty() || !(0..8).contains(&who) {
            return None;
        }
        return Some(Issued::SetTransport { who, flag, objects });
    }
    // `@spell`'s craft and target come before its point.
    if verb == "spell" {
        let [who, spell, ox, whom, x, y, ref objects @ ..] = nums[..] else {
            return None;
        };
        let objects: Vec<i16> = objects
            .iter()
            .take(32)
            .map(|&o| i16::try_from(o).ok())
            .collect::<Option<_>>()?;
        if objects.is_empty() || !(0..8).contains(&who) {
            return None;
        }
        return Some(Issued::Spell {
            who,
            spell,
            ox,
            whom,
            at: Pos::new(x, y),
            objects,
        });
    }
    let (build, nums) = if verb == "build" {
        let [who, x, y, build, ref objects @ ..] = nums[..] else {
            return None;
        };
        (build, [&[who, x, y][..], objects].concat())
    } else {
        (0, nums)
    };
    let [who, x, y, ref objects @ ..] = nums[..] else {
        return None;
    };
    let objects: Vec<i16> = objects
        .iter()
        .take(32)
        .map(|&o| i16::try_from(o).ok())
        .collect::<Option<_>>()?;
    if objects.is_empty() || !(0..8).contains(&who) {
        return None;
    }
    let to = Pos::new(x, y);
    Some(match verb {
        "patrol" => Issued::Patrol { who, to, objects },
        "guard" => Issued::Guard {
            who,
            ox: x,
            whom: y,
            objects,
        },
        "follow" => Issued::Follow {
            who,
            ox: x,
            whom: y,
            objects,
        },
        "garrison" => Issued::Garrison {
            who,
            ox: x,
            whom: y,
            objects,
        },
        "form" => Issued::Form {
            who,
            form: x,
            rotate: y,
            objects,
        },
        "attack" => Issued::Attack {
            who,
            ox: x,
            whom: y,
            objects,
        },
        "amove" => Issued::AttackMove { who, to, objects },
        "explore" => Issued::Explore { who, to, objects },
        "flee" => Issued::Flee { who, to, objects },
        "flight" => Issued::Flight {
            who,
            ox: x,
            whom: y,
            objects,
        },
        "strike" => Issued::Strike {
            who,
            ox: x,
            whom: y,
            objects,
        },
        "build" => Issued::Build {
            who,
            to,
            build,
            objects,
        },
        "repair" => Issued::Repair {
            who,
            ox: x,
            whom: y,
            objects,
        },
        _ => Issued::Move { who, to, objects },
    })
}

/// An issuer line, `@move <who> <x> <y> <o> [<o> …]`, as the turn pump
/// processes the command `rontrace.dll` issued for it (`docs/GOLDEN.md`
/// §17).
///
/// The DLL calls `CommandManager::issue_move_to@00941720` with a plain
/// right-click's arguments — `QUEUE_NEW`, no angle, `MOVE_TO`, form and
/// width −1, no disembark (`WorldMap::on_right_up@008c7050:203`) — on a
/// `GroupOut` listing the objects, and refuses by an INFO record, issuing
/// nothing, when an object is not a live captain of `who`. So the command
/// that reaches the pump is a `group` of exactly those captains and a
/// `move_to`, and [`crate::input::group_move_to`] is its entry.
fn issue(line: &Staged, built: &mut Built, done: &mut Applied) {
    let word = command_word(&line.text);
    // `@patrol` is `issue_patrol@00941800` with `QUEUE_NEW`, a `group` and
    // a `patrol`, whose entry is [`crate::input::group_patrol`] (item 693).
    let n = match parse_issuer(&line.text) {
        Some(Issued::Move { who, to, objects }) => {
            crate::input::group_move_to(built, who, &objects, to, 2, false, 0, 1)
        }
        Some(Issued::Patrol { who, to, objects }) => {
            crate::input::group_patrol(built, who, &objects, to, 2)
        }
        // `@guard` is `issue_guard@00941ed0` with `QUEUE_NEW`, a `group`
        // and a `guard`, whose entry is [`crate::input::group_guard`]
        // (item 696).
        Some(Issued::Guard {
            who,
            ox,
            whom,
            objects,
        }) => crate::input::group_guard(built, who, &objects, ox, whom, 2),
        // `@follow` is `issue_follow@00941e70` with `QUEUE_NEW`, a `group`
        // and a `follow`, whose entry is [`crate::input::group_follow`]
        // (item 714).
        Some(Issued::Follow {
            who,
            ox,
            whom,
            objects,
        }) => crate::input::group_follow(built, who, &objects, ox, whom, 2),
        // `@garrison` is `issue_garrison@00941a70` with `QUEUE_NEW`, a
        // `group` and a `garrison`, whose entry is
        // [`crate::input::group_garrison`]; `@eject` is
        // `issue_eject_all@00941ca0` on a group of buildings,
        // [`crate::input::group_eject_all`] (item 718).
        Some(Issued::Garrison {
            who,
            ox,
            whom,
            objects,
        }) => crate::input::group_garrison(built, who, &objects, ox, whom, 2),
        Some(Issued::Eject { who, buildings }) => {
            crate::input::group_eject_all(built, who, &buildings)
        }
        // `@form` is `issue_form@00941580` with `QUEUE_NEW`, a `group`
        // and a `form`, whose entry is [`crate::input::group_form`] (item
        // 723, `docs/GOLDEN.md` §22).
        Some(Issued::Form {
            who,
            form,
            rotate,
            objects,
        }) => crate::input::group_form(built, who, &objects, form, rotate, 2),
        // `@amove` is `issue_move_to@00941720` with `ATTACK_TO`, whose
        // entry is [`crate::input::group_move_to`] with `orders` 2 (item
        // 731, `docs/GOLDEN.md` §23).
        Some(Issued::AttackMove { who, to, objects }) => {
            crate::input::group_move_to(built, who, &objects, to, 2, false, 0, 2)
        }
        // `@explore` and `@flee` are `issue_move_to@00941720` with
        // `EXPLORE_TO` (3) and `FLEE_TO` (4), whose entry is
        // [`crate::input::group_move_to`] with that `orders` byte (item
        // 738, `docs/GOLDEN.md` §24).
        Some(Issued::Explore { who, to, objects }) => {
            crate::input::group_move_to(built, who, &objects, to, 2, false, 0, 3)
        }
        Some(Issued::Flee { who, to, objects }) => {
            crate::input::group_move_to(built, who, &objects, to, 2, false, 0, 4)
        }
        // `@attack` is `issue_attack@009415e0` with `ignore` 0 and
        // `QUEUE_NEW`, a `group` and an `attack`, whose entry is
        // [`crate::input::group_attack`] (item 731, `docs/GOLDEN.md` §23).
        Some(Issued::Attack {
            who,
            ox,
            whom,
            objects,
        }) => crate::input::group_attack(built, who, &objects, ox, whom, 0, 2),
        // `@flight` and `@strike` are `issue_flight@00941d40` with
        // `MOVE_TO` (1) and `ATTACK` (10), a `group` and a `flight`, whose
        // entry is [`crate::input::group_flight`] (item 746,
        // `docs/GOLDEN.md` §25).
        Some(Issued::Flight {
            who,
            ox,
            whom,
            objects,
        }) => crate::input::group_flight(built, who, &objects, ox, whom, 1),
        Some(Issued::Strike {
            who,
            ox,
            whom,
            objects,
        }) => crate::input::group_flight(built, who, &objects, ox, whom, 10),
        // `@build` is `issue_build@00941c30` with the point twice and
        // `QUEUE_NEW`, a `group` and a `build`, whose entry is
        // [`crate::input::group_build`] (item 779, `docs/GOLDEN.md` §26).
        Some(Issued::Build {
            who,
            to,
            build,
            objects,
        }) => crate::input::group_build(built, who, &objects, to, build, 2),
        // `@spell` is `issue_spell@00941b80`, a `group` and a `spell`,
        // whose entry is [`crate::input::group_spell`] (item 790,
        // `docs/GOLDEN.md` §27).
        Some(Issued::Spell {
            who,
            spell,
            ox,
            whom,
            at,
            objects,
        }) => crate::input::group_spell(built, who, &objects, spell, ox, whom, at),
        // `@settransport` is `issue_set_transport@00941910`, a `group` and a
        // `set_transport`, whose entry is
        // [`crate::input::group_set_transport`] (item 803, `docs/GOLDEN.md`
        // §28).
        Some(Issued::SetTransport { who, flag, objects }) => {
            crate::input::group_set_transport(built, who, &objects, flag)
        }
        // `@repair` is `issue_swarm_around@009416b0` with `QUEUE_NEW` and
        // `REPAIR`, a `group` and a `swarm_around`, whose entry is
        // [`crate::input::group_swarm_around`] (item 813, `docs/GOLDEN.md`
        // §29).
        Some(Issued::Repair {
            who,
            ox,
            whom,
            objects,
        }) => crate::input::group_swarm_around(built, who, &objects, ox, whom, 2, 13),
        // `@buildmask` is `issue_buildmask@00941f80` on a group of
        // buildings, a `group` and a `buildmask`, whose entry is
        // [`crate::input::group_buildmask`] (item 867, `docs/GOLDEN.md`
        // §32).
        Some(Issued::Buildmask {
            who,
            mask,
            buildings,
        }) => crate::input::group_buildmask(built, who, &buildings, mask),
        // `@queueup` is `issue_queue_up@00941be0` on a group of buildings,
        // a `group` and a `queue_up`, whose entry is
        // [`crate::input::group_queue_up`] (item 877, `docs/GOLDEN.md`
        // §33) — a technology's too, `action_queue_up`'s research arm
        // (item 883, §35).
        Some(Issued::QueueUp {
            who,
            ty,
            num,
            buildings,
        }) => crate::input::group_queue_up(built, who, &buildings, ty, num),
        // `@unqueue` is `issue_unqueue@00942c40` once per building, an
        // `unqueue` with no `group`, whose entry is [`crate::input::unqueue`]
        // (item 884, `docs/GOLDEN.md` §34).
        Some(Issued::Unqueue { who, p, buildings }) => buildings
            .iter()
            .map(|&b| crate::input::unqueue(built, who, i32::from(b), p))
            .sum(),
        // `@gatherpoint` is `issue_gather_point@00941b20` on a group of
        // buildings, a `group` and a `gather_point`, whose entry is
        // [`crate::input::group_gather_point`] (item 928, `docs/GOLDEN.md`
        // §39).
        Some(Issued::GatherPoint {
            who,
            at,
            action,
            buildings,
        }) => crate::input::group_gather_point(built, who, &buildings, at, action, false),
        None => {
            done.skip(&word, "not an issuer line the DLL runs");
            return;
        }
    };
    if n == 0 {
        done.skip(&word, "no named object is a live unit in the simulation");
        return;
    }
    done.ran += 1;
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
        "bird" => Cheat::Bird,
        _ => Cheat::Unmapped(word),
    }
}

/// **The console's cursor on the staged channel**, `console_win +0x518/
/// +0x51c` (`mouse_coord_x/y`) — the point `bird` reads.
///
/// Nothing on the channel writes it. Its two writers are `parse_cmd` with
/// `no_mouse` 0 and `CommandPackage::process_console_cmd`, and the tracer
/// reaches neither; `ConsoleWin::ConsoleWin@007e6370` and
/// `ConsoleWin::init@007e6550` leave the field alone, and the object is a
/// `malloc(0x558)` in `System::init@00599700`. So the value is what the heap
/// left there, and it is **measured, not read**. run169's packet at logger
/// frame 701 holds `(0, 6)` in the field, and the same point as the
/// one fresh `AirPatrolOrder`'s waypoint (`docs/RUNS.md` run169). With it,
/// chapter six walks run168 to its end. At `(0, 0)` the walk parts on 894,
/// the bird's third edge coin, two frames early, because `Unit::init`
/// snaps both points onto the same seat and only the patrol point differs
/// (`docs/GOLDEN.md` §10). Whether every launch leaves the same value is
/// parked 653; run168 and run169 are two that did.
pub const STAGED_CURSOR: Pos = Pos { x: 0, y: 6 };

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
        Cheat::Bird => {
            // `run_cmd` case `0x52`: `Objects::init_unit(objects, 9,
            // BASE_GAIATYPES, x, y, −1, −1, −1)` on the raw cursor — no
            // `find_nearby_spot` and no `WorldData::restrict`, where `nuke`
            // beside it has one — and `Unit::add_air_patrol_order` on the
            // same point when the unit was made. The guard in front,
            // `(semaphore & 4) == 0 || no_mouse != 0`, always passes on the
            // channel, which calls with `no_mouse` 1. It is the sampling's
            // own pair, so it is the sampling's entry point.
            if built.sim.spawn_bird_at(STAGED_CURSOR).is_some() {
                done.units += 1;
                done.ran += 1;
            } else {
                done.skip(&word, "no bird type loaded");
            }
        }
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
/// `Objects::init_build` once, then `Build::activate`, and **breaks out of
/// the count loop**, so a leading count places one finished building and
/// not `num` of them.
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
        //
        // **Placed and finished, not ordered** (item 552). The arm is
        // `Objects::init_build(who, type, x, y, 0, −1)` — no price, no site
        // test, no city limit — and then, for a line without `NEW`, the
        // building's vslot `0x1a8`, `Build::activate`, with `(captured 0,
        // announce 1, counted 0)` pushed at `0x7e055b`–`0x7e0563`. This arm
        // took [`sim::Sim::place_building`] until chapter four staged the
        // first building: that is `Group::action_build`, which charges the
        // price and leaves an unstarted site, so run132's Temple never set
        // its city's temple bit and the border never moved. `NEW`, the
        // unfinished form, is not parsed; no chapter uses it.
        // `init_build`'s own `snap_center`, with its dock arm (item 803):
        // an `add dock` on a shore tile lands on the water beside it.
        let at = built
            .sim
            .snap_center_placed(ty, at, Some(who as sim::Player));
        let b = built.sim.init_build(who as sim::Player, ty, at, false);
        built.sim.activate(b, false, false);
        done.buildings += 1;
        done.ran += 1;
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

    /// **What each chapter of the golden record asks for and the interpreter
    /// will not do**, one row per file under `tools/gamelog/golden/`
    /// (`docs/GOLDEN.md`). A word lands here when [`parse`] refuses it outright
    /// or when it parses into the bare diplomacy form, which prints the table
    /// and changes nothing (`docs/INPUT.md` §11.6) — the two ways a staged line
    /// can be carried and not acted on.
    ///
    /// The point of pinning it is that a new chapter cannot quietly spend a
    /// capture on a verb the harness drops: adding one either leaves this table
    /// alone or states the debt in the same commit.
    const CHAPTER_DEBT: &[(&str, &[&str])] = &[
        // The bare `war`: chapter one's squads engage because a Quick Battle
        // already starts at war, not because of the line (item 364).
        ("chapter1.cmd", &["war"]),
        // Chapter ten (sorted as the directory is): two `@patrol` issuer lines, the patrol line (item
        // 693, `docs/GOLDEN.md` §18).
        ("chapter10.cmd", &[]),
        // Chapter eleven: two `@guard` issuer lines and a `@move`, the guard
        // line (item 696, `docs/GOLDEN.md` §19).
        ("chapter11.cmd", &[]),
        // Chapter twelve: two `@follow` issuer lines and four `@move`s, the
        // follow line (item 714, `docs/GOLDEN.md` §20).
        ("chapter12.cmd", &[]),
        // Chapter thirteen: two `@garrison` issuer lines and an `@eject`, the
        // garrison line (item 718, `docs/GOLDEN.md` §21).
        ("chapter13.cmd", &[]),
        // Chapter fourteen: two `@form` issuer lines and a `@move`, the
        // formation line (item 723, `docs/GOLDEN.md` §22).
        ("chapter14.cmd", &[]),
        // Chapter fifteen: a `@move`, an `@attack` and an `@amove`, the
        // group attack line (item 731, `docs/GOLDEN.md` §23).
        ("chapter15.cmd", &[]),
        // Chapter sixteen: two `@explore` and two `@flee` issuer lines, the
        // move issuer's trailing selector (item 738, `docs/GOLDEN.md` §24).
        ("chapter16.cmd", &[]),
        // Chapter seventeen: two `@flight` and two `@strike` issuer lines,
        // the flight line (item 746, `docs/GOLDEN.md` §25).
        ("chapter17.cmd", &[]),
        // Chapter eighteen: two `@build` issuer lines, the build line
        // (item 779, `docs/GOLDEN.md` §26).
        ("chapter18.cmd", &[]),
        ("chapter19.cmd", &[]),
        ("chapter2.cmd", &[]),
        // Chapter twenty: two `@settransport` lines, three `@move`s and a
        // Dock, the board line (item 803, `docs/GOLDEN.md` §28).
        ("chapter20.cmd", &[]),
        ("chapter21.cmd", &[]),
        // Chapter twenty-two: chapter seventeen and one more `@strike`, on
        // the Fighter inside its base, the launch line (item 836,
        // `docs/GOLDEN.md` §31).
        ("chapter22.cmd", &[]),
        // Chapter twenty-three: chapter twenty-two and one `@buildmask`, the
        // Airbase's repeat toggled off between the landings (item 867,
        // `docs/GOLDEN.md` §32).
        ("chapter23.cmd", &[]),
        // Chapter twenty-four: two `@queueup` lines and two `@buildmask`
        // lines with the infinite-queue mask 0x40 on a Barracks, the queue
        // line (item 877, `docs/GOLDEN.md` §33).
        ("chapter24.cmd", &[]),
        // Chapter twenty-five: three `@queueup` lines, four `@unqueue`
        // lines and a `@buildmask` 0x40 on a Barracks, the cancel line
        // (item 884, `docs/GOLDEN.md` §34).
        ("chapter25.cmd", &[]),
        // Chapter twenty-six: seven `@queueup` lines, five of them a
        // technology at who=0's Library, and an `@unqueue` of one, the
        // research line (item 883, `docs/GOLDEN.md` §35).
        ("chapter26.cmd", &[]),
        // Chapter twenty-seven: a unit upgrade through the same `@queueup`,
        // the Classical age and The Art of War staged by `age` and
        // `military` (item 901, `docs/GOLDEN.md` §36).
        ("chapter27.cmd", &[]),
        // Chapter twenty-eight: a second Barracks, three `@queueup` lines
        // and two `@buildmask` lines on one or both, two buildings under one
        // command (item 888, `docs/GOLDEN.md` §37).
        ("chapter28.cmd", &[]),
        // Chapter twenty-nine: chapter twenty-two with no toggle and one
        // `@queueup` of a Biplane at the Airbase, the repeat launch (item
        // 915, `docs/GOLDEN.md` §38).
        ("chapter29.cmd", &[]),
        // Chapter thirty: chapter twenty-eight's cast and five
        // `@gatherpoint` lines (the DLL's verb 20) on two Barracks and the
        // City, three `@queueup` lines behind them: the gather point (item
        // 928, `docs/GOLDEN.md` §39).
        ("chapter30.cmd", &[]),
        ("chapter3.cmd", &[]),
        // Chapter three restaged in two arenas (item 587, run146).
        ("chapter3b.cmd", &[]),
        ("chapter4.cmd", &[]),
        ("chapter5.cmd", &[]),
        // `bird`, the one console command that issues an order, is staged
        // at the channel's cursor since item 652 (`docs/GOLDEN.md` §10).
        ("chapter6.cmd", &[]),
        // Chapter six-b: chapter six with an Airbase a side (item 651).
        ("chapter6b.cmd", &[]),
        ("chapter7.cmd", &[]),
        // Chapter seven's control, the same file less `0 !ai off` (item 578).
        ("chapter7_control.cmd", &[]),
        // Chapter seven-b: the same five for who=1, and its control (item 628).
        ("chapter7b.cmd", &[]),
        ("chapter7b_control.cmd", &[]),
        ("chapter8.cmd", &[]),
        // Chapter nine: two `@move` issuer lines, the first player orders
        // in the record (item 676, `docs/GOLDEN.md` §17).
        ("chapter9.cmd", &[]),
    ];

    /// The chapter directory as the tree has it, sorted.
    fn chapters() -> Vec<(String, Script)> {
        let dir = std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tools/gamelog/golden"
        ))
        .to_path_buf();
        let mut out: Vec<(String, Script)> = std::fs::read_dir(&dir)
            .expect("tools/gamelog/golden/")
            .map(|e| e.expect("entry").path())
            .filter(|p| p.extension().is_some_and(|e| e == "cmd"))
            .map(|p| {
                let name = p.file_name().unwrap().to_str().unwrap().to_string();
                (name, Script::read(&p).expect("a readable chapter"))
            })
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    /// **Every chapter is stageable, and what it asks for and does not get
    /// is written down.** `docs/GOLDEN.md` §3: a chapter names its cheat
    /// lines, the records the capture must dump, and what would falsify it
    /// — and this is the half of that a test can hold. A line on the wrong
    /// half of `run_cmd`'s two disjoint switches reaches a case that is not
    /// there, so the halves are checked too.
    #[test]
    fn every_chapter_stages_what_it_says_it_stages() {
        let chapters = chapters();
        assert_eq!(
            chapters.len(),
            CHAPTER_DEBT.len(),
            "tools/gamelog/golden/ holds {} chapter(s) and CHAPTER_DEBT names {}; \
             a new chapter states its debt in the same commit",
            chapters.len(),
            CHAPTER_DEBT.len()
        );
        for ((name, script), (pinned_name, pinned)) in chapters.iter().zip(CHAPTER_DEBT) {
            assert_eq!(name, pinned_name, "CHAPTER_DEBT is out of order");
            assert!(!script.is_empty(), "{name} stages nothing");
            let mut debt: Vec<String> = Vec::new();
            for line in script.lines() {
                // An issuer line never reaches `parse_cmd`: it only has to
                // be one the DLL runs (item 676).
                if line.text.starts_with('@') {
                    assert!(
                        !line.console && parse_issuer(&line.text).is_some(),
                        "{name}: `{}` is not an issuer line rontrace.dll runs",
                        line.text
                    );
                    continue;
                }
                let word = command_word(&line.text);
                let console_only =
                    matches!(word.as_str(), "ai" | "quit" | "go" | "break" | "restart");
                assert_eq!(
                    line.console, console_only,
                    "{name}: `{}` is on the wrong half of run_cmd's two switches",
                    line.text
                );
                match parse(&line.text) {
                    Cheat::Unmapped(w) => debt.push(w),
                    Cheat::Diplo { target: None, .. } => debt.push(word),
                    _ => {}
                }
            }
            debt.sort();
            debt.dedup();
            assert_eq!(
                debt,
                pinned.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                "{name}: the verbs the interpreter will not act on moved; \
                 re-pin CHAPTER_DEBT and say so in docs/GOLDEN.md"
            );
        }
    }

    /// **`@build` parses as the DLL reads it** (item 779): `who`, the
    /// point, the `TypeIndex`, then the objects; three numbers and no
    /// type, or a type and no object, is the DLL's refusal 5.
    #[test]
    fn a_build_line_is_the_dll_s_build() {
        assert_eq!(
            parse_issuer("@build 0 7296 36864 430 7 8 9"),
            Some(Issued::Build {
                who: 0,
                to: Pos::new(7296, 36864),
                build: 430,
                objects: vec![7, 8, 9],
            })
        );
        assert_eq!(parse_issuer("@build 0 7296 36864 430"), None);
        assert_eq!(parse_issuer("@build 0 7296 36864"), None);
    }

    /// **A spell line is the DLL's spell** (item 790): `who`, the craft's
    /// `TypeIndex`, the target's `ox` and `whom`, the pick's point, then the
    /// objects; a line one number short of its point, or with no object, is
    /// the DLL's refusal 5.
    #[test]
    fn a_spell_line_is_the_dll_s_spell() {
        assert_eq!(
            parse_issuer("@spell 0 639 2006 1 15360 15360 6"),
            Some(Issued::Spell {
                who: 0,
                spell: 639,
                ox: 2006,
                whom: 1,
                at: Pos::new(15360, 15360),
                objects: vec![6],
            })
        );
        assert_eq!(parse_issuer("@spell 0 639 2006 1 15360 15360"), None);
        assert_eq!(parse_issuer("@spell 0 639 2006 1 15360"), None);
    }

    /// **A set-transport line is the DLL's toggle** (item 803): `who`, the
    /// flag and the objects; the DLL refuses a line with no object by its
    /// refusal 5.
    #[test]
    fn a_set_transport_line_is_the_dll_s_toggle() {
        assert_eq!(
            parse_issuer("@settransport 0 0 7"),
            Some(Issued::SetTransport {
                who: 0,
                flag: 0,
                objects: vec![7],
            })
        );
        assert_eq!(
            parse_issuer("@settransport 0 1 6 7"),
            Some(Issued::SetTransport {
                who: 0,
                flag: 1,
                objects: vec![6, 7],
            })
        );
        assert_eq!(parse_issuer("@settransport 0 1"), None);
        assert_eq!(parse_issuer("@settransport 9 1 6"), None);
    }

    /// **A buildmask line is the DLL's repeat button** (item 867): `who`,
    /// the mask and the buildings; a line with no building is the DLL's
    /// refusal 5.
    #[test]
    fn a_buildmask_line_is_the_dll_s_repeat_button() {
        assert_eq!(
            parse_issuer("@buildmask 0 128 2007"),
            Some(Issued::Buildmask {
                who: 0,
                mask: 128,
                buildings: vec![2007],
            })
        );
        assert_eq!(parse_issuer("@buildmask 0 128"), None);
        assert_eq!(parse_issuer("@buildmask 9 128 2007"), None);
    }

    /// **A queueup line is the DLL's unit button** (item 877): `who`, the
    /// type, the count and the buildings; a line with no building is the
    /// DLL's refusal 5.
    #[test]
    fn a_queueup_line_is_the_dll_s_unit_button() {
        assert_eq!(
            parse_issuer("@queueup 0 132 1 2007"),
            Some(Issued::QueueUp {
                who: 0,
                ty: 132,
                num: 1,
                buildings: vec![2007],
            })
        );
        assert_eq!(parse_issuer("@queueup 0 132 1"), None);
        assert_eq!(parse_issuer("@queueup 9 132 1 2007"), None);
    }

    /// **A gatherpoint line is the DLL's rally point** (item 928): `who`,
    /// the point, the action and the buildings; the Clear button's −1, −1
    /// parse as given; a line with no building is the DLL's refusal 5.
    #[test]
    fn a_gatherpoint_line_is_the_dll_s_rally_point() {
        assert_eq!(
            parse_issuer("@gatherpoint 0 5000 6000 1 2007 2008"),
            Some(Issued::GatherPoint {
                who: 0,
                at: Pos { x: 5000, y: 6000 },
                action: 1,
                buildings: vec![2007, 2008],
            })
        );
        assert_eq!(
            parse_issuer("@gatherpoint 0 -1 -1 0 2007"),
            Some(Issued::GatherPoint {
                who: 0,
                at: Pos { x: -1, y: -1 },
                action: 0,
                buildings: vec![2007],
            })
        );
        assert_eq!(parse_issuer("@gatherpoint 0 5000 6000 0"), None);
        assert_eq!(parse_issuer("@gatherpoint 9 5000 6000 0 2007"), None);
    }

    /// **An unqueue line is the DLL's cancel** (item 884): `who`, the
    /// selector (a slot, or a negative) and the buildings; a line with no
    /// building is the DLL's refusal 5.
    #[test]
    fn an_unqueue_line_is_the_dll_s_cancel() {
        assert_eq!(
            parse_issuer("@unqueue 0 -1 2007"),
            Some(Issued::Unqueue {
                who: 0,
                p: -1,
                buildings: vec![2007],
            })
        );
        assert_eq!(parse_issuer("@unqueue 0 0"), None);
        assert_eq!(parse_issuer("@unqueue 9 0 2007"), None);
    }

    /// **A repair line is the DLL's swarm** (item 813): `who`, the
    /// building's `ox` and `whom`, then the objects; a line with no object
    /// is the DLL's refusal 5.
    #[test]
    fn a_repair_line_is_the_dll_s_swarm() {
        assert_eq!(
            parse_issuer("@repair 0 2006 0 6 7"),
            Some(Issued::Repair {
                who: 0,
                ox: 2006,
                whom: 0,
                objects: vec![6, 7],
            })
        );
        assert_eq!(parse_issuer("@repair 0 2006 0"), None);
    }

    /// **An issuer line parses as the DLL reads it** (item 676): the verb,
    /// `who`, the point in internal units, and the objects. The DLL
    /// refuses each of the malformed forms below by its refusal 5, so
    /// neither side may act on one.
    #[test]
    fn an_issuer_line_is_the_dll_s_move() {
        assert_eq!(
            parse_issuer("@move 0 12672 7296 6"),
            Some(Issued::Move {
                who: 0,
                to: Pos::new(12672, 7296),
                objects: vec![6],
            })
        );
        assert_eq!(
            parse_issuer("@move 0 14208 11904 7 8"),
            Some(Issued::Move {
                who: 0,
                to: Pos::new(14208, 11904),
                objects: vec![7, 8],
            })
        );
        assert_eq!(
            parse_issuer("@patrol 0 3456 11136 6"),
            Some(Issued::Patrol {
                who: 0,
                to: Pos::new(3456, 11136),
                objects: vec![6],
            })
        );
        assert_eq!(
            parse_issuer("@guard 0 7 0 6"),
            Some(Issued::Guard {
                who: 0,
                ox: 7,
                whom: 0,
                objects: vec![6],
            })
        );
        assert_eq!(
            parse_issuer("@follow 0 7 0 6"),
            Some(Issued::Follow {
                who: 0,
                ox: 7,
                whom: 0,
                objects: vec![6],
            })
        );
        assert_eq!(
            parse_issuer("@garrison 0 2007 0 6"),
            Some(Issued::Garrison {
                who: 0,
                ox: 2007,
                whom: 0,
                objects: vec![6],
            })
        );
        assert_eq!(
            parse_issuer("@eject 0 2007"),
            Some(Issued::Eject {
                who: 0,
                buildings: vec![2007],
            })
        );
        assert_eq!(
            parse_issuer("@form 0 2 0 6 9 12"),
            Some(Issued::Form {
                who: 0,
                form: 2,
                rotate: 0,
                objects: vec![6, 9, 12],
            })
        );
        assert_eq!(
            parse_issuer("@attack 0 6 1 6 9"),
            Some(Issued::Attack {
                who: 0,
                ox: 6,
                whom: 1,
                objects: vec![6, 9],
            })
        );
        assert_eq!(
            parse_issuer("@amove 0 3192 7680 6 9"),
            Some(Issued::AttackMove {
                who: 0,
                to: Pos::new(3192, 7680),
                objects: vec![6, 9],
            })
        );
        assert_eq!(
            parse_issuer("@explore 0 2400 17280 6"),
            Some(Issued::Explore {
                who: 0,
                to: Pos::new(2400, 17280),
                objects: vec![6],
            })
        );
        assert_eq!(
            parse_issuer("@flee 0 3168 7008 7"),
            Some(Issued::Flee {
                who: 0,
                to: Pos::new(3168, 7008),
                objects: vec![7],
            })
        );
        assert_eq!(
            parse_issuer("@flight 0 2007 0 7 8"),
            Some(Issued::Flight {
                who: 0,
                ox: 2007,
                whom: 0,
                objects: vec![7, 8],
            })
        );
        assert_eq!(
            parse_issuer("@strike 0 2006 1 6"),
            Some(Issued::Strike {
                who: 0,
                ox: 2006,
                whom: 1,
                objects: vec![6],
            })
        );
        for bad in [
            "@explore 0 2400 17280",
            "@flee 0 3168",
            "move 0 1 2 3",
            "@formation 0 1 2 3",
            "@attack 0 6 1",
            "@amove 0 3192 7680",
            "@form 0 2 0",
            "@eject 0",
            "@eject 9 2007",
            "@garrison 0 2007 0",
            "@follow 0 7 0",
            "@guard 0 7 0",
            "@patrol 0 1 2",
            "@move 0 1 2",
            "@move 9 1 2 3",
            "@move 0 1",
        ] {
            assert_eq!(parse_issuer(bad), None, "{bad}");
        }
        let s = Script::parse("620 @move 0 12672 7296 6\n");
        assert_eq!(s.lines().len(), 1);
        assert!(!s.lines()[0].console);
        assert_eq!(s.lines()[0].text, "@move 0 12672 7296 6");
    }

    /// `bird` takes no argument and is the chat half's (`run_cmd` case
    /// `0x52`), so a staged `700 bird` parses to [`Cheat::Bird`] and is not
    /// debt (item 652).
    #[test]
    fn bird_is_the_chat_half_s_argumentless_spawn() {
        assert_eq!(parse("bird"), Cheat::Bird);
        let s = Script::parse("700 bird\n");
        assert_eq!(s.lines().len(), 1);
        assert!(!s.lines()[0].console);
    }

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
