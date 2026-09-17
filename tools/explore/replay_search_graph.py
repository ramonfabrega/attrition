#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Replay native open/closed PathNode disposal with a captured recycler.

No allocator adapter or altered pool capacity. This is the payload-disposal
substep of cleanup, not full cleanup or suspended-search resumption.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import time
from bounded_call import BoundedCall, Region
from replay_capsule import image_bytes, require, digest
from search_graph import validate, check_trace, words, POOLS
from search_cleanup_oracle import CODE

STOP, ESP = 0x70000000, 0x71002000
CODE_DISPOSE = ((0x687c10,0x64),(0x687ba0,0x6a))
REGS = (0x11111111,0x22222222,0x33333333,ESP,0x44444444,0x55555555,0xe85e40,0x66666666,0x202)


def pack(*v): return struct.pack('<'+'I'*len(v),*v)


def data_regions(items, full=False):
    selected = items if full else [r for r in items if (r[0]==3 and r[1] in (0,2)) or
                                   (r[0] in (6,7) and r[1]==6)]
    return [Region(f'{k}_{owner}_{a:x}', a, d[:12] if k==6 else d, writable=True)
            for k,owner,a,d in selected]


def pipeline(runner, regions, roots, reverse=False):
    inputs={r.name:r.data for r in regions if not r.executable and not r.scratch}
    outputs=[]; instructions=0
    for owner in ((2,0) if reverse else (0,2)):
        inputs['arguments']=pack(STOP,roots[owner])
        result=runner.run(0x687c10 if owner==0 else 0x687ba0,REGS,inputs)
        regs,state,writes=result
        require(all(regs[i]==REGS[i] for i in (0,1,2,4)) and regs[3]==ESP+8,
                'callee-saved register or stack mismatch')
        inputs={k:v for k,v in state if k!='scratch'}
        outputs.append(result);instructions+=runner.instructions
    return outputs,instructions


def verify(outputs, regions, active, roots, reverse=False):
    state=dict(outputs[-1][1]); expected={r.name:bytearray(r.data) for r in regions if not r.executable and not r.scratch}
    owned=[n[3] for owner in (0,2) for n in active[owner].values()]
    node_names={r.address:r.name for r in regions}
    for owner in (0,2):
        for a in active[owner]: struct.pack_into('<I',expected[node_names[a]],12,0)
    pool_name=node_names[POOLS[6]]; p=words(expected[pool_name]); array_name=node_names[p[0]]
    require(p[1]-p[2]>=len(owned), 'natural pool capacity insufficient')
    struct.pack_into('<I',expected[pool_name],8,p[2]+len(owned))
    actual=state[array_name][p[2]*4:(p[2]+len(owned))*4]
    require(sorted(words(actual))==sorted(owned), 'payload ownership mismatch')
    expected[array_name][p[2]*4:(p[2]+len(owned))*4]=actual
    # The argument root belongs to the last call; all other bytes are compared.
    expected['arguments'][:]=pack(STOP,roots[0 if reverse else 2])
    require(words(expected['arguments'])[0]==STOP, 'return sentinel changed')
    for name,data in expected.items(): require(state[name]==data,f'complete-record mismatch: {name}')


def run(image, raw, repeats=64):
    require(1 <= repeats <= 4096, 'repeat count outside bound')
    report,items,active=validate(raw)
    roots={owner:words(d)[3] for k,owner,_,d in items if k==2}
    code=[Region(f'code_{a:x}',a,image_bytes(image,a,n),executable=True) for a,n in CODE_DISPOSE]
    regions=code+data_regions(items)+[Region('arguments',ESP,pack(STOP,roots[0])),
                                    Region('scratch',ESP-4096,bytes(4096),writable=True,scratch=True)]
    runner=BoundedCall(regions,STOP,budget=20000)
    started=time.perf_counter(); baseline,instructions=pipeline(runner,regions,roots);verify(baseline,regions,active,roots)
    signature=hashlib.sha256(repr(baseline).encode()).hexdigest()
    for _ in range(repeats):
        result,_=pipeline(runner,regions,roots)
        require(hashlib.sha256(repr(result).encode()).hexdigest()==signature,'reset dependence')
        verify(result,regions,active,roots)
    reused_seconds=time.perf_counter()-started
    fresh,_=pipeline(BoundedCall(regions,STOP,budget=20000),regions,roots)
    require(fresh==baseline,'fresh engine differs')
    reverse,_=pipeline(runner,regions,roots,reverse=True);verify(reverse,regions,active,roots,reverse=True)
    # Every mapped semantic region must be necessary, not merely convenient.
    refusals=[]
    for r in regions:
        reduced=[x for x in regions if x.name!=r.name]
        try: pipeline(BoundedCall(reduced,STOP,budget=20000),reduced,roots)
        except ValueError: refusals.append(r.name)
        else: raise ValueError(f'unnecessary region accepted: {r.name}')
    altered=[]
    for name,offset,value in [('arguments',4,123),
                               (next(r.name for r in regions if r.address==POOLS[6]),8,0),
                               (next(r.name for r in regions if r.address==roots[0]),12,123),
                               (next(r.name for r in regions if r.address==words(next(d for k,o,a,d in items if (k,o)==(6,6)))[0]),0,0)]:
        state=dict(baseline[-1][1]); d=bytearray(state[name]);struct.pack_into('<I',d,offset,value);state[name]=bytes(d)
        bad=[*baseline[:-1],(baseline[-1][0],tuple(state.items()),baseline[-1][2])]
        try: verify(bad,regions,active,roots)
        except ValueError: altered.append(name)
        else: raise ValueError('corrupt output accepted')
    # Retain the natural pool header. Full cleanup must stop at the first
    # allocator-dependent read, rather than fabricate spare container capacity.
    full_code={**dict(CODE),**dict(CODE_DISPOSE)}
    full=[Region(f'code_{a:x}',a,image_bytes(image,a,n),executable=True) for a,n in full_code.items()]
    full+=data_regions(items,full=True)+[Region('arguments',ESP,pack(STOP)),
                                       Region('scratch',ESP-4096,bytes(4096),writable=True,scratch=True)]
    unit=next(a-0x104 for k,_,a,_ in items if k==1)
    regs=(*REGS[:6],unit,*REGS[7:])
    try: BoundedCall(full,STOP,budget=20000).run(0x5e3920,regs,{})
    except ValueError as e:
        refusal=str(e);require('00c8d81c' in refusal,'unexpected full-cleanup boundary: '+refusal)
    else: raise ValueError('unexpected full cleanup success')
    return {**report,'disposed_pathnodes':len(active[0])+len(active[2]),
            'semantic_replay_bytes':sum(len(r.data) for r in regions if not r.executable and not r.scratch and r.name!='arguments'),
            'mapped_bytes':runner.mapped_bytes,'instructions':instructions,'repeat_runs':repeats,
            'reused_seconds':reused_seconds,'fresh_matches':True,'reverse_ownership_matches':True,
            'removed_dependency_refusals':len(refusals),'corrupt_output_refusals':len(altered),
            'full_cleanup_refusal':refusal,'result_sha256':signature}


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install',type=Path);ap.add_argument('directory',type=Path)
    ap.add_argument('--repeats',type=int,default=64);args=ap.parse_args()
    raw=(args.directory/'search-graph.bin').read_bytes();check_trace(raw,args.directory/'rontrace.log')
    binding=json.loads((args.directory/'capsule-image.json').read_text())['sha256']
    require(binding['source']==digest(args.install/'riseofnations.exe'),'original image identity mismatch')
    require(binding['tracer']==digest(args.directory/'rontrace.dll') and
            binding['traced']==digest(args.directory/'riseofnations_trace.exe'),'captured image identity mismatch')
    print(json.dumps(run(args.install/'riseofnations.exe',raw,args.repeats),indent=2))


if __name__=='__main__': main()
