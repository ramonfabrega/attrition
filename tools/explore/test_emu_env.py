"""`tools/emu`'s one shared environment (item 3354).

A bare `uv run tools/emu/<script>.py` builds the script's PEP 723
environment under `~/.cache/uv/environments-v2/<name>-<hash>`, and the
hash moves with the script's path, not its content (measured 2026-10-20,
uv 0.11.16: the same `sweep_angle_diff.py` from a second directory got a
second environment, and an edited copy at one path kept its first) — so
every worktree built one environment per sweep script, 358,890 of them.
The sweeps run instead through `tools/emu/pyproject.toml`'s project:
`uv run --locked --project tools/emu python <script>`, one `.venv` a
worktree, made on the first run. The scripts keep their inline metadata
so a bare `uv run` still works for a person; these tests hold the two
declarations together and the runners on the project.
"""
import os
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EMU = ROOT / 'tools/emu'
SUPPORT = ROOT / 'crates/sim/src/sweep/support.rs'

BLOCK = re.compile(r'^# /// script\s*$(.*?)^# ///\s*$', re.S | re.M)


def inline_metadata(path):
    """A script's PEP 723 block as a dict, or None."""
    m = BLOCK.search(path.read_text())
    if not m:
        return None
    body = '\n'.join(line[2:] if line.startswith('# ') else line[1:]
                     for line in m.group(1).strip('\n').splitlines())
    return tomllib.loads(body)


def project():
    return tomllib.loads((EMU / 'pyproject.toml').read_text())['project']


def uv_command_body():
    """The text of `support.rs`'s `fn uv_command`."""
    text = SUPPORT.read_text()
    start = text.index('fn uv_command(')
    end = text.index('\n}\n', start)
    return text[start:end]


class ProjectDeclarations(unittest.TestCase):
    def test_the_project_declares_every_scripts_dependencies(self):
        deps, pythons = set(), set()
        for script in sorted(EMU.glob('*.py')):
            meta = inline_metadata(script)
            if meta is None:
                continue
            deps.update(meta.get('dependencies', []))
            pythons.add(meta.get('requires-python'))
        self.assertTrue(deps, 'no script under tools/emu declares a dependency')
        self.assertEqual(sorted(project()['dependencies']), sorted(deps))
        self.assertEqual(pythons, {project()['requires-python']})

    def test_the_lock_pins_what_the_project_declares(self):
        lock = tomllib.loads((EMU / 'uv.lock').read_text())
        pinned = {p['name']: p['version'] for p in lock['package'] if 'source' in p
                  and 'registry' in p['source']}
        for dep in project()['dependencies']:
            name, version = dep.split('==')
            self.assertEqual(pinned.get(name), version, dep)

    def test_the_environment_is_ignored(self):
        lines = (ROOT / '.gitignore').read_text().splitlines()
        self.assertIn('/tools/emu/.venv/', lines)


class Runners(unittest.TestCase):
    def test_uv_command_runs_the_project_python(self):
        body = uv_command_body()
        self.assertIn('["run", "--locked", "--project"]', body)
        self.assertIn('.arg("python")', body)

    def test_no_other_runner_starts_uv_bare(self):
        # Every `uv run` of a tools/emu script goes through `uv_command`;
        # `tool_versions` asks uv its versions and is the one other caller.
        offenders = []
        for rs in sorted((ROOT / 'crates').rglob('*.rs')):
            for n, line in enumerate(rs.read_text().splitlines(), 1):
                if 'Command::new("uv")' in line and rs != SUPPORT:
                    offenders.append(f'{rs.relative_to(ROOT)}:{n}')
        self.assertEqual(offenders, [])


@unittest.skipUnless(shutil.which('uv'), 'uv is not on PATH')
class SharedEnvironment(unittest.TestCase):
    """The real `uv`: a script with inline metadata, at a path no run has
    used, runs on the project's `.venv` and leaves no script environment."""

    def test_a_fresh_path_builds_no_script_environment(self):
        cache = Path(subprocess.run(['uv', 'cache', 'dir'], capture_output=True, text=True,
                                    check=True).stdout.strip())
        name = f'emu-env-probe-{os.getpid()}'
        with tempfile.TemporaryDirectory() as tmp:
            probe = Path(tmp) / f'{name}.py'
            probe.write_text((EMU / 'callfn.py').read_text().split('"""', 1)[0]
                             + 'import sys, unicorn\nprint(sys.prefix)\n')
            self.assertIsNotNone(inline_metadata(probe))
            env = dict(os.environ)
            env.pop('VIRTUAL_ENV', None)
            out = subprocess.run(
                ['uv', 'run', '--locked', '--project', str(EMU), 'python', str(probe)],
                capture_output=True, text=True, env=env)
        self.assertEqual(out.returncode, 0, out.stderr)
        self.assertEqual(Path(out.stdout.strip()).resolve(), (EMU / '.venv').resolve())
        made = list((cache / 'environments-v2').glob(f"{name.replace('_', '-')}-*"))
        self.assertEqual(made, [])


if __name__ == '__main__':
    unittest.main()
