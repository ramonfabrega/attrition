"""The kill guard (Loop 3392, 3394; the forty-seventh pass): a PreToolUse hook
that refuses a Bash command signalling processes by name. A subagent's
`pkill -f "<pattern>" -n` sent SIGTERM to every process whose arguments held
`-n` — every rustc, every budgeted test, every shell with a `grep -n` — for
four and a half minutes (docs/audit/2026-10-21-signals-3392-3394.md); parked
1713's prose rule ("never pkill -f") had stood for a month. Wiring it into
`.claude/settings.json` beside `pollguard.py` is Ramon's line to add: the
classifier refuses a session's edit of its own hooks."""
import importlib.util
import json
import pathlib
import subprocess
import sys
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
GUARD = ROOT / 'tools' / 'killguard.py'
spec = importlib.util.spec_from_file_location('killguard', GUARD)
killguard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(killguard)


class TheKillGuard(unittest.TestCase):
    def test_the_incident_s_line_is_refused_for_its_stray_option(self):
        cmd = ('until grep -q "^batch" res.txt; do sleep 10; '
               'pkill -f "debug/deps/sweep_u_z-a19 unit_find_path" -n 2>/dev/null; sleep 1; done')
        why = killguard.offence(cmd)
        self.assertIsNotNone(why)
        self.assertIn('-n', why)

    def test_pkill_by_pattern_and_killall_are_refused(self):
        self.assertIsNotNone(killguard.offence('pkill -f "mutate.py --batch"'))
        self.assertIsNotNone(killguard.offence('killall rustc'))
        self.assertIsNotNone(killguard.offence('ls; pgrep -f foo -n | head'))

    def test_a_kill_by_pid_and_a_plain_pgrep_pass(self):
        for cmd in ('kill 12345', 'kill -TERM 12345', 'pgrep -f "mutate.py --batch"',
                    'pgrep -n -f sweep_u_z', 'pkill -P 4242', 'git log --oneline -n 3',
                    'grep -n pkill docs/LOOP.md', 'echo "pkill -f is banned"'):
            self.assertIsNone(killguard.offence(cmd), cmd)

    def test_the_hook_denies_through_its_json(self):
        payload = json.dumps({'tool_input': {'command': 'pkill -f sweep -n'}, 'session_id': 't'})
        out = subprocess.run([sys.executable, str(GUARD)], input=payload, capture_output=True, text=True)
        self.assertEqual(out.returncode, 0)
        decision = json.loads(out.stdout)['hookSpecificOutput']
        self.assertEqual(decision['permissionDecision'], 'deny')
        payload = json.dumps({'tool_input': {'command': 'kill 4242'}, 'session_id': 't'})
        out = subprocess.run([sys.executable, str(GUARD)], input=payload, capture_output=True, text=True)
        self.assertEqual((out.returncode, out.stdout), (0, ''))


if __name__ == '__main__':
    unittest.main()
