"""Process-local allocator policy for test children; no host configuration edits."""
import os
import sys


def bounded_test(name):
    """Measured repeated walks; other isolated tests keep the system policy."""
    return (name == 'diff::coverage::every_parsed_field_is_compared_by_the_instrument_or_pinned'
            or name.startswith('diff::third_pair::'))


# The coverage walk ran 1613-1693 s in the fifty-eighth tranche's whole
# gates and was killed at resource_budget's default 1800 s in two of the
# fifty-ninth's (green alone in 345 s; Loop 3681): it alone gets an hour
# until a pass measures it.
LONG_TESTS = {'diff::coverage::every_parsed_field_is_compared_by_the_instrument_or_pinned': 3600}


def execution_timeout(name):
    """Seconds an isolated test may run; None keeps the budget's default."""
    return LONG_TESTS.get(name)


# **`tools/check.sh test`'s execution timeout** (item 3974): the whole
# rondata lib was killed at the budget's fixed 1800 s (`third_pair::run661`),
# so a standalone test run gets a per-test allowance times the count of timed
# tests its Cargo arguments select, 1800 s the floor. The count is read off
# the lane gate's timings file (`release_gate.TIMINGS`, every rondata release
# test alone; nothing is built or listed to count it), so a selection the file
# cannot see — another package, a test it has not timed — keeps the floor.
# The allowance: the file's 2,108 tests summed 5,056 s alone (2.4 s each,
# 2026-11-07); `check.sh` runs two test threads, about 1.2 s a test, and 2 s
# leaves the build and the slower in-process run their margin. The gate and
# the isolated runner pass no flag and keep the budget's own default.
CHECK_FLOOR_SECONDS = 1800
CHECK_SECONDS_PER_TEST = 2
# Cargo options before `--` and libtest options after it whose value is the
# next word, so that value is never read as a test-name filter.
CARGO_VALUED = {'-p', '--package', '--exclude', '-F', '--features', '--target', '--target-dir',
                '--profile', '-j', '--jobs', '--manifest-path', '--test', '--bin', '--example',
                '--bench', '--message-format', '--color', '--config', '-Z', '--lockfile-path'}
LIBTEST_VALUED = {'--test-threads', '--skip', '--format', '--color', '--logfile', '-Z',
                  '--shuffle-seed'}


def timed_names():
    """The test names the lane gate's timings file times."""
    from release_gate import TIMINGS
    from suite_timings import timed
    try:
        return list(timed(TIMINGS.read_text()))
    except OSError:
        return []


def selected(cargo_args, names):
    """The `names` a `cargo test` with these arguments (after `test`) would run
    in rondata: libtest's substring filters (`--exact` makes them whole names),
    any one admitting, less `--skip`'s; no filter selects all. Another package
    alone selects none."""
    before, after = (cargo_args[:cargo_args.index('--')], cargo_args[cargo_args.index('--') + 1:]) \
        if '--' in cargo_args else (cargo_args, [])
    packages, excluded, filters, skips = [], [], [], []
    i = 0
    while i < len(before):
        arg = before[i]
        flag, eq, value = arg.partition('=')
        if arg.startswith('-p') and len(arg) > 2 and not arg.startswith('--'):
            flag, eq, value = '-p', '=', arg[2:]
        if flag in CARGO_VALUED:
            if not eq:
                i += 1
                value = before[i] if i < len(before) else ''
            if flag in ('-p', '--package'):
                packages.append(value)
            elif flag == '--exclude':
                excluded.append(value)
        elif not arg.startswith('-'):
            filters.append(arg)
        i += 1
    if (packages and 'rondata' not in packages) or 'rondata' in excluded:
        return []
    exact = False
    i = 0
    while i < len(after):
        arg = after[i]
        flag, eq, value = arg.partition('=')
        if flag in LIBTEST_VALUED:
            if not eq:
                i += 1
                value = after[i] if i < len(after) else ''
            if flag == '--skip':
                skips.append(value)
        elif arg == '--exact':
            exact = True
        elif not arg.startswith('-'):
            filters.append(arg)
        i += 1
    hit = (lambda f, n: f == n) if exact else (lambda f, n: f in n)
    return [n for n in names if (not filters or any(hit(f, n) for f in filters))
            and not any(hit(s, n) for s in skips)]


def check_timeout(command, names):
    """Seconds `check.sh test` may run `command`: the per-test allowance times
    the timed tests it selects, never under the floor."""
    if command[:2] != ['cargo', 'test']:
        return CHECK_FLOOR_SECONDS
    return max(CHECK_FLOOR_SECONDS, CHECK_SECONDS_PER_TEST * len(selected(command[2:], names)))


def child_environment(environ=None, *, platform=None, test_name=None, dispatcher=False):
    env = dict(os.environ if environ is None else environ)
    mode = env.get('RON_TEST_ALLOCATOR', 'auto')
    if mode not in ('auto', 'bounded', 'system'):
        raise ValueError('RON_TEST_ALLOCATOR must be auto, bounded or system')
    bounded = mode == 'bounded' or (mode == 'auto' and not dispatcher
                                   and (test_name is None or bounded_test(test_name)))
    if (sys.platform if platform is None else platform) == 'darwin' and bounded:
        # Apple's allocator can retain gigabytes of already-freed large blocks.
        # Reclaim them promptly for competing suites. Preserve explicit overrides;
        # this does not change type isolation, zeroing or corruption checks.
        env.setdefault('MallocLargeCache', '0')
    return env


def allocator_policy(env):
    return {'platform': sys.platform,
            'requested': env.get('RON_TEST_ALLOCATOR', 'auto'),
            'MallocLargeCache': env.get('MallocLargeCache')}
