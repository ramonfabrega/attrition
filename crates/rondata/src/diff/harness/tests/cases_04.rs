pub(super) use bodies::*;

mod bodies {
    use super::super::*;

    pub(in super::super) fn run130_s_word_is_1_64_s_lag() {
        /// The frame the detour is planned on, and the block before it.
        const DETOUR: i64 = 11_688;
        let Some(inst) = install() else { return };
        let (Some(path), Some(r130)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            dump("gamelog-run130-greatlakes-armytwo.txt"),
        ) else {
            eprintln!("skipping: no run53/run130 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r130).unwrap();
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        if let Some(t) = trace("rontrace-run53.log") {
            borrow_pasture(&mut init, &t);
        }
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let mut block = |n: i64| {
            let at = ix
                .frames()
                .iter()
                .position(|x| x.number == n)
                .expect("run130 carries the block");
            ix.frame_state(at).unwrap()
        };
        let unit = |f: &Frame, o: i64| f.units.iter().find(|t| t.who == 1 && t.o == o).cloned();
        let seat = unit(&block(DETOUR), 64).expect("1/64 on the block").pos;
        for _ in 0..DETOUR {
            built.tick();
        }
        let u64 = built.sim.unit_by_o(1, 64).expect("1/64");
        assert_eq!(
            (
                i64::from(built.sim.units[u64].pos.x),
                i64::from(built.sim.units[u64].pos.y)
            ),
            (seat.x, seat.y),
            "1/64 on the original's point before the detour, unprobed"
        );
        assert!(
            built
                .sim
                .probe_relocate(u64, sim::Pos::new(seat.x as i32, seat.y as i32))
        );
        built.tick();
        let theirs = unit(&block(DETOUR + 1), 62).expect("1/62");
        let u62 = built.sim.unit_by_o(1, 62).expect("1/62");
        let ours: Vec<(i64, i64)> = built.sim.units[u62]
            .path
            .iter()
            .map(|p| (i64::from(p.to.x), i64::from(p.to.y)))
            .collect();
        let want: Vec<(i64, i64)> = theirs.path.iter().map(|p| (p.to.0, p.to.1)).collect();
        assert_eq!(
            ours, want,
            "1/62's detour with 1/64 on the original's point"
        );
        while built.sim.frame < GREAT_LAKES_ARMY_TWO_BLOCK {
            built.tick();
        }
        let theirs = unit(&block(GREAT_LAKES_ARMY_TWO_BLOCK), 62).expect("1/62");
        let u = &built.sim.units[u62];
        let g = u.guys.first().expect("a figure");
        assert_eq!(
            (
                (i64::from(u.pos.x), i64::from(u.pos.y)),
                i64::from(g.anim),
                i64::from(g.cur_time)
            ),
            (
                (theirs.pos.x, theirs.pos.y),
                theirs.guys[0].cur_anim.unwrap(),
                theirs.guys[0].cur_time.unwrap()
            ),
            "1/62 on the word's block, with 1/64 seated"
        );
    }

    pub(in super::super) fn run100_s_word_block_is_every_record_the_dump_carries() {
        /// run100's first complete block — [`WIDENING_GREAT_LAKES`], the
        /// window the `WIDENINGS` row declares, so the guard and this test
        /// read one number.
        const FIRST: i64 = WIDENING_GREAT_LAKES.0;
        /// Thirteen blocks past the word — far enough to carry the
        /// re-convergence the last one had on 10165, and inside run100's
        /// own window. **Moved with the word on item 456** (10175 →
        /// 10246) **and again on 463** (→ 10247): a widening whose window
        /// stops short of the word it is the widening of passes by saying
        /// nothing, which is parked 449's failure one step along — and
        /// since the eighth pass the floors guard fails on it too. Item
        /// 465 moved the word inside it and left the tail alone.
        const TAIL: i64 = WIDENING_GREAT_LAKES.1;
        /// The blocks the replan schedule is read over — three before the
        /// death that opens it and one past `0/2000`'s replan, the last
        /// of the six.
        const REGEN_LO: i64 = 10_228;
        const REGEN_HI: i64 = 10_242;
        /// **The block the word last stood on** — 10834's, item 515's
        /// six rows of `1/28`. Item 518 moved the word to 11185, past
        /// run100's last block, so this test no longer straddles the
        /// headline (the [`WIDENINGS`] row says who owes that); what it
        /// holds on this block is the value diff of the move, and it is
        /// pinned **empty**.
        const LAST_WORD_BLOCK: i64 = 10_835;
        const {
            assert!(
                FIRST < LAST_WORD_BLOCK && LAST_WORD_BLOCK < TAIL,
                "the window must straddle the block it pins"
            )
        };
        let Some(inst) = install() else { return };
        let (Some(path), Some(r100)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            dump("gamelog-run100-greatlakes-valuewindow2.txt"),
        ) else {
            eprintln!("skipping: no run53/run100 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r100).unwrap();
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        if let Some(t) = trace("rontrace-run53.log") {
            borrow_pasture(&mut init, &t);
        }
        // **The clocks are this crate's own prediction here.** run53's
        // per-frame detail is `MISC`, so it carries no `frame_guys`, and
        // the siblings are another map's: nothing installs the original's
        // animation state at any frame of this window. Asserted rather
        // than assumed, because an installed clock would make every guy
        // row below vacuous.
        assert!(
            !init
                .frame_guys
                .iter()
                .any(|(f, _)| (FIRST..=TAIL).contains(f)),
            "a sibling installs the original's guy clocks inside the \
             window, so the guy rows would compare the dump with itself"
        );
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let players = built.sim.players.len();
        use std::collections::BTreeMap;
        // The block each `(who, o, field)` **first** parts on, with the
        // value diff beside it.
        let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
        let mut blocks = 0usize;
        let mut compared = 0usize;
        let mut word_census = None;
        // **The replan flag's own trajectory** (item 478): for every one
        // of the human's buildings, the block its `build_masks & 0x100`
        // goes up and the block it goes down, from the dump and from this
        // crate, over the window the death at 10230 opens.
        let mut regen_dump: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
        let mut regen_ours: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
        for f in 0..=TAIL {
            built.tick();
            let n = f + 1;
            if n < FIRST {
                continue;
            }
            let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
                continue;
            };
            let frame = ix.frame_state(at).unwrap();
            blocks += 1;
            // `RON_DEBUG_UNIT=<who>/<o>@<lo>-<hi>` on **this** loop too.
            // The rows below say which field parted; they never say what
            // this crate's own path stack and order list held while it
            // did, and on the word's frame that is the whole question
            // (item 463).
            debug_watch(&built, n);
            if (REGEN_LO..=REGEN_HI).contains(&n) {
                for b in frame.builds.iter().filter(|b| b.who == 0) {
                    if b.build_masks.is_some_and(|m| m & 0x100 != 0) {
                        regen_dump.entry(b.o).or_default().push(n);
                    }
                }
                for b in built
                    .sim
                    .buildings
                    .iter()
                    .filter(|b| b.alive && b.owner == 0 && b.regen_roads)
                {
                    regen_ours.entry(i64::from(b.index)).or_default().push(n);
                }
            }
            let (r, rows) = widen_block(&built, &frame, players, n, &mut firsts);
            compared += rows;
            if n == LAST_WORD_BLOCK {
                word_census = Some((
                    frame.units.len(),
                    frame.builds.len(),
                    frame.cities.len(),
                    frame.leaders.len(),
                    r.compared,
                    r.order_compared,
                    r.collide_compared,
                    r.city_compared,
                    r.build_compared,
                    r.queue_compared,
                    r.gather_compared,
                ));
            }
        }
        let word = LAST_WORD_BLOCK;
        eprintln!(
            "run100 word widening: {blocks} blocks [{FIRST}, {TAIL}], {compared} record rows, \
             {} keys parted",
            firsts.len()
        );
        eprintln!("  census on {word}: {word_census:?}");
        // **The word's own block, whole.** Every key that first parts on
        // it, with the dump's own numbers beside this crate's.
        let on_word: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f == word)
            .map(|((w, o, what), (_, row))| format!("{w}/{o} {what}: {row}"))
            .collect();
        for r in &on_word {
            eprintln!("  word {r}");
        }
        // `RON_DEBUG_ROWS=<lo>-<hi>` prints every key that first parts
        // inside those blocks, the value diff beside it — item 312's
        // instrument, on the widest row set this crate has (§54.3's
        // residue families were read off it).
        if let Some((lo, hi)) = site_window_named("RON_DEBUG_ROWS") {
            for ((w, o, what), (f, row)) in &firsts {
                if (lo..=hi).contains(f) {
                    eprintln!("  f{f} {w}/{o} {what}: {row}");
                }
            }
        }
        // **10162's fifteen rows are gone, all of them** — item 456
        // (`docs/COLLISION.md` §9), and that is the value diff the word's
        // move is booked on. The block the last word stood on is
        // asserted **clean**, separately from the current word's, so the
        // closure cannot come undone quietly as the headline walks away
        // from it.
        let on_10162: Vec<&String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f == 10_162)
            .map(|((_, _, _), (_, row))| row)
            .collect();
        assert!(
            on_10162.is_empty(),
            "block 10162 parts again — item 448's fifteen rows of `1/38`,              which item 456 closed: {on_10162:?}"
        );
        // **And the six raiders, which item 463 closed.** The block the
        // word stood on before this one parted on six `orders.len` rows —
        // `1/42` on 10231 and `1/27`, `1/28`, `1/29`, `1/40`, `1/41` on
        // 10232, each holding one order where the original holds two —
        // and the three that then walked on 10233 were the consequence.
        // The six are asserted on their **order lists**, over the whole
        // tail of the raid rather than on one block, so the closure
        // cannot come undone quietly as the headline walks away from it.
        // Each drops its order on the block its own `recharging` reaches
        // nought — 10231 for `1/42`, 10233 for `1/28`, 10239 for `1/27`,
        // and past this window for `1/29` — so a rule that dropped them
        // together fails here on three of the four, and one that never
        // dropped them fails on the other three.
        //
        // **Made to fail on purpose**, both ways: with the validity test
        // back in front of the reload gate the set is six `orders.len`
        // rows and six `order:length`, on 10231 and 10232; with
        // [`sim::Sim::forget`] clearing the order's `mandatory` again it
        // is `1/28`'s pair on 10233 and `1/27`'s on 10239, the orders
        // that then never die at all.
        const RAIDERS: [i64; 6] = [27, 28, 29, 40, 41, 42];
        let raiders: Vec<String> = firsts
            .iter()
            .filter(|((w, o, what), (f, _))| {
                *w == 1
                    && RAIDERS.contains(o)
                    && (what == "orders.len" || what == "order:length")
                    && (10_200..=TAIL).contains(f)
            })
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        assert!(
            raiders.is_empty(),
            "the AI's raiders' order lists part again inside the raid's tail \
             — item 463's six dropped `ATTACKORDER`s: {raiders:?}"
        );
        // The rows are the claim, not their number: a set written out is
        // what a later item diffs its own answer against.
        //
        // **Item 465 took `1/28`'s fifteen off it**, which is the whole
        // of the headline's move: the probe's six raiders hold the
        // `GROUP_MOVE` the dump gives them now, so `1/28` spends the
        // frame in `do_group_move`'s follower arm being re-seated and
        // plans on the original's own frame (`docs/ORDERS.md` §16). What
        // is left is item 464's three, and they are the citizen's.
        //
        // The word moved with them, so this set is the block **past**
        // the new word as well as the value diff of the old one.
        //
        // **Item 471 took it from sixty rows to three.** `1/28`'s whole
        // 43-node route is the original's now, node for node, because the
        // formation angle the route hangs off is: §12's probe queues its
        // walk home at `QUEUE_LAST`, so `action_move_near` asks
        // `GroupData::get_loc_to` — where the leader *ends up* — and the
        // bearing is `find_angle` from the **farm** at the other end of
        // the map, not from the leader's own feet, which is where this
        // crate read it and got no bearing at all
        // (`docs/ORDERS.md` §17). What is left is `1/28` one unit east
        // in `x`, said three times: the unit and its guy's two copies.
        // ~~**Item 478 moved the word past this block**, so what the
        // word's own block holds is now **nothing**~~ — ~~**item 483 moved
        // it again, 10237 → 10244, and this is the value diff beside that
        // move.**~~ **Item 487 moved it 10244 → 10277**, and 10245's
        // eleven rows of `1/27` are closed; they are pinned as the empty
        // set below, beside 10234, 10235 and 10238.
        //
        // **The widening is what killed the successor 483 named.** 483
        // read `f10241 1/29 order:kind ours 10 theirs 1` as "`1/29` drops
        // its move order"; it is the other way round — ours still holds
        // the `ATTACK` where the original already holds the plain
        // `MOVE_TO` its ungroup made — and this crate's `1/29` never took
        // a step in the whole window while the original's walked 28 world
        // units a frame from 10242. `docs/ORDERS.md` §20 has the two
        // mechanisms; `docs/DECISIONS.md` 42 is why the frame survived
        // the mechanism.
        //
        // ~~**What the new word holds is the same shape, one raider
        // over.**~~ **Item 489 closed all seventeen**, and the two
        // `half_step` rows were not the successor's first thread but
        // the *same event* read from the other side: `unit_masks &
        // 0x100000` is what §4.3's scan raises when it calls a
        // collision **soft**, so a frame that sets it on both units is
        // a frame on which neither hard-collides. This crate's `1/40`
        // hard-collided with `1/41` because [`sim::Sim::same_group_soft`]
        // asked `army_of` where the original asks `UnitData +0x80`, and
        // a **pushed** group's members are in no army at all — which is
        // every raider of this probe since item 465 put them there.
        // `1/41`'s row came free with it: with `1/40` standing, it was
        // out of `1/41`'s own sweep, so `1/41` found nothing to be soft
        // about either (`docs/COLLISION.md` §11).
        //
        // ~~**The new word is 10294 and its block is the human's
        // citizen**~~ — **item 494 closed all six, and the four on
        // 10294 under them, with one number**: the human's idle wait is
        // `peasants_wait`'s *switch* and not its value, so it is **12**
        // and this crate used 2. `0/5` was put back to work at
        // `idle == 2` on 10234, took its flight to `(792, 31800)` with
        // that `GATHERORDER` queued under it, and so arrived on 10294
        // holding **two** orders — which is the one thing
        // [`sim::Sim::arrive`] will not face the order's angle for, and
        // the reason the frame the original spends in `do_idle` this
        // crate spent walking away (`docs/ORDERS.md` §21).
        //
        // ~~**The new word is 10303 and its block is `1/51`'s animation
        // clock**~~ — **item 497 closed both rows, and the pair thirty
        // blocks under them that nobody had named.** The clock was never
        // the cause: `1/51` is a **scholar** seated in a university, and
        // on 10274 both sides wrapped a 118-frame slot together and
        // re-rolled. The original took `0x1d`, this crate `0x20`, and
        // thirty frames later the original's 30-frame teach slot ran
        // out where this crate's 118-frame one had ninety left to run —
        // which is the whole of 10304's two rows and of the draw the
        // word was. `set_anim:332`'s `param_2 == 0x20` walk examines
        // **every** member of the host's `inside_down` chain, the guy
        // asking included, and its `guys[0].cur_anim` is still the slot
        // it is leaving; this crate read the walk as "some *other*
        // scholar" and so never fired it on a chain with one student
        // (`docs/ANIM.md` §4.12).
        //
        // ~~**The new word is 10582 and its block is one row**~~ —
        // **item 506 closed it, and the cause was five frames under the
        // word on a step that spends no draw at all.** `1/2018
        // queue:queued` was a Horse Archer this crate could pay for and
        // the original could not: on sim-frame **10576**, the
        // rotation's `Setup`, the original's `market_speculation` buys a
        // hundred food for **128** wealth and this crate bought nothing,
        // so it entered `make_stuff` on 10582 holding `94 87 125` where
        // the original holds `194 87 0` (run117; `docs/ECONOMY.md` §13).
        // With no wealth the original's `can_pay` refuses the head, its
        // `use_market` finds wealth short of the shortfall vector and
        // spends the sell draw the sequence parted on, and its step
        // machine disarms where this crate ran a second pass.
        //
        // ~~**The new word is 10817 and its block is `1/28`**, eight
        // rows of one raider: a collision this crate takes and the
        // original does not, with the stand that follows it.~~ **Item
        // 515 closed them, and the collision was a consequence rather
        // than a cause.** On 10818 this crate's `1/28` stood at
        // `(17208, 26952)` where the original's stands at
        // `(16838, 27194)` — 370 units east, about fifteen frames of
        // its own walk — so it collided with the gaia animal `8/0`,
        // which sits at `(17278, 26821)` on **both** sides and never
        // moves in the window. The position said so and the block did
        // not, because `1/28 pos` first parted at **10235** and a
        // `firsts` map records only the first block a key parts on:
        // parked 477's three rows and this word were one cause, 583
        // blocks apart. `docs/GROUPS.md` §18.
        //
        // ~~**The new word is 10834 and its block is the same raider,
        // six rows.**~~ **Item 518 closed all six, and the position one
        // unit out under them.** 10242 was the frame the original's cap
        // went back to the slow squad's 25 and this crate's stayed at
        // 26; the writer is `Groups::process@006fa210`, which resets one
        // pool slot per player a frame, and 10241 is `group 65`'s frame
        // (`docs/GROUPS.md` §19). The block is pinned **empty**, and so
        // are 10242 and 10243 below — `1/28`'s and `1/27`'s first
        // partings — because the position is what the block never
        // showed.
        assert_eq!(
            on_word,
            Vec::<String>::new(),
            "block {word}, the word item 518 left, parts again — item \
             515's six rows of `1/28`"
        );
        // **And 10295, the block the word just left** — item 489's six
        // rows of `0/5`, closed by item 494 and pinned empty here for
        // 10162's, 10234's, 10235's, 10238's, 10245's and 10278's
        // reason: the headline walks away from a block and nothing then
        // watches it. 10294's four under them — the `heading`,
        // `dest_angle` and `g.des_angle` the arrival turn writes — are
        // pinned with it, because they are the same citizen one block
        // earlier and closed by the same number.
        //
        // **10274 and 10304 join them** (item 497), and the pair is the
        // same shape one mechanic over: 10304 is the block the word just
        // left, 10274 the block thirty under it where the cause actually
        // is. `1/51`'s teach slot is the original's on both, so a
        // tie-break that stopped including the requester would fail here
        // on the cause rather than on the symptom.
        // **10583 joins them** (item 506) — the block the word just
        // left, and the value diff for its move: the AI's Stable holds
        // the same queue as the original's now, because the leader
        // reaches the frame with the original's own purse.
        // **And 10818** (item 515), the block the word just left and the
        // value diff for *this* move: all eight rows are gone, and the
        // one that mattered is the one the block never showed —
        // `1/28`'s position, 370 units out on 10818 and **one** unit
        // out now. A regression that put the raider back where it was
        // would bring the collision back with it and fail here rather
        // than only at the word.
        // **And 10242 and 10243** (item 518): `1/28`'s position one
        // unit out and `1/27`'s a block later, the rows the cap's reset
        // closed.
        for b in [
            10_242_i64, 10_243, 10_274, 10_294, 10_295, 10_304, 10_583, 10_818,
        ] {
            let on: Vec<String> = firsts
                .iter()
                .filter(|(_, (f, _))| *f == b)
                .map(|((w, o, what), (_, row))| format!("{w}/{o} {what}: {row}"))
                .collect();
            assert_eq!(
                on,
                Vec::<String>::new(),
                "block {b} parts — item 489's `0/5`, item 497's `1/51`, item \
                 506's `1/2018`, item 515's `1/28` or item 518's raiders is \
                 back"
            );
        }
        // **And 10278, the block the word left before that** — item
        // 487's seventeen rows of `1/40` and `1/41`, every one of them
        // closed by item 489 and pinned empty here for 10162's,
        // 10234's, 10235's, 10238's and 10245's reason: the headline
        // walks away from a block and nothing then watches it.
        let on_10278: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f == 10_278)
            .map(|((w, o, what), (_, row))| format!("{w}/{o} {what}: {row}"))
            .collect();
        assert_eq!(
            on_10278,
            Vec::<String>::new(),
            "block 10278 parts — item 487's seventeen rows of `1/40` \
             and `1/41` are back"
        );
        // **And 10245, the block the word just left** — item 483's eleven
        // rows of `1/27`, every one of them closed by item 487 and pinned
        // empty here for 10162's, 10234's, 10235's and 10238's reason:
        // the headline walks away from a block and nothing then watches
        // it. `1/27` collides with nothing now because `1/29` is not
        // there to collide with.
        let on_10245: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f == 10_245)
            .map(|((w, o, what), (_, row))| format!("{w}/{o} {what}: {row}"))
            .collect();
        assert_eq!(
            on_10245,
            Vec::<String>::new(),
            "block 10245 parts — item 483's eleven rows of `1/27` are back"
        );
        // **And 10238, the block the word just left**, which was the
        // word's own until this item and was pinned empty there. It is
        // still empty, and it is pinned here for 10162's, 10234's and
        // 10235's reason: the headline walks away from a block and
        // nothing then watches it.
        let on_10238: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f == 10_238)
            .map(|((w, o, what), (_, row))| format!("{w}/{o} {what}: {row}"))
            .collect();
        assert_eq!(
            on_10238,
            Vec::<String>::new(),
            "block 10238 parts — it was empty when the word left it"
        );
        // ~~**And 10235, the block the word just left**, where item
        // 477's three rows stand: `1/28` one world unit east in `x`,
        // said by the unit and by its guy's two copies.~~ **Item 515
        // closed all three**, and they were the word's own cause 583
        // blocks below it: the raid's very first step, ours 26 and the
        // original's 25, because a group's speed cap was not modelled
        // at all (`docs/GROUPS.md` §18). Pinned **empty** here for
        // 10162's reason — the headline walks away from a block and
        // nothing then watches it — and this one is worth watching
        // twice, because everything the word was grew out of it.
        let on_10235: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f == 10_235)
            .map(|((w, o, what), (_, row))| format!("{w}/{o} {what}: {row}"))
            .collect();
        assert_eq!(
            on_10235,
            Vec::<String>::new(),
            "block 10235 parts — item 477's three rows of `1/28`, which \
             item 515 closed, are back"
        );
        // **And 10234, where the word's own cause was written sixty
        // blocks early.** `1/28`'s fifteen rows stood here until item
        // 465 and item 464's three until item 494: `0/5 orders.len ours
        // 2 theirs 1` and its `order:length` twin are the `GATHERORDER`
        // this crate queued under the citizen's flight at `idle == 2`,
        // and `0/2001 gather:gather_down` is the camp it linked itself
        // into on the way. All three are the human's idle wait read as 2
        // where the original's switch answers 12 (`docs/ORDERS.md`
        // §21.2), and they are pinned empty here for 10162's reason —
        // the headline walks away from a block and nothing then watches
        // it. **This is the row that says the citizen is not merely
        // walking elsewhere but was never re-tasked at all**: a fix that
        // only delayed the departure would leave the pair standing.
        let on_10234: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f == 10_234)
            .map(|((w, o, what), (_, row))| format!("{w}/{o} {what}: {row}"))
            .collect();
        assert_eq!(
            on_10234,
            Vec::<String>::new(),
            "block 10234 parts — item 464's three rows of the citizen's \
             queued `GATHERORDER` are back, or `1/28`'s fifteen are"
        );
        // **The replan schedule, both sides and both directions** (item
        // 478, `docs/ROADS.md` §1.2). The human's farm `0/2004` falls on
        // sim-frame 10230; `Build::remove_from_city` calls
        // `City::regen_roads` on what is left of the city, so all six of
        // the survivors carry `build_masks & 0x100` from block **10231**
        // — and the **last** block that still carries it is the frame
        // `(frame + o) % 16 == 0` names, which for every one of them is
        // `12_240 - o`: the flag is cleared *during* that frame, and
        // block `n` is the state at its start. That arithmetic is the
        // assertion: a rule that
        // flagged the city on the wrong frame moves the first number, a
        // rule that fired the replans together collapses the six spans
        // onto one block, and a rule that never flagged anything — which
        // is what this crate had until this item, `Build::activate`
        // being its only caller of `City::regen_roads` — empties
        // `regen_ours` while `regen_dump` still holds all six.
        //
        // The two blocks that matter are `0/2006`'s last (10234) and
        // `0/2005`'s (10235): those are the two road searches
        // `great_lakes_s_word_draws_are_a_road_search_and_8186_spends_none`
        // counts at 198 and 73 nodes, and they are what held the word.
        let span = |m: &BTreeMap<i64, Vec<i64>>| -> Vec<(i64, i64, i64)> {
            let mut v: Vec<(i64, i64, i64)> = m
                .iter()
                .map(|(o, bs)| (*o, bs[0], bs[bs.len() - 1]))
                .collect();
            v.sort_unstable();
            v
        };
        let expected: Vec<(i64, i64, i64)> = [2_000, 2_001, 2_002, 2_003, 2_005, 2_006]
            .into_iter()
            .map(|o| (o, 10_231, 12_240 - o))
            .collect();
        assert_eq!(
            span(&regen_dump),
            expected,
            "run100's own replan flags are not the six the farm's death sets"
        );
        assert_eq!(
            span(&regen_ours),
            expected,
            "this crate's replan flags are not the dump's —              `City::regen_roads`' second writer (`Build::remove_from_city`)"
        );
        // **Anti-vacuity, in the block's own counts.** A capture without
        // `UNITS=3`, `BUILDS=7`, `CITIES=5` or `GUYS=4` would agree
        // everywhere by saying nothing, and this is the only assertion
        // that can tell the two apart.
        assert!(
            blocks >= 836 && compared >= 2_250_000,
            "run100's own blocks are missing — the wrong file: {blocks} blocks, {compared} rows"
        );
        let (units, builds, cities, leaders, pos, order, collide, city, build, queue, gather) =
            word_census.expect("the word's own block is inside run100's window");
        // Measured on the block itself, and a floor rather than an
        // equality: a record that grows is not a failure, one that
        // vanishes is. `leaders` is 4 because `LEADERS=1` still writes
        // one block a slot — the record whose `score` and `leader_flags`
        // nothing here compares (§54.4).
        //
        // **`builds` 29 → 28 on item 456**, and it is not a record that
        // vanished from the capture: the *word* moved 10162 → 10233, and
        // the original's own dump carries 29 building records on the
        // first block and 28 on the second — one building fewer standing
        // seventy-one frames later. Counted in run100 directly before
        // re-pinning, because "a record that vanishes is a failure" and
        // "the floor was measured on a different block" look identical
        // from inside this assertion.
        assert!(
            (units, builds, cities, leaders) >= (104, 28, 3, 4)
                && [pos, order, collide, city, build, queue, gather]
                    >= [60, 60, 300, 130, 80, 50, 900],
            "the word's block does not carry every record this widening claims to \
             compare: {units} units, {builds} builds, {cities} cities, {leaders} leaders; \
             {pos} pos, {order} order, {collide} collide, {city} city, {build} build, \
             {queue} queue, {gather} gather rows"
        );
    }

    pub(in super::super) fn run100_s_scholar_job_runs_to_the_word() {
        /// The frame the job is queued on; block `QUEUED + 1` is the
        /// first that carries it.
        const QUEUED: i64 = 9_382;
        /// `SCHOLARS`' type index, the `guy` every seated scholar in the
        /// window carries.
        const SCHOLAR: i64 = 52;
        let Some(r100) = dump("gamelog-run100-greatlakes-valuewindow2.txt") else {
            eprintln!("skipping: no run100 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r100).unwrap();
        let mut seen: Vec<(i64, Vec<(i64, i64)>)> = Vec::new();
        let mut born: Option<i64> = None;
        for n in QUEUED..=GREAT_LAKES_SIXTH_SCHOLAR + 1 {
            let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
                continue;
            };
            let frame = ix.frame_state(at).unwrap();
            let Some(b) = frame.builds.iter().find(|b| (b.who, b.o) == (1, 2_019)) else {
                continue;
            };
            let live = usize::try_from(b.queued.unwrap_or(0)).unwrap_or(0);
            seen.push((
                n,
                b.queue
                    .iter()
                    .take(live)
                    .map(|q| (q.ty, q.job_counter))
                    .collect(),
            ));
            if born.is_none() && frame.units.iter().any(|u| (u.who, u.o) == (1, 51)) {
                born = Some(n);
            }
        }
        assert_eq!(
            seen.len(),
            (GREAT_LAKES_SIXTH_SCHOLAR + 2 - QUEUED) as usize,
            "run100 does not cover [{QUEUED}, {}]: {} blocks",
            GREAT_LAKES_SIXTH_SCHOLAR + 1,
            seen.len()
        );
        // The block the job is not yet on, and the block it is gone from.
        assert_eq!(seen.first().map(|s| s.1.as_slice()), Some(&[][..]));
        assert_eq!(seen.last().map(|s| s.1.as_slice()), Some(&[][..]));
        let wrong: Vec<&(i64, Vec<(i64, i64)>)> = seen
            .iter()
            .filter(|(n, q)| {
                let want = if *n <= QUEUED || *n > GREAT_LAKES_SIXTH_SCHOLAR {
                    Vec::new()
                } else {
                    vec![(SCHOLAR, (100 * (n - QUEUED)).min(12_750))]
                };
                *q != want
            })
            .collect();
        assert!(
            wrong.is_empty(),
            "1/2019's scholar job is not +100 a frame to a clamped 12750: {wrong:?}"
        );
        // The price the ramp put on the sixth scholar, off the queue
        // slot's own `(good, cost)` pair: 40 **wealth**, where the four
        // already seated at `1/2020` were bought at 38.
        let at = ix
            .frames()
            .iter()
            .position(|x| x.number == GREAT_LAKES_SIXTH_SCHOLAR)
            .unwrap();
        let frame = ix.frame_state(at).unwrap();
        let b = frame
            .builds
            .iter()
            .find(|b| (b.who, b.o) == (1, 2_019))
            .unwrap();
        assert_eq!(
            (b.queue[0].good, b.queue[0].cost),
            ([2, -1, -1], [40, 0, 0]),
            "the sixth scholar's price is not 40 wealth"
        );
        assert_eq!(
            born,
            Some(GREAT_LAKES_SIXTH_SCHOLAR + 1),
            "1/51 is not standing in the block after the word"
        );
    }

    pub(in super::super) fn run97_s_window_clocks_are_the_original_s() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(trace), Some(r97)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            trace("rontrace-run53.log"),
            dump("gamelog-run97-greatlakes-valuewindow.txt"),
        ) else {
            eprintln!("skipping: no run53/run97 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let mut source = crate::capture::indexed::IndexedCapture::open(&path).unwrap();
        let setup = source.read_replay_setup().unwrap();
        let setup_log = Log::parse(&setup);
        let setup_init = setup_log.initial().unwrap();
        let (seeds, animations, guys) = source
            .with_replay_initial(|observed| {
                (
                    observed.frame_seeds.clone(),
                    observed.anim_lengths.clone(),
                    observed.frame_guys.clone(),
                )
            })
            .unwrap();
        let mut r97_source = crate::capture::indexed::IndexedCapture::open(&r97).unwrap();
        let theirs = r97_source
            .with_replay_initial(|initial| initial.frame_guys.clone())
            .unwrap();
        let window: Vec<i64> = theirs.iter().map(|(n, _)| *n).collect();
        assert!(
            window.first() == Some(&8_029)
                && window.last() == Some(&9_348)
                && window.len() >= 1_320,
            "run97's clock window is not [8029, 9348] in sim-frames: {:?}..{:?} ({})",
            window.first(),
            window.last(),
            window.len()
        );
        let last = *window.last().unwrap();
        with_sibling_initials(|refs| {
            let mut init = setup_init;
            init.frame_seeds = seeds;
            init.anim_lengths = animations;
            init.frame_guys = guys;
            init.frame_seeds.retain(|(frame, _)| *frame <= last);
            init.frame_guys.retain(|(frame, _)| *frame <= last);
            borrow_from_siblings(&mut init, refs);
            borrow_pasture(&mut init, &trace);
            let mut built = build_sim(&loaded, &init, Tuning::RON);
            built.frame_guys.clear();
            let mut compared = 0usize;
            let mut above = 0usize;
            let mut unmatched = 0usize;
            let mut walk = 0usize;
            // The walk-slot rows themselves, not only their count — item 352
            // closed the band at zero, so anything here is a handful and worth
            // naming rather than summing.
            let mut walkw: Vec<String> = Vec::new();
            let mut wrong: Vec<String> = Vec::new();
            let mut posw: Vec<String> = Vec::new();
            let mut body_bad = 0usize;
            let mut body_all = 0usize;
            let mut body_trio = 0usize;
            let mut frames_below = 0usize;
            let mut posfirst: std::collections::BTreeMap<(i64, i64), i64> = Default::default();
            for f in 0..=last {
                built.tick();
                let Some((_, units)) = theirs.iter().find(|(n, _)| *n == f) else {
                    continue;
                };
                if f < LONG_WORD_GREAT_LAKES {
                    frames_below += 1;
                }
                for state in units {
                    if !(0..8).contains(&state.who) {
                        continue;
                    }
                    let Some(u) = (0..built.sim.units.len()).find(|&i| {
                        let x = &built.sim.units[i];
                        x.alive()
                            && i64::from(x.owner) == state.who
                            && i64::from(x.index) == state.o
                    }) else {
                        unmatched += 1;
                        continue;
                    };
                    // **The rest of the record** (item 350). The clocks
                    // below are four fields of a `GUY` block; the `UNITDATA`
                    // that carries them prints the unit's own point and the
                    // goal of its leading move order beside them, and for a
                    // month neither was compared on this capture. The first
                    // run of this widening found sixteen units off position
                    // from 8442 and named the cause in twenty minutes
                    // (`docs/ARMY.md` §3.4).
                    {
                        let un = &built.sim.units[u];
                        let ourgoal = un.orders.front().and_then(|o| match &o.body {
                            sim::orders::Body::Move(m) => Some((m.dest.x, m.dest.y)),
                            _ => None,
                        });
                        // `-1` for "no leading move order" on both sides:
                        // whether a unit *has* a goal is half the comparison,
                        // and a `None`/`Some` mismatch is a divergence.
                        let rows: [(&str, i64, i64); 4] = [
                            ("x", i64::from(un.pos.x), state.pos.x),
                            ("y", i64::from(un.pos.y), state.pos.y),
                            (
                                "goal_x",
                                ourgoal.map_or(-1, |(x, _)| i64::from(x)),
                                state.goal.map_or(-1, |g| g.x),
                            ),
                            (
                                "goal_y",
                                ourgoal.map_or(-1, |(_, y)| i64::from(y)),
                                state.goal.map_or(-1, |g| g.y),
                            ),
                        ];
                        for (name, o, t) in rows {
                            if f >= LONG_WORD_GREAT_LAKES {
                                continue;
                            }
                            body_all += 1;
                            if o == t {
                                continue;
                            }
                            // The **standing trio**, older than any window
                            // on this map and already excused: `1/24`, `1/25`
                            // and `1/26` stand a constant **(24, 24)** off
                            // their cell, from before this capture opens
                            // (`docs/ORACLE.md`, run76/run83's rows). They
                            // never move and never draw, so they are counted
                            // apart rather than mixed into the live residue.
                            if state.who == 1 && (24..=26).contains(&state.o) {
                                body_trio += 1;
                            } else {
                                body_bad += 1;
                            }
                            posfirst.entry((state.who, state.o)).or_insert(f);
                            if posw.len() < 40 {
                                posw.push(format!(
                                    "frame {f}: {}/{} {name} ours {o} theirs {t}",
                                    state.who, state.o
                                ));
                            }
                        }
                    }
                    for (n, g) in state.guys.iter().enumerate() {
                        if !g.has_clock() {
                            continue;
                        }
                        let Some(ours) = built.sim.units[u].guys.get(n) else {
                            continue;
                        };
                        // The known seam, and the only one left below the
                        // word: a guy on the **walk category** whose slot
                        // this crate resolves to a different one of `SLOG` /
                        // `WALK` / `JOG` (`docs/ANIM.md` §4.3's speed test).
                        // Three slots of one category cost no draw, which is
                        // why the stream runs 139 frames past the first of
                        // them. Counted and floored rather than listed; a
                        // disagreement anywhere else is a failure.
                        let walking = i64::from(sim::anim::category(ours.anim)) == 8
                            || g.cur_anim.map(|a| sim::anim::category(a as i8)) == Some(8);
                        let rows: [(&str, i64, Option<i64>); 4] = [
                            ("cur_anim", i64::from(ours.anim), g.cur_anim),
                            ("cur_time", i64::from(ours.cur_time), g.cur_time),
                            ("end_time", i64::from(ours.end_time), g.end_time),
                            ("last_time", i64::from(ours.last_time), g.last_time),
                        ];
                        for (name, o, t) in rows {
                            let Some(t) = t else { continue };
                            if f >= LONG_WORD_GREAT_LAKES {
                                above += 1;
                                continue;
                            }
                            compared += 1;
                            if o == t {
                                continue;
                            }
                            if walking {
                                walk += 1;
                                if walkw.len() < 20 {
                                    walkw.push(format!(
                                        "frame {f}: {}/{} guy {n} {name} ours {o} theirs {t}",
                                        state.who, state.o
                                    ));
                                }
                                continue;
                            }
                            if wrong.len() < 20 {
                                wrong.push(format!(
                                    "frame {f}: {}/{} guy {n} {name} ours {o} theirs {t}",
                                    state.who, state.o
                                ));
                            }
                        }
                    }
                }
            }
            eprintln!(
                "run97 clocks: {compared} fields below the word, {walk} of them the \
             walk-slot seam, {above} above (printed, not pinned), {unmatched} \
             unmatched"
            );
            for w in &wrong {
                eprintln!("  {w}");
            }
            for w in &walkw {
                eprintln!("  WALK {w}");
            }
            eprintln!(
                "run97 bodies: {body_bad} of {body_all} point-and-goal fields below the word wrong ({body_trio} more on the standing trio over {frames_below} frames), on {} unit(s) — {posfirst:?}",
                posfirst.len()
            );
            for w in &posw {
                eprintln!("  BODY {w}");
            }
            assert!(
                compared >= 120_000,
                "run97's own clock rows are missing — the wrong file: {compared} below"
            );
            assert!(
                wrong.is_empty(),
                "run97's clocks parted outside the walk-slot seam: {wrong:?}"
            );
            // **The walk-slot band is closed** (item 352). It was on player
            // 1's army from run97 block 8443 to the word, and it cost the
            // same ~33.3 fields a frame while it was fifteen units' — 4,615
            // over the 139 frames `[8443, 8582)` on item 346 and 5,853 over
            // the 176 of `[8443, 8619)` on item 348, 33.5 apiece. Item 350
            // took it to 1,747 over the 185 frames `[8443, 8628)`, 9.4
            // apiece, by taking the probe's six out of the army
            // (`docs/ARMY.md` §3.4); item 352 took it to **zero** by putting
            // the army's muster on the original's own cell (§13's two
            // predicates — the `<= 4` spacing test and `mask_me`'s `0x4000`).
            // Measured on 352's own window, `[8443, 8663)`, the base is
            // **2,898, 13.2 a frame** — which is the division this comment
            // keeps asking for, run against itself: 1,747 and 2,898 are the
            // same simulation through two window widths.
            //
            // So this is an **equality** now, not a floor: item 347 read the
            // band as a free `walk_variant` choice costing no draw, and the
            // truth was that the nine were walking somewhere else. There is
            // no remaining seam for a walking guy's slot to differ in below
            // the word, and a successor that reopens one should say so here
            // rather than raise a ceiling.
            //
            // **Item 385 says so.** Moving the word 9182 → 9415 brought 233
            // frames that had never been compared under it, and two of them
            // carry a row: `1/35`'s guy 0 on 9338 and 9339, `cur_anim` 8
            // against 7 — one slot of the walk category, the seam this band
            // was always about and not a new one. The band over item 352's
            // own `[8443, 9182)` is still **empty**, which is what the
            // assertion below says by naming the two rows rather than
            // counting them: a count would have hidden which frames they are
            // on, and the frames are the whole point.
            //
            // **Item 539 closed those two**: `1/35`'s walk now keeps the
            // original's step, because the soft flag that halved it came from
            // the fast path's stride over an empty world cell
            // (`docs/COLLISION.md` §4.2). The band is empty over the whole
            // window below the word again.
            assert_eq!(
                walkw,
                Vec::<String>::new(),
                "run97's walk-slot band is not the empty set item 539 left it, \
             over the {} frames below the word",
                LONG_WORD_GREAT_LAKES - 8_443
            );
            assert!(
                walkw.iter().all(|w| !w.starts_with("frame 91")
                    && !w.starts_with("frame 90")
                    && !w.starts_with("frame 8")),
                "the walk-slot band reopened *below* 9182, which item 352 closed \
             at zero: {walkw:?}"
            );
            // **The rest of the record, floored** (item 350). Three families,
            // and only the second is anyone's current item:
            //
            // - the **standing trio** `1/24`/`1/25`/`1/26`, a constant
            //   (24, 24) older than every window on this map and already
            //   excused — counted apart, and exactly `3 × 2 × frames`, which
            //   is what the equality below says rather than a literal that a
            //   wider window breaks;
            // - ~~**`1/31`–`1/39` from 8442**~~ — **closed by item 352**: the
            //   frame is an `Army::find_target`, both sides spend its two
            //   `+0x7df` score draws, and the army's muster is the original's
            //   cell (58, 29) from `docs/ARMY.md` §13's two corrected
            //   predicates. All nine stand on the original's own
            //   `x_internal`/`y_internal` and hold its own path goal on 8442;
            // - **`1/27`/`1/28`/`1/29`/`1/41` from 8579**, and `1/33` from
            //   8584, the late drift on the long walk southwest — what the
            //   601 below is, together with `1/24`–`1/26`'s goal rows;
            // - and **`1/35` from 9329** — item 385's own two fields, on
            //   frames the word had always covered until it moved 9182 →
            //   9415. Below 9182 the count is still 6. Its neighbour is the
            //   walk-slot pair on 9338–9339 above: one unit, one late walk,
            //   and the only thing this item changed about it is that it is
            //   now compared.
            assert!(
                body_all >= 121_224,
                "run97's own bodies are missing — the wrong file: {body_all} fields"
            );
            // **Closed on item 629**: the trio are Great Lakes' Merchants,
            // and the (24, 24) was their seat — `cast_unpack`'s merchant arm
            // puts each on its tile corner (`docs/MERCHANT.md` §3.2). Their
            // two fields agree on every frame below the word.
            assert_eq!(
                body_trio, 0,
                "the standing trio is off again over {frames_below} frames: {body_trio}"
            );
            assert!(
                body_bad <= 8,
                "run97's point-and-goal residue grew: {body_bad} of {body_all} \
             fields below the word, the floor is 8 — {posfirst:?}"
            );
        });
    }

    pub(in super::super) fn run98_s_window_clocks_are_the_original_s() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr), Some(r98)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
            dump("gamelog-run98-eastindies-valuewindow.txt"),
        ) else {
            eprintln!("skipping: no run54/run98 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);

        let text98 = crate::capture::read(&r98);
        let theirs = Log::parse(&text98)
            .initial()
            .expect("run98 carries a start block")
            .frame_guys;
        let window: Vec<i64> = theirs.iter().map(|(n, _)| *n).collect();
        assert!(
            window.first() == Some(&7_879) && window.last() == Some(&8_788) && window.len() >= 910,
            "run98's clock window is not [7879, 8788]: {:?}..{:?} ({})",
            window.first(),
            window.last(),
            window.len()
        );
        let last = *window.last().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.frame_guys.clear();
        let mut compared = 0usize;
        let mut above = 0usize;
        let mut unmatched = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        // The seating's own rows, by value: the first frame this crate
        // and the dump agree the scholar is on a teach slot, and the
        // clock it runs on afterwards.
        let mut seating: Vec<(i64, i64, i64, i64)> = Vec::new();
        for f in 0..=last {
            built.tick();
            let Some((_, units)) = theirs.iter().find(|(n, _)| *n == f) else {
                continue;
            };
            for state in units {
                if !(0..8).contains(&state.who) {
                    continue;
                }
                let Some(u) = (0..built.sim.units.len()).find(|&i| {
                    let x = &built.sim.units[i];
                    x.alive() && i64::from(x.owner) == state.who && i64::from(x.index) == state.o
                }) else {
                    unmatched += 1;
                    continue;
                };
                let is_scholar = built.sim.worker_of(u) == sim::orders::Worker::Scholar;
                for (n, g) in state.guys.iter().enumerate() {
                    if !g.has_clock() {
                        continue;
                    }
                    let Some(ours) = built.sim.units[u].guys.get(n) else {
                        continue;
                    };
                    if is_scholar
                        && n == 0
                        && (EAST_INDIES_FIRST_SCHOLAR..EAST_INDIES_FIRST_SCHOLAR + 3).contains(&f)
                        && let (Some(a), Some(t), Some(e)) = (g.cur_anim, g.cur_time, g.end_time)
                    {
                        seating.push((f, a, t, e));
                    }
                    let rows: [(&str, i64, Option<i64>); 4] = [
                        ("cur_anim", i64::from(ours.anim), g.cur_anim),
                        ("cur_time", i64::from(ours.cur_time), g.cur_time),
                        ("end_time", i64::from(ours.end_time), g.end_time),
                        ("last_time", i64::from(ours.last_time), g.last_time),
                    ];
                    for (name, o, t) in rows {
                        let Some(t) = t else { continue };
                        // Past the word both streams are nobody's.
                        if f >= LONG_WORD_EAST_INDIES {
                            above += 1;
                            continue;
                        }
                        compared += 1;
                        if o == t {
                            continue;
                        }
                        if wrong.len() < 20 {
                            wrong.push(format!(
                                "frame {f}: {}/{} guy {n} {name} ours {o} theirs {t}",
                                state.who, state.o
                            ));
                        }
                    }
                }
            }
        }
        eprintln!(
            "run98 clocks: {compared} fields below the word, {above} above \
             (printed, not pinned), {unmatched} unmatched"
        );
        for w in &wrong {
            eprintln!("  {w}");
        }
        // **The whole window is below the word since item 346** — East
        // Indies' 8495 was the seated scholar's first wrap and the fix
        // carried the word to 9711, past 8788. So `above` is zero by
        // construction now, and the floor is the one that matters: every
        // one of run98's 126,508 clock fields is a checked prediction.
        assert!(
            compared >= 126_000,
            "run98's own clock rows are missing — the wrong file: \
             {compared} below, {above} above"
        );
        // **The scholar's teach slot, off the original's own dump.**
        assert_eq!(
            seating,
            vec![
                (EAST_INDIES_FIRST_SCHOLAR, 25, 1, 30),
                (EAST_INDIES_FIRST_SCHOLAR + 1, 25, 2, 30),
                (EAST_INDIES_FIRST_SCHOLAR + 2, 25, 3, 30),
            ],
            "run98's first scholar is seated on slot 25 — `variant 0 + \
             0x19` — and runs a 30-frame clock from the frame it enters"
        );
        // The whole residue, pinned by value: `1/0`'s walk slot on seven
        // frames, and nothing else in 84,888 fields — until item 1164.
        assert_eq!(
            wrong,
            // Item 1164 took the whole of it: `1/0`'s walk slot on seven
            // frames, 8242..8248, ours 7 against 8, was the scout ashore
            // turning at a zeroed average; it lands at the one it boarded
            // with (`docs/TRANSPORT.md` §6.4).
            Vec::<String>::new(),
            "run98's clock residue below the word, whole"
        );
    }

    pub(in super::super) fn run82_s_leader_record_is_the_original_s_wealth() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr), Some(r82)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
            dump("gamelog-run82-islands-merchantcast.txt"),
        ) else {
            eprintln!("skipping: no run54/run82 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r82).unwrap();
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        const GOODS: [&str; 6] = ["food", "timber", "wealth", "knowledge", "metal", "oil"];
        let mut stock: Vec<(i64, [i64; 6], [i64; 6])> = Vec::new();
        let mut rows = 0usize;
        for f in 0..6_930 {
            built.tick();
            let n = f + 1;
            if n < 6_860 {
                continue;
            }
            let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
                continue;
            };
            let raw = ix.read_frame(at).unwrap();
            let flog = Log::parse(&raw);
            let Some(block) = flog.leader_block(n, 1) else {
                continue;
            };
            let t = crate::diff::leader::theirs(&block);
            let mine = crate::diff::leader::rows(&loaded, &built, 1);
            let mut o = [0i64; 6];
            let mut y = [0i64; 6];
            for (g, name) in GOODS.iter().enumerate() {
                let k = format!("bucket[{g}:{name}]");
                o[g] = mine
                    .iter()
                    .find(|(m, _)| *m == k)
                    .map_or(i64::MIN, |(_, v)| *v);
                y[g] = t.get(&k).copied().unwrap_or(i64::MAX);
                rows += 1;
            }
            stock.push((n, o, y));
        }
        assert_eq!(stock.len(), 70, "run82's window is 6860..6929");
        let parted: Vec<String> = stock
            .iter()
            .filter(|(_, o, y)| o != y)
            .map(|(n, o, y)| format!("{n}: ours {o:?} theirs {y:?}"))
            .collect();
        assert_eq!(
            parted,
            Vec::<String>::new(),
            "player 1's stockpile on run82, {rows} rows"
        );
        // The value itself, off the dump, on the window's first block.
        assert_eq!(stock[0].2[2], 246, "run82's block 6860: wealth 246");
    }

    pub(in super::super) fn run99_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_EAST_INDIES.0;
        const TAIL: i64 = WIDENING_EAST_INDIES.1;
        /// Each capture's first block the walk reads it from: run99 opens
        /// on 8780, and the nine blocks it shares with run98 are the
        /// stanza's overlap check, so the walk takes them from run98.
        const FROM: [i64; 2] = [FIRST, 8_789];
        const WORD_BLOCK: i64 = EAST_INDIES_WORD_BLOCK;
        const MAKE_BLOCK: i64 = EAST_INDIES_MAKE_BLOCK;
        const SCHOLAR_BLOCK: i64 = EAST_INDIES_SCHOLAR_BLOCK;
        /// The block the purchase behind the sixth Scholar lands on:
        /// sim-frame 9576 writes block 9577.
        const PURCHASE_BLOCK: i64 = 9_577;
        /// The block warship `1/32` is born on (item 579): sim-frame 10186
        /// writes it, the first parting any of its inputs reaches.
        const BIRTH_BLOCK: i64 = 10_187;
        /// The word 10232's block (item 576's successor): sim-frame 10232
        /// writes it. Item 579 moved the word past it.
        const OLD_WORD_BLOCK: i64 = 10_233;
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr), Some(r98), Some(r99)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
            dump("gamelog-run98-eastindies-valuewindow.txt"),
            dump("gamelog-run99-eastindies-valuewindow2.txt"),
        ) else {
            eprintln!("skipping: no run54/run98/run99 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let mut ixs: Vec<crate::capture::indexed::IndexedCapture> = [&r98, &r99]
            .iter()
            .map(|p| crate::capture::indexed::IndexedCapture::open(p).unwrap())
            .collect();
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let players = built.sim.players.len();
        use std::collections::{BTreeMap, BTreeSet};
        let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
        let mut missing: BTreeSet<String> = BTreeSet::new();
        let (mut blocks, mut compared, mut leader_rows) = (0usize, 0usize, 0usize);
        let row_walk: Vec<(i64, i64)> = std::env::var("RON_ROW_WALK")
            .map(|w| {
                w.split(',')
                    .filter_map(|u| {
                        let (a, b) = u.split_once('/')?;
                        Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut walked: BTreeMap<(i64, i64), Vec<String>> = BTreeMap::new();
        let mut anims: BTreeMap<(i64, i64), (Option<i64>, Option<i64>)> = BTreeMap::new();
        // Every figure whose animation changes on either side on the
        // word's blocks: `(block, who, o, theirs changed, ours changed)`.
        let mut changed: Vec<(i64, i64, i64, bool, bool)> = Vec::new();
        for f in 0..=TAIL {
            built.tick();
            let n = f + 1;
            if n < FIRST {
                continue;
            }
            let c = FROM.iter().rposition(|&lo| n >= lo).unwrap();
            let ix = &mut ixs[c];
            let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
                continue;
            };
            let frame = ix.frame_state(at).unwrap();
            blocks += 1;
            debug_watch(&built, n);
            let mut here: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
            let (_, rows) = widen_block(&built, &frame, players, n, &mut here);
            compared += rows;
            for &(w, o) in &row_walk {
                let now: Vec<String> = here
                    .iter()
                    .filter(|((hw, ho, _), _)| (*hw, *ho) == (w, o))
                    .map(|((_, _, what), (_, row))| format!("{what}: {row}"))
                    .collect();
                let e = walked.entry((w, o)).or_default();
                if *e != now {
                    eprintln!("  row walk {n} {w}/{o}: [{}]", now.join(" · "));
                    *e = now;
                }
            }
            // The first parting block prints by default (parked 830, the
            // sixteenth pass; the note at the first of these four sites).
            if firsts.is_empty()
                || site_window_named("RON_STANDING").is_some_and(|(a, b)| (a..=b).contains(&n))
            {
                for ((w, o, what), (_, row)) in &here {
                    eprintln!("  standing {n} {w}/{o} {what}: {row}");
                }
            }
            for (k, v) in here {
                firsts.entry(k).or_insert(v);
            }
            // `RON_MAKE_WALK=<lo>-<hi>`: this crate's player-1 make list,
            // and the unit offers of the last `create_units`, on each block
            // of the window — the half of the purchase run99's `LEADERS=1`
            // record cannot print.
            if site_window_named("RON_MAKE_WALK").is_some_and(|(a, b)| (a..=b).contains(&n)) {
                let ai = &built.sim.ai[1];
                eprintln!(
                    "  make {n}: step {:?} script_step {} bucket {:?} offers@{} {:?}",
                    ai.step,
                    ai.script_step,
                    built.sim.ledgers[1].bucket,
                    ai.unit_offers_frame,
                    ai.unit_offers
                );
                if ai.make_list.list.iter().any(|m| m.t >= 0) {
                    for (k, m) in ai.make_list.list.iter().enumerate() {
                        eprintln!("    MAKE[{k}] {m:?}");
                    }
                }
            }
            // **Who changes animation, both sides** (`docs/COMBAT.md`
            // §44.2.1: the `theirs` side printed before a quiet row is
            // trusted): every figure whose guy 0 `cur_anim` changes on
            // either side on the word's blocks.
            for t in &frame.units {
                let Ok(o16) = i16::try_from(t.o) else {
                    continue;
                };
                let ours = u8::try_from(t.who)
                    .ok()
                    .and_then(|w| built.sim.unit_by_o(w, o16));
                let ta = t.guys.first().and_then(|g| g.cur_anim);
                let oa =
                    ours.and_then(|u| built.sim.units[u].guys.first().map(|g| i64::from(g.anim)));
                let before = anims.insert((t.who, t.o), (ta, oa));
                if !(WORD_BLOCK - 2..=WORD_BLOCK + 1).contains(&n) {
                    continue;
                }
                let Some((pt, po)) = before else { continue };
                if pt == ta && po == oa {
                    continue;
                }
                eprintln!(
                    "  anim {n} {}/{}: theirs ({},{}) a{:?}->{:?} | ours {:?} a{:?}->{:?}",
                    t.who,
                    t.o,
                    t.pos.x,
                    t.pos.y,
                    pt,
                    ta,
                    ours.map(|u| (built.sim.units[u].pos.x, built.sim.units[u].pos.y)),
                    po,
                    oa
                );
                changed.push((n, t.who, t.o, pt != ta, po != oa));
            }
            let raw = ix.read_frame(at).unwrap();
            let flog = Log::parse(&raw);
            for who in 0..2usize {
                let Some(block) = flog.leader_block(n, who as i64) else {
                    continue;
                };
                let t = crate::diff::leader::theirs(&block);
                let mine = crate::diff::leader::rows(&loaded, &built, who);
                for (k, v) in &mine {
                    let Some(&y) = t.get(k) else {
                        missing.insert(k.clone());
                        continue;
                    };
                    leader_rows += 1;
                    if *v != y {
                        firsts
                            .entry((who as i64, -1, format!("leader:{k}")))
                            .or_insert((n, format!("ours {v} theirs {y}")));
                    }
                }
            }
        }
        eprintln!(
            "run99 widening: {blocks} blocks [{FIRST}, {TAIL}], {compared} record rows, \
             {leader_rows} leader rows, {} keys parted",
            firsts.len()
        );
        let mut by_block: BTreeMap<i64, Vec<String>> = BTreeMap::new();
        for ((w, o, what), (f, row)) in &firsts {
            by_block
                .entry(*f)
                .or_default()
                .push(format!("{w}/{o} {what}: {row}"));
        }
        for (f, rows) in &by_block {
            let near = (WORD_BLOCK - 40..=WORD_BLOCK + 40).contains(f);
            let window =
                site_window_named("RON_DEBUG_ROWS").is_some_and(|(lo, hi)| (lo..=hi).contains(f));
            if near || window {
                for r in rows {
                    eprintln!("  f{f} {r}");
                }
            } else {
                eprintln!("  f{f}: {} keys", rows.len());
            }
        }
        pin_eq!(blocks, (TAIL - FIRST + 1) as usize, "the walk is whole");
        // **The leader half is blind, and that is a finding**: both
        // captures print `LEADERS=1`'s seven-key stub — `who tribe
        // defeated_by gov score leader_flags leader_flags2` — and not one
        // key of `diff::leader::rows`. The make list, the stockpile and
        // the sites are not on this disk anywhere in [7880, 10399]. Since
        // item 706 the stub's `gov` is compared, one key a leader a block.
        pin_eq!(
            leader_rows,
            2 * blocks,
            "run98/run99's leader record is the stub, and `gov` is its one key read"
        );
        pin!(missing.contains("bucket[2:wealth]"), "{missing:?}");
        // **The old word's blocks, 9710..9712, and the move's value diff**
        // (item 573). Until the fix they held one row, `1/28 unlinked`:
        // the original's Scholar `1/28` (type 52) is born seated at
        // University `1/2015`, (36000, 35808), on block 9712 —
        // `Guy::init_real+0x52` and `Guy::set_anim+0x97a <
        // Unit::go_inside+0x280`, the word's two extra draws — and this
        // crate had no such unit. It is born now, and what is left is the
        // birth record every seated Scholar in the window carries.
        let scholar: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (SCHOLAR_BLOCK - 2..=SCHOLAR_BLOCK).contains(f))
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        pin_eq!(
            scholar,
            ["9712 1/28 g.angle[0]: ours 1431655765 theirs 0"],
            "the sixth Scholar's birth"
        );
        // **The purchase, block 9577**, 135 blocks under the birth: the
        // original's University `1/2015` queues one Scholar on sim-frame
        // 9576 (`queue[scan].type 52`, `cost[0] 42` wealth), through
        // `economic.bhs`'s `at_least_type(who, 130, "Wealth")`. Until the
        // fix this crate's queued nothing: its player 1 held **114**
        // wealth, twenty short since the caravan's first homecoming, where
        // `City::new_caravan` paid `(Civic + 1) · 10` for the original's
        // `(Commerce + 1) · 10` (`docs/CARAVAN.md` §7.3, §9). A purchase
        // spends no draw (`docs/AI.md` §47.3), so the stream agreed for the
        // whole job. Nothing parts between it and the old word now.
        let purchase: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (PURCHASE_BLOCK..SCHOLAR_BLOCK - 2).contains(f))
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        pin_eq!(purchase, Vec::<String>::new(), "the purchase agrees");
        // **The old word's blocks, 9982..9984, and the move's value diff**
        // (item 576). Sim-frame 9983 is a `make_stuff` on both sides, and
        // until the fix they bought different things: the original queued
        // `1/2010` two, `1/2014` one and `1/2005` one, and this crate queued
        // two at `1/2005`, placed a Mine, `1/2017`, and sent `1/6` to build
        // it — 27 rows on 9984, ours 212 draws against 15. The make lists
        // run139 prints parted two frames under it, on `create_units`' ship
        // offers at a Dock that belongs to no city (`docs/AI.md` §57). With
        // the sea branch's `find_building`, nothing parts here.
        let on_make: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (MAKE_BLOCK - 2..=MAKE_BLOCK).contains(f))
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        pin_eq!(on_make, Vec::<String>::new(), "the make_stuff agrees");
        // **The old word's blocks, 10231..10233, and the move's value
        // diff** (item 579). Sim-frame 10232 spent 33 draws here against
        // 34, parting at index 30: the original's `Unit::do_move+0xe84`.
        // The navy's first tick ordered Trireme `1/32` to the muster, and
        // the original's straight line refused it — the ship stood inside
        // its dock's `BAD_PATH` margin, which a warship may not cross
        // (`invalid_loc` answers 3) — so it paid the grid draw and gave
        // up; this crate had no navy (the ship stood idle with `stance 0`,
        // `idle 5`, `form_mod -1`) and, once it had one, no margin. With
        // `init_navy`, the margin and the hazard arm, nothing parts here.
        let on_old: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (OLD_WORD_BLOCK - 2..=OLD_WORD_BLOCK).contains(f))
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        pin_eq!(
            on_old,
            Vec::<String>::new(),
            "the navy's first order agrees"
        );
        // **The old word's blocks, 10397..10399, and the move's value
        // diff** (item 588) — run99's last three. Sim-frame 10398 spent 4
        // draws here against 5, parting at index 0: the original's
        // `Guy::set_anim+0x97a < Unit::do_idle+0x7d`. The Bark `1/34`,
        // trained on 10323, had walked to the navy 21 units behind the
        // original's since its first step and arrived still moving; the
        // original's idled on 10399. Its first step was halved: `do_move`'s
        // waypoint probe found Trireme `1/32` soft beside the muster and
        // raised the one-shot half step, where the original's
        // `detect_unit_collision` never scans for a ship on a probe that
        // does not ask for `boats` (`docs/COLLISION.md` §13). With the
        // sea arm, nothing parts here — the seated Scholar `1/24`'s
        // one-sided animation change on 10399 with it.
        let on_word: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (WORD_BLOCK - 2..=WORD_BLOCK).contains(f))
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        pin_eq!(
            on_word,
            Vec::<String>::new(),
            "the Bark arrives with the original's"
        );
        // **And under it, the standing floor**: 132 keys under the
        // purchase — 75 on the walk's first block (7880: `form -1`
        // against 9 on every figure, the human city's census, `1/19`/
        // `1/20`'s 24-unit offset), the scout `1/0`'s walk slot and step on
        // 7969..8481, the census's gatherer count on 8576, and the seated
        // Scholars' birth records, `1/22`..`1/27` on 8467, 8675, 8882,
        // 9090, 9297 and 9505, each `orders` (+24, +24), `angle` 0 against
        // 1431655765 and `form` 9 against -1, born on the original's frame
        // at the original's University every time — then `1/28`'s four on
        // 9712, and between the old word and the new only `1/0`'s clock on
        // 9815/9878/9879 and the newborn `1/29`'s `form` and facing (9815,
        // 9945). None of it spends a draw. **Item 576 added 78** between the
        // old word and the new, and none of them spends a draw either:
        // the seated Scholars `1/30` (10127) and `1/31` (10149, with its
        // `group` and `form_mod` on 10150), born with the same four-row
        // record as the five before them; every Citizen's `myhits` and
        // `hits_left` 40 against the original's 50, and `mylos` 2 against
        // 4, on 10165; and `1/32`, born on 10187 at (44184, 42888) against
        // (45192, 41880), in group 68 against 66 from 10188. **Item 579
        // took five of them and added 28** (219 → 242): `1/32` is born on the
        // original's point and is in the navy (group 68 against 66, a
        // numbering, from 10188); the seated Scholar `1/33`'s four-row
        // birth record on 10278; and the Bark `1/34`, in group 68 against
        // 66 from 10324 and a step behind the original's from its first
        // step on 10325 — its speed, its figure, its angle, its path's
        // length and its move's `last` — none of which spends a draw until
        // it arrives on the word. **Item 588 took 23 of them** (242 → 219):
        // every row of the Bark's walk but its group number, 68 against 66
        // from 10324 — the navy's numbering, as `1/32`'s. **Item 629 took
        // 22 more** (219 → 197): the Merchants `1/19` and `1/20` on their
        // tile corners, and what their footprints and the leader's rate
        // flag had held off (`docs/MERCHANT.md` §3.2).
        let under = firsts.values().filter(|(f, _)| *f < WORD_BLOCK - 2).count();
        // **Item 632 took three** (197 → 194): three computer
        // citizens' `form_mod`, ours −1 against the original's 50 from the
        // first block, now agreeing. `action_move_near` writes the width
        // twin on every member, citizens too (`docs/GROUPS.md` §24).
        // **Item 661 took four** (194 → 190): who=1's two cities'
        // `trade_val` and `vans.length`, which were compared against a
        // literal 0 and agree once the rows read the sim's own (`docs/AI.md`
        // §63).
        // **Item 718 took four (190 → 186)**: trained units' `orders_x`/`orders_y`
        // on their exit block, `come_out`'s `update_action` on the captain
        // (`docs/ORDERS.md` §29), and nothing arrived.
        // **Item 723 added one (186 → 187)**, all `order:move.facing`, the
        // formation mirror (parked 275): with `push_group` keeping an equal
        // group's record, `1/0`'s first parts on 7969 (was 8481), `1/22`
        // gains one on 7993, and `1/29`'s moves 9945 → 9815. None spends a
        // draw.
        // **Item 736 added one (187 → 188)**: the new `mirror` row
        // (`unit_masks & 2`, `docs/GROUPS.md` §26.4) parts on the scout `1/0`
        // from 8242, ours 0 against 1, beside its `order:move.facing`, and it parts so with or without item 736's fix: a turn
        // this crate flips the bit on and the original does not.
        // **Item 752 took two (188 → 186)**: who=1's two cities'
        // `gatherers`, a gatherer counted in its building's city and not
        // the nearest (`docs/AI.md` §73).
        // **Item 803 took one (186 → 185)**: the scout `1/0`'s `dest_angle`
        // on 8242, ours 531955712 against 479789056, agreeing now:
        // `eject_contents`' `update_action` is step 1's and `come_out`'s
        // `set_angle` leaves `+0x58` (`docs/TRANSPORT.md` §15). Its
        // `orders_x/y` there still part, ours (29592, 11160) → (23223,
        // 17572) against (28680, 12792): a later write in the frame.
        // **Item 850 took four (185 → 181)**: a unit that goes inside in
        // its own work takes its figures' frame (`docs/ANIM.md` §15). Three
        // keys leave 7993 and two of them first part on 8242, the scout
        // `1/0`'s disembark; one each leaves 9815, 9878 and 9879.
        // **Item 870 took seven (181 → 174)**: `1/11`'s `group` on 7880
        // (64 here, 65 there), the scout `1/0`'s on 7969 (66 against 68)
        // and its `mirror` on 9960, and `order:move.facing` on `1/22` and
        // `1/29`: who=1's pool numbering, the original's since item 870 (`docs/GROUPS.md` §29). The scout's `order:move.facing` now first
        // parts on 8481, 0 here against 1 there: the slot's kept `facing`
        // (§28), not built.
        // **Item 880 took that one (174 → 173)**: the scout `1/0`'s
        // `order:move.facing` on 8481, 0 here against 1 there, agrees now —
        // the slot keeps its record's `facing` and the record's history is
        // the original's (`Group::kill`'s clear, `docs/GROUPS.md` §30).
        // **Item 1164 took twenty-four (169 → 145)**: the scout `1/0`'s
        // disembark on 8242 and what followed it, its figures ashore at the
        // average they boarded with and its order point the landing
        // (`docs/TRANSPORT.md` §6.4).
        // Item 1291 took one: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1354 took fifteen (118 → 103): who=1's Citizens' `mylos`
        // standing on 10164, ours 4 against the original's cached 2 — the
        // Militia term the original had not yet refreshed, which this
        // crate's cache now holds back too (`docs/VISION.md` §2).
        pin_eq!(under, 67, "the floor under the word");
        // **The birth under the old word, 10187..10188** (item 579): the
        // first block any of `1/32`'s inputs parts on is its own birth.
        // Trireme `1/32` (type 340) is trained at Dock `1/2010`, (44160,
        // 41856), on the same frame on both sides, from a queue that agrees
        // on every block. `come_out`'s ring is 1008 on both, and the
        // original put it due **east**, (45192, 41880), where this crate
        // put it due **south**, (44184, 42888). Two things were missing
        // (`docs/ORDERS.md` §25): `find_nearby_spot`'s warship clause,
        // which refuses a `0x2400` tile to a sea type with an attack, and
        // `BuildType::mask_me`'s `is(DOCK)` arm, which marks every ocean
        // tile within three of a dock's footprint `BAD_PATH`. The first
        // alone put it on the ring's third bearing, (44568, 42792); with
        // the margin it is the original's point, and the position, the
        // figure's and its `des` agree from the birth on. What is left is
        // the order point (the Dock's own, here), `form`, and the navy's
        // group number, 68 against 66.
        let birth: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (BIRTH_BLOCK..=BIRTH_BLOCK + 1).contains(f))
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        pin_eq!(
            birth,
            [
                // Item 870: the navy's `group` on 10188, 68 here against
                // 66, agrees (`docs/GROUPS.md` §29).
            ] as [&str; 0],
            "the warship's birth"
        );
        let placed: Vec<&String> = firsts
            .keys()
            .filter(|(w, o, what)| {
                (*w, *o) == (1, 32)
                    && ["pos", "g.x[0]", "g.y[0]", "g.des_x[0]", "g.des_y[0]"]
                        .contains(&what.as_str())
            })
            .map(|(_, _, what)| what)
            .collect();
        pin_eq!(
            placed,
            Vec::<&String>::new(),
            "1/32 is born on (45192, 41880)"
        );
        // **Who changes animation, both sides** (`docs/COMBAT.md`
        // §44.2.1): on the old word's blocks, 10397..10399 (run99 ends
        // there). Until item 588 three changes were one-sided — `1/34` on
        // ours on 10398, and on theirs on 10399 `1/34` (8 → 0, the stand)
        // and the seated Scholar `1/24` (29 → 30). Now both change on
        // 10399 on both sides, and nothing is one-sided. (On the old word
        // 10232, 10231..10234, five figures changed and every one on both
        // sides; on 9984 it was `1/6` and gaia's `8/0`, and parked 577 is
        // `compare`'s blind spot for gaia.)
        let one_sided: Vec<(i64, i64, i64, bool, bool)> =
            changed.iter().filter(|c| c.3 != c.4).copied().collect();
        pin_eq!(one_sided, [], "a figure moves on one side only");
        pin_eq!(
            changed,
            [(10399, 1, 24, true, true), (10399, 1, 34, true, true)],
            "the Bark stands and the Scholar turns a page, both sides"
        );
    }

    pub(in super::super) fn run139_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_EAST_INDIES_MAKE.0;
        const TAIL: i64 = WIDENING_EAST_INDIES_MAKE.1;
        const MAKE_BLOCK: i64 = EAST_INDIES_MAKE_BLOCK;
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr), Some(r139)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
            dump("gamelog-run139-eastindies-makelist.txt"),
        ) else {
            eprintln!("skipping: no run54/run139 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r139).unwrap();
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let players = built.sim.players.len();
        use std::collections::{BTreeMap, BTreeSet};
        let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
        let mut missing: BTreeSet<String> = BTreeSet::new();
        let (mut blocks, mut compared, mut leader_rows) = (0usize, 0usize, 0usize);
        for f in 0..=TAIL {
            built.tick();
            let n = f + 1;
            if n < FIRST {
                continue;
            }
            let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
                continue;
            };
            let frame = ix.frame_state(at).unwrap();
            blocks += 1;
            let (_, rows) = widen_block(&built, &frame, players, n, &mut firsts);
            compared += rows;
            let raw = ix.read_frame(at).unwrap();
            let flog = Log::parse(&raw);
            for who in 0..2usize {
                let Some(block) = flog.leader_block(n, who as i64) else {
                    continue;
                };
                let t = crate::diff::leader::theirs(&block);
                let mine = crate::diff::leader::rows(&loaded, &built, who);
                let slots = site_window_named("RON_MAKE_SLOTS")
                    .is_some_and(|(a, b)| who == 1 && (a..=b).contains(&n));
                if slots {
                    let o: BTreeMap<&str, i64> =
                        mine.iter().map(|(k, v)| (k.as_str(), *v)).collect();
                    eprintln!(
                        "  block {n}, leader 1: step ours {:?} theirs {:?}",
                        o.get("production_step"),
                        t.get("production_step")
                    );
                    for i in 0..11 {
                        let g = |k: &str| {
                            let key = format!("MAKE[{i}].{k}");
                            (o.get(key.as_str()).copied(), t.get(&key).copied())
                        };
                        let (ot, tt) = g("t");
                        let (ov, tv) = g("val");
                        let (oc, tc) = g("city");
                        let (on, tn) = g("num");
                        let (ok, tk) = g("cat");
                        eprintln!(
                            "    {i:>2}  ours t {ot:?} val {ov:?} city {oc:?} num {on:?} cat {ok:?}   \
                             theirs t {tt:?} val {tv:?} city {tc:?} num {tn:?} cat {tk:?}"
                        );
                    }
                }
                for (k, v) in &mine {
                    let Some(&y) = t.get(k) else {
                        missing.insert(k.clone());
                        continue;
                    };
                    leader_rows += 1;
                    if *v != y {
                        firsts
                            .entry((who as i64, -1, format!("leader:{k}")))
                            .or_insert((n, format!("ours {v} theirs {y}")));
                    }
                }
            }
        }
        eprintln!(
            "run139 widening: {blocks} blocks [{FIRST}, {TAIL}], {compared} record rows, \
             {leader_rows} leader rows, {} keys parted, {} keys unprinted",
            firsts.len(),
            missing.len()
        );
        let mut by_block: BTreeMap<i64, Vec<String>> = BTreeMap::new();
        for ((w, o, what), (f, row)) in &firsts {
            by_block
                .entry(*f)
                .or_default()
                .push(format!("{w}/{o} {what}: {row}"));
        }
        for (f, rows) in &by_block {
            for r in rows {
                eprintln!("  f{f} {r}");
            }
        }
        pin_eq!(blocks, (TAIL - FIRST + 1) as usize, "the walk is whole");
        // **The leader half is whole**: every key `leader::rows` carries is
        // on run139's record, both leaders, all forty blocks. 1,055 before
        // item 785, which compares `wonder_mark` (`docs/AI.md` §75), and
        // 1,056 before item 883, which compares the three tech counters,
        // and 1,059 before item 1147, which compares `pop_cap`, and 1,060
        // before item 1209, which compares `agendas[·]`, and 1,062 before
        // item 1293, which compares `flock_stamp`.
        pin_eq!(leader_rows, 85760, "40 blocks x 2 leaders x 1,065 keys");
        pin_eq!(missing, BTreeSet::new(), "no key unprinted");
        // **Where each key first parts, block by block.** The 169 on the
        // window's first block are standing residue under the cycle: the
        // human leader's census, which this crate does not keep; the site
        // list's order and regions; `leftover`'s fractions; and
        // `tech_frame`/`tech_cat_frame`, which this crate never writes.
        // Nothing there feeds `create_units`' ship offers (§57).
        let counts: Vec<(i64, usize)> = by_block.iter().map(|(f, r)| (*f, r.len())).collect();
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was [(9960, 101), (9982, 7), (9992, 1)].
        // Item 1458: [(9960, 77)] → [(9960, 76)]; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
        pin_eq!(counts, [(9960, 36)], "first partings per block");
        // **The lists agree on block 9982, `create_units`' frame** (sim-frame
        // 9981). Both are empty on 9979, and `research_techs`,
        // `upgrade_units` and `create_units` fill them slot for slot. Until
        // item 576 the original's `create_units` offered three ships here,
        // at `val 9999999`, city 0, cat 6 — types 340 (slot 2), 334 (slot 3)
        // and 323 (slot 6) — and this crate none: seventeen rows on 9982,
        // with Mercenaries and Militia in slots 2 and 3, and on 9983 the
        // Mine in slot 3 where the original's ship stands; on 9984, the
        // `make_stuff` that bought it, 38 rows. The sea branch's dock is a
        // `find_building` around the city, and East Indies' Dock `1/2010`
        // belongs to no city (§57). What is left are the `city` rows, §52.2's
        // index shift, and not a value.
        let list: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (MAKE_BLOCK - 2..=MAKE_BLOCK).contains(f))
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(list, Vec::<String>::new(), "the make list, slot for slot");
        // **One row past the word, and it spends no draw**: player 1's
        // `gather_stamp` on 9992, ours 9991 against the original's 9719 —
        // this crate's gather clock stamped 9991 and the original's has not
        // moved since 9719. Before the fix `income[1]` and `resources[1]`
        // parted beside it (1280 against 1600); now nothing moves with it
        // inside the window. Not this item's (§57.4).
        let past: Vec<String> = firsts
            .iter()
            .filter(|(k, (f, _))| *f > MAKE_BLOCK && k.2 == "leader:gather_stamp")
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        pin_eq!(past, Vec::<String>::new(), "the gather clock");
    }

    pub(in super::super) fn run155_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_EAST_INDIES_EXPLORE.0;
        const TAIL: i64 = WIDENING_EAST_INDIES_EXPLORE.1;
        /// The block run155 was taken to widen, the word 11069's.
        const WORD_BLOCK: i64 = EAST_INDIES_EXPLORE_BLOCK;
        let Some(Widened {
            firsts,
            missing,
            blocks,
            leader_rows,
            changed,
            housed,
            ..
        }) = widen_east_indies(
            "run155",
            "gamelog-run155-eastindies-longword.txt",
            WIDENING_EAST_INDIES_EXPLORE,
            &[WORD_BLOCK],
            true,
        )
        else {
            return;
        };
        pin_eq!(blocks, (TAIL - FIRST + 1) as usize, "the walk is whole");
        pin_eq!(
            leader_rows,
            1301500,
            "250 blocks x 2 leaders x (1,065 + 1,531) keys"
        );
        pin_eq!(
            missing,
            std::collections::BTreeSet::new(),
            "no key unprinted"
        );
        let row = |((w, o, what), (f, row)): (&(i64, i64, String), &(i64, String))| {
            format!("{f} {w}/{o} {what}: {row}")
        };
        // **Under the word and on it, both directions, and the move's
        // value diff** (item 620). Two leader rows stand, 613's residue:
        // food a unit off from 11064, metal from 11037. Until the fix three
        // more parted on the word: the citizen `1/11` walked its
        // `EXPLORE_TO` on the original's line displaced by (15, −29), about
        // a step and a half behind, on every block of the window. So the
        // original popped its waypoint on frame 11068 (block 11069: path 2
        // against 3, and `dest` cleared) and met the refused straight line
        // on 11069, where it spent `do_move`'s grid roll and planned ten
        // waypoints (block 11070); this crate popped a frame later and
        // rolled on 11070. The displacement began on 10959, a Mine tile
        // scored with `dist_mod` 10 where the original caps it at 3
        // (`run152_s_word_frame_is_widened_whole`).
        let under: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (FIRST + 1..=WORD_BLOCK).contains(f))
            .map(row)
            .collect();
        // Item 629 closed the two leader rows too: the Merchants' seat
        // (`docs/MERCHANT.md` §3.2) raises the leader's `0x2000000`, and
        // the rates reassemble on the original's cadence.
        pin_eq!(
            under,
            Vec::<String>::new(),
            "the rows under the word and on it"
        );
        // **The position that carried it**: until the fix `1/11` stood on
        // the window's first block at (33963, 33784) against (33978,
        // 33755). Now its walk agrees on every block.
        pin_eq!(firsts.get(&(1, 11, "pos".into())), None, "`1/11`'s walk");
        // **Who changes animation, both sides** (`docs/COMBAT.md`
        // §44.2.1): nobody on one side only. Our `set_anim+0x97a` at
        // index 0 of the word is the original's index 1.
        let one_sided: Vec<(i64, i64, i64, bool, bool)> =
            changed.iter().filter(|c| c.3 != c.4).copied().collect();
        pin_eq!(
            one_sided,
            Vec::<(i64, i64, i64, bool, bool)>::new(),
            "a figure moves on one side only"
        );
        // **The outermost container, `compare`'s own row now** (parked
        // 598): every unit, every block, and no parting. The row reads a
        // housed unit 3,000 times over the window: twelve on block 11070,
        // the Scholars seated in the University 2015 and the unit in 2016,
        // so the agreement is not a window with no garrison in it.
        pin!(
            !firsts.keys().any(|(_, _, what)| what == "inside"),
            "a unit in another container"
        );
        pin_eq!(housed, 3_000, "housed unit-blocks the row compared");
        // **The floor**: 289 keys standing on the window's first block
        // (run152's residue on its last), 291 before the word, and 307 in
        // all. Before the fix it was 369/371/897: `1/11`'s walk, and past
        // the word every unit the unfixed stream parted from 11070 — `1/36`
        // on the same Mine from 11084, the human's `0/4` from 11081, gaia's
        // animals and the rest.
        let under_n = firsts.values().filter(|(f, _)| *f < WORD_BLOCK - 2).count();
        let first = firsts.values().filter(|(f, _)| *f == FIRST).count();
        // Item 629: 289/291/307 → 261/261/277, the two Merchants' seats,
        // city `1/2007`'s space and the rate rows (`docs/MERCHANT.md` §3.2).
        // **Item 632 took three** (261/261/277 → 258/258/274): three computer
        // citizens' `form_mod`, ours −1 against the original's 50 from the
        // first block, now agreeing. `action_move_near` writes the width
        // twin on every member, citizens too (`docs/GROUPS.md` §24).
        // **Item 661 took four** (258/258/274 → 254/254/270): who=1's two cities'
        // `trade_val` and `vans.length`, which were compared against a
        // literal 0 and agree once the rows read the sim's own (`docs/AI.md`
        // §63).
        // **Item 688 took thirty-one** (254/254/270 → 223/223/233), all
        // who=1 `SITE` rows under the word, and six more past it: a founded
        // city wears the site values down (`City::fix_world_vals`,
        // `docs/AI.md` §67).
        // **Item 718 took two (223/223/233 → 223/223/231)**: trained units' `orders_x`/`orders_y`
        // on their exit block, `come_out`'s `update_action` on the captain
        // (`docs/ORDERS.md` §29), and nothing arrived.
        // **Item 723 took one (223/223/231 → 222/222/230)**: the scout's
        // formation mirror (parked 275), an equal group's record kept.
        // **Item 870 took seven (222/222/230 → 215/215/223)**: the
        // first block's `group` rows — `1/0` 70 here against 69, `1/11` 64
        // against −1, `1/31` 66 against 65 and `1/32`..`1/38` 68 against
        // 66: who=1's pool numbering, the original's since item 870 (`docs/GROUPS.md` §29).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (147, 147, 151).
        // Item 1330 took 30 (139/139/143 → 109/109/112): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        pin_eq!((first, under_n, firsts.len()), (38, 38, 38), "the floor");
    }

    pub(in super::super) fn run159_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_EAST_INDIES_IDLE.0;
        const TAIL: i64 = WIDENING_EAST_INDIES_IDLE.1;
        /// The block run159 was taken to widen, the word 11590's.
        const WORD_BLOCK: i64 = EAST_INDIES_IDLE_BLOCK;
        /// The block of the word 629 moved to, 11747's (item 642).
        const CAST_BLOCK: i64 = EAST_INDIES_CAST_BLOCK;
        let Some(Widened {
            firsts,
            missing,
            blocks,
            leader_rows,
            changed,
            housed,
            standing,
            ..
        }) = widen_east_indies(
            "run159",
            "gamelog-run159-eastindies-idleword.txt",
            WIDENING_EAST_INDIES_IDLE,
            &[WORD_BLOCK, CAST_BLOCK],
            true,
        )
        else {
            return;
        };
        pin_eq!(blocks, (TAIL - FIRST + 1) as usize, "the walk is whole");
        pin_eq!(
            leader_rows,
            3279780,
            "630 blocks x 2 leaders x (1,065 + 1,531) keys"
        );
        pin_eq!(
            missing,
            std::collections::BTreeSet::new(),
            "no key unprinted"
        );
        let row = |((w, o, what), (f, row)): (&(i64, i64, String), &(i64, String))| {
            format!("{f} {w}/{o} {what}: {row}")
        };
        // **Under the word and on it, both directions** (item 629). The
        // leader rows are the census and, until item 688, the site list
        // (parked 622; `docs/AI.md` §67); until the fix 613's food and
        // metal a unit off stood here too, from
        // 11272 and 11275, and the sheep's three rows on 11578..11591. `1/41`, `1/42` and `1/43` are citizens born under the
        // window with 40 hit points against 50 (parked 622's `1/36`), and
        // `form` −1 against 9 beside them.
        let under: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (FIRST + 1..=WORD_BLOCK).contains(f))
            .map(row)
            .collect();
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            under,
            ["11363 1/42 g.angle[0]: ours 1431655765 theirs 0"],
            "the rows under the word and on it"
        );
        let scout: Vec<(i64, usize)> = {
            let mut by: std::collections::BTreeMap<i64, usize> = Default::default();
            for ((w, o, _), (f, _)) in &firsts {
                if (*w, *o) == (1, 0) && (FIRST + 1..=WORD_BLOCK).contains(f) {
                    *by.entry(*f).or_default() += 1;
                }
            }
            by.into_iter().collect()
        };
        // **The scout's island** (item 642). Until the fix the scout `1/0`
        // re-planned on 11550 for another target, 47 rows and then 20 more
        // on 11578..11579. Out of land on 11549, it asks
        // `think_civilian_transport` for an unscouted island. The original
        // sends it to cell (19, 13), on the human's home island: `go_here`
        // answers bit 2 there, because the human holds a city in the region
        // and this AI does not. This crate read the human's count from a
        // census it never runs, so it saw 0, and it sent the scout to (25,
        // 43) instead.
        pin_eq!(scout, [], "the scout's rows");
        // **The word was the sheep `8/1`'s arrival** (W1). Both sides roll
        // the same wander on 11576, from (28872, 23880) to (28680, 23736).
        // This crate walks it straight in 13 frames; the original plans
        // two waypoints, south to (28884, 23724) and then west, and walks
        // 21. It arrives on 11597, so its `Animal::do_idle` asks for
        // `DEFAULT` on 11598 and 11599, the pair this crate spends on 11590
        // and 11591. Until the fix that was the one-sided list, 11591 and
        // 11592; now nobody changes animation on one side only.
        let one_sided: Vec<(i64, i64, i64, bool, bool)> = changed
            .iter()
            .filter(|c| c.3 != c.4 && (WORD_BLOCK - 2..=WORD_BLOCK + 2).contains(&c.0))
            .copied()
            .collect();
        pin_eq!(
            one_sided,
            Vec::<(i64, i64, i64, bool, bool)>::new(),
            "a figure moves on one side only"
        );
        // **What the waypoint goes round** (W2): the Merchant `1/20`,
        // deployed since 7663, stood at the tile corner (28608, 24000)
        // there and at the unit-cell centre (28632, 24024) here — eleven
        // rows from the window's first block, both figures and the order
        // point with it — and its four tiles were blocked there only. The
        // corner tile (149, 124) is what the original's tile search refused
        // the sheep's diagonal through. The fix seats it and `1/19` on
        // their corners and blocks both footprints; `form` ~~is not this
        // arm's and stands~~ agrees since item 1330 (`Unit::init`'s).
        let merchant: Vec<String> = firsts
            .iter()
            .filter(|((w, o, _), _)| (*w, *o) == (1, 20))
            .map(row)
            .collect();
        pin_eq!(merchant, [] as [&str; 0], "the Merchant's seat");
        // **The word 11747's own blocks, every row, both directions**
        // (item 642). `firsts` keeps a key's first parting, so a key that
        // parted before the word is silent about the word's frame; these
        // are the rows standing on 11747 and 11748 that did not stand on
        // 11746, the block before the frame.
        let before = standing.get(&(CAST_BLOCK - 2)).cloned().unwrap_or_default();
        let on_word: Vec<String> = (CAST_BLOCK - 1..=CAST_BLOCK)
            .flat_map(|b| {
                let before = &before;
                standing
                    .get(&b)
                    .into_iter()
                    .flatten()
                    .filter(move |(k, _)| !before.contains_key(*k))
                    .map(move |((w, o, what), row)| format!("{b} {w}/{o} {what}: {row}"))
            })
            .collect();
        // Before the fix (committed as the floor, `6a86aeb`), 16 rows:
        // the scout `1/0` stopped on 11747 and took a cast order (kind 14,
        // two orders against one). On 11748 it was inside `1/44`, a boat
        // only this crate held, with its order list and path empty and its
        // figures' clocks reset. The original's scout was still walking a
        // 28-waypoint path. Now nothing is new on the frame.
        pin_eq!(
            on_word,
            Vec::<String>::new(),
            "the rows new on the word's blocks"
        );
        let one_sided_cast: Vec<(i64, i64, i64, bool, bool)> = changed
            .iter()
            .filter(|c| c.3 != c.4 && (CAST_BLOCK - 2..=CAST_BLOCK + 2).contains(&c.0))
            .copied()
            .collect();
        // Before the fix: the scout's boarding on 11748, and past it the
        // wraps of `1/28` and `1/27`, each moved a frame by the frame's
        // three extra draws.
        pin_eq!(
            one_sided_cast,
            Vec::<(i64, i64, i64, bool, bool)>::new(),
            "a figure moves on one side only on the word's blocks"
        );
        // **The outermost container** (parked 598). Until item 642 there
        // was one row, `1/0` inside `1/44` here from 11748 and on the map
        // there. Now both sides board it on the same frame.
        pin!(
            !firsts.keys().any(|(_, _, what)| what == "inside"),
            "a unit in another container"
        );
        // **What the boarding leaves** (item 642): the boat `1/44` is born
        // on both sides for block 11793. Its `form` is −1 here against 0
        // there, as the Merchant's is. Its order's `facing` is the scout's
        // own, which parted on the window's first block. The scout's
        // `avg_speed` and its second figure's `stopped` part on the same
        // block. None of them spends a draw before the word.
        let past: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f > WORD_BLOCK)
            .map(row)
            .collect();
        pin_eq!(
            past,
            [
                // Item 850: the scout `1/0`'s `g.avg_speed[0]` 25 against
                // 18, `g.avg_speed[1]` 24 against 18 and `g.stopped[1]` 0
                // against 1, agree (`docs/ANIM.md` §15).
                "11793 1/44 form: ours -1 theirs 0",
                // Item 723: `1/44`'s formation mirror (parked 275) is gone.
            ],
            "the rows past the old word"
        );
        pin_eq!(housed, 8_204, "housed unit-blocks the row compared");
        // **The floor**: 267 keys standing on the window's first block
        // (run155's residue on its last), 291 before the word, 296 in all.
        // Item 642 took it 267/358/803 → 267/291/296: the scout's 67 rows
        // under the old word, and 440 of the 445 past it, which the unfixed
        // boarding parted.
        // Before item 629 it was 297/391/920: the two Merchants' 22 seat
        // rows, city `1/2007`'s `filled` and `space[2]`, the leader's
        // `reg_land[11]` and `leftover` food and metal on 11270, the
        // buckets, the sheep, and past the word everything the unfixed
        // stream parted from 11594. Every row the fix moves in comes on
        // 11749 or later, past the new word's block.
        let under_n = firsts.values().filter(|(f, _)| *f < WORD_BLOCK - 2).count();
        let first = firsts.values().filter(|(f, _)| *f == FIRST).count();
        // **Item 632 took three** (270/361/806 → 267/358/803): three computer
        // citizens' `form_mod`, ours −1 against the original's 50 from the
        // first block, now agreeing. `action_move_near` writes the width
        // twin on every member, citizens too (`docs/GROUPS.md` §24).
        // **Item 661 took four** (267/291/296 → 263/287/292): who=1's two cities'
        // `trade_val` and `vans.length`, which were compared against a
        // literal 0 and agree once the rows read the sim's own (`docs/AI.md`
        // §63).
        // **Item 688 took thirty-seven** (263/287/292 → 228/250/255), all who=1 `SITE` rows and nothing
        // arriving: a founded city wears the site values down
        // (`City::fix_world_vals`, `docs/AI.md` §67).
        // Item 718: trained units' `orders_x`/`orders_y` on their exit block
        // now agree (`come_out`'s `update_action`, `docs/ORDERS.md` §29).
        // Item 723 took one under and one past (228/246/251 →
        // 227/245/249): the formation mirror (parked 275) on `1/44` and
        // the scout, an equal group's record kept.
        // **Item 850 took three past the word (227/245/249 →
        // 227/245/246)**: the scout `1/0`'s `g.avg_speed[0]`, `[1]` and
        // `g.stopped[1]` on 11793, agreeing (`docs/ANIM.md` §15).
        // **Item 870 took seven (227/245/246 → 220/238/239)**: the first
        // block's `group` rows — `1/0` 65 here against 64, `1/11` 64
        // against −1, `1/31` 66 against 65, `1/32`..`1/38` 68 against 66:
        // who=1's pool numbering, the original's since item 870 (`docs/GROUPS.md` §29).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (148, 158, 159).
        // Item 1330 took 31 (140/149/150 → 109/115/116): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        pin_eq!((first, under_n, firsts.len()), (37, 38, 39), "the floor");
    }
}
