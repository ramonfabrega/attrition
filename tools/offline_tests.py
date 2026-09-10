#!/usr/bin/env python3
"""Run the reviewed offline acquisition and gate tests without an install.

Use explicit module names: new experimental tests are not discovered implicitly.
The cleanup suite mocks live-session operations and uses authored temporary
profiles; its launch-argument test runs a temporary zsh helper and /usr/bin/true.
The gate suite substitutes its child runner, so it does not recurse into a gate.
The memcap suite uses fake RSS samples and small disposable child processes.
"""
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent / 'explore'))

SUITES = (
    'test_autostart_receipt',
    'test_compare_unattended',
    'test_unattended_capture',
    'test_release_gate',
    'test_memcap',
)

if __name__ == '__main__':
    unittest.main(module=None, argv=[sys.argv[0], *SUITES], verbosity=2)
