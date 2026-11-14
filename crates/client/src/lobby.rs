//! **The lobby, a plain page** (item 4184; `docs/LOBBY.md`; DECISIONS 128
//! §6). The client opens on it under `--lobby`: one row a choice — every
//! setting the `GAMEINFO` block prints, the seed, the flags, and each
//! seat's kind, nation, name, team and handicap — moved through with the
//! arrows, typed into for a seed or a name, and started with Enter on its
//! last row through the client's `--lobby` door
//! (`rondata::free::FreeRunner::from_choices`). It is not the original's
//! interface; it is drawn in the debug overlay's font.
//!
//! Every change the page makes is one `rondata::lobby::Event`, applied to
//! its `Choices` and nothing else: what the page holds is what its events
//! made.

use rondata::lobby::{Choices, Event, NOT_OFFERED, PLAYERS, Roster, SETTINGS};

/// One row of the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    Setting(usize),
    Seed,
    Flag(u8),
    Kind(usize),
    Nation(usize),
    Name(usize),
    Team(usize),
    Handicap(usize),
    AddSeat,
    RemoveSeat,
    Start,
}

/// A key the page reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Backspace,
    Char(char),
}

/// What a key did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Nothing,
    /// The event the key made, applied.
    Event(Event),
    /// The event refused, and why.
    Refused(Event, String),
    /// Enter on the last row: start the game from the page's choices.
    Start,
}

/// The page: its choices, the install's lists, the row the cursor is on,
/// and the text typed into a seed or name row since the cursor came to it.
pub struct Page {
    pub choices: Choices,
    pub roster: Roster,
    pub row: usize,
    draft: Option<String>,
    /// The last refusal, or the start's notes, shown under the rows.
    pub note: Option<String>,
}

impl Page {
    /// The page on the harness lobby, the client's own start.
    pub fn new(roster: Roster) -> Result<Page, String> {
        let choices = Choices::harness(&roster)?;
        Ok(Page::on(choices, roster))
    }

    /// **The page set to a capture's lobby by its own keys** (item 4215,
    /// `--page-of`): a blank page, and for each of the lobby's events
    /// (`rondata::lobby::events_for`) the keys a person presses to make it
    /// (`keys_for`), pressed one by one. Refused when a key is refused or
    /// the page does not end on the capture's `GAMEINFO` block, field for
    /// field.
    pub fn of_capture(path: &std::path::Path, roster: Roster) -> Result<Page, String> {
        use std::io::BufRead;
        let file = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut text = String::new();
        for line in std::io::BufReader::new(file).split(b'\n') {
            let line = line.map_err(|e| e.to_string())?;
            let line: String = line.iter().map(|&b| b as char).collect();
            let line = line.trim_end_matches('\r');
            if line == "BEGIN GAME" || line.trim_start().starts_with("BEGIN FRAME") {
                break;
            }
            text.push_str(line);
            text.push('\n');
        }
        let lobby = rondata::lobby::Lobby::parse(&text)
            .ok_or_else(|| format!("{}: no GAMEINFO block", path.display()))?;
        let mut page = Page::on(Choices::blank(), roster);
        let roster = page.roster.clone();
        for e in rondata::lobby::events_for(&lobby, &roster) {
            for k in page.keys_for(&e) {
                if let Outcome::Refused(e, why) = page.key(k) {
                    return Err(format!("{e:?} refused: {why}"));
                }
            }
        }
        let parted = page.choices.lobby().partings(&lobby);
        if !parted.is_empty() {
            return Err(format!("the page parts: {}", parted.join("; ")));
        }
        Ok(page)
    }

    /// The page holding `choices`.
    pub fn on(choices: Choices, roster: Roster) -> Page {
        Page {
            choices,
            roster,
            row: 0,
            draft: None,
            note: None,
        }
    }

    /// The rows, top to bottom.
    pub fn rows(&self) -> Vec<Row> {
        let mut r: Vec<Row> = (0..SETTINGS.len())
            .filter(|&i| i != PLAYERS && !NOT_OFFERED.contains(&i))
            .map(Row::Setting)
            .collect();
        r.push(Row::Seed);
        r.extend((0..self.roster.flags().len() as u8).map(Row::Flag));
        for s in 0..self.choices.seats.len() {
            r.extend([
                Row::Kind(s),
                Row::Nation(s),
                Row::Name(s),
                Row::Team(s),
                Row::Handicap(s),
            ]);
        }
        r.extend([Row::AddSeat, Row::RemoveSeat, Row::Start]);
        r
    }

    /// One key.
    pub fn key(&mut self, k: Key) -> Outcome {
        let rows = self.rows();
        let row = rows[self.row.min(rows.len() - 1)];
        match k {
            Key::Up | Key::Down => {
                let n = rows.len();
                self.row = if k == Key::Up {
                    (self.row + n - 1) % n
                } else {
                    (self.row + 1) % n
                };
                self.draft = None;
                return Outcome::Nothing;
            }
            Key::Enter => {
                return match row {
                    Row::AddSeat => self.event(Event::AddSeat),
                    Row::RemoveSeat => self.event(Event::RemoveSeat),
                    Row::Start => Outcome::Start,
                    _ => Outcome::Nothing,
                };
            }
            _ => {}
        }
        let step = |n: usize, v: usize| -> usize {
            if n == 0 {
                v
            } else if k == Key::Left {
                (v + n - 1) % n
            } else {
                (v + 1) % n
            }
        };
        let arrow = matches!(k, Key::Left | Key::Right);
        let c = &self.choices;
        let e = match row {
            Row::Setting(i) if arrow => Event::Set {
                setting: i,
                value: step(self.roster.entries(i), usize::from(c.settings[i])) as u8,
            },
            Row::Flag(b) if arrow => Event::Flag {
                bit: b,
                on: c.flags & (1 << b) == 0,
            },
            Row::Kind(s) if arrow => Event::Kind {
                seat: s,
                human: !c.seats[s].human,
            },
            Row::Nation(s) if arrow => Event::Nation {
                seat: s,
                tribe: step(self.roster.nations.len(), usize::from(c.seats[s].tribe)) as u8,
            },
            Row::Name(s) if arrow => {
                let leaders = self.leaders(s);
                let at = leaders.iter().position(|l| *l == c.seats[s].name);
                let index = match at {
                    Some(i) => step(leaders.len(), i),
                    None if k == Key::Left => leaders.len().saturating_sub(1),
                    None => 0,
                };
                self.draft = None;
                Event::Leader { seat: s, index }
            }
            Row::Team(s) if arrow => Event::Team {
                seat: s,
                team: step(
                    rondata::lobby::NO_TEAM as usize + 1,
                    c.seats[s].team as usize,
                ) as i32,
            },
            Row::Handicap(s) if arrow => Event::Handicap {
                seat: s,
                handicap: step(
                    self.roster
                        .category("handicaps")
                        .map_or(0, |c| c.names.len()),
                    c.seats[s].handicap as usize,
                ) as i32,
            },
            Row::Seed | Row::Name(_) => {
                let mut d = self.draft.take().unwrap_or_else(|| match row {
                    Row::Seed => c.seed.to_string(),
                    Row::Name(s) => c.seats[s].name.clone(),
                    _ => unreachable!(),
                });
                match (row, k) {
                    (_, Key::Backspace) => {
                        d.pop();
                    }
                    (Row::Seed, Key::Char(ch)) if ch.is_ascii_digit() => {
                        if d == "0" {
                            d.clear();
                        }
                        d.push(ch);
                    }
                    (Row::Name(_), Key::Char(ch)) => d.push(ch),
                    _ => {
                        self.draft = Some(d);
                        return Outcome::Nothing;
                    }
                }
                let e = match row {
                    Row::Seed => match d.parse::<u32>() {
                        Ok(v) => Some(Event::Seed(v)),
                        Err(_) if d.is_empty() => Some(Event::Seed(0)),
                        Err(_) => None,
                    },
                    Row::Name(s) => Some(Event::Name {
                        seat: s,
                        name: d.clone(),
                    }),
                    _ => None,
                };
                self.draft = Some(d);
                // A draft the block cannot print yet — an empty name, a
                // trailing space — waits for the next key, and says why.
                return match e {
                    Some(e) => match self.choices.apply(&e, &self.roster) {
                        Ok(()) => {
                            self.note = None;
                            Outcome::Event(e)
                        }
                        Err(why) => {
                            self.note = Some(why);
                            Outcome::Nothing
                        }
                    },
                    None => Outcome::Nothing,
                };
            }
            _ => return Outcome::Nothing,
        };
        self.event(e)
    }

    fn event(&mut self, e: Event) -> Outcome {
        match self.choices.apply(&e, &self.roster) {
            Ok(()) => {
                self.note = None;
                let n = self.rows().len();
                self.row = self.row.min(n - 1);
                Outcome::Event(e)
            }
            Err(why) => {
                self.note = Some(why.clone());
                Outcome::Refused(e, why)
            }
        }
    }

    fn leaders(&self, seat: usize) -> &[String] {
        self.roster
            .nations
            .get(usize::from(self.choices.seats[seat].tribe))
            .map_or(&[][..], |n| n.leaders.as_slice())
    }

    /// A row's text: its title and its value.
    fn text(&self, row: Row) -> String {
        let c = &self.choices;
        let entry = |id: &str, v: usize| -> String {
            self.roster
                .category(id)
                .and_then(|c| c.names.get(v))
                .cloned()
                .unwrap_or_else(|| v.to_string())
        };
        match row {
            Row::Setting(i) => {
                let id = rondata::lobby::CATEGORY[i].unwrap_or("");
                let title = self
                    .roster
                    .category(id)
                    .map(|c| c.title.trim_end_matches(':').to_string())
                    .filter(|t| !t.is_empty())
                    .unwrap_or_else(|| SETTINGS[i].replace('_', " "));
                format!("{title}: {}", entry(id, usize::from(c.settings[i])))
            }
            Row::Seed => format!("Seed: {}", self.draft_or(row, c.seed.to_string())),
            Row::Flag(b) => format!(
                "{}: {}",
                self.roster.flags()[usize::from(b)],
                if c.flags & (1 << b) != 0 { "on" } else { "off" }
            ),
            Row::Kind(s) => format!(
                "Seat {}: {}",
                s + 1,
                if c.seats[s].human {
                    "a person"
                } else {
                    "the computer"
                }
            ),
            Row::Nation(s) => format!(
                "  Nation: {}",
                self.roster
                    .nations
                    .get(usize::from(c.seats[s].tribe))
                    .map_or("?", |n| n.name.as_str())
            ),
            Row::Name(s) => format!("  Name: {}", self.draft_or(row, c.seats[s].name.clone())),
            Row::Team(s) => match c.seats[s].team {
                rondata::lobby::NO_TEAM => "  Team: none".to_string(),
                t => format!("  Team: {}", t + 1),
            },
            Row::Handicap(s) => format!(
                "  Handicap: {}",
                entry("handicaps", c.seats[s].handicap as usize)
            ),
            Row::AddSeat => "Add a seat".into(),
            Row::RemoveSeat => "Remove the last seat".into(),
            Row::Start => "Start".into(),
        }
    }

    fn draft_or(&self, row: Row, value: String) -> String {
        if self.rows().get(self.row) == Some(&row) {
            self.draft.clone().unwrap_or(value)
        } else {
            value
        }
    }

    /// The page's lines: a title, at most `room` rows around the cursor,
    /// the keys, and the note.
    pub fn lines(&self, room: usize) -> Vec<String> {
        let rows = self.rows();
        let room = room.max(1).min(rows.len());
        let first = self.row.saturating_sub(room / 2).min(rows.len() - room);
        let mut out = vec!["ATTRITION - THE LOBBY".to_string(), String::new()];
        for (i, r) in rows.iter().enumerate().skip(first).take(room) {
            let mark = if i == self.row { "> " } else { "  " };
            out.push(format!("{mark}{}", self.text(*r)));
        }
        out.push(String::new());
        out.push("UP/DOWN: A ROW. LEFT/RIGHT: ITS VALUE. TYPE: A SEED OR A NAME.".into());
        out.push("ENTER: ADD OR REMOVE A SEAT, OR START.".into());
        if let Some(n) = &self.note {
            out.push(n.clone());
        }
        out
    }

    /// **The keys a person presses to make `e`** from where the page
    /// stands: down to its row, then the arrows, the typing or Enter that
    /// row takes. The door that replays a lobby as key presses: the tests'
    /// and `--page-of`'s ([`Page::of_capture`], item 4215).
    pub fn keys_for(&self, e: &Event) -> Vec<Key> {
        let c = &self.choices;
        let (row, mut keys) = match e {
            Event::Set { setting, value } => {
                let n = self.roster.entries(*setting);
                let d = (usize::from(*value) + n - usize::from(c.settings[*setting])) % n.max(1);
                (Row::Setting(*setting), vec![Key::Right; d])
            }
            Event::Seed(v) => {
                let now = c.seed.to_string();
                let mut k = vec![Key::Backspace; now.len()];
                k.extend(v.to_string().chars().map(Key::Char));
                (Row::Seed, k)
            }
            Event::Flag { bit, on } => {
                let is = c.flags & (1 << bit) != 0;
                (
                    Row::Flag(*bit),
                    if is == *on { vec![] } else { vec![Key::Right] },
                )
            }
            Event::AddSeat => (Row::AddSeat, vec![Key::Enter]),
            Event::RemoveSeat => (Row::RemoveSeat, vec![Key::Enter]),
            Event::Kind { seat, human } => (
                Row::Kind(*seat),
                if c.seats[*seat].human == *human {
                    vec![]
                } else {
                    vec![Key::Right]
                },
            ),
            Event::Nation { seat, tribe } => {
                let n = self.roster.nations.len();
                let d = (usize::from(*tribe) + n - usize::from(c.seats[*seat].tribe)) % n;
                (Row::Nation(*seat), vec![Key::Right; d])
            }
            Event::Leader { seat, index } => {
                let leaders = self.leaders(*seat);
                let n = leaders.len();
                let d = match leaders.iter().position(|l| *l == c.seats[*seat].name) {
                    Some(at) => (index + n - at) % n,
                    None => index + 1,
                };
                (Row::Name(*seat), vec![Key::Right; d])
            }
            Event::Name { seat, name } => {
                let mut k = vec![Key::Backspace; c.seats[*seat].name.chars().count()];
                k.extend(name.chars().map(Key::Char));
                (Row::Name(*seat), k)
            }
            Event::Team { seat, team } => {
                let n = rondata::lobby::NO_TEAM as usize + 1;
                let d = (*team as usize + n - c.seats[*seat].team as usize) % n;
                (Row::Team(*seat), vec![Key::Right; d])
            }
            Event::Handicap { seat, handicap } => {
                let n = self
                    .roster
                    .category("handicaps")
                    .map_or(1, |c| c.names.len());
                let d = (*handicap as usize + n - c.seats[*seat].handicap as usize) % n;
                (Row::Handicap(*seat), vec![Key::Right; d])
            }
        };
        let to = self.rows().iter().position(|r| *r == row).expect("the row");
        let mut nav: Vec<Key> = if to >= self.row {
            vec![Key::Down; to - self.row]
        } else {
            vec![Key::Up; self.row - to]
        };
        // A typed row starts its draft afresh: leave and come back.
        if matches!(row, Row::Seed | Row::Name(_)) && nav.is_empty() {
            nav = vec![Key::Down, Key::Up];
        }
        nav.append(&mut keys);
        nav
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rondata::lobby::{Lobby, events_for, lobbies_on_disk};
    use std::path::PathBuf;

    fn install() -> Option<PathBuf> {
        let root = PathBuf::from(std::env::var_os("RON_INSTALL")?);
        root.join("Data/rules.xml").is_file().then_some(root)
    }

    /// **The page's own keys set every lobby on disk** (item 4184): for
    /// each lobby, a blank page is driven by the key presses a person would
    /// make — `keys_for` each of the lobby's events, pressed one by one —
    /// and only the events the page itself emits are kept. Its choices must
    /// then print the dump's `GAMEINFO` block, line for line and field for
    /// field, and the events it emitted must be the lobby's.
    #[test]
    fn the_page_s_keys_set_every_lobby_on_disk() {
        let Some(root) = install() else {
            eprintln!("skipping: no install (set RON_INSTALL)");
            return;
        };
        let dir = crate::world::gamelog_dir();
        // The tree's lobbies, not the machine's (`rondata::corpus`).
        let Ok(lobbies) = lobbies_on_disk(&dir, rondata::corpus::Ledger::of_tree()) else {
            eprintln!("skipping: no gamelog directory (set RON_GAMELOG_DIR)");
            return;
        };
        if lobbies.is_empty() {
            eprintln!("skipping: no lobbies under {}", dir.display());
            return;
        }
        let roster = Roster::read(&root).unwrap();
        let mut set = 0;
        let mut parted = Vec::new();
        for l in &lobbies {
            let theirs = Lobby::parse(&l.block).unwrap();
            let mut page = Page::on(Choices::blank(), roster.clone());
            let mut emitted = Vec::new();
            for e in events_for(&theirs, &roster) {
                for k in page.keys_for(&e) {
                    match page.key(k) {
                        Outcome::Event(e) => emitted.push(e),
                        Outcome::Refused(e, why) => {
                            panic!("{}: {e:?} refused: {why}", l.captures[0])
                        }
                        _ => {}
                    }
                }
            }
            let ours = page.choices.lobby();
            if ours == theirs && page.choices.lines() == l.lines {
                set += 1;
            } else {
                parted.push((l.captures[0].clone(), ours.partings(&theirs)));
            }
            // What the page holds is its emitted events' doing alone.
            let mut replay = Choices::blank();
            for e in &emitted {
                replay.apply(e, &roster).unwrap();
            }
            assert_eq!(replay, page.choices, "{}", l.captures[0]);
        }
        println!("the page's keys set {set} of {} lobbies", lobbies.len());
        assert!(parted.is_empty(), "{parted:#?}");
        assert_eq!(set, rondata::lobby::LOBBY_PAGE);
    }

    /// **East Indies from the page, through the client** (item 4215): the
    /// first East Indies lobby on disk set by the page's own keys, opened
    /// as Start opens it (`Game::Page`), against the lobby's built start
    /// (`FreeRunner::from_lobby` on its capture): the scene on frame 0 and
    /// after `LOBBY_WALK` frames walked off-screen, and the ground the
    /// client draws — the built start's own tileset and `World.seed`, the
    /// capture's `WORLD seed`, not the pair's Great Lakes.
    #[test]
    fn east_indies_from_the_page_opens_its_own_start() {
        let Some(root) = install() else {
            eprintln!("skipping: no install (set RON_INSTALL)");
            return;
        };
        let dir = crate::world::gamelog_dir();
        let Ok(lobbies) = lobbies_on_disk(&dir, rondata::corpus::Ledger::of_tree()) else {
            eprintln!("skipping: no gamelog directory (set RON_GAMELOG_DIR)");
            return;
        };
        let Some((l, theirs)) = lobbies
            .iter()
            .filter_map(|l| Some((l, Lobby::parse(&l.block)?)))
            .find(|(_, b)| b.get("MAP_STYLE") == Some("18"))
        else {
            eprintln!("skipping: no East Indies lobby under {}", dir.display());
            return;
        };
        let roster = Roster::read(&root).unwrap();
        let page = Page::of_capture(&dir.join(&l.captures[0]), roster).unwrap();
        assert_eq!(page.choices.lobby(), theirs, "{}", l.captures[0]);
        let mut ours = crate::world::World::open_game(
            &root,
            &dir,
            crate::world::Game::Page(Box::new(page.choices.clone())),
        )
        .unwrap();
        let loaded = rondata::load::load(&rondata::Install::new(&root)).unwrap();
        let capture = dir.join(&l.captures[0]);
        let mut built =
            rondata::free::FreeRunner::from_lobby(&loaded, &capture, sim::Tuning::RON).unwrap();
        let seed: u32 = rondata::capture::indexed::IndexedCapture::open(&capture)
            .unwrap()
            .with_replay_initial(|init| {
                init.world
                    .iter()
                    .find(|(k, _)| *k == "seed")
                    .and_then(|(_, v)| v.trim().parse().ok())
            })
            .unwrap()
            .expect("the start's WORLD seed");
        let ground = ours.ground_art().map(|(t, s)| (t.to_string(), s));
        println!(
            "{}: the client's ground {ground:?}, the dump's WORLD seed {seed}",
            l.captures[0]
        );
        assert_eq!(
            ground.as_ref().map(|g| (g.0.as_str(), g.1)),
            built.ground(),
            "the client draws the built start's ground"
        );
        assert_eq!(ground.map(|g| g.1), Some(seed), "World.seed");
        assert!(ours.runner.scene(None) == built.scene(None), "frame 0");
        let walk = rondata::lobby::LOBBY_WALK as i64;
        ours.run_to(walk);
        while built.frame() < walk {
            built.step();
        }
        assert!(ours.runner.scene(None) == built.scene(None), "frame {walk}");
        assert!(!built.scene(None).units.is_empty(), "a scene with units");
    }

    /// **Start on the page opens the minted lobby's game**: the page's
    /// opening through `Game::Page` (the window's Start) against
    /// `Game::Lobby` (the headless `--lobby`, run53's block), the scene
    /// compared on frame 0 and after 40 frames.
    #[test]
    fn start_on_the_page_opens_the_minted_lobby_s_game() {
        let Some(root) = install() else {
            eprintln!("skipping: no install (set RON_INSTALL)");
            return;
        };
        let logs = crate::world::gamelog_dir();
        if !logs.join(rondata::script::LOBBY_DUMP).is_file() {
            eprintln!("skipping: no {}", rondata::script::LOBBY_DUMP);
            return;
        }
        let mut page = Page::new(Roster::read(&root).unwrap()).unwrap();
        page.key(Key::Up);
        assert_eq!(page.key(Key::Enter), Outcome::Start);
        let open = |g| crate::world::World::open_game(&root, &logs, g).unwrap();
        let mut ours = open(crate::world::Game::Page(Box::new(page.choices.clone())));
        let mut theirs = open(crate::world::Game::Lobby);
        // Item 4215: a built start draws its own ground; on the harness
        // lobby that is the ground the client drew before from the pair's
        // start dump (the same map: Great Lakes, seed 12345).
        let start = logs.join(crate::world::START_DUMP);
        let pair = (
            crate::world::lobby_tileset(&root, &start),
            rondata::mapgen::start_head(&start)
                .ok()
                .flatten()
                .map(|h| rondata::mapgen::start_grid(&h).seed as u32),
        );
        let built = theirs.ground_art().map(|(t, s)| (t.to_string(), s));
        println!("the harness lobby's ground {built:?}, the pair's {pair:?}");
        assert_eq!(built.as_ref().map(|g| g.0.clone()), pair.0);
        assert_eq!(built.map(|g| g.1), pair.1);
        assert_eq!(ours.ground_art(), theirs.ground_art());
        assert!(
            ours.runner.scene(None) == theirs.runner.scene(None),
            "frame 0"
        );
        ours.run_to(40);
        theirs.run_to(40);
        assert!(
            ours.runner.scene(None) == theirs.runner.scene(None),
            "frame 40"
        );
        assert!(
            !ours.runner.scene(None).units.is_empty(),
            "a scene with units"
        );
    }

    /// The page opens on the harness lobby and starts on its last row.
    #[test]
    fn the_page_opens_on_the_harness_lobby_and_starts_from_its_last_row() {
        let Some(root) = install() else {
            eprintln!("skipping: no install (set RON_INSTALL)");
            return;
        };
        let roster = Roster::read(&root).unwrap();
        let mut page = Page::new(roster.clone()).unwrap();
        assert_eq!(page.choices, Choices::harness(&roster).unwrap());
        assert_eq!(page.key(Key::Up), Outcome::Nothing);
        assert_eq!(page.rows()[page.row], Row::Start);
        assert_eq!(page.key(Key::Enter), Outcome::Start);
        // The map style's row moves Great Lakes on by one, and back.
        page.row = 1;
        let Outcome::Event(Event::Set { setting: 1, value }) = page.key(Key::Right) else {
            panic!("the style's row")
        };
        assert_eq!(value, 15);
        page.key(Key::Left);
        assert_eq!(page.choices, Choices::harness(&roster).unwrap());
        let lines = page.lines(40);
        assert!(
            lines.iter().any(|l| l == "> Map Style: Great Lakes"),
            "{lines:#?}"
        );
        // Every character the page prints has a glyph.
        let all: Vec<String> = (0..page.rows().len())
            .flat_map(|r| {
                page.row = r;
                page.lines(80)
            })
            .collect();
        for ch in all.iter().flat_map(|l| l.chars()).filter(|c| *c != ' ') {
            assert!(crate::overlay::has_glyph(ch), "no glyph for {ch:?}");
        }
    }
}
