"""Exercise sampling failures without launching a memory-heavy workload."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


MEMCAP = Path(__file__).resolve().parents[1] / "memcap.sh"


class SamplingFailureTests(unittest.TestCase):
    def run_guard(self, fake_ps):
        with tempfile.TemporaryDirectory(prefix="attrition-memcap-") as directory:
            root = Path(directory)
            ps = root / "ps"
            ps.write_text("#!/bin/sh\n" + fake_ps)
            ps.chmod(0o755)
            env = dict(os.environ, PATH=f"{root}:{os.environ['PATH']}",
                       MEMCAP_TEST_STATE=str(root / "sampled"))
            result = subprocess.run(
                ["zsh", str(MEMCAP), "20", "sleep", "20"],
                env=env, capture_output=True, text=True, timeout=8,
            )
            self.assertEqual(result.returncode, 125, result.stderr)
            return result.stderr

    def test_denied_sampling_refuses_launch(self):
        self.assertIn("refusing to launch", self.run_guard("exit 1\n"))

    def test_invalid_sample_refuses_launch(self):
        self.assertIn("invalid RSS", self.run_guard("echo unavailable\n"))

    def test_sampling_loss_stops_live_command(self):
        # The preflight succeeds, then the first live sample fails. Returning
        # within eight seconds also checks that the twenty-second child died.
        message = self.run_guard(
            'if [ -e "$MEMCAP_TEST_STATE" ]; then exit 1; fi\n'
            ': > "$MEMCAP_TEST_STATE"\n'
            'echo 1024\n'
        )
        self.assertIn("while the command is live", message)


if __name__ == "__main__":
    unittest.main()
