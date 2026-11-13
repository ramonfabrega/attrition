//! **The lobby's choices** (item 4184; `docs/LOBBY.md`): what a person sets
//! before a game starts, the events that set it, and the `GAMEINFO` block
//! those choices print — the block `GameInfo::log_data@005d6040` writes at
//! the head of every dump, and the one the built start reads its lobby
//! from ([`crate::diff::from_lobby`], `docs/START.md` §2).
//!
//! The page a person sees is the client's (`client::lobby`), a plain page
//! and not the original's interface (DECISIONS 128 §6). This is its model,
//! kept beside the walk it feeds so that the counter — the lobbies on disk
//! whose block the page's events reproduce, field for field — runs where
//! the dumps are read.
//!
//! Nothing here comes from the install at compile time: the names the page
//! shows are `rules.xml`'s categories and the nations' own files, read by
//! [`Roster::read`] from the install a person points the client at.

use std::io::BufRead;
use std::path::Path;

/// **The thirty settings** `GameInfo::log_data` prints after the version,
/// in its order: `gameinfo_strings`
/// (`_dynamic_initializer_for__gameinfo_strings__@00405400`), each one
/// `uchar` of `GameInfo +0x18` (`GameInfo_u_24_s_0` in the type stream).
pub const SETTINGS: [&str; 30] = [
    "TEAM_STYLE",
    "MAP_STYLE",
    "MAP_SIZE",
    "PLAYERS",
    "MAX_OBSERVERS",
    "GAME_SPEED",
    "GAME_RULES",
    "DIFFICULTY",
    "STARTING_TOWN",
    "STARTING_RESOURCES",
    "STARTING_RESOURCES2",
    "TECH_COST",
    "REVEAL_MAP",
    "POP_LIMIT",
    "RUSH_RULES",
    "CANNON_TIMES",
    "STARTING_TECHNOLOGY",
    "STARTING_TECHNOLOGY2",
    "ENDING_TECHNOLOGY",
    "ELIMINATION",
    "VICTORY",
    "WONDERWIN",
    "SCORE_GOAL",
    "POPWIN",
    "TIME_LIMIT",
    "CHAIRS",
    "ECONWIN",
    "SCENARIO_TYPE",
    "SCRIPT_TYPE",
    "MODS",
];

/// **The `rules.xml` category each setting's byte indexes**, by name: the
/// list the page offers for it, its title and its entries' names. The pairing
/// is this item's reading of the two name lists side by side (`MAP_SIZE`
/// and `mapsizes`, `STARTING_RESOURCES2` and the `defenderresources`
/// "Defender Bonus:"), not a reading of the original's lobby code; that the
/// byte is the entry's index is confirmed where a consumer reads it
/// (`world_extent`'s `mapsizes`, `Loaded::starting_resources`) and assumed
/// elsewhere (`docs/LOBBY.md` §4).
pub const CATEGORY: [Option<&str>; 30] = [
    Some("gamestyles"),
    Some("mapstyles"),
    Some("mapsizes"),
    Some("numberofplayers"),
    Some("maxobservers"),
    Some("gamespeeds"),
    Some("gamerules"),
    Some("difficulties"),
    Some("startingtowns"),
    Some("startingresources"),
    Some("defenderresources"),
    Some("techcosts"),
    Some("revealmaps"),
    Some("poplimits"),
    Some("rushrules"),
    Some("cannontimes"),
    Some("startingtechs"),
    Some("defendertechs"),
    Some("endingtechs"),
    Some("eliminations"),
    Some("victories"),
    Some("wonderwins"),
    Some("scores"),
    Some("popwins"),
    Some("timelimits"),
    Some("chairs"),
    Some("econwins"),
    None,
    None,
    Some("mods"),
];

/// `PLAYERS`' place in [`SETTINGS`]: the page derives it — the seats less
/// one — and offers no event for it.
pub const PLAYERS: usize = 3;

/// The settings the page prints at the value every lobby on disk holds
/// and offers no event for: `SCENARIO_TYPE`, `SCRIPT_TYPE` and `MODS` —
/// scenarios and mods are cut from v1 (`CLAUDE.md`).
pub const NOT_OFFERED: [usize; 3] = [27, 28, 29];

/// The executable's build, as the block's first two lines print it on
/// every dump on disk (`Version::get_string`, `GameInfo::version`).
pub const VERSION_STRING: &str = "00.2024.06.2000";
pub const VERSION: u32 = 111_935_788;

/// `GameInfo::checksum_window_size` on every dump on disk.
pub const CHECKSUM_WINDOW_SIZE: i32 = 256;

/// `GameInfo::player` is `Player[8]`.
pub const MAX_SEATS: usize = 8;

/// A seat's `team` when it has none, on every seat on disk:
/// `LeaderData::get_team`'s "no team", the simulation's own constant.
pub use sim::mapgen::map_east_west::NO_TEAM;

/// The name a person's seat is added with: the profile name on 156 of the
/// 161 lobbies on disk (`Player 1` on the other five, run53's among them).
pub const PERSON: &str = "Player";

/// **What the page cannot set**, each field the block prints that no event
/// writes, and why. The block prints each at the value every lobby on disk
/// holds, so a lobby that held another would part on it and be counted
/// out; none does.
pub const NOT_SET: &[(&str, &str)] = &[
    ("(Version: …)", "the executable's build string"),
    ("(int) version", "the executable's build number"),
    (
        "PLAYERS",
        "derived: the seats less one (1 on every two-seat lobby on disk, 2 on the four three-seat ones)",
    ),
    ("SCENARIO_TYPE", "scenarios are cut from v1"),
    ("SCRIPT_TYPE", "scenarios are cut from v1"),
    ("MODS", "mods are cut from v1"),
    ("checksum_deep", "the engine's, not a lobby's"),
    ("checksum_window_size", "the engine's, not a lobby's"),
    ("checksum_failure_threshold", "the engine's, not a lobby's"),
    (
        "PLAYER flags",
        "derived from the seat's kind: 7 a person, 1 the computer; what bit 0x2 means is not established",
    ),
    ("PLAYER who", "the seat's place"),
    ("PLAYER play", "the seat's place"),
    ("PLAYER pauses", "a count the game keeps, 0 at the start"),
    (
        "PLAYER diff",
        "2 on every seat on disk whatever DIFFICULTY says; its writer is not read",
    ),
    (
        "PLAYER pop_cap_frame",
        "a frame the game keeps, 0 at the start",
    ),
    (
        "PLAYER synced_*",
        "the ten counters the game keeps of a person's play, 0 at the start",
    ),
    (
        "a random nation",
        "the block prints a nation, and whether it prints the drawn one is not established",
    ),
    (
        "an open, closed or observing seat",
        "no lobby on disk has one; MAX_OBSERVERS is offered, an observer seat is not",
    ),
];

/// One seat: a person or the computer, its nation, team, handicap and the
/// name its row ends on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seat {
    pub human: bool,
    /// The nation: `rules.xml`'s `TRIBES` record, as `PLAYER tribe`.
    pub tribe: u8,
    pub team: i32,
    pub handicap: i32,
    pub name: String,
}

/// **The start's choices**: what the page holds, and all the block prints
/// that a person can set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choices {
    /// [`SETTINGS`]' bytes; `PLAYERS`' is derived and this copy unread.
    pub settings: [u8; 30],
    pub seed: u32,
    /// `GameInfo::flags`: `rules.xml`'s `gameinfo_flags`, a bit an entry.
    pub flags: u32,
    pub seats: Vec<Seat>,
}

/// **One event of the page**: each is one thing a person does on it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// A setting's entry: its place in [`SETTINGS`] and the entry's index.
    Set {
        setting: usize,
        value: u8,
    },
    /// The game's seed, typed.
    Seed(u32),
    /// One of `gameinfo_flags`' entries, on or off.
    Flag {
        bit: u8,
        on: bool,
    },
    /// A seat at the end: the computer, the first nation, its first leader.
    AddSeat,
    /// The last seat, taken away.
    RemoveSeat,
    Kind {
        seat: usize,
        human: bool,
    },
    /// A nation; a computer's seat takes the nation's first leader's name.
    Nation {
        seat: usize,
        tribe: u8,
    },
    /// One of the seat's nation's leaders, by its place in the nation's file.
    Leader {
        seat: usize,
        index: usize,
    },
    /// A name, typed.
    Name {
        seat: usize,
        name: String,
    },
    Team {
        seat: usize,
        team: i32,
    },
    /// The `handicaps` entry's index.
    Handicap {
        seat: usize,
        handicap: i32,
    },
}

/// One `rules.xml` category: its `id`, its `title` and its entries' names
/// (an entry with no `name` is named by its `DATA`, as `mapsizes`' are).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Category {
    pub id: String,
    pub title: String,
    pub names: Vec<String>,
}

/// One nation: its file's `<TRIBE name>` and its `<LEADER>`s in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Nation {
    pub name: String,
    pub leaders: Vec<String>,
}

/// **The install's lists the page offers**: every category, and the
/// nations in `rules.xml`'s `TRIBES` order (the index `PLAYER tribe` is).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Roster {
    pub categories: Vec<Category>,
    pub nations: Vec<Nation>,
}

impl Roster {
    /// Read from the install at `root`: `Data/rules.xml` and `tribes/`.
    pub fn read(root: &Path) -> Result<Roster, String> {
        let path = root.join("Data/rules.xml");
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let doc =
            roxmltree::Document::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut categories = Vec::new();
        for c in doc.descendants().filter(|n| n.has_tag_name("CATEGORIES")) {
            let names = c
                .children()
                .filter(|n| n.has_tag_name("CATEGORY"))
                .map(|n| match n.attribute("name") {
                    Some(name) => display(name),
                    None => n
                        .children()
                        .filter(|d| d.has_tag_name("DATA"))
                        .filter_map(|d| d.text())
                        .map(str::trim)
                        .collect::<Vec<_>>()
                        .join(" "),
                })
                .collect();
            categories.push(Category {
                id: c.attribute("id").unwrap_or_default().to_string(),
                title: c.attribute("title").unwrap_or_default().trim().to_string(),
                names,
            });
        }
        let tribes = doc
            .descendants()
            .find(|n| n.has_tag_name("TRIBES"))
            .ok_or("rules.xml has no TRIBES")?;
        let mut nations = Vec::new();
        for t in tribes.children().filter(|n| n.has_tag_name("TRIBE")) {
            let file = t
                .children()
                .find(|n| n.has_tag_name("FILE"))
                .and_then(|n| n.text())
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase();
            let path = root.join("tribes").join(&file);
            let nation = std::fs::read_to_string(&path).ok().and_then(|text| {
                let doc = roxmltree::Document::parse(&text).ok()?;
                let r = doc.root_element();
                Some(Nation {
                    name: r.attribute("name").unwrap_or_default().to_string(),
                    leaders: r
                        .descendants()
                        .filter(|n| n.has_tag_name("LEADER"))
                        .filter_map(|n| n.text())
                        .map(|s| s.trim().to_string())
                        .collect(),
                })
            });
            nations.push(nation.unwrap_or(Nation {
                name: file,
                leaders: Vec::new(),
            }));
        }
        Ok(Roster {
            categories,
            nations,
        })
    }

    /// The category of that `id`.
    pub fn category(&self, id: &str) -> Option<&Category> {
        self.categories.iter().find(|c| c.id == id)
    }

    /// The entries setting `setting` offers: its category's count, 0 when
    /// the page offers none.
    pub fn entries(&self, setting: usize) -> usize {
        if setting == PLAYERS || NOT_OFFERED.contains(&setting) {
            return 0;
        }
        CATEGORY
            .get(setting)
            .copied()
            .flatten()
            .and_then(|id| self.category(id))
            .map_or(0, |c| c.names.len())
    }

    /// `gameinfo_flags`' entries, one bit each, by position (the comment
    /// in the shipped file is not an entry; `docs/FORMATS.md`).
    pub fn flags(&self) -> &[String] {
        self.category("gameinfo_flags")
            .map_or(&[][..], |c| c.names.as_slice())
    }
}

/// A category entry's name as the page shows it: the `#ICONnn` prefix the
/// original draws an icon from is dropped.
fn display(name: &str) -> String {
    let s = name.trim();
    let s = s
        .strip_prefix("#ICON")
        .map(|r| r.trim_start_matches(|c: char| c.is_ascii_digit()))
        .unwrap_or(s);
    s.to_string()
}

impl Choices {
    /// No seat and every byte 0: the page's events build a lobby from here.
    pub fn blank() -> Choices {
        Choices {
            settings: [0; 30],
            seed: 0,
            flags: 0,
            seats: Vec::new(),
        }
    }

    /// **The harness lobby** (run9, run53: seed 12345, Great Lakes, Small,
    /// a person's Nubians against the computer's British): the page's
    /// opening, so that the client's start is the one every walk is proved
    /// on (`docs/START.md` §7.4). It is [`harness_events`] on
    /// [`blank`](Self::blank).
    pub fn harness(roster: &Roster) -> Result<Choices, String> {
        let mut c = Choices::blank();
        for e in harness_events() {
            c.apply(&e, roster)?;
        }
        Ok(c)
    }

    /// One event. A value the page cannot hold is refused, and the
    /// choices are left as they were.
    pub fn apply(&mut self, e: &Event, roster: &Roster) -> Result<(), String> {
        let seat_of = |seats: &[Seat], s: usize| -> Result<(), String> {
            (s < seats.len())
                .then_some(())
                .ok_or(format!("no seat {s}"))
        };
        match e {
            Event::Set { setting, value } => {
                let n = roster.entries(*setting);
                if n == 0 {
                    return Err(format!(
                        "the page offers no {}",
                        SETTINGS.get(*setting).unwrap_or(&"?")
                    ));
                }
                if usize::from(*value) >= n {
                    return Err(format!(
                        "{} has {n} entries, not {value}",
                        SETTINGS[*setting]
                    ));
                }
                self.settings[*setting] = *value;
            }
            Event::Seed(s) => self.seed = *s,
            Event::Flag { bit, on } => {
                if usize::from(*bit) >= roster.flags().len() {
                    return Err(format!(
                        "gameinfo_flags has {} entries, not bit {bit}",
                        roster.flags().len()
                    ));
                }
                if *on {
                    self.flags |= 1 << bit;
                } else {
                    self.flags &= !(1 << bit);
                }
            }
            Event::AddSeat => {
                if self.seats.len() >= MAX_SEATS {
                    return Err(format!("{MAX_SEATS} seats at most"));
                }
                let name = roster
                    .nations
                    .first()
                    .and_then(|n| n.leaders.first())
                    .cloned()
                    .unwrap_or_else(|| PERSON.to_string());
                self.seats.push(Seat {
                    human: false,
                    tribe: 0,
                    team: NO_TEAM,
                    handicap: 0,
                    name,
                });
            }
            Event::RemoveSeat => {
                self.seats.pop().ok_or("no seat to remove")?;
            }
            Event::Kind { seat, human } => {
                seat_of(&self.seats, *seat)?;
                self.seats[*seat].human = *human;
            }
            Event::Nation { seat, tribe } => {
                seat_of(&self.seats, *seat)?;
                let n = roster
                    .nations
                    .get(usize::from(*tribe))
                    .ok_or(format!("{} nations, not {tribe}", roster.nations.len()))?;
                let s = &mut self.seats[*seat];
                s.tribe = *tribe;
                if let (false, Some(l)) = (s.human, n.leaders.first()) {
                    s.name = l.clone();
                }
            }
            Event::Leader { seat, index } => {
                seat_of(&self.seats, *seat)?;
                let s = &mut self.seats[*seat];
                let l = roster
                    .nations
                    .get(usize::from(s.tribe))
                    .and_then(|n| n.leaders.get(*index))
                    .ok_or(format!("no leader {index} of nation {}", s.tribe))?;
                s.name = l.clone();
            }
            Event::Name { seat, name } => {
                seat_of(&self.seats, *seat)?;
                if !printable(name) {
                    return Err(format!("{name:?} is not a name the block can print"));
                }
                self.seats[*seat].name = name.clone();
            }
            Event::Team { seat, team } => {
                seat_of(&self.seats, *seat)?;
                if !(0..=NO_TEAM).contains(team) {
                    return Err(format!("team {team}: 0 to {NO_TEAM}, {NO_TEAM} none"));
                }
                self.seats[*seat].team = *team;
            }
            Event::Handicap { seat, handicap } => {
                seat_of(&self.seats, *seat)?;
                let n = roster.category("handicaps").map_or(0, |c| c.names.len()) as i32;
                if !(0..n).contains(handicap) {
                    return Err(format!("handicaps has {n} entries, not {handicap}"));
                }
                self.seats[*seat].handicap = *handicap;
            }
        }
        Ok(())
    }

    /// `PLAYERS` as the block prints it: the seats less one.
    pub fn players(&self) -> u8 {
        self.seats.len().saturating_sub(1) as u8
    }

    /// **The `GAME INFO` block** these choices print, in
    /// `GameInfo::log_data`'s order and the dump's indentation: the
    /// version, the thirty settings, the seed, the three checksum fields,
    /// the flags, and a `PLAYER` row a seat (`Player::log_data@006ee720`,
    /// which prints a seat only when its `flags & 1`).
    pub fn block(&self) -> String {
        let mut out = String::from("BEGIN GAME INFO\n BEGIN GAMEINFO\n");
        let mut line = |indent: usize, s: String| {
            out.push_str(&" ".repeat(indent));
            out.push_str(&s);
            out.push('\n');
        };
        line(2, format!("(Version: {VERSION_STRING})"));
        line(2, format!("(int) version {VERSION}"));
        for (i, k) in SETTINGS.iter().enumerate() {
            let v = if i == PLAYERS {
                self.players()
            } else {
                self.settings[i]
            };
            line(2, format!("{k} {v}"));
        }
        line(2, format!("(int)seed {}", self.seed));
        line(2, "checksum_deep 0".into());
        line(2, format!("checksum_window_size {CHECKSUM_WINDOW_SIZE}"));
        line(2, "checksum_failure_threshold 0".into());
        line(2, format!("flags {}", self.flags));
        for (who, s) in self.seats.iter().enumerate() {
            line(2, "BEGIN PLAYER".into());
            line(3, format!("flags {}", if s.human { 7 } else { 1 }));
            line(3, format!("tribe {}", s.tribe));
            line(3, format!("who {who}"));
            line(3, format!("team {}", s.team));
            line(3, format!("handicap {}", s.handicap));
            line(3, format!("play {who}"));
            line(3, "pauses 0".into());
            line(3, "diff 2".into());
            line(3, "pop_cap_frame 0".into());
            for k in SYNCED {
                line(3, format!("{k} 0"));
            }
            line(3, s.name.clone());
        }
        out
    }

    /// The `GAMEINFO` block's lines as [`block`](Self::block) writes them,
    /// indentation off: the text side of the counter, read by no parser.
    pub fn lines(&self) -> Vec<String> {
        self.block()
            .lines()
            .skip(2)
            .map(|l| l.trim_start_matches(' ').to_string())
            .collect()
    }

    /// The block, parsed as the dump's is.
    pub fn lobby(&self) -> Lobby {
        Lobby::parse(&self.block()).expect("the page's block parses")
    }
}

/// The ten counters `Player::log_data` prints after `pop_cap_frame`.
const SYNCED: [&str; 10] = [
    "synced_used_zoom_control",
    "synced_frames_zoomed_in",
    "synced_frames_zoomed_out",
    "synced_clicks",
    "synced_hotkeys",
    "synced_minimap_clicks",
    "synced_mainmap_clicks",
    "synced_cheated",
    "synced_control_groups_formed",
    "synced_control_groups_activated",
];

/// The fields a `PLAYER` row prints ahead of its name.
const ROW_KEYS: [&str; 9] = [
    "flags",
    "tribe",
    "who",
    "team",
    "handicap",
    "play",
    "pauses",
    "diff",
    "pop_cap_frame",
];

/// A name the block can print as its row's last line and read back: one
/// line, not empty, not led by a space, and not a key the row prints.
fn printable(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with(' ')
        && !name.ends_with(' ')
        && !name.contains("  ")
        && name
            .chars()
            .all(|c| c != '\n' && c != '\r' && !c.is_control())
        && !name.starts_with("BEGIN")
        && !name.starts_with("END")
        && !ROW_KEYS.contains(&name.split(' ').next().unwrap_or(""))
        && !name.starts_with("synced_")
}

/// The harness lobby's events, in the page's order: every setting, the
/// seed, then the two seats — the person's typed name is run53's own,
/// `Player 1`.
pub fn harness_events() -> Vec<Event> {
    let values: [u8; 30] = [
        1, 14, 2, 0, 0, 2, 1, 0, 2, 1, 1, 3, 1, 5, 0, 0, 0, 1, 7, 1, 0, 5, 4, 8, 1, 3, 2, 0, 0, 0,
    ];
    let mut e: Vec<Event> = (0..30)
        .filter(|&i| i != PLAYERS && !NOT_OFFERED.contains(&i))
        .map(|i| Event::Set {
            setting: i,
            value: values[i],
        })
        .collect();
    e.push(Event::Seed(12345));
    e.extend([
        Event::AddSeat,
        Event::Kind {
            seat: 0,
            human: true,
        },
        Event::Nation { seat: 0, tribe: 4 },
        Event::Name {
            seat: 0,
            name: "Player 1".into(),
        },
        Event::AddSeat,
        Event::Nation { seat: 1, tribe: 11 },
        // `tribes/british.xml`'s fifth leader, the name run53's computer
        // prints.
        Event::Leader { seat: 1, index: 4 },
    ]);
    e
}

/// One block's fields, borrowed.
pub type Fields<'a> = Vec<(&'a str, &'a str)>;

/// **A lobby as its block prints it**: the `GAMEINFO` fields and each
/// `PLAYER` row's, in file order, as [`crate::gamelog::Log`] parses them —
/// `Initial::game_info` and `Initial::players`. Two lobbies are one when
/// both lists are equal: `same_lobby`'s key, and the rows besides.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lobby {
    pub info: Vec<(String, String)>,
    pub players: Vec<Vec<(String, String)>>,
}

impl Lobby {
    /// From a text holding a `GAME INFO` block.
    pub fn parse(text: &str) -> Option<Lobby> {
        let log = crate::gamelog::Log::parse(text);
        let gi = log.root("GAME INFO")?.kid("GAMEINFO")?;
        let own = |f: Vec<(&str, &str)>| -> Vec<(String, String)> {
            f.into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        };
        Some(Lobby {
            info: own(gi.fields().to_vec()),
            players: gi
                .kids("PLAYER")
                .map(|p| own(p.fields().to_vec()))
                .collect(),
        })
    }

    /// The `GAMEINFO` field `key`.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.info
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// The fields as [`crate::diff::from_lobby`] takes them.
    pub fn borrowed(&self) -> (Fields<'_>, Vec<Fields<'_>>) {
        fn b(f: &[(String, String)]) -> Vec<(&str, &str)> {
            f.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect()
        }
        (b(&self.info), self.players.iter().map(|p| b(p)).collect())
    }

    /// Every field where `self` and `other` part, as `record field: ours
    /// theirs` (`—` where a side has none): `GAMEINFO` first, then
    /// `PLAYER n`.
    pub fn partings(&self, other: &Lobby) -> Vec<String> {
        let mut out = Vec::new();
        let mut list = |rec: &str, a: &[(String, String)], b: &[(String, String)]| {
            for i in 0..a.len().max(b.len()) {
                let (x, y) = (a.get(i), b.get(i));
                if x != y {
                    let side = |f: Option<&(String, String)>| {
                        f.map_or("—".to_string(), |(k, v)| format!("{k} {v}"))
                    };
                    out.push(format!("{rec}: ours {} theirs {}", side(x), side(y)));
                }
            }
        };
        list("GAMEINFO", &self.info, &other.info);
        for i in 0..self.players.len().max(other.players.len()) {
            let none = Vec::new();
            list(
                &format!("PLAYER {i}"),
                self.players.get(i).unwrap_or(&none),
                other.players.get(i).unwrap_or(&none),
            );
        }
        out
    }
}

/// **The page's events for a lobby**: what a person does on the page to
/// set `lobby`'s choices, from [`Choices::blank`] — each offered setting,
/// the seed, each flag, then each seat: added, its kind, nation, team,
/// handicap and name (a nation's leader by its place when the name is one,
/// else typed). A field the page cannot set emits nothing; the block then
/// prints its own value, and a lobby that holds another parts on it.
pub fn events_for(lobby: &Lobby, roster: &Roster) -> Vec<Event> {
    let mut e = Vec::new();
    for (i, k) in SETTINGS.iter().enumerate() {
        if i == PLAYERS || NOT_OFFERED.contains(&i) {
            continue;
        }
        if let Some(value) = lobby.get(k).and_then(|v| v.trim().parse().ok()) {
            e.push(Event::Set { setting: i, value });
        }
    }
    if let Some(s) = lobby.get("(int)seed").and_then(|v| v.trim().parse().ok()) {
        e.push(Event::Seed(s));
    }
    let flags: u32 = lobby
        .get("flags")
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(0);
    for bit in 0..32u8 {
        if flags & (1 << bit) != 0 {
            e.push(Event::Flag { bit, on: true });
        }
    }
    for (seat, row) in lobby.players.iter().enumerate() {
        let int = |key: &str| -> Option<i64> {
            row.iter()
                .find(|(k, _)| k == key)
                .and_then(|(_, v)| v.trim().parse().ok())
        };
        e.push(Event::AddSeat);
        let human = int("flags").is_some_and(|f| f & 4 != 0);
        e.push(Event::Kind { seat, human });
        let tribe = int("tribe").unwrap_or(0) as u8;
        e.push(Event::Nation { seat, tribe });
        if let Some(team) = int("team") {
            e.push(Event::Team {
                seat,
                team: team as i32,
            });
        }
        if let Some(handicap) = int("handicap") {
            e.push(Event::Handicap {
                seat,
                handicap: handicap as i32,
            });
        }
        // The name is the row's last line, which the parser splits at its
        // first space into a key and a value.
        if let Some((k, v)) = row.last().filter(|(k, _)| !ROW_KEYS.contains(&k.as_str())) {
            let name = if v.is_empty() {
                k.clone()
            } else {
                format!("{k} {v}")
            };
            let leaders = roster
                .nations
                .get(usize::from(tribe))
                .map_or(&[][..], |n| n.leaders.as_slice());
            match leaders.iter().position(|l| *l == name) {
                Some(index) if !human => e.push(Event::Leader { seat, index }),
                _ => e.push(Event::Name { seat, name }),
            }
        }
    }
    e
}

/// The kids of `GAME` that mark a start dump (`tools/start_lobbies.py`):
/// `GameLog::begin_game`'s records ahead of the first frame.
const START_KIDS: [&str; 4] = [
    "BEGIN WORLD",
    "BEGIN UNITDATA",
    "BEGIN BUILDDATA",
    "BEGIN FULL DUMP",
];

/// One lobby on disk: the `GAME INFO` block's text and every capture that
/// opens on it, by file name.
#[derive(Clone, Debug)]
pub struct OnDisk {
    pub block: String,
    /// The `GAMEINFO` block's lines, indentation off, as the scan read
    /// them: no parser between the file and these.
    pub lines: Vec<String>,
    pub captures: Vec<String>,
}

/// **The lobbies on disk with a `[Start Game]` dump**
/// (`tools/start_lobbies.py`'s scan, `docs/START.md` §5): every
/// `gamelog-*.txt` under `dir` whose `GAME` carries start records ahead of
/// its first frame, grouped by its `GAMEINFO` block's text, in the order of
/// each group's first capture. Each file is read only to that point.
///
/// Only the captures `ledger` puts in the corpus are read
/// ([`crate::corpus`]): a test pins this count, so a capture another
/// worktree filed and the tree's `docs/RUNS.md` does not head is skipped
/// and named, never counted (the pass of 2026-11-13).
pub fn lobbies_on_disk(dir: &Path, ledger: &crate::corpus::Ledger) -> std::io::Result<Vec<OnDisk>> {
    let names = crate::corpus::listed(dir, ledger, crate::corpus::is_gamelog)?.kept;
    let mut out: Vec<OnDisk> = Vec::new();
    let mut key_at: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for name in names {
        let Some((block, key)) = head(&dir.join(&name))? else {
            continue;
        };
        match key_at.get(&key) {
            Some(&i) => out[i].captures.push(name),
            None => {
                key_at.insert(key.clone(), out.len());
                out.push(OnDisk {
                    block,
                    lines: key.split('\n').map(str::to_string).collect(),
                    captures: vec![name],
                });
            }
        }
    }
    Ok(out)
}

/// A capture's `GAME INFO` block and its `GAMEINFO` key, when it is a
/// start dump: the text from `BEGIN GAME INFO` to `BEGIN GAME`, and the
/// `GAMEINFO` block's lines with their indentation off.
fn head(path: &Path) -> std::io::Result<Option<(String, String)>> {
    let mut r = std::io::BufReader::new(std::fs::File::open(path)?);
    let mut raw = Vec::new();
    let (mut block, mut key) = (String::new(), Vec::<String>::new());
    let (mut in_root, mut info_depth, mut in_game) = (false, None::<usize>, false);
    loop {
        raw.clear();
        if r.read_until(b'\n', &mut raw)? == 0 {
            return Ok(None);
        }
        let line: String = raw
            .iter()
            .map(|&b| b as char)
            .collect::<String>()
            .trim_end_matches(['\r', '\n'])
            .to_string();
        let body = line.trim_start_matches(' ');
        let indent = line.len() - body.len();
        if body.starts_with("BEGIN FRAME") {
            return Ok(None);
        }
        if in_game {
            if indent >= 1 && START_KIDS.contains(&body) {
                return Ok((!key.is_empty()).then(|| (block, key.join("\n"))));
            }
            continue;
        }
        if line == "BEGIN GAME" {
            in_game = true;
            in_root = false;
            continue;
        }
        if line == "BEGIN GAME INFO" {
            in_root = true;
        } else if indent == 0 {
            in_root = false;
        }
        if !in_root {
            continue;
        }
        block.push_str(&line);
        block.push('\n');
        match info_depth {
            Some(d) if indent > d => key.push(body.to_string()),
            Some(_) => info_depth = None,
            None => {}
        }
        if body == "BEGIN GAMEINFO" && key.is_empty() {
            info_depth = Some(indent);
        }
    }
}

/// **The lobby page's counter** (item 4184, `docs/LOBBY.md` §5): the
/// lobbies on disk with a `[Start Game]` dump whose `GAMEINFO` block the
/// page's events ([`events_for`] on [`Choices::blank`]) reproduce field for
/// field — `GAMEINFO`'s and every `PLAYER` row's — of
/// [`crate::diff::START_LOBBIES`]. The harness lobby is first, and its
/// start built from the page is the built one
/// (`tests::the_page_s_harness_lobby_builds_the_built_start`).
pub const LOBBY_PAGE: usize = 161;

#[cfg(test)]
mod tests {
    use super::*;

    fn roster() -> Option<Roster> {
        let Some(root) = crate::testenv::install_root() else {
            eprintln!("skipping: no install (set RON_INSTALL)");
            return None;
        };
        Some(Roster::read(Path::new(&root)).expect("the install's roster"))
    }

    /// **The counter**: every lobby on disk, set on a blank page by the
    /// events a person makes, printed and parsed as the dump's block is,
    /// against that block field for field. The harness lobby first.
    #[test]
    fn the_page_sets_every_lobby_on_disk() {
        let Some(roster) = roster() else { return };
        let Some(dir) = crate::testenv::gamelog_dir() else {
            eprintln!("skipping: no gamelog directory (set RON_GAMELOG_DIR)");
            return;
        };
        let lobbies = lobbies_on_disk(&dir, crate::corpus::Ledger::of_tree()).expect("the logs");
        assert_eq!(
            lobbies.len(),
            crate::diff::START_LOBBIES,
            "tools/start_lobbies.py's count"
        );
        let harness = lobbies
            .iter()
            .position(|l| l.captures.iter().any(|c| c == crate::script::LOBBY_DUMP))
            .expect("the harness lobby is on disk");
        let mut order: Vec<usize> = vec![harness];
        order.extend((0..lobbies.len()).filter(|&i| i != harness));
        let (mut set, mut parted, mut opening) = (0, Vec::new(), false);
        for (n, &i) in order.iter().enumerate() {
            let theirs = Lobby::parse(&lobbies[i].block).expect("the dump's block parses");
            let mut ours = Choices::blank();
            let refused: Vec<String> = events_for(&theirs, &roster)
                .iter()
                .filter_map(|e| ours.apply(e, &roster).err())
                .collect();
            let mut p = ours.lobby().partings(&theirs);
            let text = ours.lines();
            if text != lobbies[i].lines {
                let at = (0..text.len().max(lobbies[i].lines.len()))
                    .find(|&k| text.get(k) != lobbies[i].lines.get(k))
                    .unwrap();
                p.push(format!(
                    "text line {at}: ours {:?} theirs {:?}",
                    text.get(at),
                    lobbies[i].lines.get(at)
                ));
            }
            if n == 0 {
                let fields = theirs.info.len() + theirs.players.iter().map(Vec::len).sum::<usize>();
                println!(
                    "the parser reads {fields} fields of the block's {} lines (less {} `BEGIN PLAYER`); \
                     first fields {:?}",
                    lobbies[i].lines.len(),
                    theirs.players.len(),
                    &theirs.info[..3]
                );
            }
            if n == 0 {
                opening = Choices::harness(&roster).ok() == Some(ours.clone());
            }
            if p.is_empty() && refused.is_empty() {
                set += 1;
            } else {
                parted.push(format!(
                    "{}: refused {refused:?}; {}",
                    lobbies[i].captures[0],
                    p.join("; ")
                ));
            }
        }
        for p in &parted {
            println!("parted: {p}");
        }
        println!("the page sets {set} of {} lobbies", lobbies.len());
        assert!(opening, "the harness lobby's events are the page's opening");
        assert!(parted.is_empty() || !parted[0].contains(crate::script::LOBBY_DUMP));
        assert_eq!(set, LOBBY_PAGE, "LOBBY_PAGE; parted: {parted:#?}");
    }

    /// **The comparison can fail**: one event bent — the computer's nation
    /// on the harness lobby — and the block parts on that row's `tribe`
    /// and its name, and nothing else.
    #[test]
    fn a_bent_event_parts_the_lobby() {
        let Some(roster) = roster() else { return };
        let want = Choices::harness(&roster).unwrap().lobby();
        let mut c = Choices::blank();
        for e in harness_events() {
            let e = match e {
                Event::Nation { seat: 1, .. } => Event::Nation { seat: 1, tribe: 10 },
                Event::Leader { seat: 1, index } => Event::Leader {
                    seat: 1,
                    index: index % 2,
                },
                e => e,
            };
            c.apply(&e, &roster).unwrap();
        }
        let p = c.lobby().partings(&want);
        assert_eq!(p.len(), 2, "{p:#?}");
        assert!(
            p[0].starts_with("PLAYER 1: ours tribe 10 theirs tribe 11"),
            "{p:#?}"
        );
        // And a typed seed alone parts on the seed.
        let mut c = Choices::harness(&roster).unwrap();
        c.apply(&Event::Seed(12346), &roster).unwrap();
        assert_eq!(
            c.lobby().partings(&want),
            ["GAMEINFO: ours (int)seed 12346 theirs (int)seed 12345"]
        );
    }

    /// **The harness lobby's start, built from the page, is the built
    /// start**: the page's opening through [`crate::free::FreeRunner::from_choices`]
    /// against [`crate::free::FreeRunner::from_lobby`] on run53's own block
    /// — the start `START_WALKED` walks 24,000 of 24,000 frames from — the
    /// whole `Sim` compared by its `{:?}` print, and both stepped 40 frames
    /// (the first frame the walk once parted on, `docs/START.md` §7.2).
    /// Bent once: a typed seed one off builds another start.
    #[test]
    fn the_page_s_harness_lobby_builds_the_built_start() {
        let Some(roster) = roster() else { return };
        let Some(inst) = crate::testenv::install() else {
            return;
        };
        let Some(dump) = crate::testenv::dump(crate::script::LOBBY_DUMP) else {
            eprintln!(
                "skipping: no {} (set RON_GAMELOG_DIR)",
                crate::script::LOBBY_DUMP
            );
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let tuning = sim::Tuning::RON;
        let page = Choices::harness(&roster).unwrap();
        let (mut ours, notes) = crate::free::FreeRunner::from_choices(&loaded, &page, tuning);
        let mut theirs =
            crate::free::FreeRunner::from_lobby(&loaded, Path::new(&dump), tuning).unwrap();
        println!("the walk's notes: {notes:?}");
        let print = |r: &crate::free::FreeRunner| format!("{:?}", r.sim());
        assert!(
            print(&ours) == print(&theirs),
            "the page's start parts from run53's"
        );
        for _ in 0..40 {
            ours.step();
            theirs.step();
        }
        assert!(print(&ours) == print(&theirs), "frame 40 parts");
        let mut bent = page.clone();
        bent.apply(&Event::Seed(12346), &roster).unwrap();
        let (bent, _) = crate::free::FreeRunner::from_choices(&loaded, &bent, tuning);
        let fresh = crate::free::FreeRunner::from_lobby(&loaded, Path::new(&dump), tuning).unwrap();
        assert!(
            print(&bent) != print(&fresh),
            "a bent seed builds the same start"
        );
    }

    /// The page refuses what it cannot hold, and leaves the choices.
    #[test]
    fn the_page_refuses_what_it_cannot_hold() {
        let Some(roster) = roster() else { return };
        let mut c = Choices::harness(&roster).unwrap();
        let before = c.clone();
        let n = roster.entries(1);
        assert!(n > 14, "mapstyles has Great Lakes");
        for bad in [
            Event::Set {
                setting: 1,
                value: n as u8,
            },
            Event::Set {
                setting: PLAYERS,
                value: 1,
            },
            Event::Set {
                setting: 27,
                value: 0,
            },
            Event::Nation { seat: 2, tribe: 0 },
            Event::Nation {
                seat: 0,
                tribe: roster.nations.len() as u8,
            },
            Event::Name {
                seat: 0,
                name: "tribe 3".into(),
            },
            Event::Name {
                seat: 0,
                name: String::new(),
            },
            Event::Team { seat: 0, team: 9 },
            Event::Flag { bit: 31, on: true },
        ] {
            assert!(c.apply(&bad, &roster).is_err(), "{bad:?} was let through");
        }
        assert_eq!(c, before);
        for _ in 2..MAX_SEATS {
            c.apply(&Event::AddSeat, &roster).unwrap();
        }
        assert!(c.apply(&Event::AddSeat, &roster).is_err());
        assert_eq!(c.lobby().get("PLAYERS"), Some("7"));
    }

    /// The roster reads the lists the page shows: the thirty settings'
    /// categories that it offers, and 24 nations each with a leader.
    #[test]
    fn the_roster_reads_every_offered_list() {
        let Some(roster) = roster() else { return };
        for (i, name) in SETTINGS.iter().enumerate() {
            let offered = i != PLAYERS && !NOT_OFFERED.contains(&i);
            assert_eq!(roster.entries(i) > 0, offered, "{name}");
        }
        assert_eq!(roster.nations.len(), 24);
        assert!(roster.nations.iter().all(|n| !n.leaders.is_empty()));
        assert_eq!(
            roster.category("mapstyles").unwrap().names[14],
            "Great Lakes"
        );
        assert_eq!(roster.category("mapsizes").unwrap().names.len(), 7);
    }
}
