use super::*;

#[test]
fn the_table_binds_every_shipped_call_and_nothing_else() {
    assert_eq!(
        HOST.len(),
        55,
        "the report's 55: 54 ScenarioFuncSet calls and rand_int"
    );
    assert_eq!(
        Signatures.signature("find_build_at_city", 4),
        Some((vec![Ty::Int, Ty::Str, Ty::Str, Ty::Int], Ty::Int))
    );
    assert_eq!(Signatures.signature("find_build_at_city", 3), None);
    assert_eq!(
        Signatures.signature("get_mapstyle", 0).map(|s| s.1),
        Some(Ty::Str)
    );
    assert!(Signatures.signature("print_game_msg", 1).is_none());
}

#[test]
fn timers_are_ticks_keyed_by_string_and_expiry_consumes() {
    let mut sim = Sim::new(crate::Tuning::RON, crate::World::new(2, 2), 1);
    let mut h = ScriptHost {
        sim: &mut sim,
        who: 0,
    };
    assert_eq!(h.set_timer("1", 0), BAD);
    assert_eq!(h.set_timer("", 5), BAD);
    assert_eq!(h.set_timer("1", 2), OK);
    assert_eq!(
        h.call("timer_expired", &[Value::Str("1".into())]).unwrap(),
        NO
    );
    assert_eq!(
        h.call("timer_expired", &[Value::Str("2".into())]).unwrap(),
        BAD
    );
    h.sim.frame = 2 * FRAMES_PER_TICK;
    assert_eq!(
        h.call("timer_expired", &[Value::Str("1".into())]).unwrap(),
        OK
    );
    // Consumed: asking again finds nothing.
    assert_eq!(
        h.call("timer_expired", &[Value::Str("1".into())]).unwrap(),
        BAD
    );
    assert_eq!(
        h.call("stop_timer", &[Value::Str("1".into())]).unwrap(),
        BAD
    );
    assert_eq!(h.set_timer("1", 3), OK);
    assert_eq!(h.call("stop_timer", &[Value::Str("1".into())]).unwrap(), OK);
}

#[test]
fn rand_int_is_the_sync_stream_with_an_exclusive_bound() {
    let mut sim = Sim::new(crate::Tuning::RON, crate::World::new(2, 2), 1);
    let mut oracle = sim.rng;
    let mut h = ScriptHost {
        sim: &mut sim,
        who: 0,
    };
    for _ in 0..50 {
        let v = h
            .call("rand_int", &[Value::Int(1), Value::Int(10)])
            .unwrap()
            .as_int();
        assert_eq!(v, oracle.get(1, 10));
        assert!((1..10).contains(&v));
    }
}

#[test]
fn a_bad_who_is_minus_one_everywhere_and_a_bad_name_too() {
    let mut sim = Sim::new(crate::Tuning::RON, crate::World::new(2, 2), 1);
    let mut h = ScriptHost {
        sim: &mut sim,
        who: 0,
    };
    for who in [0, 2, -1] {
        assert_eq!(h.call("age", &[Value::Int(who)]).unwrap(), BAD, "who {who}");
        assert_eq!(h.call("num_cities", &[Value::Int(who)]).unwrap(), BAD);
        assert_eq!(
            h.call("find_nation", &[Value::Int(who)]).unwrap(),
            Value::Str(String::new())
        );
    }
    assert_eq!(h.call("age", &[Value::Int(1)]).unwrap(), NO);
    assert_eq!(
        h.call("num_type", &[Value::Int(1), Value::Str("Nothing".into())])
            .unwrap(),
        BAD
    );
    assert_eq!(
        h.call("find_city_with_num", &[Value::Int(1), Value::Int(1)])
            .unwrap(),
        Value::Str(String::new())
    );
    assert_eq!(
        h.call("find_city_id", &[Value::Str(String::new())])
            .unwrap(),
        BAD
    );
}
