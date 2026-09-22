#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Find aligned pointer words in a validated retained payload; no live reads.

Following words are raw observations, not automatically recognized objects.
Output is packet-derived and belongs outside git.
"""
import argparse
import json
import struct
from pathlib import Path
from memory_payload import validate
from replay_capsule import require


def find_refs(stream, spans, pointer, limit=64):
    require(0 <= pointer < 2**32 and 1 <= limit <= 256, 'invalid pointer scan bounds')
    needle, hits = struct.pack('<I', pointer), []
    for span in spans:
        require(int(span['base'], 16) % 4 == 0 and span['bytes'] % 4 == 0, 'unaligned pointer scan span')
        for done in range(0, span['bytes'], 1024*1024):
            stream.seek(span['file_offset']+done)
            size = min(1024*1024, span['bytes']-done)
            data = stream.read(size)
            require(len(data) == size, 'short pointer scan read')
            at = data.find(needle)
            while at != -1:
                address = int(span['base'], 16)+done+at
                if address % 4 == 0:
                    if len(hits) == limit:
                        return dict(hits=hits, truncated=True)
                    stream.seek(span['file_offset']+done+at)
                    count = min(16, span['bytes']-done-at)
                    following = stream.read(count)
                    require(len(following) == count, 'short pointer context read')
                    hits.append(dict(address=hex(address), words=list(struct.unpack(
                        '<'+'I'*(len(following)//4), following[:len(following)//4*4]))))
                at = data.find(needle, at+1)
    return dict(hits=hits, truncated=False)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install', type=Path)
    ap.add_argument('directory', type=Path)
    ap.add_argument('pointer', type=lambda s: int(s, 0))
    ap.add_argument('--limit', type=int, default=64)
    args = ap.parse_args()
    report = validate(args.install, args.directory)
    with (args.directory/'memory-payload.bin').open('rb') as stream:
        result = find_refs(stream, report['spans'], args.pointer, args.limit)
    print(json.dumps(dict(payload_sha256=report['sha256'], pointer=hex(args.pointer), **result), indent=2))


if __name__ == '__main__':
    main()
