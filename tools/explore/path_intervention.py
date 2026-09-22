#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""One-field request and continuation-limit trials on a validated prepared replay.

No live process or packet edits. Reuse only the baseline's declared memory;
new dependencies refuse. Predictions are not native intervention witnesses.
Prints captured/derived data: redirect outside git.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import time
from unicorn import UC_HOOK_CODE, x86_const as x
from payload_replay import explore, fingerprint, observe_unit_path, parse_borrow
from replay_capsule import require
from resume_frontier import ENTRY, registers
from restore_poststate import decode_post, check_receipt, compare, require_agreement
from search_census import records

FIELDS={'x':0,'y':4,'tolerance':8}
LIMIT=0xe85ec0  # PathFinderData.limit: pathfinder +0x40 +0x40; matched PDB.


def captured_region(regions,address,size):
    r=next((r for r in regions if r.address<=address and address+size<=r.address+len(r.data)),None)
    require(r is not None and not r.executable and not r.scratch,'request must be declared captured data')
    return r


def request(regions,unit):
    r=captured_region(regions,unit+0xb8,12)
    pointer,capacity,length=struct.unpack_from('<3I',r.data,unit+0xb8-r.address)
    require(0<length<=capacity<=4096 and pointer>=0x10000 and pointer+capacity*16<=2**32,
            'invalid captured request extent')
    address=pointer+(length-1)*16
    r=captured_region(regions,address,16)
    return address,dict(zip(('x','y','tolerance','flags'),struct.unpack_from('<4i',r.data,address-r.address)))


def input_patch(regions,unit,field,value):
    require(field in (*FIELDS,'limit') and type(value) is int and 0<=value<2**31,'invalid path intervention')
    if field=='limit':
        require(value<=4096,'experimental work limit exceeds cap')
        address=LIMIT;r=captured_region(regions,address,4)
        before=struct.unpack_from('<i',r.data,address-r.address)[0]
    else:
        address,values=request(regions,unit);address+=FIELDS[field]
        r=captured_region(regions,address,4);before=values[field]
    require(r.writable,'request region is read-only')
    data=bytearray(r.data);struct.pack_into('<I',data,address-r.address,value)
    return {r.name:bytes(data)},dict(field=field,address=hex(address),before=before,after=value,changed=before!=value)



def execute(runner,regs,inputs=None,entry=ENTRY):
    started=time.monotonic();error=None
    try:runner.run(entry,regs,inputs)
    except ValueError as exc:error=str(exc)
    return dict(returned=error is None,reason=error,instructions=runner.instructions,
                eax=runner.uc.reg_read(x.UC_X86_REG_EAX),fingerprint=fingerprint(runner,error),
                seconds=time.monotonic()-started)


def parse_trial(value):
    field,number=value.split(':');require(field in (*FIELDS,'limit'),'unknown request field')
    return field,int(number,0)


def experiment(runner,context,baseline,native,trials):
    require(1<=len(trials)<=12,'trial count outside bound')
    require_agreement(compare(native,baseline,baseline['payload_sha256']))
    expected=baseline['last']['final_fingerprint'];regs=registers(context)
    control=execute(runner,regs)
    require(control['fingerprint']==expected,'prepared control differs before intervention')
    address,original=request(runner.regions,context['unit'])
    mode_region=captured_region(runner.regions,LIMIT,8)
    mode_values=struct.unpack_from('<2i',mode_region.data,LIMIT-mode_region.address)
    astar=[];trace=hashlib.sha256();control_trace=None
    def record(u,a,n,d):
        trace.update(struct.pack('<II',a,n))
        if a==0x68335e:astar.append(u.reg_read(x.UC_X86_REG_EAX))
    hook=runner.uc.hook_add(UC_HOOK_CODE,record)
    results=[]
    try:
        for field,value in trials:
            inputs,patch=input_patch(runner.regions,context['unit'],field,value)
            astar.clear();trace=hashlib.sha256()
            one=execute(runner,regs,inputs);one['astar_returns']=list(astar)
            one['instruction_trace_sha256']=trace.hexdigest()
            if one['returned']:
                one['unit_path_observation']=observe_unit_path(runner,context['unit'])
                variant={'payload_sha256':baseline['payload_sha256'],'last':one}
                one['comparison_to_unmodified_native_control']=compare(native,variant,baseline['payload_sha256'])
            astar.clear();trace=hashlib.sha256();repeat=execute(runner,regs,inputs)
            require(repeat['fingerprint']==one['fingerprint'] and astar==one['astar_returns'] and
                    trace.hexdigest()==one['instruction_trace_sha256'],
                    'intervention repeat differs')
            trace=hashlib.sha256();after=execute(runner,regs)
            require(after['fingerprint']==expected,'control differs after intervention')
            if control_trace is None:control_trace=trace.hexdigest()
            require(trace.hexdigest()==control_trace,'control instruction stream differs')
            one['instruction_stream_matches_control']=one['instruction_trace_sha256']==control_trace
            one.update(intervention=patch,repeat_matches=True,control_restored=True,native_intervention_compared=False)
            results.append(one)
    finally:
        runner.uc.hook_del(hook)
    return dict(captured_request_address=hex(address),captured_request=original,
                captured_modes=dict(limit=mode_values[0],saving=mode_values[1]),
                prepared_control=control,control_instruction_trace_sha256=control_trace,results=results,new_regions_added=0,
                scope='One request field changed; all other captured state restored. Native comparison only for unmodified control.')


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install',type=Path);ap.add_argument('directory',type=Path)
    ap.add_argument('--borrow',action='append',default=[],type=parse_borrow)
    ap.add_argument('--trial',action='append',required=True,type=parse_trial,metavar='FIELD:VALUE')
    args=ap.parse_args();require(1<=len(args.trial)<=12,'trial count outside bound')
    native=decode_post((args.directory/'restore-poststate.bin').read_bytes(),(args.directory/'restore-prefix.bin').read_bytes())
    check_receipt(list(records(args.directory/'rontrace.log')),native)
    result=explore(args.install,args.directory,services='malloc+memset+memcpy+free',borrowed=args.borrow,
                   mutable_arguments=True,observe_path=True,
                   on_prepared=lambda r,c,b:experiment(r,c,b,native,args.trial))
    print(json.dumps(result,indent=2))


if __name__=='__main__':main()
