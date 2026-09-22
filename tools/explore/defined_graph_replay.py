#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Observe model post-search structure with explicit undefined bytes.

First requires the paired native limit-95 unit/path agreement. Tree observations
remain model-only; generated reports contain original-derived data, keep outside git.
"""
import argparse
import json
from pathlib import Path
from defined_bytes import observe_bytes
from defined_search_graph import GraphObserver
from native_limit_replay import experiment as native_experiment
from path_intervention import execute,input_patch
from payload_replay import explore,parse_borrow,fingerprint
from resume_frontier import registers
from restore_poststate import decode_post,check_receipt,require_agreement
from search_census import records
from replay_capsule import require


def experiment(runner,context,baseline,native):
    witness=native_experiment(runner,context,baseline,native)
    require_agreement(witness['comparison'])
    inputs,_=input_patch(runner.regions,context['unit'],'limit',95)
    regs=registers(context)
    one=execute(runner,regs,inputs);require(one['returned'],'intervention did not return')
    def observe():
        return GraphObserver(lambda a,n:observe_bytes(runner,a,n),context['unit'],context['frame']).run()
    graph=observe()
    require(fingerprint(runner,None)==one['fingerprint'],'graph observer mutated model state')
    repeat=execute(runner,regs,inputs)
    require(repeat['fingerprint']==one['fingerprint'] and observe()==graph,'graph observation does not repeat')
    require(execute(runner,regs)['fingerprint']==baseline['last']['final_fingerprint'],'control did not restore')
    return dict(native_unit_path=witness,graph=graph,graph_repeats=True,observer_read_only=True,
                control_restored=True,new_regions_added=0,native_tree_contents_compared=False)


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install',type=Path);ap.add_argument('directory',type=Path)
    ap.add_argument('--borrow',action='append',default=[],type=parse_borrow)
    ap.add_argument('--require-closure',action='store_true')
    args=ap.parse_args()
    native=decode_post((args.directory/'restore-poststate.bin').read_bytes(),
                       (args.directory/'restore-prefix.bin').read_bytes())
    check_receipt(list(records(args.directory/'rontrace.log')),native)
    require(native.get('intervention') is not None,'explicit native intervention witness required')
    result=explore(args.install,args.directory,services='malloc+memset+memcpy+free',
                   borrowed=args.borrow,mutable_arguments=True,observe_path=True,
                   on_prepared=lambda r,c,b:experiment(r,c,b,native))
    print(json.dumps(result,indent=2))
    if args.require_closure:
        graph=result['prepared_experiments']['graph'];require(graph['complete'],graph['reason'])


if __name__=='__main__':main()
