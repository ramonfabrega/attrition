//! Decodes `CommandPackage.data` — the command payload inside a recorded
//! game's package stream, and the same bytes a lockstep peer receives.
//!
//! The format is `docs/COMMANDS.md`: a sequence of commands, each opened by
//! one type byte (0x00–0x51) that alone determines the size; four commands
//! carry their own count field. The wire structs are the PDB's `*Command`
//! layouts, packed from offset +0x1, copied raw by
//! `CommandPackage::add_command` and walked back by `process_all`/`process`
//! (0x94c500/0x94a700).
//!
//! Single-player payloads are plain. Multiplayer payloads carry the network
//! obfuscation (`docs/COMMANDS.md` §5): each u16 XORed with
//! `(seed >> 8) & 0xffff` (a trailing odd byte plain), and a
//! `Random::get(0, 2)` padding gap after every command, drawn from the
//! game's `Random` seeded with `GameInfo.seed` — [`decode_mp`] reverses
//! both.

use crate::Error;

/// One decoded command. Field names are the PDB's; semantics live with the
/// mechanic each `process_*` hands its fields to (`docs/COMMANDS.md` §4).
///
/// Camera floats (`Hotkey`, `Spline`) are carried as raw bit patterns — they
/// are renderer-only state and never enter the simulation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// 0x00 — sets the package's selection for the unit-targeted commands
    /// that follow. Empty `objects` is the wire's `num = 0`: "the same
    /// selection as this player's previous group command".
    Group {
        who: i8,
        objects: Vec<i16>,
    },
    /// 0x01 — processed as a no-op.
    Begin,
    Stance {
        stance: i32,
    },
    Form {
        form: i32,
        rotate: i32,
        queued: i32,
    },
    Attack {
        ox: i32,
        whom: i32,
        ignore: i32,
        queued: i32,
    },
    SiegeAttack {
        ox: i32,
        whom: i32,
        queued: i32,
    },
    SwarmAround {
        ox: i32,
        whom: i32,
        queued: i32,
        orders: i32,
    },
    MoveTo {
        to_x: i32,
        to_y: i32,
        set_angle: i32,
        angle: i32,
        orders: i8,
        queued: i8,
        form: i8,
        width: i8,
        disembark: i8,
    },
    MoveNear {
        to_x: i32,
        to_y: i32,
        tolerance: i32,
        set_angle: i32,
        angle: i32,
        orders: i8,
        queued: i8,
        form: i8,
        width: i8,
        disembark: i8,
    },
    AttackGround {
        to_x: i32,
        to_y: i32,
        queued: i8,
    },
    Patrol {
        to_x: i32,
        to_y: i32,
        queued: i8,
    },
    LaunchPatrol {
        to_x: i32,
        to_y: i32,
        queued: i32,
        shift: i32,
        ctrl: i32,
        alt: i32,
    },
    Halt,
    Transport,
    SetTransport {
        flag: i32,
    },
    BoardShip {
        ox: i32,
        queued: i32,
    },
    Repair {
        ox: i32,
        whom: i32,
        queued: i32,
    },
    Trade {
        ox: i32,
        whom: i32,
        oxx: i32,
        whose: i32,
        queued: i32,
    },
    CityGather {
        t: i32,
        queued: i32,
    },
    Gather {
        ox: i32,
        queued: i32,
    },
    Garrison {
        ox: i32,
        whom: i32,
        queued: i32,
    },
    Disband {
        all: i32,
    },
    GatherPoint {
        x: i32,
        y: i32,
        action: i32,
        add_to_end: i32,
    },
    Spell {
        ox: i32,
        whom: i32,
        spell: i32,
        x: i32,
        y: i32,
    },
    QueueUp {
        unit: i32,
        num: i32,
    },
    Build {
        x: i32,
        y: i32,
        x2: i32,
        y2: i32,
        build: i32,
        queued: i32,
    },
    EjectAll {
        back_to_work: i32,
        who: i32,
        eject_o: i32,
        eject_who: i32,
    },
    Alarm,
    Flight {
        ox: i32,
        whom: i32,
        shift: i32,
        ctrl: i32,
        alt: i32,
        orders: i32,
    },
    StopSpell,
    Follow {
        ox: i32,
        whom: i32,
        queued: i32,
    },
    Guard {
        ox: i32,
        whom: i32,
        queued: i32,
    },
    Unitmask {
        mask: i32,
        set: i32,
    },
    Buildmask {
        mask: i32,
        set: i32,
    },
    /// 0x22 — `x`/`y` are camera floats, raw bits.
    Hotkey {
        group: i32,
        clear: i32,
        valid: i32,
        x: u32,
        y: u32,
        zoom: i32,
    },
    Recall,
    Scramble,
    Treaty {
        who: i32,
        whom: i32,
        treaty: i32,
    },
    Declare {
        who: i32,
        whom: i32,
        treaty: i32,
    },
    ClearTributes {
        who: i32,
        whom: i32,
    },
    ClearAll {
        who: i32,
        whom: i32,
    },
    Accept {
        who: i32,
        whom: i32,
    },
    Reject {
        who: i32,
        whom: i32,
    },
    Tribute {
        who: i32,
        whom: i32,
        good: i32,
        amount: i32,
    },
    DemandTribute {
        who: i32,
        whom: i32,
        good: i32,
        amount: i32,
    },
    ProposeAttack {
        who: i32,
        whom: i32,
        whose: i32,
        onoff: i32,
    },
    Buy {
        who: i32,
        good: i32,
        flags: i32,
    },
    Sell {
        who: i32,
        good: i32,
        flags: i32,
    },
    Unqueue {
        who: i32,
        o: i32,
        unit: i32,
        uid: i16,
    },
    ComeOut {
        who: i32,
        o: i32,
        uid: i16,
    },
    Ping {
        x: i32,
        y: i32,
    },
    /// 0x33 — drawn annotations; vertex floats as raw bits.
    Spline {
        kind: u8,
        flags: u8,
        cmd: u8,
        verts: Vec<(u32, u32)>,
    },
    SpeedSet {
        speed: i32,
    },
    SpeedUp,
    SpeedDown,
    MpLog,
    CheckRandom {
        seed: u32,
    },
    /// 0x39 — the sixteen per-subsystem checksums, `CheckSumsCommand`'s
    /// field order (units, builds, walls, ammo, deaths, groups, guys,
    /// leaders, cities, items, goods, world, rules, scenario_data,
    /// script_run_time, all).
    CheckSums {
        sums: [u32; 16],
    },
    NextCheckSum {
        kind: u8,
        checksum: u32,
    },
    CheatViewAll {
        who: i32,
    },
    CheatGiveTechs {
        who: i32,
    },
    CheatZeroTechs {
        who: i32,
    },
    CheatAiSpeedIncrease,
    CheatAiSpeedNormal,
    CheatAiToggle,
    CheatIncreaseBuckets {
        who: i32,
    },
    CheatZeroBuckets {
        who: i32,
    },
    CheatInitUnit {
        who: i32,
        t: i32,
        x: i32,
        y: i32,
    },
    /// 0x44 — `bits` is the recipient mask, −1 = everyone; the wire carries
    /// a null terminator `len` excludes.
    Chat {
        bits: i32,
        taunt: i32,
        taunt_num: i32,
        text: String,
    },
    ChatSet {
        status: [u8; 8],
    },
    Resign {
        play: i32,
    },
    Quit {
        play: i32,
        replay: u8,
        system_quit: u8,
    },
    /// 0x48 — the recorded view; acted on only during playback.
    Camera {
        zoom: u8,
        x: i32,
        y: i32,
    },
    /// 0x49 — the auto-manage settings; the trailing `BitMask<8>` struct
    /// rides raw (bits, size, flags, ptr[4]).
    LeaderOptions {
        who: i32,
        peasants: i32,
        peasants_wait: i32,
        buildings: i32,
        mask: [u8; 16],
    },
    TurnData {
        ping_time: u16,
        frame_average: u16,
        wait_time: u16,
        game_lag: u16,
        forced_loads: u16,
    },
    /// 0x4b — the name field is a fixed 22 wchars; decoded to the NUL.
    RenameCity {
        who: i32,
        o: i32,
        name: String,
    },
    Pause {
        state: u8,
    },
    CannonTime {
        state: u8,
    },
    /// 0x4e — fixed 256 wchars. Dispatched but unsendable: its 0x209 wire
    /// size exceeds the 512-byte package cap, so nothing ever emits it
    /// (`docs/COMMANDS.md` §4).
    ConsoleCmd {
        mouse_x: i32,
        mouse_y: i32,
        cmd: String,
    },
    /// 0x4f — the eight `accum_*` input-telemetry counters.
    PlayerSpeed {
        accum: [u8; 8],
    },
    /// 0x50 — injected by the net layer, exempt from the MP XOR.
    UngracefulPlayerDrop {
        play: u8,
        state: u8,
    },
    Marwan {
        start: u8,
    },
}

impl Command {
    /// The engine-side name, `CommandPackage::process_<name>`.
    pub fn name(&self) -> &'static str {
        match self {
            Command::Group { .. } => "group",
            Command::Begin => "begin",
            Command::Stance { .. } => "stance",
            Command::Form { .. } => "form",
            Command::Attack { .. } => "attack",
            Command::SiegeAttack { .. } => "siege_attack",
            Command::SwarmAround { .. } => "swarm_around",
            Command::MoveTo { .. } => "move_to",
            Command::MoveNear { .. } => "move_near",
            Command::AttackGround { .. } => "attack_ground",
            Command::Patrol { .. } => "patrol",
            Command::LaunchPatrol { .. } => "launch_patrol",
            Command::Halt => "halt",
            Command::Transport => "transport",
            Command::SetTransport { .. } => "set_transport",
            Command::BoardShip { .. } => "board_ship",
            Command::Repair { .. } => "repair",
            Command::Trade { .. } => "trade",
            Command::CityGather { .. } => "city_gather",
            Command::Gather { .. } => "gather",
            Command::Garrison { .. } => "garrison",
            Command::Disband { .. } => "disband",
            Command::GatherPoint { .. } => "gather_point",
            Command::Spell { .. } => "spell",
            Command::QueueUp { .. } => "queue_up",
            Command::Build { .. } => "build",
            Command::EjectAll { .. } => "eject_all",
            Command::Alarm => "alarm",
            Command::Flight { .. } => "flight",
            Command::StopSpell => "stop_spell",
            Command::Follow { .. } => "follow",
            Command::Guard { .. } => "guard",
            Command::Unitmask { .. } => "unitmask",
            Command::Buildmask { .. } => "buildmask",
            Command::Hotkey { .. } => "hotkey",
            Command::Recall => "recall",
            Command::Scramble => "scramble",
            Command::Treaty { .. } => "treaty",
            Command::Declare { .. } => "declare",
            Command::ClearTributes { .. } => "clear_tributes",
            Command::ClearAll { .. } => "clear_all",
            Command::Accept { .. } => "accept",
            Command::Reject { .. } => "reject",
            Command::Tribute { .. } => "tribute",
            Command::DemandTribute { .. } => "demand_tribute",
            Command::ProposeAttack { .. } => "propose_attack",
            Command::Buy { .. } => "buy",
            Command::Sell { .. } => "sell",
            Command::Unqueue { .. } => "unqueue",
            Command::ComeOut { .. } => "come_out",
            Command::Ping { .. } => "ping",
            Command::Spline { .. } => "spline",
            Command::SpeedSet { .. } => "speed_set",
            Command::SpeedUp => "speed_up",
            Command::SpeedDown => "speed_down",
            Command::MpLog => "mp_log",
            Command::CheckRandom { .. } => "check_random",
            Command::CheckSums { .. } => "check_sums",
            Command::NextCheckSum { .. } => "next_check_sum",
            Command::CheatViewAll { .. } => "cheat_view_all",
            Command::CheatGiveTechs { .. } => "cheat_give_techs",
            Command::CheatZeroTechs { .. } => "cheat_zero_techs",
            Command::CheatAiSpeedIncrease => "cheat_ai_speed_increase",
            Command::CheatAiSpeedNormal => "cheat_ai_speed_normal",
            Command::CheatAiToggle => "cheat_ai_toggle",
            Command::CheatIncreaseBuckets { .. } => "cheat_increase_buckets",
            Command::CheatZeroBuckets { .. } => "cheat_zero_buckets",
            Command::CheatInitUnit { .. } => "cheat_init_unit",
            Command::Chat { .. } => "chat",
            Command::ChatSet { .. } => "chat_set",
            Command::Resign { .. } => "resign",
            Command::Quit { .. } => "quit",
            Command::Camera { .. } => "camera",
            Command::LeaderOptions { .. } => "leader_options",
            Command::TurnData { .. } => "turn_data",
            Command::RenameCity { .. } => "rename_city",
            Command::Pause { .. } => "pause",
            Command::CannonTime { .. } => "cannon_time",
            Command::ConsoleCmd { .. } => "console_cmd",
            Command::PlayerSpeed { .. } => "player_speed",
            Command::UngracefulPlayerDrop { .. } => "ungraceful_player_drop",
            Command::Marwan { .. } => "marwan",
        }
    }
}

/// A cursor over one payload. Errors carry the byte offset into the payload.
struct Cur<'a> {
    data: &'a [u8],
    off: usize,
}

impl<'a> Cur<'a> {
    fn bad(&self, what: impl Into<String>) -> Error {
        Error::Format {
            path: "command payload".into(),
            at: self.off,
            what: what.into(),
        }
    }

    fn raw(&mut self, n: usize, what: &str) -> Result<&'a [u8], Error> {
        let end = self.off.checked_add(n).filter(|&e| e <= self.data.len());
        let Some(end) = end else {
            return Err(self.bad(format!("{what}: {n} bytes past end of payload")));
        };
        let s = &self.data[self.off..end];
        self.off = end;
        Ok(s)
    }

    fn u8(&mut self, what: &str) -> Result<u8, Error> {
        Ok(self.raw(1, what)?[0])
    }

    fn i8(&mut self, what: &str) -> Result<i8, Error> {
        Ok(self.u8(what)? as i8)
    }

    fn u16(&mut self, what: &str) -> Result<u16, Error> {
        let b = self.raw(2, what)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    fn i16(&mut self, what: &str) -> Result<i16, Error> {
        Ok(self.u16(what)? as i16)
    }

    fn u32(&mut self, what: &str) -> Result<u32, Error> {
        let b = self.raw(4, what)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn i32(&mut self, what: &str) -> Result<i32, Error> {
        Ok(self.u32(what)? as i32)
    }

    /// A fixed field of `n` UTF-16 code units, decoded up to the first NUL.
    fn wchars(&mut self, n: usize, what: &str) -> Result<String, Error> {
        let b = self.raw(2 * n, what)?;
        let units: Vec<u16> = b
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .take_while(|&u| u != 0)
            .collect();
        String::from_utf16(&units).map_err(|_| self.bad(format!("{what}: not UTF-16")))
    }
}

/// Decodes a plain (single-player) payload: the whole `data` slice, walked
/// front to back, must be consumed exactly.
pub fn decode(data: &[u8]) -> Result<Vec<Command>, Error> {
    let mut c = Cur { data, off: 0 };
    let mut out = Vec::new();
    while c.off < data.len() {
        out.push(one(&mut c)?);
    }
    Ok(out)
}

/// Decodes a multiplayer payload (`docs/COMMANDS.md` §5): XORs each u16 with
/// `(seed >> 8) & 0xffff` (odd tail byte plain), then walks commands skipping
/// the `Random::get(0, 2)` padding gap drawn per command from the game's
/// `Random` seeded with `seed` — the exact inverse of
/// `CommandPackage::send` + `add_command`.
///
/// A package that is a lone `ungraceful_player_drop` (`data[0] == b'P'`,
/// size ≤ 4) travels unencoded even in MP and must go through [`decode`].
pub fn decode_mp(data: &[u8], seed: u32) -> Result<Vec<Command>, Error> {
    let key = (seed >> 8) as u16;
    let mut plain = data.to_vec();
    for ch in plain.chunks_exact_mut(2) {
        let v = u16::from_le_bytes([ch[0], ch[1]]) ^ key;
        ch.copy_from_slice(&v.to_le_bytes());
    }
    let mut c = Cur {
        data: &plain,
        off: 0,
    };
    let mut rng = sim::combat::Rng::new(seed);
    let mut out = Vec::new();
    while c.off < plain.len() {
        out.push(one(&mut c)?);
        let pad = rng.get(0, 2) as usize;
        // The encoder grew `size` by the same draw; the last command's
        // padding may be part of the recorded size too.
        c.off = (c.off + pad).min(plain.len());
    }
    Ok(out)
}

/// One command, dispatched on the type byte exactly as `CommandPackage::
/// process` (0x94a700); sizes are the `process_*` returns.
fn one(c: &mut Cur) -> Result<Command, Error> {
    let t = c.u8("command type")?;
    Ok(match t {
        0x00 => {
            let num = c.u8("group num")? as usize;
            let who = c.i8("group who")?;
            let mut objects = Vec::with_capacity(num);
            for _ in 0..num {
                objects.push(c.i16("group object")?);
            }
            Command::Group { who, objects }
        }
        0x01 => Command::Begin,
        0x02 => Command::Stance {
            stance: c.i32("stance")?,
        },
        0x03 => Command::Form {
            form: c.i32("form")?,
            rotate: c.i32("rotate")?,
            queued: c.i32("queued")?,
        },
        0x04 => Command::Attack {
            ox: c.i32("ox")?,
            whom: c.i32("whom")?,
            ignore: c.i32("ignore")?,
            queued: c.i32("queued")?,
        },
        0x05 => Command::SiegeAttack {
            ox: c.i32("ox")?,
            whom: c.i32("whom")?,
            queued: c.i32("queued")?,
        },
        0x06 => Command::SwarmAround {
            ox: c.i32("ox")?,
            whom: c.i32("whom")?,
            queued: c.i32("queued")?,
            orders: c.i32("orders")?,
        },
        0x07 => Command::MoveTo {
            to_x: c.i32("to_x")?,
            to_y: c.i32("to_y")?,
            set_angle: c.i32("set_angle")?,
            angle: c.i32("angle")?,
            orders: c.i8("orders")?,
            queued: c.i8("queued")?,
            form: c.i8("form")?,
            width: c.i8("width")?,
            disembark: c.i8("disembark")?,
        },
        0x08 => Command::MoveNear {
            to_x: c.i32("to_x")?,
            to_y: c.i32("to_y")?,
            tolerance: c.i32("tolerance")?,
            set_angle: c.i32("set_angle")?,
            angle: c.i32("angle")?,
            orders: c.i8("orders")?,
            queued: c.i8("queued")?,
            form: c.i8("form")?,
            width: c.i8("width")?,
            disembark: c.i8("disembark")?,
        },
        0x09 => Command::AttackGround {
            to_x: c.i32("to_x")?,
            to_y: c.i32("to_y")?,
            queued: c.i8("queued")?,
        },
        0x0a => Command::Patrol {
            to_x: c.i32("to_x")?,
            to_y: c.i32("to_y")?,
            queued: c.i8("queued")?,
        },
        0x0b => Command::LaunchPatrol {
            to_x: c.i32("to_x")?,
            to_y: c.i32("to_y")?,
            queued: c.i32("queued")?,
            shift: c.i32("shift")?,
            ctrl: c.i32("ctrl")?,
            alt: c.i32("alt")?,
        },
        0x0c => Command::Halt,
        0x0d => Command::Transport,
        0x0e => Command::SetTransport {
            flag: c.i32("flag")?,
        },
        0x0f => Command::BoardShip {
            ox: c.i32("ox")?,
            queued: c.i32("queued")?,
        },
        0x10 => Command::Repair {
            ox: c.i32("ox")?,
            whom: c.i32("whom")?,
            queued: c.i32("queued")?,
        },
        0x11 => Command::Trade {
            ox: c.i32("ox")?,
            whom: c.i32("whom")?,
            oxx: c.i32("oxx")?,
            whose: c.i32("whose")?,
            queued: c.i32("queued")?,
        },
        0x12 => Command::CityGather {
            t: c.i32("t")?,
            queued: c.i32("queued")?,
        },
        0x13 => Command::Gather {
            ox: c.i32("ox")?,
            queued: c.i32("queued")?,
        },
        0x14 => Command::Garrison {
            ox: c.i32("ox")?,
            whom: c.i32("whom")?,
            queued: c.i32("queued")?,
        },
        0x15 => Command::Disband { all: c.i32("all")? },
        0x16 => Command::GatherPoint {
            x: c.i32("x")?,
            y: c.i32("y")?,
            action: c.i32("action")?,
            add_to_end: c.i32("add_to_end")?,
        },
        0x17 => Command::Spell {
            ox: c.i32("ox")?,
            whom: c.i32("whom")?,
            spell: c.i32("type")?,
            x: c.i32("x")?,
            y: c.i32("y")?,
        },
        0x18 => Command::QueueUp {
            unit: c.i32("type")?,
            num: c.i32("num")?,
        },
        0x19 => Command::Build {
            x: c.i32("x")?,
            y: c.i32("y")?,
            x2: c.i32("x2")?,
            y2: c.i32("y2")?,
            build: c.i32("type")?,
            queued: c.i32("queued")?,
        },
        0x1a => Command::EjectAll {
            back_to_work: c.i32("back_to_work")?,
            who: c.i32("who")?,
            eject_o: c.i32("eject_o")?,
            eject_who: c.i32("eject_who")?,
        },
        0x1b => Command::Alarm,
        0x1c => Command::Flight {
            ox: c.i32("ox")?,
            whom: c.i32("whom")?,
            shift: c.i32("shift")?,
            ctrl: c.i32("ctrl")?,
            alt: c.i32("alt")?,
            orders: c.i32("orders")?,
        },
        0x1d => Command::StopSpell,
        0x1e => Command::Follow {
            ox: c.i32("ox")?,
            whom: c.i32("whom")?,
            queued: c.i32("queued")?,
        },
        0x1f => Command::Guard {
            ox: c.i32("ox")?,
            whom: c.i32("whom")?,
            queued: c.i32("queued")?,
        },
        0x20 => Command::Unitmask {
            mask: c.i32("unitmask")?,
            set: c.i32("set")?,
        },
        0x21 => Command::Buildmask {
            mask: c.i32("buildmask")?,
            set: c.i32("set")?,
        },
        0x22 => Command::Hotkey {
            group: c.i32("group")?,
            clear: c.i32("clear")?,
            valid: c.i32("valid")?,
            x: c.u32("x")?,
            y: c.u32("y")?,
            zoom: c.i32("zoom")?,
        },
        0x23 => Command::Recall,
        0x24 => Command::Scramble,
        0x25 => Command::Treaty {
            who: c.i32("who")?,
            whom: c.i32("whom")?,
            treaty: c.i32("treaty")?,
        },
        0x26 => Command::Declare {
            who: c.i32("who")?,
            whom: c.i32("whom")?,
            treaty: c.i32("treaty")?,
        },
        0x27 => Command::ClearTributes {
            who: c.i32("who")?,
            whom: c.i32("whom")?,
        },
        0x28 => Command::ClearAll {
            who: c.i32("who")?,
            whom: c.i32("whom")?,
        },
        0x29 => Command::Accept {
            who: c.i32("who")?,
            whom: c.i32("whom")?,
        },
        0x2a => Command::Reject {
            who: c.i32("who")?,
            whom: c.i32("whom")?,
        },
        0x2b => Command::Tribute {
            who: c.i32("who")?,
            whom: c.i32("whom")?,
            good: c.i32("good")?,
            amount: c.i32("amount")?,
        },
        0x2c => Command::DemandTribute {
            who: c.i32("who")?,
            whom: c.i32("whom")?,
            good: c.i32("good")?,
            amount: c.i32("amount")?,
        },
        0x2d => Command::ProposeAttack {
            who: c.i32("who")?,
            whom: c.i32("whom")?,
            whose: c.i32("whose")?,
            onoff: c.i32("onoff")?,
        },
        0x2e => Command::Buy {
            who: c.i32("who")?,
            good: c.i32("good")?,
            flags: c.i32("flags")?,
        },
        0x2f => Command::Sell {
            who: c.i32("who")?,
            good: c.i32("good")?,
            flags: c.i32("flags")?,
        },
        0x30 => Command::Unqueue {
            who: c.i32("who")?,
            o: c.i32("o")?,
            unit: c.i32("type")?,
            uid: c.i16("uid")?,
        },
        0x31 => Command::ComeOut {
            who: c.i32("who")?,
            o: c.i32("o")?,
            uid: c.i16("uid")?,
        },
        0x32 => Command::Ping {
            x: c.i32("x")?,
            y: c.i32("y")?,
        },
        0x33 => {
            let kind = c.u8("spline_type")?;
            let flags = c.u8("spline_flags")?;
            let cmd = c.u8("spline_cmd")?;
            let len = c.u16("spline len")? as usize;
            let mut verts = Vec::with_capacity(len);
            for _ in 0..len {
                verts.push((c.u32("vert x")?, c.u32("vert y")?));
            }
            Command::Spline {
                kind,
                flags,
                cmd,
                verts,
            }
        }
        0x34 => Command::SpeedSet {
            speed: c.i32("speed")?,
        },
        0x35 => Command::SpeedUp,
        0x36 => Command::SpeedDown,
        0x37 => Command::MpLog,
        0x38 => Command::CheckRandom {
            seed: c.u32("seed")?,
        },
        0x39 => {
            let mut sums = [0u32; 16];
            for s in &mut sums {
                *s = c.u32("checksum")?;
            }
            Command::CheckSums { sums }
        }
        0x3a => Command::NextCheckSum {
            kind: c.u8("checksum_type")?,
            checksum: c.u32("checksum")?,
        },
        0x3b => Command::CheatViewAll { who: c.i32("who")? },
        0x3c => Command::CheatGiveTechs { who: c.i32("who")? },
        0x3d => Command::CheatZeroTechs { who: c.i32("who")? },
        0x3e => Command::CheatAiSpeedIncrease,
        0x3f => Command::CheatAiSpeedNormal,
        0x40 => Command::CheatAiToggle,
        0x41 => Command::CheatIncreaseBuckets { who: c.i32("who")? },
        0x42 => Command::CheatZeroBuckets { who: c.i32("who")? },
        0x43 => Command::CheatInitUnit {
            who: c.i32("who")?,
            t: c.i32("t")?,
            x: c.i32("x")?,
            y: c.i32("y")?,
        },
        0x44 => {
            let bits = c.i32("bits")?;
            let taunt = c.i32("taunt")?;
            let taunt_num = c.i32("taunt_num")?;
            let len = c.u32("chat len")? as usize;
            if len > 256 {
                return Err(c.bad(format!("implausible chat length {len}")));
            }
            // len + 1: the wire carries the null terminator.
            let text = c.wchars(len + 1, "chat text")?;
            Command::Chat {
                bits,
                taunt,
                taunt_num,
                text,
            }
        }
        0x45 => {
            let mut status = [0u8; 8];
            status.copy_from_slice(c.raw(8, "chat_set status")?);
            Command::ChatSet { status }
        }
        0x46 => Command::Resign {
            play: c.i32("play")?,
        },
        0x47 => Command::Quit {
            play: c.i32("play")?,
            replay: c.u8("replay")?,
            system_quit: c.u8("system_quit")?,
        },
        0x48 => Command::Camera {
            zoom: c.u8("zoom")?,
            x: c.i32("x_loc")?,
            y: c.i32("y_loc")?,
        },
        0x49 => {
            let who = c.i32("who")?;
            let peasants = c.i32("peasants")?;
            let peasants_wait = c.i32("peasants_wait")?;
            let buildings = c.i32("buildings")?;
            let mut mask = [0u8; 16];
            mask.copy_from_slice(c.raw(16, "leader_options mask")?);
            Command::LeaderOptions {
                who,
                peasants,
                peasants_wait,
                buildings,
                mask,
            }
        }
        0x4a => Command::TurnData {
            ping_time: c.u16("ping_time")?,
            frame_average: c.u16("frame_average")?,
            wait_time: c.u16("wait_time")?,
            game_lag: c.u16("game_lag")?,
            forced_loads: c.u16("forced_loads")?,
        },
        0x4b => Command::RenameCity {
            who: c.i32("who")?,
            o: c.i32("o")?,
            name: c.wchars(22, "city name")?,
        },
        0x4c => Command::Pause {
            state: c.u8("state")?,
        },
        0x4d => Command::CannonTime {
            state: c.u8("state")?,
        },
        0x4e => Command::ConsoleCmd {
            mouse_x: c.i32("mouse_x")?,
            mouse_y: c.i32("mouse_y")?,
            cmd: c.wchars(256, "console cmd")?,
        },
        0x4f => {
            let mut accum = [0u8; 8];
            accum.copy_from_slice(c.raw(8, "player_speed accums")?);
            Command::PlayerSpeed { accum }
        }
        0x50 => Command::UngracefulPlayerDrop {
            play: c.u8("play")?,
            state: c.u8("state")?,
        },
        0x51 => Command::Marwan {
            start: c.u8("start")?,
        },
        _ => return Err(c.bad(format!("unknown command type {t:#04x}"))),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_group_and_a_move_decode() {
        // group: 2 units of leader 0; move_to (16,32), plain flags.
        let mut d: Vec<u8> = vec![0x00, 2, 0, 5, 0, 7, 0];
        d.push(0x07);
        d.extend(16i32.to_le_bytes());
        d.extend(32i32.to_le_bytes());
        d.extend(0i32.to_le_bytes());
        d.extend(0i32.to_le_bytes());
        d.extend([1, 0, 2, 3, 0]);
        let cmds = decode(&d).unwrap();
        assert_eq!(
            cmds,
            vec![
                Command::Group {
                    who: 0,
                    objects: vec![5, 7]
                },
                Command::MoveTo {
                    to_x: 16,
                    to_y: 32,
                    set_angle: 0,
                    angle: 0,
                    orders: 1,
                    queued: 0,
                    form: 2,
                    width: 3,
                    disembark: 0
                }
            ]
        );
    }

    #[test]
    fn the_empty_group_is_the_same_selection() {
        let cmds = decode(&[0x00, 0, 3]).unwrap();
        assert_eq!(
            cmds,
            vec![Command::Group {
                who: 3,
                objects: vec![]
            }]
        );
    }

    #[test]
    fn chat_carries_its_terminator() {
        // "hi", len 2, wire has 3 wchars.
        let mut d: Vec<u8> = vec![0x44];
        d.extend((-1i32).to_le_bytes());
        d.extend((-1i32).to_le_bytes());
        d.extend(0i32.to_le_bytes());
        d.extend(2u32.to_le_bytes());
        for u in "hi\0".encode_utf16() {
            d.extend(u.to_le_bytes());
        }
        assert_eq!(d.len(), 0x13 + 2 * 2);
        let cmds = decode(&d).unwrap();
        assert_eq!(cmds.len(), 1);
        let Command::Chat { text, bits, .. } = &cmds[0] else {
            panic!("not chat")
        };
        assert_eq!(text, "hi");
        assert_eq!(*bits, -1);
    }

    #[test]
    fn a_truncated_or_unknown_payload_is_refused() {
        assert!(decode(&[0x07, 1, 2]).is_err());
        assert!(decode(&[0x52]).is_err());
        // A trailing partial command fails rather than being dropped.
        let mut d: Vec<u8> = vec![0x35];
        d.push(0x48);
        assert!(decode(&d).is_err());
    }

    /// Install-gated: every one of the heavengames sample's 21,884 payloads
    /// decodes with this table, each consumed to its exact size — the check
    /// that verified `docs/COMMANDS.md` §3/§4 in the first place, pinned
    /// with the histogram's load-bearing rows.
    #[test]
    fn the_sample_s_every_payload_decodes() {
        let Some(root) = crate::testenv::install_root() else {
            eprintln!("skipping: no install (set RON_INSTALL)");
            return;
        };
        let dir = std::path::Path::new(&root).join("external-recgames");
        let Some(path) = std::fs::read_dir(dir).ok().and_then(|mut d| {
            d.find_map(|e| {
                let p = e.ok()?.path();
                (p.extension()? == "rcx").then(|| p.display().to_string())
            })
        }) else {
            eprintln!("no .rcx under the install's external-recgames; skipping");
            return;
        };
        let rec = crate::recgame::read(&path).unwrap();
        let mut histogram: std::collections::BTreeMap<&'static str, usize> = Default::default();
        for p in &rec.packages {
            for cmd in decode(&p.data).expect("every payload decodes") {
                *histogram.entry(cmd.name()).or_default() += 1;
            }
        }
        assert_eq!(histogram.get("camera"), Some(&21044));
        assert_eq!(histogram.get("player_speed"), Some(&2631));
        assert_eq!(histogram.get("group"), Some(&1010));
        assert_eq!(histogram.get("move_to"), Some(&374));
        assert_eq!(histogram.get("queue_up"), Some(&261));
        assert_eq!(histogram.get("attack"), Some(&80));
        assert_eq!(histogram.get("build"), Some(&31));
        // The first package — frame 0, valid 1 — is the game-start
        // leader_options push, alone.
        let first = decode(&rec.packages[0].data).unwrap();
        assert_eq!(first.len(), 1);
        assert!(matches!(first[0], Command::LeaderOptions { .. }));
    }

    /// Round-trip through the MP encode: XOR each u16 with (seed>>8)&0xffff,
    /// pad 0–2 seeded-random bytes per command — built here exactly as
    /// `CommandPackage::send`/`add_command` build it, then undone.
    #[test]
    fn the_mp_obfuscation_reverses() {
        let seed = 0x144b_d480u32;
        // Two commands: speed_up (1 byte) and camera (10 bytes).
        let mut plain: Vec<u8> = Vec::new();
        let mut rng = sim::combat::Rng::new(seed);
        plain.push(0x35);
        // The encoder's garbage gaps, one draw per command.
        plain.extend(std::iter::repeat_n(0xEE, rng.get(0, 2) as usize));
        plain.push(0x48);
        plain.push(4);
        plain.extend(100i32.to_le_bytes());
        plain.extend(200i32.to_le_bytes());
        plain.extend(std::iter::repeat_n(0xEE, rng.get(0, 2) as usize));
        // The network XOR, odd tail byte plain.
        let key = (seed >> 8) as u16;
        let mut wire = plain.clone();
        for ch in wire.chunks_exact_mut(2) {
            let v = u16::from_le_bytes([ch[0], ch[1]]) ^ key;
            ch.copy_from_slice(&v.to_le_bytes());
        }
        let cmds = decode_mp(&wire, seed).unwrap();
        assert_eq!(
            cmds,
            vec![
                Command::SpeedUp,
                Command::Camera {
                    zoom: 4,
                    x: 100,
                    y: 200
                }
            ]
        );
    }
}
