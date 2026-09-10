#!/usr/bin/env python3
"""Run the full local gate with an explicit install, including its data survey."""
import argparse
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def gate(install, *, run=subprocess.run):
    install = Path(install).resolve()
    if not (install / 'Data/rules.xml').is_file():
        raise ValueError(f'not an install: missing {install / "Data/rules.xml"}')
    env = os.environ.copy()
    env['RON_INSTALL'] = str(install)
    # No executable launch: rondata surveys the user's data files.
    commands = [
        ['cargo', 'run', '-p', 'rondata', '--', str(install)],
        ['zsh', 'tools/memcap.sh', '20', 'cargo', 'test', '--release'],
        ['cargo', 'clippy', '--all-targets', '--', '-D', 'warnings'],
        ['cargo', 'fmt', '--check'],
        ['zsh', 'tools/guard.sh'],
    ]
    for command in commands:
        run(command, cwd=ROOT, env=env, check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path, help='owned install directory; always exported as RON_INSTALL')
    args = parser.parse_args()
    try:
        gate(args.install)
    except ValueError as exc:
        parser.error(str(exc))


if __name__ == '__main__':
    main()
