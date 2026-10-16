# Shared isolated runner and persistent setup observations

User explicitly requests production isolation followed by persistent setup
observations, targeting 10× with lower CPU/RSS. Four to six simultaneous suites
now compete on this 16-core, 128-GiB host. Single-agent work, commander paused.
Baseline df3ea74b; previous isolated eight-slot experiment on2d61d180:
876 pass, 6 ignored, 3276 fixture requests, 0 missing; 236.87s vs ordinary
375.88s. CPU1553.73s; sampled tree6211MiB vs10562MiB. Not a10× result.

## Runner implementation checkpoint

Added resource_budget.py with cross-worktree kernel-owned CPU/memory tokens,
individual owned-process cleanup, preflight sampling, timeout and RSS enforcement.
Added isolated_tests.py with Cargo artifact discovery, every exact rondata test,
strict one-test verdicts, fresh output manifests/logs/counters and all remaining
workspace targets/doctests. --isolated gate mode coordinates other phases too.
Default legacy gate unchanged; explicit candidate option pending validation.
Scheduling profile is the measured 2d61d180 eight-slot experiment; unknown tests
reserve16GiB and run alone. No test is omitted for being new/slow/large.

Focused tests cover atomic admission, inherited orphan locks, six simultaneous
suite supervisors, memory/timeout/nonzero failures, unavailable sampling,
manifest duplicates, zero-test false success, unknown tests, once-only execution
and continued execution after a test failure. Initial killpg attempts were denied
by macOS even elevated; individual PID signaling, as used by memcap, succeeds.
This is an observed permission failure, not evidence of a product test defect.

This checkpoint is pending complete committed-tree validation. No simulation,
word, score, value, fixture selection or coverage change; pins/scoreboard remain
unchanged. Independent review owed. Persistent observations are next sequential
work after this runner is validated; no implementation or speed claim yet.
