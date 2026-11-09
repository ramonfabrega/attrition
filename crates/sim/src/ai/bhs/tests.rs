use super::*;

/// A host with a few functions and a log of the calls it saw.
#[derive(Default)]
struct TestHost {
    log: Vec<(String, Vec<Value>)>,
    counter: i32,
}

impl Host for TestHost {
    fn signature(&self, name: &str, _nargs: usize) -> Option<(Vec<Ty>, Ty)> {
        Some(match name {
            "mark" => (vec![Ty::Int], Ty::Int),
            "next" => (vec![], Ty::Int),
            "name_of" => (vec![Ty::Int], Ty::Str),
            "set_timer" => (vec![Ty::Str, Ty::Int], Ty::Int),
            "minus_one" => (vec![], Ty::Int),
            _ => return None,
        })
    }
    fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, String> {
        self.log.push((name.to_string(), args.to_vec()));
        Ok(match name {
            "mark" => args[0].clone(),
            "next" => {
                self.counter += 1;
                Value::Int(self.counter)
            }
            "name_of" => Value::Str(if args[0].as_int() == 1 {
                "Capital".into()
            } else {
                String::new()
            }),
            "set_timer" => Value::Int(1),
            "minus_one" => Value::Int(-1),
            _ => return Err(format!("no host function {name}")),
        })
    }
}

fn load(src: &str) -> (Program, State, TestHost) {
    let host = TestHost::default();
    let p = Program::load(
        &["main.bhs"],
        &[Source {
            name: "main.bhs",
            text: src,
        }],
        &host,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let s = State::new(&p);
    (p, s, host)
}

fn run_int(p: &Program, s: &mut State, h: &mut TestHost, f: &str, args: &mut [Value]) -> i32 {
    run(p, s, h, f, args).unwrap().as_int()
}

#[test]
fn truth_is_greater_than_zero() {
    let (p, mut s, mut h) = load(
        "int ai t(int x) { if (x) return 10; return 20; }
             int ai n(int x) { return !x; }",
    );
    assert_eq!(run_int(&p, &mut s, &mut h, "t", &mut [Value::Int(1)]), 10);
    assert_eq!(run_int(&p, &mut s, &mut h, "t", &mut [Value::Int(0)]), 20);
    assert_eq!(
        run_int(&p, &mut s, &mut h, "t", &mut [Value::Int(-1)]),
        20,
        "-1 is false"
    );
    assert_eq!(run_int(&p, &mut s, &mut h, "n", &mut [Value::Int(-5)]), 1);
    assert_eq!(run_int(&p, &mut s, &mut h, "n", &mut [Value::Int(3)]), 0);
}

#[test]
fn arguments_evaluate_right_to_left() {
    let (p, mut s, mut h) = load(
        "int ai two(int a, int b) { return a * 10 + b; }
             int ai go() { return two(next(), next()); }",
    );
    // `next()` yields 1 then 2; the second argument is evaluated first.
    assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut []), 2 * 10 + 1);
}

#[test]
fn short_circuit_skips_the_host_call_and_yields_a_bit() {
    let (p, mut s, mut h) = load(
        "int ai go(int x) { return (x > 0) && mark(7); }
             int ai orr(int x) { return (x > 0) || mark(9); }",
    );
    assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(0)]), 0);
    assert!(h.log.is_empty(), "mark not called");
    assert_eq!(
        run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(1)]),
        1,
        "0/1, not 7"
    );
    assert_eq!(h.log.len(), 1);
    assert_eq!(run_int(&p, &mut s, &mut h, "orr", &mut [Value::Int(1)]), 1);
    assert_eq!(h.log.len(), 1);
}

#[test]
fn a_static_initialises_once_ever_and_persists() {
    let (p, mut s, mut h) = load(
        "int ai go(int who) {
               static int seed = next();
               static int hits = 0;
               hits++;
               return seed * 100 + hits;
             }",
    );
    assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(1)]), 101);
    assert_eq!(
        run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(2)]),
        102,
        "shared by callers"
    );
    assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(1)]), 103);
    assert_eq!(h.counter, 1, "the initialiser ran once");
}

/// A statement above the declarations does not wipe the statics.
///
/// `VirtualMachine::get_value`/`set_value` read and write a static on
/// `Script::static_vars` rather than through the call's frame, so the
/// value the last call left is live from the first instruction. This
/// simulation gives a static a frame slot and mirrors it back, and the
/// mirror used to run over slots the call had not yet declared — so any
/// script with an expression statement above its `static` block lost
/// every one of them on its *second* call, and only on the second.
/// `economic.bhs` has three such statements, and its `needed_citizens`
/// went back to zero on every call after the first: the AI stopped
/// training citizens for the rest of the game (`docs/AI.md` §17).
#[test]
fn a_statement_above_the_declarations_does_not_wipe_the_statics() {
    let (p, mut s, mut h) = load(
        "int ai go(int who) {
               mark(who);
               static int hits = 0;
               hits++;
               return hits;
             }",
    );
    assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(1)]), 1);
    assert_eq!(
        run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(1)]),
        2,
        "the second call is the one that used to lose it"
    );
    assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(1)]), 3);
}

#[test]
fn ref_parameters_write_back_and_labels_count_from_one() {
    let (p, mut s, mut h) = load(
        "labels { BLOCK_ON_THIS = 1, DONT_BLOCK_ON_THIS, SCRIPT_DONE, }
             int ai economic(int who, ref int step, int bvr, int loops) {
               step += 2;
               if (step > 4) return SCRIPT_DONE;
               return BLOCK_ON_THIS;
             }",
    );
    let mut args = [Value::Int(1), Value::Int(1), Value::Int(2), Value::Int(5)];
    assert_eq!(run_int(&p, &mut s, &mut h, "economic", &mut args), 1);
    assert_eq!(args[1], Value::Int(3), "the ref came back");
    assert_eq!(run_int(&p, &mut s, &mut h, "economic", &mut args), 3);
    assert_eq!(args[1], Value::Int(5));
}

#[test]
fn triggers_are_one_shot_inline_and_rearm() {
    let (p, mut s, mut h) = load(
        "int ai place(int who) {
               new_city = 0;
               enable_trigger(\"city_build\");
               trigger city_build() {
                 if (mark(who) > 1) { enable_trigger(\"health_check\"); }
                 else { enable_trigger(\"city_build\"); }
               }
               trigger health_check() {
                 return 1;
               }
               return -1;
             }",
    );
    // who = 1: city_build fires, re-arms itself (deferred), returns -1.
    assert_eq!(
        run_int(&p, &mut s, &mut h, "place", &mut [Value::Int(1)]),
        -1
    );
    assert_eq!(s.trigger_armed(&p, "place", "city_build"), Some(true));
    // who = 2: city_build fires and arms health_check, which runs in
    // the same call and returns 1.
    assert_eq!(
        run_int(&p, &mut s, &mut h, "place", &mut [Value::Int(2)]),
        1
    );
    assert_eq!(
        s.trigger_armed(&p, "place", "health_check"),
        Some(false),
        "fired, so cleared"
    );
}

#[test]
fn switch_falls_through_and_break_leaves_only_the_switch() {
    let (p, mut s, mut h) = load(
        "int ai go(int n) {
               int acc = 0;
               for (i = 0; i < 3; i++) {
                 switch (n) {
                 case 1: acc += 1;
                 case 2: acc += 10; break;
                 default: acc += 100; break;
                 }
               }
               return acc;
             }",
    );
    assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(1)]), 33);
    assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(2)]), 30);
    assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(9)]), 300);
}

#[test]
fn strings_compare_case_insensitively_and_mix_with_ints() {
    let (p, mut s, mut h) = load(
        "int ai go() {
               String n = name_of(1);
               int a = (n == \"CAPITAL\");
               int b = (n > -1);
               String none = name_of(2);
               int c = (none > -1);
               int d = !none;
               return a * 1000 + b * 100 + c * 10 + d;
             }",
    );
    assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut []), 1101);
}

#[test]
fn host_arguments_are_cast_to_the_declared_types() {
    let (p, mut s, mut h) = load("int ai go(int who) { return set_timer(who, 300); }");
    run_int(&p, &mut s, &mut h, "go", &mut [Value::Int(3)]);
    assert_eq!(h.log[0].1, vec![Value::Str("3".into()), Value::Int(300)]);
}

#[test]
fn division_truncates_and_a_zero_divisor_aborts() {
    let (p, mut s, mut h) = load("int ai go(int a, int b) { return a / b * 100 + a % b; }");
    assert_eq!(
        run_int(
            &p,
            &mut s,
            &mut h,
            "go",
            &mut [Value::Int(-7), Value::Int(2)]
        ),
        -300 - 1
    );
    assert!(matches!(
        run(
            &p,
            &mut s,
            &mut h,
            "go",
            &mut [Value::Int(1), Value::Int(0)]
        ),
        Err(RunError::Runtime(_))
    ));
}

#[test]
fn implicit_variables_are_function_scoped_and_typed_by_first_assignment() {
    let (p, mut s, mut h) = load(
        "int ai go() {
               if (1) { who_nation = name_of(1); }
               if (who_nation == \"Capital\") return 1;
               return 0;
             }",
    );
    assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut []), 1);
}

#[test]
fn falling_off_the_end_returns_the_default_and_not_found_is_an_error() {
    let (p, mut s, mut h) = load("int ai go() { int x = 3; } String ai nm() { }");
    assert_eq!(run_int(&p, &mut s, &mut h, "go", &mut []), 0);
    assert_eq!(
        run(&p, &mut s, &mut h, "nm", &mut []).unwrap(),
        Value::Str(String::new())
    );
    assert_eq!(
        run(&p, &mut s, &mut h, "nope", &mut []),
        Err(RunError::NotFound)
    );
    assert!(matches!(
        run(&p, &mut s, &mut h, "go", &mut [Value::Int(1)]),
        Err(RunError::Params(_))
    ));
}

#[test]
fn includes_share_labels_and_functions_and_a_library_static_is_shared() {
    let host = TestHost::default();
    let p = Program::load(
        &["a.bhs", "b.bhs"],
        &[
            Source {
                name: "a.bhs",
                text: "include \"lib.bhs\"\n int ai a(int who) { return helper(who) + DONE; }",
            },
            Source {
                name: "b.bhs",
                text: "include \"lib.bhs\"\n int ai b(int who) { return helper(who); }",
            },
            Source {
                name: "lib.bhs",
                text: "labels { DONE = 3, }\n int ai helper(int who);\n int ai helper(int who) { static int calls = 0; calls++; return calls; }",
            },
        ],
        &host,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let mut s = State::new(&p);
    let mut h = host;
    assert_eq!(
        run_int(&p, &mut s, &mut h, "a", &mut [Value::Int(1)]),
        1 + 3
    );
    assert_eq!(
        run_int(&p, &mut s, &mut h, "b", &mut [Value::Int(1)]),
        2,
        "one library, one static"
    );
    assert_eq!(p.files().len(), 3);
}

/// A host that accepts any function at any arity, answering `1` — enough
/// to load the shipped scripts and walk their first call.
struct AnyHost;

impl Host for AnyHost {
    fn signature(&self, name: &str, nargs: usize) -> Option<(Vec<Ty>, Ty)> {
        let ret = match name {
            "find_nation" | "find_city_with_num" | "get_mapstyle" => Ty::Str,
            _ => Ty::Int,
        };
        let mut params = vec![Ty::Int; nargs];
        // The string-taking parameters the shipped scripts use.
        for (i, p) in params.iter_mut().enumerate() {
            let is_str = match name {
                "num_type"
                | "num_type_with_queued"
                | "have_tech"
                | "can_pay_cost"
                | "research_tech_with_cost"
                | "researching_tech"
                | "find_build"
                | "find_inactive_build"
                | "find_unit"
                | "find_num_idle_unit" => i == 1,
                "at_least_type" | "train_unit_with_cost" | "train_unit_at_with_cost" => i == 2,
                "place_building_with_cost" | "place_building_upgrade_with_cost" => i >= 1,
                "place_orphan_building_with_cost" => i == 1,
                "find_build_at_city" | "num_city_buildings" => i == 1 || i == 2,
                "num_type_queued" => i == 2,
                "find_city_id" => i == 0,
                "was_city_attacked" | "was_city_raided" => i == 1,
                "set_timer" | "stop_timer" | "timer_expired" => i == 0,
                _ => false,
            };
            if is_str {
                *p = Ty::Str;
            }
        }
        Some((params, ret))
    }
    fn call(&mut self, name: &str, _args: &[Value]) -> Result<Value, String> {
        Ok(match name {
            "find_nation" | "find_city_with_num" | "get_mapstyle" => Value::Str("x".into()),
            _ => Value::Int(1),
        })
    }
}

/// The three shipped scripts load and run their first call — gated on
/// the install, as every install-backed test is.
#[test]
fn the_shipped_scripts_load_and_run_a_first_call() {
    let Some(install) = crate::testenv::install_root() else {
        eprintln!("skipping: no install (set RON_INSTALL)");
        return;
    };
    let dir = std::path::Path::new(&install).join("ai").join("scripts");
    let read =
        |n: &str| std::fs::read_to_string(dir.join(n)).unwrap_or_else(|e| panic!("{n}: {e}"));
    let (e, d, l) = (
        read("economic.bhs"),
        read("defensive.bhs"),
        read("aibestbuildlibrary.bhs"),
    );
    let sources = [
        Source {
            name: "economic.bhs",
            text: &e,
        },
        Source {
            name: "defensive.bhs",
            text: &d,
        },
        Source {
            name: "aibestbuildlibrary.bhs",
            text: &l,
        },
    ];
    let p = Program::load(&["economic.bhs", "defensive.bhs"], &sources, &AnyHost)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(p.files().len(), 3, "the library is loaded once");
    let mut s = State::new(&p);
    let mut h = AnyHost;
    for name in ["economic", "defensive"] {
        let mut args = [Value::Int(2), Value::Int(1), Value::Int(2), Value::Int(5)];
        let r =
            run(&p, &mut s, &mut h, name, &mut args).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        // Every host answer is 1: "attacked" → a barracks placed → SCRIPT_DONE.
        assert_eq!(r, Value::Int(3), "{name}");
    }
}

#[test]
fn a_float_declaration_fails_to_load() {
    let host = TestHost::default();
    let r = Program::load(
        &["m.bhs"],
        &[Source {
            name: "m.bhs",
            text: "int ai go() { float f = 1; return 0; }",
        }],
        &host,
    );
    assert!(r.is_err());
}

#[test]
fn the_shipped_shapes_parse() {
    // The constructs the three shipped scripts use, in one function.
    let (p, mut s, mut h) = load(
        "//Mark Sobota\n
             labels {\n BLOCK_ON_THIS = 1,\n DONT_BLOCK_ON_THIS,\n SCRIPT_DONE,\n}
             int ai assign_idle (int who);
             int ai assign_idle (int who) { return -1;; }
             int ai economic (int who, ref int step, int boom_vs_rush, int num_loops)
             {
               if ((mark(who) < 1)&&(mark(who) < 1)) return SCRIPT_DONE;
               my_capital = name_of(who);
               int return_value = BLOCK_ON_THIS;
               static int prev_step0 = 0;
               static int needed_techs = mark(who);
               int old_step;
               switch (who-1) {
               case 0:
                 old_step = prev_step0;
                 break;
               default:
                 old_step = 0;
                 break;
               }
               for (j = mark(who); j < 3; j++) {
                 if (mark(j) < 1) break;
               }
               while (mark(who) < 0) { break; }
               if (step == 1) {
                 if (mark(who) > 4 || mark(who) > 1) { return SCRIPT_DONE; }
                 size = mark(2);
                 if (size == 0) step = 1;
                 else if (size >= 2) step = 6;
                 else return SCRIPT_DONE;
               }
               for (i = 0; i < num_loops; i++) {
                 if (old_step != step) old_step = step;
                 switch (step) {
                 case 6:
                   step++;
                   return_value = BLOCK_ON_THIS;
                   break;
                 case 7:
                   step+=2;
                   return BLOCK_ON_THIS;
                   break;
                 default:
                   return_value = SCRIPT_DONE;
                   break;
                 }
               }
               switch (who-1) {
               case 0:
                 prev_step0 = old_step;
                 break;
               }
               return return_value;
             }",
    );
    let mut args = [Value::Int(1), Value::Int(1), Value::Int(2), Value::Int(5)];
    assert_eq!(run_int(&p, &mut s, &mut h, "economic", &mut args), 1);
    assert_eq!(args[1], Value::Int(9), "6 → 7 → +2");
    assert!(s.any_static_set());
}
