#!/usr/bin/env python3
"""Recompute the exploratory Jev report from retained requests and responses."""
import collections
import hashlib
import json
import math
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]


def main():
    archive = json.loads((ROOT / 'docs/lab/2026-09-19-jev-observations.json').read_text())
    for digest, state in archive['states'].items():
        assert hashlib.sha256(json.dumps(state, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest() == digest
    tokens_total = 0
    requests = 0
    for name, run in archive['runs'].items():
        if 'rows' not in run:
            answer = run['response']['answers']
            correct = sum(v['choice'] == run['expected'][k] for k, v in answer.items())
            print(name, f'{correct}/{len(answer)}', f'{run["elapsed"]:.3f}s', run['response']['usage'])
            tokens_total += run['response']['usage']['input_tokens']
            requests += 1
            continue
        families = collections.defaultdict(lambda: [0, 0])
        for row in run['rows']:
            if 'error' in row:
                raise ValueError(f'Incomplete run: {name}/{row["id"]}')
            requests += 1
            tokens_total += row['response']['usage']['input_tokens']
            # Case 00 has an ambiguous/metaclaim label; preserve but exclude it.
            if row['id'] in ('evidence-00', 'evidence_explicit-00'):
                continue
            families[row['family']][0] += row['response']['answers']['decision']['choice'] == row['expected']
            families[row['family']][1] += 1
        print(name, dict(families))
    original = {r['id']: r for r in archive['runs']['followup']['rows']}
    flips = collections.Counter(r['family'] for r in archive['runs']['permuted']['rows']
        if r['response']['answers']['decision']['choice'] != original[r['id']]['response']['answers']['decision']['choice'])
    print('Reordered/repeated request flips (not causally isolated):', dict(flips))
    rows = archive['runs']['retrieval']['rows']
    documents = archive['states'][rows[0]['request']['state_ref']]['documents']
    stop = set('the and for that with from this what where which our did its was are not but have can how is it to a of in on as be at an or we'.split())

    def tokens(text):
        return [x for x in re.findall('[a-z0-9]+', text.lower()) if x not in stop]

    corpus = {k: tokens(v['file'] + ' ' + v['excerpt']) for k, v in documents.items()}
    average = sum(map(len, corpus.values())) / len(corpus)
    frequencies = collections.Counter(t for ts in corpus.values() for t in set(ts))
    correct = count = 0
    for row in rows:
        if row['expected'] == 'none':
            continue  # No tuned rejection threshold exists for this baseline.
        query = tokens(archive['states'][row['request']['state_ref']]['query'])
        scores = {}
        for key, ts in corpus.items():
            terms = collections.Counter(ts)
            scores[key] = sum(math.log(1 + (len(corpus) - frequencies[t] + .5) / (frequencies[t] + .5)) *
                terms[t] * 2.5 / (terms[t] + 1.5 * (.25 + .75 * len(ts) / average)) for t in query if terms[t])
        correct += max(scores, key=scores.get) == row['expected']
        count += 1
    print(f'BM25 top-1 on answerable queries: {correct}/{count}')
    serial = archive['runs']['serial']['summary']['wall_seconds']
    batch = archive['runs']['batch']['elapsed']
    print(f'One measured serial/batch latency ratio: {serial / batch:.2f}x')
    print(f'Retained calls={requests}; input tokens={tokens_total}; estimated USD={tokens_total * .042 / 1_000_000:.6f}')


if __name__ == '__main__':
    main()
