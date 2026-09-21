#!/usr/bin/env python3
"""Verify the frozen manifests and recompute phase-two Jev evidence results."""
import collections
import gzip
import hashlib
import json

from jev_evidence_lab import ROOT, encoded, PRICE, summarize


def load_archive():
    path=ROOT/'docs/lab/2026-09-19-jev-evidence-observations.json.gz'
    archive=json.loads(gzip.decompress(path.read_bytes()))
    for digest,state in archive['states'].items():
        assert hashlib.sha256(encoded(state)).hexdigest()==digest
    for name,run in archive['runs'].items():
        specs=[]
        for row in run['rows']:
            request=row['request']
            row['request']=dict(model=request['model'],state=archive['states'][request['state_ref']],questions=request['questions'])
            assert 'error' not in row
            spec={k:v for k,v in row.items() if k not in ('response','elapsed')}
            specs.append(spec)
        manifest={'queries':run['queries'],'specs':specs} if run['queries'] is not None else specs
        assert hashlib.sha256(encoded(manifest)).hexdigest()==run['manifest_sha256'],name
    return archive


def main():
    a=load_archive();runs=a['runs']
    for name in ['packets-v1','missing-v1','missing-v2']:
        summary=summarize(runs[name]['rows'])
        if name=='packets-v1':
            print(name,{k:(v['verdict_correct'],v['total']) for k,v in summary['packets']['variants'].items()})
        else:
            print(name,summary['missing']['correct'],'/',summary['missing']['requests'])
    for name in ['packets-v1','citations-v1']:
        for variant in ['base','reordered','distractors','long']:
            counts=collections.Counter()
            for r in runs[name]['rows']:
                if r['variant']!=variant:
                    continue
                scores=collections.defaultdict(float)
                for k,v in r['response']['answers'].items():
                    if k in ('verdict','decidable'):
                        continue
                    source=k.split('_',1)[1]
                    scores[source]=max(scores[source],v['noul'])
                predicted={k for k,v in scores.items() if v>=.5};gold=set(r['expected']['sources'])
                counts.update(tp=len(predicted&gold),extra=len(predicted-gold),missed=len(gold-predicted))
            print(name,variant,dict(counts))
    scans=runs['scan-v1']['rows'];queries=runs['retrieval-v1']['rows'];counts=collections.Counter()
    for query in queries:
        qid=query['id'];gold=set(query['expected']['files'])
        if not gold:
            continue
        bm=list(dict.fromkeys(query['bm25_files']))
        ranking=sorted(scans,key=lambda r:(-r['response']['answers'][qid]['noul'],r['id']))
        files=list(dict.fromkeys(r['request']['state']['passage']['file'] for r in ranking))
        for k in [1,3,5]:
            counts['bm25_all_sources_at_'+str(k)]+=gold<=set(bm[:k])
            counts['scan_all_sources_at_'+str(k)]+=gold<=set(files[:k])
    print('16 answerable queries; nominated-source recall:',dict(counts))
    for name in ['scan-v1','source-search-v1']:
        print(name,'wall seconds',runs[name]['summary']['wall_seconds'])
    print('cached repeat seconds',a['cached_search']['wall_seconds'],'hits',a['cached_search']['cached'])
    rows=[r for run in runs.values() for r in run['rows']]
    tokens=sum(r['response']['usage']['input_tokens'] for r in rows)
    print('Phase two:',len(rows),'requests,',sum(len(r['response']['answers']) for r in rows),'answers,',tokens,'input tokens, estimated USD',tokens*PRICE)
    ledger=a['budget']
    print('Cumulative charged/reserved estimate USD',ledger['prior_usd']+sum(c['charged_or_reserved_usd'] for c in ledger['calls'].values()))
    print('Every frozen request manifest and interned state hash verified.')


if __name__=='__main__':
    main()
