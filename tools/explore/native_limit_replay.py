#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Compare an explicitly recorded native limit-95 run with its modeled intervention.

The packet retains limit 300. Prepare that unmodified model, change only the
limit, repeat, then restore the control. No native control output is asserted
for this packet. All generated data must remain outside git.
"""
import argparse
import json
from pathlib import Path
import struct
from path_intervention import captured_region, input_patch, execute, LIMIT
from payload_replay import explore, observe_unit_path, parse_borrow
from resume_frontier import registers
from restore_poststate import decode_post, check_receipt, compare, require_agreement
from replay_capsule import require
from search_census import records


def experiment(runner, context, baseline, native):
    provenance=dict(field='limit',address='0xe85ec0',before=300,after=95,saving=1)
    require(native.get('intervention')==provenance,'requires an explicit native limit-95 witness')
    region=captured_region(runner.regions,LIMIT,8)
    require(struct.unpack_from('<2I',region.data,LIMIT-region.address)==(300,1),
            'captured delegated modes differ')
    regs=registers(context);expected=baseline['last']['final_fingerprint']
    require(execute(runner,regs)['fingerprint']==expected,'prepared control differs')
    inputs,patch=input_patch(runner.regions,context['unit'],'limit',95)
    one=execute(runner,regs,inputs)
    require(one['returned'],'intervened model refused: '+str(one['reason']))
    one['unit_path_observation']=observe_unit_path(runner,context['unit'])
    repeat=execute(runner,regs,inputs)
    require(repeat['fingerprint']==one['fingerprint'],'intervention repeat differs')
    restored=execute(runner,regs)
    require(restored['fingerprint']==expected,'control did not restore')
    variant=dict(payload_sha256=baseline['payload_sha256'],native_intervention=provenance,last=one)
    result=compare(native,variant,baseline['payload_sha256'])
    return dict(replay=variant,comparison=result,intervention=patch,repeat_matches=True,
                control_restored=True,new_regions_added=0,native_control_compared=False,
                native_saved_tree_contents_compared=False)


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install',type=Path);ap.add_argument('directory',type=Path)
    ap.add_argument('--borrow',action='append',default=[],type=parse_borrow)
    ap.add_argument('--require-agreement',action='store_true')
    args=ap.parse_args()
    native=decode_post((args.directory/'restore-poststate.bin').read_bytes(),
                       (args.directory/'restore-prefix.bin').read_bytes())
    check_receipt(list(records(args.directory/'rontrace.log')),native)
    require(native.get('intervention') is not None,'unchanged control is not an intervention witness')
    result=explore(args.install,args.directory,services='malloc+memset+memcpy+free',
                   borrowed=args.borrow,mutable_arguments=True,observe_path=True,
                   on_prepared=lambda r,c,b:experiment(r,c,b,native))
    print(json.dumps(result,indent=2))
    if args.require_agreement:require_agreement(result['prepared_experiments']['comparison'])


if __name__=='__main__':main()
