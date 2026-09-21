#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Replay captured delegation to A* entry; refuse its first absent unit byte.

Only the prefix's observed GPR/arithmetic flags and consumed thread slot are
restored. No FXSAVE import or complete search-return fidelity claim is made.
"""
import argparse
import json
import struct
import re
from pathlib import Path
from unicorn import UC_HOOK_MEM_INVALID, x86_const as x
from bounded_call import Region
from replay_capsule import image_bytes, require
from restore_context import decode_context, check_unit_graph, validate as validate_context
from restore_prefix import layout, pack
from search_census import records
from search_graph import validate as validate_graph
from thread_context_probe import ThreadCall

ENTRY, ASTAR, RETURN = 0x682f30, 0x683770, 0x688faa
OUTPUTS=(0xe85e94,0xe85ed4,0xe85e98,0xe85e9c,0xe85eac,0xe85ea8,0xe85ea4)


def regions_for(image,c,graph_raw,inside=False):
    p=c['prefix'];_,items,_=validate_graph(graph_raw);esp=p['after'][3]-4
    regions=[r for r in layout(image,p) if r.name not in ('code','arguments','scratch','modes')]
    regions += [Region('modes',0xe85ec0,pack(*p['modes']),writable=True),
                Region('callee_code',ENTRY,image_bytes(image,ENTRY,0x42e),executable=True),
                Region('arguments',esp,pack(RETURN,*p['out'])),
                Region('scratch',esp-4096,bytes(4096),writable=True,scratch=True),
                Region('seh',c['teb'],pack(c['tib'][0]),writable=True),
                Region('table',c['origin'],c['table']),
                Region('center',0xcae5fc,pack(c['center'])),
                Region('registry',0xc0aeb4+p['stack'][2]*28,pack(*p['registry'])),
                Region('registry_unit',p['registry'][3]+p['stack'][3]*4,pack(p['unit']))]
    if c['unit_data'] is not None:
        check_unit_graph(c,graph_raw)
        regions=[r for r in regions if r.name!='coordinates']
        regions.append(Region('unit_data',c['unit'],c['unit_data'],writable=True))
        items=[item for item in items if item[:2]!=(1,0)]
    regions += [Region(f'graph_{k}_{o}_{a:x}',a,d,writable=True) for k,o,a,d in items]
    # These seven words are outputs, not zero-valued captured inputs. A read
    # before the original writes each byte still fails the scratch guard.
    regions += [Region(f'output_{a:x}',a,bytes(4),writable=True,scratch=True) for a in OUTPUTS]
    if inside:
        regions.extend([Region('astar_code',ASTAR,image_bytes(image,ASTAR,0x100),executable=True),
                        Region('astar_unit_mask',0xe85eb4,bytes(4),writable=True,scratch=True)])
    return regions


def make_runner(image,c,graph_raw,inside=False,omit=()):
    regions=[r for r in regions_for(image,c,graph_raw,inside) if r.name not in omit]
    return ThreadCall(regions,RETURN if inside else ASTAR,
                      fs_address=c['teb'],gdt_address=0x71000000,budget=128)


def registers(c):
    regs=list(c['prefix']['after']);regs[3]-=4;regs[8]|=2
    return regs


def native_call(rows,c):
    boundary=next(i for i,r in enumerate(rows) if r[:2]==(5,164))
    row=next(r for r in rows[boundary+1:] if r[:2]==(7,0))
    require(row[7]==c['frame'],'native call frame differs')
    return row[2:6]  # this, path stack, step budget, anti


def verify_entry(runner,c,expected):
    esp=runner.uc.reg_read(x.UC_X86_REG_ESP)
    words=struct.unpack('<4I',runner.uc.mem_read(esp,16))
    observed=(runner.uc.reg_read(x.UC_X86_REG_ECX),*words[1:])
    require(words[0]==0x68335e and observed==expected,'native A* entry differs')
    p=c['prefix'];values=(p['unit'],0,(p['out'][1]>>6)//3,(p['out'][2]>>6)//3,0,0,1)
    for address,value in zip(OUTPUTS,values):
        require(bytes(runner.uc.mem_read(address,4))==pack(value),'working output differs')
    require(bytes(runner.uc.mem_read(c['teb'],4))==pack(p['after'][3]-20),'SEH link differs')
    return observed


def run(image,c,graph_raw,expected,repeats=64):
    require(1<=repeats<=1024,'repeat count outside bound')
    runner=make_runner(image,c,graph_raw);regs=registers(c)
    baseline=runner.run(ENTRY,regs);observed=verify_entry(runner,c,expected)
    for _ in range(repeats):
        require(runner.run(ENTRY,regs)==baseline,'reused context differs')
        verify_entry(runner,c,expected)
    fresh=make_runner(image,c,graph_raw)
    require(fresh.run(ENTRY,regs)==baseline,'fresh context differs')
    # This narrow prefix zeroes XMM0 before its two stores; no x87 operation
    # executes. Empirically vary all extended state without claiming an import.
    changed=make_runner(image,c,graph_raw)
    for i in range(8):
        changed.uc.reg_write(getattr(x,f'UC_X86_REG_XMM{i}'),(i+1)*(2**120+17))
        changed.uc.reg_write(getattr(x,f'UC_X86_REG_FP{i}'),(1<<63,0x4000+i))
    changed.uc.reg_write(x.UC_X86_REG_MXCSR,0x3f80)
    changed.uc.reg_write(x.UC_X86_REG_FPCW,0x27f)
    changed.initial_context=changed.uc.context_save()
    require(changed.run(ENTRY,regs)==baseline,'prefix depends on unimported extended state')
    refusals=[]
    for name in ('table','center','registry','registry_unit'):
        try:make_runner(image,c,graph_raw,omit=(name,)).run(ENTRY,regs)
        except ValueError:refusals.append(name)
        else:raise ValueError('missing dependency accepted: '+name)
    inside=make_runner(image,c,graph_raw,inside=True)
    faults=[]
    inside.uc.hook_add(UC_HOOK_MEM_INVALID,lambda u,k,a,n,v,d: faults.append((k,a,n)) or False)
    try:inside.run(ENTRY,regs)
    except ValueError as error:
        pc=inside.uc.reg_read(x.UC_X86_REG_EIP)
        refusal=str(error)
        missing=re.fullmatch(r'undeclared access ([0-9a-f]+)\+(\d+)',refusal)
        if c['version']==1:
            require(pc==0x6837c8 and refusal==f"undeclared access {c['prefix']['unit']+9:08x}+1",
                    f'unexpected frontier {pc:x}: {error}')
        require(missing is not None or (len(faults)==1 and 'UNMAPPED' in refusal),
                'failure is not a missing-input boundary: '+refusal)
        address,size=(int(missing[1],16),int(missing[2])) if missing else faults[0][1:]
        in_unit=c['unit']<=address<c['unit']+0x158
    else:raise ValueError('absent unit header accepted')
    return {'frame':c['frame'],'owner':c['prefix']['stack'][2],'id':c['prefix']['stack'][3],
            'native_astar_arguments_match':True,'astar_arguments':list(observed),
            'prefix_instructions':runner.instructions,'repeated_resets':repeats,
            'fresh_matches':True,'extended_state_perturbation_matches':True,
            'missing_input_refusals':refusals,'working_output_words':len(OUTPUTS),
            'frontier_pc':hex(pc),'frontier_attempted_instructions':inside.instructions,
            'context_version':c['version'],'missing_address':hex(address),
            'missing_region':'current unit body' if in_unit else 'undeclared memory',
            'missing_offset':address-c['unit'] if in_unit else None,'missing_bytes':size,
            'refusal':refusal,
            'full_resumption_returned':False,'fxsave_imported':False}


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install',type=Path);ap.add_argument('directory',type=Path)
    args=ap.parse_args();d=args.directory
    validate_context(args.install,d)
    c=decode_context((d/'restore-context.bin').read_bytes(),(d/'restore-prefix.bin').read_bytes())
    result=run(args.install/'riseofnations.exe',c,(d/'search-graph.bin').read_bytes(),
               native_call(list(records(d/'rontrace.log')),c))
    print(json.dumps(result,indent=2))


if __name__=='__main__':main()
