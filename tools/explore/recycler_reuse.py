"""Audit PathNode reads separately for each observed recycler acquisition.

Pool 6 and the 36-byte PathNode extent are established by the owned PDB and the
existing search-graph contract. This observes one guarded model invocation; it
neither normalizes target contents nor proves native or universal reuse behavior.
Modeled memcpy/memset execute guest loads/stores and remain visible to the hook.
"""
import struct
from unicorn import UC_HOOK_MEM_READ,UC_HOOK_MEM_WRITE,UC_MEM_WRITE
from replay_capsule import require
from defined_bytes import observe_bytes
POOL=0xc8da70
SIZE=36

def audit_pathnode_reuse(runner,invoke):
    epochs=[];active={};owners={};events=[]
    def read_words(address,count):
        return struct.unpack('<'+'I'*count,bytes(observe_bytes(runner,address,count*4))) if count else ()
    # Observe before hooks, since observation invokes the runner's access guard.
    initial=read_words(POOL,4)
    require(initial[2]<=initial[1]<=4096,'invalid initial recycler bounds')
    initial_pointers=read_words(initial[0],initial[2])
    require(len(set(initial_pointers))==len(initial_pointers),'duplicate initial pool objects')
    def valid_pointer(p):
        require(p>=0x10000 and p%4==0 and p+SIZE<=2**32,'invalid PathNode address')
    for p in initial_pointers:valid_pointer(p)
    ordered=sorted(initial_pointers)
    require(all(a+SIZE<=b for a,b in zip(ordered,ordered[1:])),'overlapping initial PathNodes')
    pooled=set(initial_pointers)
    def hook(uc,access,address,size,value,user):
        if access==UC_MEM_WRITE and address<POOL+12 and address+size>POOL+8:
            require(address==POOL+8 and size==4,'partial recycler length store')
            array,capacity,old,_=read_words(POOL,4)
            new=value&0xffffffff
            require(old<=capacity<=4096 and new<=capacity,'invalid recycler transition')
            require(old==len(pooled),'recycler length changed outside observed stores')
            if new==old:return
            require(len(events)<4096,'recycler event cap')
            pointers=read_words(array+min(new,old)*4,abs(new-old))
            require(len(set(pointers))==len(pointers),'duplicate transition objects')
            for p in pointers:
                valid_pointer(p)
                if new<old:
                    require(p in pooled and p not in active,'acquisition outside available pool')
                    require(len(epochs)<4096,'acquisition cap')
                    pooled.remove(p)
                    for b in range(p,p+SIZE):require(b not in owners,'overlapping active objects')
                    e=dict(address=p,written=set(),reads=set(),old_reads=set(),released=False)
                    epochs.append(e);active[p]=e
                    for b in range(p,p+SIZE):owners[b]=e
                else:
                    require(p not in pooled,'duplicate pooled release')
                    require(all(p==a or p+SIZE<=a or a+SIZE<=p for a in active),'release overlaps another active object')
                    require(all(p+SIZE<=a or a+SIZE<=p for a in pooled),'overlapping pooled release')
                    pooled.add(p)
                    if p in active:
                        e=active.pop(p);e['released']=True
                        for b in range(p,p+SIZE):del owners[b]
            events.append(dict(old=old,new=new,pointers=[hex(p) for p in pointers]))
            return
        for b in range(address,address+size):
            if b not in owners:continue
            e=owners[b];off=b-e['address']
            if access==UC_MEM_WRITE:e['written'].add(off)
            else:
                e['reads'].add(off)
                if off not in e['written']:e['old_reads'].add(off)
    h=runner.uc.hook_add(UC_HOOK_MEM_READ|UC_HOOK_MEM_WRITE,hook)
    try:result=invoke()
    finally:runner.uc.hook_del(h)
    result['reuse_epochs']=dict(schema='pathnode-reuse-epochs-v1',
      scope='guest accesses during one invocation; history restarts on each pool acquisition',
      successful_invocation=result.get('returned') is True,initial_pool=list(initial),events=events,epochs=[
      dict(address=hex(e['address']),written_offsets=sorted(e['written']),read_offsets=sorted(e['reads']),
           read_before_write_offsets=sorted(e['old_reads']),released=e['released']) for e in epochs])
    return result
