#!/usr/bin/env python3
"""Exploration-only full-gate timer. Run from the repository root.

Temporarily edits capture.rs and gamelog.rs, restoring them in finally.
Do not run alongside source edits/builds in the same worktree.
Logs stay in /tmp. Set RON_INSTALL; process sampling must work.
The original measurement preceded the sampling preflight retained here.
"""
import json
import os
from pathlib import Path
import subprocess
import time

if not os.environ.get('RON_INSTALL'):
    raise SystemExit('set RON_INSTALL to the owned game install')
try:
    sample = subprocess.run(
        ['ps', '-o', 'rss=', '-p', str(os.getpid())],
        capture_output=True, text=True)
except OSError as error:
    raise SystemExit(f'process sampling unavailable: {error}')
if sample.returncode or not sample.stdout.strip().isdigit():
    raise SystemExit('process sampling unavailable; run in a context that permits ps')

paths = [Path('crates/rondata/src/capture.rs'),
         Path('crates/rondata/src/gamelog.rs')]
original = {path: path.read_bytes() for path in paths}
try:
    path = paths[0]
    source = path.read_text()
    old = 'std::fs::read_to_string(path).unwrap_or_default()'
    new = '''let start = std::time::Instant::now();
    let result = std::fs::read_to_string(path.as_ref()).unwrap_or_default();
    eprintln!("EXPLORE read {} {} {:?}", start.elapsed().as_nanos(), result.len(), path.as_ref());
    result'''
    assert old in source
    path.write_text(source.replace(old, new))

    path = paths[1]
    source = path.read_text()
    marker = "impl<'a> Log<'a> {"
    helper = '''struct ExplorationSpan(&'static str, std::time::Instant);
impl Drop for ExplorationSpan {
    fn drop(&mut self) {
        eprintln!("EXPLORE {} {} {}", self.0, self.1.elapsed().as_nanos(), std::thread::current().name().unwrap_or("unnamed"));
    }
}
'''
    assert marker in source
    source = source.replace(marker, helper + marker, 1)
    for signature, label in [
        ("pub fn parse(text: &'a str) -> Log<'a> {", 'parse'),
        ("fn scan_children(&self, mut f: impl FnMut(Block<'_>) -> bool) {", 'scan'),
    ]:
        assert signature in source
        timer = f'\n        let _exploration = ExplorationSpan("{label}", std::time::Instant::now());'
        source = source.replace(signature, signature + timer, 1)
    path.write_text(source)

    start = time.perf_counter()
    with open('/tmp/attrition-profile-gate.log', 'w') as log:
        result = subprocess.run(
            ['zsh', 'tools/memcap.sh', '20', 'cargo', 'test', '-p', 'rondata',
             '--release', '--lib', '--', '--nocapture'],
            stdout=log, stderr=subprocess.STDOUT)
    print(json.dumps(dict(exit_code=result.returncode,
                          wall_seconds=time.perf_counter() - start)))
finally:
    for path, data in original.items():
        path.write_bytes(data)
raise SystemExit(result.returncode)
