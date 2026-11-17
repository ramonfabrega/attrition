//! **The offers arm** — item 4159's step VII-c (`docs/LEADERS.md` §7.2,
//! §8.2): `Leader::diplomacy@006bc950` from the team-style arms
//! (`LAB_006bdf83`) through `LAB_006be428` to the offers arm
//! (`LAB_006bf0e0`), where a computer answers a proposal another leader
//! has committed to; with the treaty a proposal carries
//! (`Leader::action_treaty@006d1570`) and the test for one
//! (`LeaderData::any_proposals@006d5ad0`).
//!
//! What is carried is the path a peace asked for nothing walks, and every
//! store on it: the elimination and victory arms of team styles 0, 8 and
//! 11, the quiet coin, the clear of a committed deal, the invaders'
//! message, the valuation of the goods in the deal, the shortfall under
//! both of its selectors (`aggression[t]`, the one store at `6bcd78`), and
//! the three outcomes of a proposal that asks no alliance: the accept, the
//! counteroffer and the refusal. Each arm the path does not take stops at
//! a `SEAM` that names it.

use super::Sim;
use super::alliance::Rivals;
use crate::Player;

/// `Leader::diplomacy+0x1b12`'s coin is [`super::SITE_QUIET_COIN`]; the
/// valuation's early-game multiplier ends at 4500 frames (`0x1194`) …
const EARLY_GAME: i64 = 4500;
/// … and its three-halves at 9000.
const MIDDLE_GAME: i64 = 9000;

/// The ally pricing's window: a gift to an ally inside 4500 frames of the
/// last is charged (`gift_stamp`, `0x1194`).
const GIFT_WINDOW: i64 = 4500;

/// What the shortfall forgives a pending counteroffer (`6bfe1c`, `−0x63`).
const COUNTEROFFER_GRACE: i32 = 99;

/// `TypeIndex` 0x244's name, `GLOBAL_GOVERNMENT`, the final tech the
/// victory arm asks whether the target is researching.
const GLOBAL_GOVERNMENT: &str = "Global Government";

impl Sim {
    /// **`Leader::action_treaty@006d1570(me, p, w)`**: `me` asks `p` for
    /// treaty `w` (1 peace, 2 alliance). `me.dip[p].any_offer` is set, both
    /// agreements are withdrawn ([`Sim::clear_agree`] each way, which
    /// zeroes that `any_offer` again), and `w` is written into both
    /// records. The diplomacy window's click stamp is the interface's.
    pub fn action_treaty(&mut self, me: Player, p: Player, w: i32) {
        let Some((m, x)) = self.deal_pair(me, p) else {
            return;
        };
        self.leaders.tribute[m].dip[x].any_offer = 1;
        self.clear_agree(m, x);
        self.clear_agree(x, m);
        self.leaders.tribute[m].dip[x].treaty = w;
        self.leaders.tribute[x].dip[m].treaty = w;
    }

    /// **`Leader::action_attack@006d14e0(me, p, whose, onoff)`**: `me` asks
    /// `p` to attack `whose` (`onoff` 1) or withdraws the ask (0), the
    /// diplomacy window's Mutual Attack check (`CommandPackage::
    /// process_propose_attack@00946b30`). `me.dip[p].any_offer` is set;
    /// with a slot named (`whose >= 0`), both agreements are withdrawn
    /// ([`Sim::clear_agree`] each way) and `onoff` is written into both
    /// records' `attacks[whose]` (item 4298, `docs/LEADERS.md` §7.2). The
    /// window's click stamp is the interface's.
    pub fn action_attack(&mut self, me: Player, p: Player, whose: i32, onoff: i32) {
        let Some((m, x)) = self.deal_pair(me, p) else {
            return;
        };
        self.leaders.tribute[m].dip[x].any_offer = 1;
        let Some(w) = usize::try_from(whose)
            .ok()
            .filter(|&w| w < self.leaders.tribute[m].dip[x].attacks.len())
        else {
            return;
        };
        self.clear_agree(m, x);
        self.clear_agree(x, m);
        self.leaders.tribute[m].dip[x].attacks[w] = onoff;
        self.leaders.tribute[x].dip[m].attacks[w] = onoff;
    }

    /// **`LeaderData::any_proposals@006d5ad0(me, p)`**: a treaty asked, or
    /// any good or attack in `me.dip[p]`.
    pub fn any_proposals(&self, me: Player, p: Player) -> bool {
        let Some(d) = self
            .leaders
            .tribute
            .get(me as usize)
            .and_then(|t| t.dip.get(p as usize))
        else {
            return false;
        };
        d.treaty != -1 || d.offers.iter().any(|&o| o != 0) || d.attacks.iter().any(|&a| a != 0)
    }

    /// **`LeaderData::resource_value@006c18d0(me, r)`**: what a unit of
    /// good `r` is worth to `me`, in 256ths. `econ[r] & 2` under 1000 in
    /// stock is 0x200; `econ & 4` under 500 is 0x180; `econ & 8`, or over
    /// 1000 in stock, is 0xaa; else 0x100.
    pub(crate) fn resource_value(&self, me: usize, r: usize) -> i32 {
        let econ = self.ai.ai[me].econ[r];
        let stock = self.economy.ledgers[me].bucket[r];
        if econ & 2 != 0 && stock < 1000 {
            0x200
        } else if econ & 4 != 0 && stock < 500 {
            0x180
        } else if econ & 8 != 0 || stock > 1000 {
            0xaa
        } else {
            0x100
        }
    }

    /// **The arms of team styles 0, 8 and 11** (`LAB_006bdf83` to
    /// `LAB_006be428`), after the view is stored: `None` when an arm ends
    /// the pass at `t`, else the two third parties they noted
    /// ([`Rivals`], which only an alliance's price reads).
    ///
    /// - **Elimination** (`ELIMINATION` 1): when my capital is held by
    ///   another, by neither `t` nor an ally of `t`, `aggression[t]` is
    ///   cleared and the holder noted; held by `t` or `t`'s ally, I answer
    ///   a committed proposal of `t`'s and turn on `t` ([`Sim::turn_on`]).
    /// - **Victory** (`VICTORY` 0): a target neither me nor my mutual ally
    ///   that runs a territory or wonder clock (`popwin_timer`,
    ///   `wonderwin_timer`), or is researching Global Government, is turned
    ///   on the same way; else a third party racing a clock is noted
    ///   (`LAB_006be372`).
    ///
    /// SEAM: the top-scorer arm (`LAB_006be004`'s `bVar33`) needs the score
    /// count and the top scorer, which this pass takes as never the target
    /// (the head's own `SEAM`), so it is false; the victory arm's
    /// `GLOBAL_GOVERNMENT_BONUS` preq is read as held by no one (the crate
    /// keeps no such preq); the human third party racing a clock that
    /// `LAB_006be372` finds (`local_58`) feeds the attack valuation, which
    /// is not carried, and the alliance's price; the elimination arm reads
    /// `gamec`'s flag as the game's.
    pub(super) fn style_zero_arms(&mut self, me: usize, t: usize) -> Option<Rivals> {
        if std::env::var("RON_PROBE_4331").is_ok() && (5480..=5500).contains(&self.frame) {
            eprintln!(
                "P4331y f{} me{me} t{t} style {} elim {} victory {} holder {}",
                self.frame,
                self.ai.lobby.team_style,
                self.ai.lobby.elimination,
                self.ai.lobby.victory,
                self.capital_holder(me as Player)
            );
        }
        let mut rivals = Rivals::default();
        if self.ai.lobby.elimination == 1 {
            // `LeaderData::find_capital(−1, −1)@006eb930`'s holder.
            let holder = self.capital_holder(me as Player) as usize;
            if holder != me {
                if holder != t && !self.allied_both(t, holder) {
                    self.ai.ai[me].aggression[t] = 0;
                    rivals.capital = Some(holder);
                } else {
                    self.turn_on(me, t);
                    return None;
                }
            }
        }
        if self.ai.lobby.victory == 0 && t != me && !self.allied_both(me, t) {
            let gg = self
                .tables
                .tech_tree
                .types
                .iter()
                .position(|ty| ty.name.eq_ignore_ascii_case(GLOBAL_GOVERNMENT));
            let racing = self.ai.ai[t].popwin_timer != 0
                || self
                    .leaders
                    .endgame
                    .get(t)
                    .is_some_and(|e| e.wonderwin_timer != 0)
                || gg.is_some_and(|g| self.researching(t as Player, g));
            if racing {
                self.turn_on(me, t);
                return None;
            }
        }
        if std::env::var("RON_PROBE_4331").is_ok() && (5480..=5500).contains(&self.frame) {
            let gg = self
                .tables
                .tech_tree
                .types
                .iter()
                .position(|ty| ty.name.eq_ignore_ascii_case(GLOBAL_GOVERNMENT));
            eprintln!(
                "P4331z f{} me{me} t{t} holder {} popwin {} gg {:?} res {:?} ages {:?}",
                self.frame,
                self.capital_holder(me as Player),
                self.ai.ai[t].popwin_timer,
                gg,
                gg.map(|g| self.researching(t as Player, g)),
                (0..3).map(|l| self.tech[l].ages).collect::<Vec<_>>()
            );
        }
        rivals.racer = self.racing_third_party(me, t);
        Some(rivals)
    }

    /// The elimination and victory arms' tail (`LAB_006be17f` to
    /// `LAB_006be187`): a proposal `t` has committed to is answered —
    /// withdrawn if I had committed too, else rejected — and then, unless
    /// already at war, I declare (`action_declare(me, t, 0, quiet)`) where
    /// the rush rules allow it; `agendas[t] |= 0x20` either way. The
    /// console's message is the interface's.
    fn turn_on(&mut self, me: usize, t: usize) {
        if self.leaders.tribute[t].dip[me].agree == 1 {
            let code = if self.leaders.tribute[me].dip[t].agree == 1 {
                0
            } else {
                2
            };
            self.action_respond(me as Player, t as Player, code);
        }
        if self.not_at_war(me, t) {
            let rr = self.ai.lobby.rush_rules;
            let declare = rr == 0
                || self.current_age() >= rr
                || (rr > 8 && i64::from(self.ai.lobby.rush_minutes) * 900 <= self.frame);
            if declare {
                self.action_declare(me as Player, t as Player, 0, true, false);
            }
        }
        self.leaders.agendas[me][t] |= 0x20;
    }

    /// **`LAB_006be428` onward**, for every team style: the quiet coin, the
    /// clear of a committed deal, then the arm `aggression[t]` picks.
    ///
    /// - The quiet coin (`SITE_QUIET_COIN`): with no commitment of mine to
    ///   `t`, an unforced pass, `aggression[t]` 0 and `agendas[t] & 8`, two
    ///   draws in three end the pass.
    /// - When I had committed to `t`, or `t.dip[me].agree` is 2, the deal
    ///   is cleared both ways (`action_clear_all`) and noted.
    /// - `aggression[t]` 0: at peace, unforced, with `t`'s units in my land
    ///   (`invaders[t]`), `agendas[t] |= 0x18` and `t`'s
    ///   `got_diplo_message` is this frame; then the offers arm.
    /// - `aggression[t]` set, at war: the offers arm. At peace: at team
    ///   styles 0, 8 and 11 with war allowed, the demand arm
    ///   ([`Sim::demand_or_declare`]); else, or when its gate refuses, the
    ///   rush-rules message (`LAB_006bed3f`).
    ///
    /// SEAM: the scenario's fixed diplomacy (`semaphore[1] & 0x10` or
    /// `semaphore[2] & 2` with `ScenarioData::diplomacy_setting`) is not
    /// carried, nor team style 7's way into the demand arm (a team-8
    /// leader on either side); every capture is style 0 or 1.
    pub(super) fn diplomacy_tail(
        &mut self,
        me: usize,
        t: usize,
        forced: bool,
        rivals: Rivals,
        plan_war: bool,
        plan_frame: i64,
    ) {
        let frame = self.frame;
        let aggression = self.ai.ai[me].aggression[t];
        if self.leaders.tribute[me].dip[t].agree != 1
            && !forced
            && aggression == 0
            && self.leaders.agendas[me][t] & 8 != 0
        {
            self.mark(super::SITE_QUIET_COIN);
            if self.rng.roll() % 3 != 0 {
                return;
            }
        }
        let mut cleared = false;
        if self.leaders.tribute[me].dip[t].agree == 1 || self.leaders.tribute[t].dip[me].agree == 2
        {
            self.action_clear_all(me as Player, t as Player);
            cleared = true;
        }
        if aggression == 0 {
            let invaders = self.ai.ai[me].census.invaders.get(t).copied().unwrap_or(0);
            if !forced
                && invaders != 0
                && t != me
                && self.diplos(me, t) != 0
                && self.diplos(t, me) != 0
                && !self.allied_both(me, t)
            {
                // The message (`receive_chat` to the console player) draws
                // only from the interface's own stream.
                self.leaders.agendas[me][t] |= 0x18;
                self.ai.ai[t].got_diplo_message = frame;
            }
        } else if self.not_at_war(me, t) {
            if matches!(self.ai.lobby.team_style, 0 | 8 | 11)
                && self.war_allowed()
                && !self.is_tribute_period(me as Player, t as Player)
                && frame >= plan_frame
            {
                self.demand_or_declare(me, t, cleared, plan_war);
                return;
            }
            // `LAB_006bed3f`: the rush rules forbid the war I want, or the
            // demand's gate refused. A proposal `t` has committed to is
            // answered first (withdrawn if I had committed, else rejected).
            self.answer_committed(me, t);
            let quiet = {
                let stamp = self.ai.ai[t].got_diplo_message;
                stamp != 0 && frame - stamp < 0x1c2
            };
            if self.leaders.agendas[me][t] & 0x48 != 0 || quiet {
                return;
            }
            // The message itself (`receive_chat` to the console player)
            // draws only from the interface's own streams.
            self.ai.ai[t].got_diplo_message = frame;
            self.leaders.agendas[me][t] |= 0x40;
            return;
        }
        self.offers_arm(me, t, cleared, rivals);
    }

    /// `t` committed to a proposal to me (`t.dip[me].agree == 1`): I answer
    /// it, withdrawing if I had committed too, else rejecting.
    fn answer_committed(&mut self, me: usize, t: usize) {
        if self.leaders.tribute[t].dip[me].agree == 1 {
            let code = if self.leaders.tribute[me].dip[t].agree == 1 {
                0
            } else {
                2
            };
            self.action_respond(me as Player, t as Player, code);
        }
    }

    /// **The demand arm** (`LAB_006be54c`, `006be5f4`–`006be98d`; item 4331,
    /// `docs/LEADERS.md` §8.2): `aggression[t]` set, at peace, at team
    /// styles 0, 8 and 11 with war allowed, past `is_tribute_period(me, t)`
    /// and the planned war's frame. `D` is `(Military_me + 1) · (t.blacken
    /// + 1) · 100`.
    ///
    /// - **War** when this pass cleared a commitment (`cleared`), `t`'s
    ///   `blacken` is past 3, a war is planned (`plan_war`, `local_34`), I
    ///   hired against `t` inside 13,501 frames (`hire_stamp[t]`), or `t`
    ///   holds less WEALTH than `D`: a proposal `t` committed to is answered,
    ///   and where the rush rules allow, `t.got_diplo_message` is this frame,
    ///   `agendas[t] |= 8` and I declare (`action_declare(me, t, ally ? 1 :
    ///   0, quiet)`).
    /// - **Else the demand**: the deal is cleared both ways; a draw picks
    ///   the good the walk starts from (`% 6`); each good not WEALTH or
    ///   KNOWLEDGE that both of us can deal in, of which I am short
    ///   (`econ & 0xc == 4`) and hold under 1000, is counted, and past the
    ///   first draws again, kept on `% count == 0`; a kept good `t` holds
    ///   more than 49 of is asked for (`−min(stock, D)`), and any other
    ///   counted good adds `(25 · Military_me + 25) · (t.blacken + 1)` to the
    ///   WEALTH ask. Then the WEALTH ask (`−min(ask, t's WEALTH)`), my
    ///   commitment (`action_respond(me, t, 1)`), `agendas[t] |= 8` and
    ///   `tribute_demanded[t] = 1`. The console's message draws only the
    ///   interface's stream.
    pub(super) fn demand_or_declare(&mut self, me: usize, t: usize, cleared: bool, plan_war: bool) {
        use crate::economy::Resource;
        let frame = self.frame;
        let (mp, tp) = (me as Player, t as Player);
        let military = crate::tech::Line::Military.index();
        let epoch_me = self.tech[me].epoch[military];
        let blacken = self.leaders.declare[t].blacken;
        let hired = self.leaders.declare[me].hire_stamp[t];
        let wealth = Resource::Wealth.index();
        let d = (epoch_me + 1).wrapping_mul(blacken + 1).wrapping_mul(100);
        if cleared
            || blacken > 3
            || plan_war
            || (hired != 0 && frame - hired < 0x34bd)
            || self.economy.ledgers[t].bucket[wealth] < d
        {
            self.answer_committed(me, t);
            if self.war_allowed() {
                self.ai.ai[t].got_diplo_message = frame;
                self.leaders.agendas[me][t] |= 8;
                let level = if self.is_ally(mp, tp) { 1 } else { 0 };
                self.action_declare(mp, tp, level, true, false);
            }
            return;
        }
        self.action_clear_all(mp, tp);
        let mut ask = d;
        self.mark(super::SITE_DEMAND_START);
        let start = (self.rng.roll() % 6) as usize;
        let mut count = 0;
        for i in 0..6 {
            let g = (start + i) % 6;
            let r = Resource::ALL[g];
            if matches!(r, Resource::Wealth | Resource::Knowledge)
                || !self.deal_good(me, g)
                || !self.deal_good(t, g)
                || self.ai.ai[me].econ[g] & 0xc != 4
                || self.economy.ledgers[me].bucket[g] >= 1000
            {
                continue;
            }
            let first = count < 1;
            count += 1;
            let kept = first || {
                self.mark(super::SITE_DEMAND_PICK);
                self.rng.roll() as i32 % count == 0
            };
            let stock = self.economy.ledgers[t].bucket[g];
            if kept && stock > 49 {
                self.action_offer(mp, tp, r, -stock.min(d));
            } else {
                ask = ask.wrapping_add((epoch_me * 25 + 25).wrapping_mul(blacken + 1));
            }
        }
        let theirs = self.economy.ledgers[t].bucket[wealth];
        self.action_offer(mp, tp, Resource::Wealth, -ask.min(theirs));
        self.action_respond(mp, tp, 1);
        self.leaders.agendas[me][t] |= 8;
        self.leaders.tribute[me].tribute_demanded[t] = 1;
    }

    /// **The offers arm** (`LAB_006bf0e0`): `t` has committed to a
    /// proposal to me (`t.dip[me].agree`), and I value it.
    ///
    /// 1. `S` is seeded with `good_deeds[t]`. An alliance asked runs its
    ///    gates first ([`Sim::alliance_asked`]): they answer the proposal,
    ///    or take the alliance's price off `S`.
    /// 2. **The valuation**, per good in `me.dip[t].offers` (positive, mine
    ///    to give; negative, asked of `t`): a good either side cannot use
    ///    answers the deal (withdraw if I had committed, else reject) and
    ///    stops the walk. A give over my stock sets the over-ask flag. A
    ///    give, and an ask after a give, is multiplied by four before frame
    ///    4500 and by three halves before 9000 while `weak[t]` is 0. Then
    ///    `S −= resource_value · v / 256`, except a give to a mutual ally of
    ///    no more than a third of my stock, which is free at team styles 0,
    ///    8 and 11 and free elsewhere unless I gave `t` within 4500 frames.
    /// 3. With no proposal left, the pass ends. **The attack requests**
    ///    ([`Sim::attack_terms`]) add their terms to `S`, or refuse; with no
    ///    proposal left after them, the pass ends (`LAB_006bfa57`).
    /// 4. **The shortfall** (`LAB_006bfa57`), selected by `aggression[t]`:
    ///    - 0: unless `t` holds a fresh tribute of mine
    ///      (`is_tribute_period(t, me)`), `S += epoch_t · x` (`x` 100 at
    ///      difficulty under 2, else 50, and only while `weak[t]`); for a
    ///      peace asked, either `S −= (epoch_me + 1) · strong[t]` when I am
    ///      strong, not weak, and `t` fights one war, or `+100` per
    ///      difficulty gate (under 2, under 3) while `t.dow[me] < 2` and
    ///      `100 · epoch_t + 100` while `t.dow[me] < 3` and I am weak. The
    ///      need is `−S`.
    ///    - set: `S' = S + epoch_t · x − (epoch_me + 1) · strong[t]`, less
    ///      50 for a rusher or plus 100/50 for an early boomer, plus 100 at
    ///      difficulty under 2; the need is `(epoch_me + t.blacken + 1) ·
    ///      100 · ((strong[t] != 0) + 1) − S'`.
    ///
    ///    `epoch` is the Military line (`data_encrypted + 0xe8`). A pending
    ///    counteroffer forgives 99.
    /// 5. **The outcome.** An over-ask refuses (withdraw or reject) unless
    ///    the need is positive and I give. A need of 0 or less accepts:
    ///    `counteroffer[t] = 0`, and with a proposal standing
    ///    `action_respond(me, t, 1)`, then `good_deeds[t] −= need` (50 more
    ///    while weak and `t` holds no fresh tribute of mine). A positive need
    ///    with no give of mine is a counteroffer: `counteroffer[t] = 1` and
    ///    both agreements withdrawn; the good it names is the message's.
    ///
    /// The attack requests' terms ([`Sim::attack_terms`], item 4298) come
    /// between the valuation and the shortfall.
    ///
    /// SEAM: a positive need with a give
    /// of mine (`6c035d`: a coin and a war) is not carried; with nothing
    /// committed to me, the computer's own proposals (`6c08f4` on: the
    /// top-scorer arm, read as never the target, then a peace at war and an
    /// alliance at peace) are not carried; nor the scenario's fixed answers.
    pub(super) fn offers_arm(&mut self, me: usize, t: usize, cleared: bool, rivals: Rivals) {
        let frame = self.frame;
        let aggression = self.ai.ai[me].aggression[t];
        if cleared && aggression == 0 {
            return;
        }
        let (mp, tp) = (me as Player, t as Player);
        if self.leaders.tribute[t].dip[me].agree != 1 {
            if self.any_proposals(mp, tp) || aggression != 0 {
                return;
            }
            // SEAM: the computer's own proposals (`6c0a3f` on).
            return;
        }
        let mut s = self.leaders.declare[me].good_deeds[t];
        if self.leaders.tribute[me].dip[t].treaty == 2 {
            // Item 4159's step VII-d: the alliance's gates and price.
            let Some(price) = self.alliance_asked(me, t, rivals) else {
                return;
            };
            s = s.wrapping_sub(price);
        }
        let weak = self.ai.ai[me].weak[t];
        let mut gives = false;
        let mut overask = false;
        for r in 0..crate::economy::RESOURCES {
            let o = self.leaders.tribute[me].dip[t].offers[r];
            if o == 0 {
                continue;
            }
            if !self.deal_good(me, r) || !self.deal_good(t, r) {
                let code = if self.leaders.tribute[me].dip[t].agree == 1 {
                    0
                } else {
                    2
                };
                self.action_respond(mp, tp, code);
                break;
            }
            let stock = self.economy.ledgers[me].bucket[r];
            if stock < o {
                overask = true;
            }
            let mut v = o;
            if o >= 1 || gives {
                gives |= o >= 1;
                if frame < EARLY_GAME {
                    v = v.wrapping_shl(2);
                } else if frame < MIDDLE_GAME && weak == 0 {
                    v = v.wrapping_mul(3) / 2;
                }
            }
            let free = v >= 1 && self.is_ally(mp, tp) && v <= stock / 3 && {
                let stamp = self.leaders.tribute[me].gift_stamp[t];
                // SEAM: team style 7's neutral (a team-8 leader under
                // style 7) prices as styles 0, 8 and 11 do not.
                matches!(self.ai.lobby.team_style, 0 | 8 | 11)
                    || stamp == 0
                    || frame - stamp >= GIFT_WINDOW
            };
            if !free {
                s = s.wrapping_sub(self.resource_value(me, r).wrapping_mul(v) / 256);
            }
        }
        if !self.any_proposals(mp, tp) {
            return;
        }
        s = s.wrapping_add(self.attack_terms(me, t, rivals));
        // `LAB_006bfa57`: a refusal of the attack terms has answered.
        if !self.any_proposals(mp, tp) {
            return;
        }
        let strong = self.ai.ai[me].strong[t];
        let diff = self.ai_difficulty();
        let military = crate::tech::Line::Military.index();
        let (epoch_t, epoch_me) = (self.tech[t].epoch[military], self.tech[me].epoch[military]);
        let x = if weak != 0 {
            if diff < 2 { 100 } else { 50 }
        } else {
            0
        };
        let mut need = if aggression == 0 {
            if !self.is_tribute_period(tp, mp) {
                s += epoch_t * x;
                if self.leaders.tribute[me].dip[t].treaty == 1 {
                    if strong != 0 && weak == 0 && self.ai.ai[t].census.wars == 1 {
                        s -= (epoch_me + 1) * strong;
                    } else {
                        let dow = self.leaders.declare[t].dow[me];
                        if dow < 2 {
                            if diff < 2 {
                                s += 100;
                            }
                            if diff < 3 {
                                s += 100;
                            }
                        }
                        if dow < 3 && weak != 0 {
                            s += 100 * epoch_t + 100;
                        }
                    }
                }
            }
            -s
        } else {
            // SEAM: multiplayer (`semaphore[0] & 4`) is never set, so the
            // strength term is taken and the scale is 100.
            let mut s2 = s + epoch_t * x - (epoch_me + 1) * strong;
            let flags = self.ai.ai[me].player_flags;
            if flags >> 12 & 1 != 0 {
                s2 -= 50;
            } else if flags >> 13 & 1 != 0 && epoch_me < 4 {
                s2 += if diff < 3 { 100 } else { 50 };
            }
            if diff < 2 {
                s2 += 100;
            }
            let blacken = self.leaders.declare[t].blacken;
            (epoch_me + blacken + 1) * 100 * (i32::from(strong != 0) + 1) - s2
        };
        if self.leaders.tribute[me].counteroffer[t] != 0 {
            need -= COUNTEROFFER_GRACE;
        }
        if overask && !(need > 0 && gives) {
            // The refusal's message is the interface's.
            let code = if self.leaders.tribute[me].dip[t].agree == 1 {
                0
            } else {
                2
            };
            self.action_respond(mp, tp, code);
            return;
        }
        if need <= 0 {
            self.leaders.tribute[me].counteroffer[t] = 0;
            if self.any_proposals(mp, tp) {
                self.action_respond(mp, tp, 1);
                if !self.is_tribute_period(tp, mp) && weak != 0 {
                    need += 50;
                }
                self.leaders.declare[me].good_deeds[t] -= need;
            }
            return;
        }
        if gives {
            // SEAM: a positive need with a give of mine (`6c035d`).
            return;
        }
        // The counteroffer: the good it asks for (the best
        // `resource_value`, rounded up to 100) is the message's alone.
        self.leaders.tribute[me].counteroffer[t] = 1;
        self.clear_agree(t, me);
        self.clear_agree(me, t);
    }

    /// **The attack requests' terms** (`Leader::diplomacy@006bc950`,
    /// `6bf6b2`–`6bfa57`; item 4298, `docs/LEADERS.md` §7.2): `me` the
    /// computer valuing `t`'s committed proposal, for each target `X ∉ {me,
    /// t}` that `me.dip[t].attacks[X]` names, in slot order.
    ///
    /// 1. **Refusals**, each answering the proposal (withdraw if I had
    ///    committed, else reject) and ending the walk: `me` and `X` mutual
    ///    allies (`6bfc0e`); `t` and `X` mutual allies (`6bfb4d`); `t` my
    ///    enemy (`is_enemy`) with no treaty asked (`treaty < 1`, `6bfb2f`).
    /// 2. **The term.** At peace with `X` both ways: `(25 · X.blacken − 100
    ///    · ((t.blacken != 0) + 1)) · (X.mil + 1) + (t.civ + 1) · (weak[X] ?
    ///    20 : 0)`. At war either way: `(t.civ + 1) · (weak[X] ? 120 : 20)`
    ///    while `t` is at peace with `X` both ways, else 0. `mil` and `civ`
    ///    are the Military and Civic lines (`data_encrypted + 0xe8`,
    ///    `+0xec`).
    /// 3. `+ X.civ · 100` for my capital's holder (`local_6c`, [`Rivals`]'
    ///    `capital`), `+ 200` for the racing third party (`local_58`); a
    ///    negative term is halved toward zero when `t` is my ally
    ///    (`is_ally(me, t)`).
    ///
    /// Returns the terms' sum. SEAM: multiplayer (`semaphore[0] & 4`, which
    /// halves the 100) is never set; the mutual-ally term (`6bf7b6`) is
    /// unreachable behind the first refusal (parked 4298a); the refusals'
    /// console messages are the interface's.
    fn attack_terms(&mut self, me: usize, t: usize, rivals: Rivals) -> i32 {
        let (mp, tp) = (me as Player, t as Player);
        let military = crate::tech::Line::Military.index();
        let civic = crate::tech::Line::Civic.index();
        let slots = self.leaders.tribute[me].dip[t].attacks.len();
        let players = self.leaders.tribute.len().min(slots);
        let mut sum: i32 = 0;
        for x in (0..players).filter(|&x| x != me && x != t) {
            if self.leaders.tribute[me].dip[t].attacks[x] == 0 {
                continue;
            }
            let refused = self.allied_both(me, x)
                || self.allied_both(t, x)
                || (self.is_enemy(mp, tp) && self.leaders.tribute[me].dip[t].treaty < 1);
            if refused {
                let code = if self.leaders.tribute[me].dip[t].agree == 1 {
                    0
                } else {
                    2
                };
                self.action_respond(mp, tp, code);
                break;
            }
            let weak = self.ai.ai[me].weak[x] != 0;
            let civ_t = self.tech[t].epoch[civic];
            let mut term = if self.diplos(me, x) != 0 && self.diplos(x, me) != 0 {
                let blacken_t = i32::from(self.leaders.declare[t].blacken != 0);
                (25 * self.leaders.declare[x].blacken - 100 * (blacken_t + 1))
                    * (self.tech[x].epoch[military] + 1)
                    + (civ_t + 1) * if weak { 20 } else { 0 }
            } else if self.diplos(t, x) != 0 && self.diplos(x, t) != 0 {
                (civ_t + 1) * if weak { 120 } else { 20 }
            } else {
                0
            };
            if rivals.capital == Some(x) {
                term += self.tech[x].epoch[civic] * 100;
            }
            if rivals.racer == Some(x) {
                term += 200;
            }
            if self.is_ally(mp, tp) && term < 0 {
                term /= 2;
            }
            sum = sum.wrapping_add(term);
        }
        sum
    }
}

#[cfg(test)]
mod tests;
