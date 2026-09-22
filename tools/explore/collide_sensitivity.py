"""Test whether the L82 collide correction changes this callee's computation.

Uses only a trusted L83 local fixture and the owned executable. Interventions
are emulator predictions, never additional native witnesses. Reports contain
capture-derived bytes and must be retained outside Git.
"""
import argparse,hashlib,json,struct,time
from pathlib import Path
from unicorn import UC_HOOK_CODE,UC_HOOK_MEM_READ,UC_HOOK_MEM_WRITE,UC_MEM_READ
from compact_chain import load
from continued_call import continue_once
from defined_bytes import observe_bytes
from path_intervention import execute,input_patch
from payload_replay import observe_unit_path
from replay_capsule import require
from restore_poststate import compare_boundary,require_agreement
from resume_frontier import registers

VALUES=(0,1,2,3,4,255,32767,32768,65535)
OFFSET=0x88

class AccessTrace:
    """Guest callbacks only; host observations do not count as guest reads."""
    def __init__(self,uc,address,size,runner=None):
        self.events=[];self.trace=hashlib.sha256();self.count=0
        def access(u,kind,at,n,value,data):
            if at<address+size and at+n>address:
                self.events.append(dict(kind='read' if kind==UC_MEM_READ else 'write',address=at,size=n))
        def code(u,at,n,data):
            self.trace.update(struct.pack('<II',at,n));self.count+=1
        self.uc=uc;self.runner=runner;self.host_reads=[]
        if runner is not None:
            self.checked_word=runner.checked_word
            def checked(at):
                if at<address+size and at+4>address:self.host_reads.append(at)
                return self.checked_word(at)
            runner.checked_word=checked
        self.hooks=[uc.hook_add(UC_HOOK_MEM_READ|UC_HOOK_MEM_WRITE,access),uc.hook_add(UC_HOOK_CODE,code)]
    def close(self):
        for h in self.hooks:self.uc.hook_del(h)
        if self.runner is not None:self.runner.checked_word=self.checked_word
    def result(self):return dict(events=self.events,host_service_reads=self.host_reads,instructions=self.count,instruction_trace_sha256=self.trace.hexdigest())


def require_passthrough(trials):
    require([t['value'] for t in trials]==list(VALUES),'trial coverage differs')
    reference=next(t['second'] for t in trials if t['value']==3)
    for trial in trials:
        s=trial['second'];require(s['returned'] and trial['repeat_matches'],'trial failed or did not repeat')
        require(s['output_collide']==trial['value'],'collide was not retained')
        trace=s['guest_trace']
        require(not trace['events'] and not trace['host_service_reads'],'collide was accessed')
        require(trace==reference['guest_trace'],'instruction stream differs')
        require(s['unit_except_collide_sha256']==reference['unit_except_collide_sha256'],'other unit bytes differ')
        for key,value in s['fingerprint'].items():
            if key!='declared_memory_sha256':require(value==reference['fingerprint'][key],'model fingerprint differs: '+key)
        c=s['comparison']
        require(c['outer_return_equal'] and c['return_modes_equal'] and c['all_path_slots_equal'] and
                c['active_path_equal'] and c['native_length']==c['replay_length'] and
                c['native_capacity']==c['replay_capacity'] and
                set(c['unit_changes_outside_path_pointer'])<=set((OFFSET,OFFSET+1)),
                'native output gap extends beyond collide')


def experiment(install,directory):
    started=time.monotonic();r,contexts,natives,meta=load(install,directory)
    c=contexts[0];regs=registers(c)
    control=execute(r,regs);require(control['returned'] and control['fingerprint']==meta['baseline_fingerprint'],'fixture baseline differs')
    inputs,_=input_patch(r.regions,c['unit'],'limit',95)
    boundary={name:next(x.data for x in r.regions if x.name==name) for name in ('callee_arguments','modes')}
    trials=[]
    for value in VALUES:
        attempts=[]
        for repeat in range(2):
            first=execute(r,regs,inputs);require(first['returned'],'first call refused')
            before=bytes(observe_bytes(r,c['unit'],344))
            require(int.from_bytes(before[OFFSET:OFFSET+2],'little')==2,'first collide differs')
            r.uc.mem_write(c['unit']+OFFSET,struct.pack('<H',value))
            trace=AccessTrace(r.uc,c['unit']+OFFSET,2,r)
            try:second=continue_once(r,regs,boundary)
            finally:trace.close()
            second['guest_trace']=trace.result()
            if second['returned']:
                second['unit_path_observation']=observe_unit_path(r,c['unit'])
                second['return_modes']=list(struct.unpack('<2I',bytes(observe_bytes(r,0xe85ec0,8))))
                second['comparison']=compare_boundary(natives[1],second)
                unit=bytes(observe_bytes(r,c['unit'],344))
                second['output_collide']=int.from_bytes(unit[OFFSET:OFFSET+2],'little')
                second['unit_except_collide_sha256']=hashlib.sha256(unit[:OFFSET]+unit[OFFSET+2:]).hexdigest()
            attempts.append(second)
        require(attempts[0]==attempts[1],'sensitivity repeat differs')
        trials.append(dict(value=value,repeat_matches=True,second=attempts[0],native_intervention_compared=False))
    require_passthrough(trials)
    reference=next(t['second'] for t in trials if t['value']==3)
    require(reference['returned'],'native-valued control refused');require_agreement(reference['comparison'])
    restored=execute(r,regs)
    require(restored['fingerprint']==meta['baseline_fingerprint'],'baseline failed to restore')
    return dict(schema='collide-sensitivity-v1',fixture_sha256=hashlib.sha256((directory/'fixture.json').read_bytes()).hexdigest(),
                scope='Modeled second callee only; no caller/world or native intervention claim',
                baseline_restored=True,trials=trials,seconds=time.monotonic()-started)


def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('install',type=Path);ap.add_argument('fixture',type=Path)
    a=ap.parse_args();print(json.dumps(experiment(a.install,a.fixture),indent=2))
if __name__=='__main__':main()
