"""Fail-closed reader for the end-frame snapshot stream; no atomicity claim."""
import argparse, hashlib, json, struct
from pathlib import Path
from typed_state import require
CAP=1024**3;CHUNK=1024**2

def decode(stream,plan):
    digest=hashlib.sha256();offset=0
    def read(n):
        nonlocal offset
        require(0<=n<=CHUNK,'unbounded read');b=stream.read(n)
        require(len(b)==n,'truncated snapshot');digest.update(b);offset+=n;return b
    h=struct.unpack('<32I',read(128))
    require(h[:3]==(0x31534652,1,1),'unsupported snapshot phase/version')
    require(h[3]>0 and h[4] in (h[3],h[3]-1),'unexpected trace/logger frame relationship')
    require(h[5]>0 and h[6]>=65536 and h[7]==plan['logger'] and h[8]==0x400000,'boundary identity differs')
    require(h[9]>=65536 and h[9]!=h[8] and 65536<=h[10]<h[11]<=2**32-1,'invalid exclusions')
    require(65536<=h[12]<h[13]<=2**32-1 and 0<h[14]<=65536 and not h[14]&(h[14]-1),'invalid inventory bounds')
    require(h[12]%h[14]==h[13]%h[14]==0,'unaligned inventory')
    require(0<h[15]<=8192 and 0<h[16]<=h[15] and 0<h[17]<=CAP,'invalid range/byte count')
    require(h[18]==len(plan['roots'])+2<=16 and 0<h[19]<=524288,'invalid anchors')
    require(h[20]<2000 and h[21]==0 and h[22:27]==(CAP,CHUNK,8192,2000,5000),'limits differ')
    require(h[27:29]==(plan['game_slot'],plan['frame_offset']) and h[31]==0,'plan differs')
    require(h[29]>=2**31 or h[29]<=h[3]<h[30],'logger window excludes frame')
    ranges=[];cursor=h[12];observer=False
    for i in range(h[15]):
        r=struct.unpack('<7I',read(28));base,allocation,ap,size,state,protect,kind=r
        require(base==cursor and size>0 and base+size<=2**32-1 and base%h[14]==size%h[14]==0,'invalid inventory extent')
        require(state in (0x1000,0x2000,0x10000),'invalid memory state')
        if allocation==h[9] and state==0x1000 and kind==0x1000000:observer=True
        cursor=min(base+size,h[13]);ranges.append(r)
        require(i==h[15]-1 or cursor<h[13],'extra inventory rows')
    require(cursor==h[13] and observer,'incomplete inventory or absent observer')
    expected=[(plan['game_slot'],4),(h[6]+plan['frame_offset'],4)]+[(r['address'],r['size']) for r in plan['roots']]
    anchors=[];total=0
    for address,size in expected:
        a=struct.unpack('<3I',read(12));require(a==(address,size,total),'anchor plan differs')
        require(0<size and 65536<=address<address+size<=2**32-1,'invalid anchor extent')
        anchors.append(a);total+=size
    require(total==h[19],'anchor bytes differ');saved=read(total)
    require(struct.unpack_from('<2I',saved)==(h[6],h[3]),'anchor identity differs')
    selected=[]
    for i,r in enumerate(ranges):
        base,allocation,ap,size,state,protect,kind=r;size=min(size,h[13]-base)
        if state==0x1000 and not protect&0x100 and protect&255 in (2,4,8) and allocation!=h[9] and not (base<h[11] and base+size>h[10]) and (kind==0x20000 or kind==0x1000000 and allocation==h[8]):
            selected.append((i,base,size))
    require(len(selected)==h[16] and sum(s[2] for s in selected)==h[17],'selection differs')
    covered=[0]*len(anchors);spans=[]
    for record in selected:
        require(struct.unpack('<3I',read(12))==record,'range record differs')
        i,base,size=record;spans.append(dict(inventory_index=i,base=hex(base),bytes=size,file_offset=offset))
        for done in range(0,size,CHUNK):
            raw=read(min(CHUNK,size-done));address=base+done
            for k,(a,n,o) in enumerate(anchors):
                lo=max(a,address);hi=min(a+n,address+len(raw))
                if lo<hi:
                    require(raw[lo-address:hi-address]==saved[o+lo-a:o+hi-a],'copied anchor drift')
                    covered[k]+=hi-lo
    require(covered==[a[1] for a in anchors],'anchor not retained')
    footer=struct.unpack('<7I',read(28))
    require(footer[:5]==(0x45465352,1,h[3],h[16],h[17]) and footer[5]<5000 and footer[6]==h[18],'invalid completion footer')
    require(not stream.read(1),'trailing snapshot bytes')
    return dict(schema='frame-snapshot-v1',frame=h[3],trace_frame=h[4],thread=h[5],game=h[6],
                capture_boundary='GameLog::end_frame entry, before full_dump',atomic_snapshot=False,
                logger_compared=False,all_threads_quiescent=False,root_anchors_checked=True,
                inventory_ms=h[20],copy_ms=footer[5],payload_bytes=h[17],range_count=h[16],
                spans=spans,rows=ranges,sha256=digest.hexdigest(),anchor_bytes=h[19],anchor_count=h[18])


def check_receipts(rows,report):
    successes=[(i,r) for i,r in enumerate(rows) if r[:2]==(5,180)]
    require(len(successes)==1 and not any(r[:2]==(5,181) for r in rows),'missing/duplicate success or failure receipt')
    index,r=successes[0]
    require(r[2:5]==(report['frame'],report['range_count'],report['payload_bytes']) and
            report['copy_ms']<=r[5]<5000 and r[6]==report['anchor_count'] and r[7]==report['trace_frame'],'receipt differs')
    require(any(x[:3]==(5,3,0x5329d0) for x in rows[:index]),'boundary hook not installed')
    require(not any(x[0]==5 and x[1] in (2,5,14) for x in rows),'trace health failure')
    require(any(x[0]==2 and x[1]==report['trace_frame'] for x in rows[:index]),'missing preceding trace frame')
    require(any(x[0]==2 and x[1]>report['trace_frame'] for x in rows[index+1:]),'no subsequent frame; capture continuation unproved')
    after=[(i,x) for i,x in enumerate(rows) if x[:2]==(5,182)]
    require(len(after)==1 and after[0][0]>index,'missing/duplicate logger return check')
    ai,a=after[0]
    require(a[2:4]==(report['frame'],report['anchor_count']) and a[6]==0 and a[7]==report['trace_frame'],'logger return check failed')
    require(any(x[:3]==(5,3,0x192586) for x in rows[:index]),'logger return hook not installed')
    changed=[x for x in rows[index+1:ai] if x[:2]==(5,184)]
    require(len(changed)==a[4] and sum(x[6] for x in changed)==a[5],'logger drift accounting differs')
    require(len({x[3] for x in changed})==len(changed) and all(x[2]==report['frame'] and x[3]<report['anchor_count'] and 0<x[6]<=x[5] and x[7]==report['trace_frame'] for x in changed),'invalid logger drift records')
    report['logger_return_roots_unchanged']=not changed
    report['logger_changed_anchors']=[dict(index=x[3],address=x[4],bytes=x[5],changed_bytes=x[6]) for x in changed]
    return r[5]


def validate(capture,plan):
    from search_census import records
    require(hashlib.sha256((capture/'riseofnations.exe').read_bytes()).hexdigest()==plan['image_sha256'],'capture image differs')
    with (capture/'frame-snapshot.bin').open('rb') as stream:report=decode(stream,plan)
    report['receipt_ms']=check_receipts(list(records(capture/'rontrace.log')),report)
    report['completion_receipt_checked']=True
    return report

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('capture',type=Path);p.add_argument('plan',type=Path)
    a=p.parse_args()
    print(json.dumps(validate(a.capture,json.loads(a.plan.read_text())),indent=2))
