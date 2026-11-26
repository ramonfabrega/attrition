//! **The play clock** (item 4696, `docs/CLIENT_INPUT.md` §40): when the
//! window steps the simulation, paced by the wall clock at the game's speed
//! rather than once a presented frame.
//!
//! The original's solo turn control steps a frame when
//! `TurnControl::check_new_frame_solo@00956320` answers 1. It reads
//! `timeGetTime`, in milliseconds, against `frame_counter ×
//! target_frame_time + frame_base_time`. While the time is short of that, it
//! answers 0. When the time is more than `target_frame_time + 2000` past
//! it, it rebases on the time, so a stall is never made up. Otherwise it
//! counts the frame. `target_frame_time` is `TurnControl::timings[speed]`
//! (`TurnControl::init@00957e20`, `speed_set@009567d0`), and `speed` is the
//! lobby's `GAME_SPEED`, an index into `rules.xml`'s `gamespeeds`.
//!
//! The milliseconds come in as an argument, so a test drives the clock
//! with no wall time at all.

/// `TurnControl::timings`, milliseconds a frame for each of `rules.xml`'s
/// `gamespeeds`: Very Slow, Slow, Normal, Fast, Hyper Fast. Read from the
/// executable at VA `0xafc4a4` (`S_GDATA32 TurnControl::timings`, `0002:226468`,
/// `int[5]`) and pinned against it by
/// `tests::the_timings_are_the_executable_s`.
pub const TIMINGS: [u32; 5] = [200, 125, 67, 50, 1];

/// Normal, `gamespeeds`' third entry and `PlayerProfile::default_profile`'s
/// `gs_game_speed`: the speed when nothing names one.
pub const NORMAL: u8 = 2;

/// How far past due the clock may fall before it rebases rather than
/// catching up (`check_new_frame_solo`'s `target_frame_time + 2000`).
pub const STALL_MS: u64 = 2000;

/// Ours: at most this many frames are stepped before one present. It only
/// binds at Hyper Fast, or when a slow step leaves the clock behind; past
/// [`STALL_MS`] the clock rebases as the original's does.
pub const STEPS_PER_PRESENT: u32 = 4;

/// The turn control's three fields that pace a solo game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayClock {
    /// `target_frame_time`, milliseconds.
    pub target: u32,
    /// `frame_base_time`.
    pub base: u64,
    /// `frame_counter`.
    pub counter: u64,
}

impl PlayClock {
    /// The clock at `speed` (an index into [`TIMINGS`], [`NORMAL`] for any
    /// other), its base `now`: the first frame is due one frame later.
    pub fn new(speed: u8, now: u64) -> PlayClock {
        let target = TIMINGS.get(usize::from(speed)).copied();
        PlayClock {
            target: target.unwrap_or(TIMINGS[usize::from(NORMAL)]),
            base: now,
            counter: 1,
        }
    }

    /// `TurnControl::reset_base_time@009578d0`: the base is `now` and the
    /// count starts again. Ours calls it when play resumes, so a pause is
    /// never caught up.
    pub fn rebase(&mut self, now: u64) {
        self.base = now;
        self.counter = 1;
    }

    /// `check_new_frame_solo`'s answer at `now`: whether a frame is due,
    /// counting it when one is.
    pub fn due(&mut self, now: u64) -> bool {
        let at = self.counter * u64::from(self.target) + self.base;
        if now < at {
            return false;
        }
        if now > at + u64::from(self.target) + STALL_MS {
            self.rebase(now);
            return true;
        }
        self.counter += 1;
        true
    }

    /// How many frames the window steps before the present at `now`:
    /// each one [`due`](Self::due), at most [`STEPS_PER_PRESENT`].
    pub fn steps(&mut self, now: u64) -> u32 {
        let mut n = 0;
        while n < STEPS_PER_PRESENT && self.due(now) {
            n += 1;
        }
        n
    }
}

/// The game's speed as the lobby chose it: `GAME_SPEED` of the choices the
/// world was opened from, or [`NORMAL`] when it was opened from none.
pub fn speed_of(choices: Option<&rondata::lobby::Choices>) -> u8 {
    let at = rondata::lobby::SETTINGS
        .iter()
        .position(|s| *s == "GAME_SPEED");
    match (choices, at) {
        (Some(c), Some(i)) if usize::from(c.settings[i]) < TIMINGS.len() => c.settings[i],
        _ => NORMAL,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The row: ticks over a measured span against the rate** (`PLAY_CLOCK`).
    /// A 60 Hz window presents every 16 or 17 ms for 60 s of wall time. At
    /// Normal it steps `60_000 / 67` frames, 895, where stepping once a
    /// present stepped 3,600. Made to fail: `steps` answering 1, 3,600.
    const PLAY_CLOCK: (u32, u64) = (895, 60_000);

    /// The presents of a `hz` display over `span` ms, at whole milliseconds.
    fn presents(hz: u64, span: u64) -> impl Iterator<Item = u64> {
        (1..=span * hz / 1000).map(move |i| i * 1000 / hz)
    }

    #[test]
    fn a_minute_at_normal_steps_the_original_s_frames() {
        let (want, span) = PLAY_CLOCK;
        let mut clock = PlayClock::new(NORMAL, 0);
        let stepped: u32 = presents(60, span).map(|t| clock.steps(t)).sum();
        assert_eq!(stepped, want, "frames over {span} ms at Normal");
        assert_eq!(u64::from(want), span / u64::from(TIMINGS[2]));
        // The display's rate does not move it: 144 Hz steps the same frames.
        let mut fast = PlayClock::new(NORMAL, 0);
        let stepped: u32 = presents(144, span).map(|t| fast.steps(t)).sum();
        assert_eq!(stepped, want, "at 144 Hz");
        // No present steps two frames at Normal.
        let mut one = PlayClock::new(NORMAL, 0);
        assert!(presents(60, span).all(|t| one.steps(t) <= 1));
    }

    #[test]
    fn every_speed_steps_at_its_timing() {
        // Hyper Fast's 1 ms is bound by the cap: four a present at 60 Hz.
        let want = [300, 480, 895, 1200, 3600 * STEPS_PER_PRESENT];
        for (speed, want) in want.iter().enumerate() {
            let mut clock = PlayClock::new(speed as u8, 0);
            let stepped: u32 = presents(60, 60_000).map(|t| clock.steps(t)).sum();
            if speed == 4 {
                // The cap leaves it behind, and each stall past 2 s rebases
                // on a present that steps one: under the cap, never near
                // Normal.
                assert!(stepped <= *want && stepped > want * 9 / 10, "{stepped}");
                continue;
            }
            assert_eq!(stepped, *want, "speed {speed}");
        }
    }

    /// The photographs' clocks read `tick`, `frame / 15` (item 4203,
    /// `docs/HUD.md` §6): 3279 is `00:03:38`, 3327 is `00:03:41`. At
    /// Normal, fifteen frames take 1,005 ms, so the bar's second is the
    /// wall's within half a percent.
    #[test]
    fn the_photographs_clock_runs_at_the_wall_s_second() {
        for (frame, seconds) in [(3279u64, 218u64), (3327, 221)] {
            assert_eq!(frame / 15, seconds);
            let wall = frame * u64::from(TIMINGS[usize::from(NORMAL)]);
            // 3279 frames are 219.7 s of wall time at Normal and the bar
            // reads 218: its second lasts 1.005 of the wall's, never four.
            assert!(wall / 1000 >= seconds && wall / 1000 <= seconds * 1010 / 1000);
        }
    }

    #[test]
    fn a_stall_is_not_made_up() {
        let mut clock = PlayClock::new(NORMAL, 0);
        assert_eq!(clock.steps(67), 1);
        // Five seconds with no present: one frame, and the clock rebases.
        assert_eq!(clock.steps(5_067), 1);
        assert_eq!(clock.base, 5_067);
        assert_eq!(clock.steps(5_100), 0);
        assert_eq!(clock.steps(5_134), 1);
        // Within the 2 s, the frames behind are caught up, four a present.
        let mut late = PlayClock::new(NORMAL, 0);
        assert_eq!(late.steps(670), 4);
        assert_eq!(late.steps(670), 4);
        assert_eq!(late.steps(670), 2);
        assert_eq!(late.steps(670), 0);
    }

    #[test]
    fn a_resumed_clock_starts_from_the_resume() {
        let mut clock = PlayClock::new(NORMAL, 0);
        assert_eq!(clock.steps(1_000), 4);
        clock.rebase(1_500);
        assert_eq!(clock.steps(1_500), 0);
        assert_eq!(clock.steps(1_567), 1);
    }

    #[test]
    fn the_speed_is_the_lobby_s() {
        assert_eq!(speed_of(None), NORMAL);
        let mut c = rondata::lobby::Choices::blank();
        assert_eq!(speed_of(Some(&c)), 0, "a blank lobby's Very Slow");
        c.settings[5] = 3;
        assert_eq!(speed_of(Some(&c)), 3);
        c.settings[5] = 9;
        assert_eq!(speed_of(Some(&c)), NORMAL, "no such speed");
        assert_eq!(PlayClock::new(9, 0).target, 67);
    }

    /// `TIMINGS` against the executable's own bytes, and Escape's option
    /// (`KeyMap::keymap_options[KEYMAP_OPTION_CANCEL]`, entry 107 at VA
    /// `0xae7770`, `short[2]`) as `OPTION_CANCEL`, 67; entry 86,
    /// `KEYMAP_OPTION_MENU`, is option 89.
    #[test]
    fn the_timings_are_the_executable_s() {
        // `TIMINGS`' virtual address in `riseofnations.exe`.
        const TIMINGS_VA: u32 = 0x00af_c4a4;
        let root = std::env::var_os("RON_INSTALL")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("../../game"));
        let exe = root.join("riseofnations.exe");
        if !exe.is_file() {
            eprintln!("no install at {}: TIMINGS unchecked", root.display());
            return;
        }
        let pe = rondata::pe::Pe::open(&exe.to_string_lossy()).expect("a PE");
        let got = pe.i32s(TIMINGS_VA, 5).expect("the table");
        let want: Vec<i32> = TIMINGS.iter().map(|&t| t as i32).collect();
        assert_eq!(got, want);
        let option = |k: u32| pe.i32s(0x00ae_7770 + 4 * k, 1).map(|v| v[0] as i16);
        assert_eq!(option(107), Some(crate::input::OPTION_CANCEL as i16));
        assert_eq!(option(86), Some(89));
    }
}
