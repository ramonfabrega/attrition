#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Compare a native restore-wrapper prefix with its live delegation boundary.

The larger pathfinder is the stop boundary, never a stubbed successful callee.
Reports and captured data must stay outside git.
"""
import argparse
import json
from pathlib import Path
import struct
import time
from bounded_call import BoundedCall, Region
from replay_capsule import image_bytes, digest, require
from search_graph import validate, words
from search_census import records, scan

ENTRY, STOP = 0x688f40,0x688fa5


def pack(*v): return struct.pack('<'+'I'*len(v),*v)


def decode(raw):
    require(len(raw) in (196,216),'invalid restore prefix size')
    w=words(raw)
    require(w[0]==0x31545352 and (w[1],len(raw)) in ((1,196),(2,216)),
            'unsupported restore prefix')
    before,stack,deps,after,out,modes=w[4:13],w[13:17],w[17:26],w[26:35],w[35:47],w[47:49]
    flags_mask=0xffffffff if w[1]==1 else w[49]
    registry=None if w[1]==1 else w[50:54]
    if w[1]==2:
        require(flags_mask==0x8d5 and not ((before[8]|after[8]) & ~flags_mask),
                'invalid observed flags')
        require(stack[3]<registry[0]<=registry[1]<=32768 and registry[3]>=0x10000
                and registry[3]+4*(stack[3]+1)<=2**32 and w[3]>=0x10000,
                'invalid captured registry')
        # Bit 1 is architectural, not a captured control flag. Unobserved flags
        # are not silently promoted into evidence by the emulator's defaults.
        before=(*before[:8],before[8]|2)
    require(stack[2]<8 and stack[3]<512,'owner/id outside capture bound')
    require(after[3]+32==before[3] and out[8:]==stack,'unexpected wrapper stack layout')
    require(out[0]==stack[1] and out[3:6]==(*stack[2:],0),'wrong delegated identity/anti')
    require(out[1:3]==tuple(v^0x63637 for v in deps[5:7]),'wrong delegated coordinates')
    require(out[6:8]==(before[1],before[2]),'saved register stack mismatch')
    require(modes[1]==1,'restore did not enable saving')
    return {'version':w[1],'packet_bytes':len(raw),'flags_mask':flags_mask,'registry':registry,
            'frame':w[2],'unit':w[3],'before':before,'stack':stack,'deps':deps,
            'after':after,'out':out,'modes':modes}


def layout(image,capture):
    c=capture;d=c['deps'];owner,uid=c['stack'][2:];esp=c['before'][3]
    return [Region('code',ENTRY,image_bytes(image,ENTRY,STOP-ENTRY),executable=True),
            Region('daemon_pointer',0xc061bc,pack(d[0])),
            Region('objects_pointer',0xc0618c,pack(d[1])),
            Region('repaths',d[0]+owner*4,pack(d[4])),
            Region('slots_pointer',d[1]+0x14+owner*0x1c,pack(d[2])),
            Region('object_pointer',d[2]+uid*4,pack(d[3])),
            Region('coordinates',d[3]+0x10,pack(*d[5:7])),
            Region('modes',0xe85ec0,pack(*d[7:9]),writable=True),
            Region('arguments',esp,pack(*c['stack'])),
            Region('scratch',esp-32,bytes(32),writable=True,scratch=True)]


def verify(result,regions,c):
    regs,state,_=result;state=dict(state)
    require(regs[:8]==c['after'][:8] and (regs[8]^c['after'][8]) & c['flags_mask']==0,
            'live delegation registers differ')
    require(state['scratch']+state['arguments']==pack(*c['out']),'live delegation stack differs')
    for r in regions:
        if r.executable or r.scratch: continue
        expected=pack(*c['modes']) if r.name=='modes' else r.data
        require(state[r.name]==expected,f'complete-record mismatch: {r.name}')


def execute(image,c,repeats=256):
    require(1<=repeats<=4096,'repeat count outside bound')
    regions=layout(image,c);runner=BoundedCall(regions,STOP,budget=128)
    start=time.perf_counter();baseline=runner.run(ENTRY,c['before'],{});verify(baseline,regions,c)
    for _ in range(repeats):
        result=runner.run(ENTRY,c['before'],{});verify(result,regions,c)
        require(result==baseline,'reset or write-history dependence')
    elapsed=time.perf_counter()-start
    fresh=BoundedCall(regions,STOP,budget=128).run(ENTRY,c['before'],{})
    require(fresh==baseline,'fresh engine differs')
    missing=[]
    for omitted in regions:
        try: BoundedCall([r for r in regions if r!=omitted],STOP,budget=128).run(ENTRY,c['before'],{})
        except ValueError: missing.append(omitted.name)
        else: raise ValueError('unused declared dependency: '+omitted.name)
    corrupt=[]
    for name,offset in (('coordinates',0),('modes',0),('scratch',0),('arguments',0)):
        state=dict(baseline[1]);value=bytearray(state[name]);value[offset]^=1;state[name]=bytes(value)
        try:verify((baseline[0],tuple(state.items()),baseline[2]),regions,c)
        except ValueError:corrupt.append(name)
        else:raise ValueError('corrupt output accepted')
    return {'frame':c['frame'],'owner':c['stack'][2],'id':c['stack'][3],
            'packet_version':c['version'],'observed_flags_mask':hex(c['flags_mask']),
            'repaths':c['deps'][4],'limit':c['modes'][0],'saving':c['modes'][1],
            'following_native_astar_result':c.get('native_astar_result'),
            'current_coordinates':list(c['out'][1:3]),'semantic_bytes':36,'argument_bytes':16,
            'scratch_bytes':32,'code_bytes':STOP-ENTRY,'mapped_bytes':runner.mapped_bytes,
            'instructions':runner.instructions,'repeats':repeats,'reused_seconds':elapsed,
            'fresh_matches':True,'dependency_refusals':missing,'corrupt_output_refusals':corrupt,
            'stop':'before find_upath call; resumption not executed'}


def check(directory):
    c=decode((directory/'restore-prefix.bin').read_bytes())
    rows=list(records(directory/'rontrace.log'));scan(iter(rows))
    require(not any(r[:2] in ((5,151),(5,163)) for r in rows),'restore/graph capture failure')
    require([r[2:7] for r in rows if r[:2]==(5,160)]==[(c['version'],ENTRY,STOP,0x682f30,0)],'missing hook setup')
    require([r[2:] for r in rows if r[:2]==(5,161)]==[(c['unit'],*c['stack'][2:],c['stack'][1],c['deps'][4],c['frame'])],
            'missing/mismatched restore entry')
    require([r[2:] for r in rows if r[:2]==(5,162)]==[(0,c['unit'],c['packet_bytes'],c['packet_bytes'],c['modes'][0],c['frame'])],
            'missing/mismatched restore delegation')
    raw=(directory/'search-graph.bin').read_bytes();graph,_,_=validate(raw);h=words(raw[:32])
    require(h[2:4]==(c['frame'],c['unit']),'graph belongs to another invocation')
    require([r[2:] for r in rows if r[:2]==(5,150)]==[(0,c['unit'],h[6],h[7],h[5],c['frame'])],
            'missing restore graph receipt')
    boundary=next(i for i,r in enumerate(rows) if r[:2]==(5,162))
    following=[r for r in rows[boundary+1:] if r[:2] in ((7,0),(8,0))][:2]
    require(len(following)==2 and following[0][:2]==(7,0) and following[1][:2]==(8,0) and
            following[0][3:6]==(c['stack'][1],48,0) and
            following[0][7]==following[1][7]==c['frame'], 'missing native resumed A* call')
    value=following[1][2];c['native_astar_result']=value if value<2**31 else value-2**32
    return c,graph


def delegation_frontier(image,c,graph_raw):
    # Start at the real callee with the native handoff stack plus CALL's return
    # address. Keep the saved graph available; do not invent a Windows TEB.
    from unicorn.x86_const import UC_X86_REG_EIP
    _,items,_=validate(graph_raw)
    regions=[r for r in layout(image,c) if r.name not in ('code','arguments','scratch','modes')]
    regions += [Region('modes',0xe85ec0,pack(*c['modes']),writable=True)]
    regions += [Region(f'graph_{k}_{o}_{a:x}',a,d,writable=True) for k,o,a,d in items]
    esp=c['after'][3]-4
    regions += [Region('callee_code',0x682f30,image_bytes(image,0x682f30,0x100),executable=True),
                Region('arguments',esp,pack(0x688faa,*c['out'])),
                Region('scratch',esp-4096,bytes(4096),writable=True,scratch=True)]
    regs=list(c['after']);regs[3]=esp
    runner=BoundedCall(regions,0x688faa,budget=128)
    try:runner.run(0x682f30,regs,{})
    except ValueError as error:
        ip=runner.uc.reg_read(UC_X86_REG_EIP)
        require(ip==0x682f3a and 'READ_UNMAPPED' in str(error),f'unexpected delegation frontier at {ip:x}: '+str(error))
        return {'instruction':hex(ip),'dependency':'FS:[0], Windows exception-chain head',
                'instructions':runner.instructions,'saved_graph_was_available':True,
                'resumption_executed':False}
    raise ValueError('uncaptured thread environment was accepted')


def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('install',type=Path);ap.add_argument('directory',type=Path)
    args=ap.parse_args();c,graph=check(args.directory)
    binding=json.loads((args.directory/'capsule-image.json').read_text())['sha256']
    require(binding['source']==digest(args.install/'riseofnations.exe'),'original image mismatch')
    require(binding['tracer']==digest(args.directory/'rontrace.dll') and
            binding['traced']==digest(args.directory/'riseofnations_trace.exe'),'capture image mismatch')
    print(json.dumps({'prefix':execute(args.install/'riseofnations.exe',c),'entry_graph':graph,
                      'delegation_frontier':delegation_frontier(args.install/'riseofnations.exe',c,
                          (args.directory/'search-graph.bin').read_bytes())},indent=2))


if __name__=='__main__':main()
