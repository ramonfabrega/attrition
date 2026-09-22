"""Immediate reentry experiment; frozen captured environment, not a native second-call witness."""
import argparse,json,struct
from pathlib import Path
from replay_capsule import require
from continued_call import continue_once
from payload_replay import explore,parse_borrow,fingerprint,observe_unit_path
from path_intervention import execute,input_patch
from resume_frontier import ENTRY,registers
from defined_bytes import observe_bytes
from defined_search_graph import GraphObserver
from absent_search_state import AbsentSearchObserver
from compare_search_graph import compare,derive_correspondence
from native_limit_replay import experiment as native_experiment
from restore_poststate import decode_post,check_receipt,require_agreement
from search_census import records


def counter_writes(runner,unit):
    return [dict(address=hex(a),bytes=n,value=v) for a,n,v in runner.writes
            if any(a<unit+off+4 and a+n>unit+off for off in (0x134,0x148))]


def experiment(runner,c,baseline,native):
    witness=native_experiment(runner,c,baseline,native)
    require_agreement(witness['comparison'])
    regs=registers(c);original=fingerprint(runner,None)
    control_counter_writes=counter_writes(runner,c['unit'])
    control_path=observe_unit_path(runner,c['unit'])
    control_state=AbsentSearchObserver(lambda a,n:observe_bytes(runner,a,n),c['unit'],c['frame']).run()
    require(control_state['complete'],str(control_state['reason']))
    require(fingerprint(runner,None)==original,'control observer mutated state')
    first_inputs,_=input_patch(runner.regions,c['unit'],'limit',95)
    modes=next(r for r in runner.regions if r.name=='modes')
    arguments=next(r for r in runner.regions if r.name=='callee_arguments')
    trials=[]
    for limit in (300,95):
        first=execute(runner,regs,first_inputs);require(first['returned'],'first call refused')
        first['counter_writes']=counter_writes(runner,c['unit'])
        first_graph=GraphObserver(lambda a,n:observe_bytes(runner,a,n),c['unit'],c['frame']).run()
        require(first_graph['complete'],'first graph incomplete')
        before=dict(cursor=runner.cursor,allocations=len(runner.allocations),retired=list(runner.retired),initialized=len(runner.initialized))
        mode_bytes=bytearray(modes.data);struct.pack_into('<I',mode_bytes,0,limit)
        second=continue_once(runner,regs,dict(callee_arguments=arguments.data,modes=bytes(mode_bytes)))
        second['counter_writes']=counter_writes(runner,c['unit'])
        second['model_before']=before
        second['model_after']=dict(cursor=runner.cursor,allocations=len(runner.allocations),retired=list(runner.retired),initialized=len(runner.initialized))
        if second['returned']:
            try:second['unit_path']=observe_unit_path(runner,c['unit'])
            except ValueError as exc:second['unit_path_refusal']=str(exc)
            second['graph']=AbsentSearchObserver(lambda a,n:observe_bytes(runner,a,n),c['unit'],c['frame']).run()
            require(second['graph']['complete'],str(second['graph']['reason']))
            require(fingerprint(runner,None)==second['fingerprint'],'second observer mutated state')
            mapping=derive_correspondence(control_state,second['graph'])
            second['state_comparison']=compare(control_state,second['graph'],mapping['pairs'])
        execute(runner,regs,first_inputs)
        repeated=continue_once(runner,regs,dict(callee_arguments=arguments.data,modes=bytes(mode_bytes)))
        require(repeated['fingerprint']==second['fingerprint'],'chain repeat differs')
        restored=execute(runner,regs);require(restored['fingerprint']==baseline['last']['final_fingerprint'],'control failed to restore')
        trials.append(dict(second_limit=limit,first=first,second=second,repeat_matches=True,control_restored=True))
    return dict(scope='Immediate modeled second invocation with frozen captured world/frame and original boundary GPR/arguments; not native second-call fidelity',native_first_call=witness,control_path=control_path,control_state=control_state,control_counter_writes=control_counter_writes,trials=trials)


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install',type=Path);ap.add_argument('directory',type=Path)
    ap.add_argument('--borrow',action='append',default=[],type=parse_borrow)
    args=ap.parse_args();p=args.directory
    native=decode_post((p/'restore-poststate.bin').read_bytes(),(p/'restore-prefix.bin').read_bytes())
    check_receipt(list(records(p/'rontrace.log')),native)
    result=explore(args.install,p,services='malloc+memset+memcpy+free',borrowed=args.borrow,
                   mutable_arguments=True,observe_path=True,
                   on_prepared=lambda r,c,b:experiment(r,c,b,native))
    print(json.dumps(result,indent=2))


if __name__=='__main__':main()
