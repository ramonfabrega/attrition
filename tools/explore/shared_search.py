"""Validate bounded native A* shared-state boundary records; JSON belongs outside Git."""
import argparse,json,struct
from pathlib import Path
from replay_capsule import require
from search_census import records

SIZES=(136,344,28,24,24,28,24,16,16)

def parse(rows):
    rows=list(rows);calls=[];active=None;pending=None;i=0
    require(not any(r[:2] in ((5,204),(5,205)) for r in rows),'shared collector failed or capped')
    while i<len(rows):
        row=rows[i];i+=1
        if row[:2]==(7,0) and 224<=row[7]<=225:
            require(active is None and pending is None and len(calls)<8,'nested or unfinished shared call')
            active=dict(sequence=len(calls)+1,frame=row[7],self=row[2],path=row[3],step=row[4],anti=row[5])
            calls.append(active)
        elif row[:2]==(8,0) and 224<=row[7]<=225:
            require(active is not None and 'entry' in active and active['frame']==row[7],'unmatched shared return')
            active['return']=row[2];pending=active;active=None
        elif row[:2]==(5,200):
            seq,phase,self,path,ret,frame=row[2:]
            call=active if phase==0 else pending if phase==1 else None
            require(call is not None and (seq,self,path,frame)==(call['sequence'],call['self'],call['path'],call['frame']),
                    'snapshot call identity differs')
            name='entry' if phase==0 else 'exit'
            require(name not in call and ret==(0xffffffff if phase==0 else call['return']),'snapshot phase/return differs')
            blocks=[];total=0
            for index,size in enumerate(SIZES):
                require(i<len(rows),'missing descriptor');d=rows[i];i+=1
                require(d[:4]==(5,201,seq,index) and d[6:]==(0,frame),'invalid descriptor')
                address,n=d[4:6]
                require(n==size or (2<=index<=6 and address==n==0),'invalid block size')
                require(not n or 0x10000<=address<address+n<=2**32,'invalid block address')
                data=bytearray()
                for offset in range(0,n,8):
                    require(i<len(rows),'missing block words');w=rows[i];i+=1
                    require(w[:5]==(5,202,seq,index,offset) and w[7]==frame,'word order/identity differs')
                    remaining=min(8,n-offset)
                    require(remaining==8 or w[6]==0,'nonzero tail padding')
                    data.extend(struct.pack('<2I',*w[5:7])[:remaining])
                blocks.append(dict(address=address,bytes=data.hex()));total+=n
            require(i<len(rows) and rows[i]==(5,203,seq,phase,9,total,0,frame),'missing snapshot completion');i+=1
            words=struct.unpack('<34I',bytes.fromhex(blocks[0]['bytes']))
            require(blocks[0]['address']==self+0x40 and self==0xe85e40 and blocks[1]['address']==words[5] and words[5]+0xb8==path,
                    'pathfinder/unit identity differs')
            require([b['address'] for b in blocks[2:7]]==list(words[:5]),'active header identity differs')
            require([b['address'] for b in blocks[7:]]==[0xc8d9b0,0xc8d820],'pool identity differs')
            call[name]=blocks
            if phase==1:pending=None
        elif row[0]==5 and 201<=row[1]<=203:
            raise ValueError('orphan shared record')
    require(calls and active is None and pending is None and all('exit' in c for c in calls),'incomplete shared calls')
    return calls


def field_values(blocks):
    def word(index,offset):
        b=bytes.fromhex(blocks[index]['bytes']);return struct.unpack_from('<I',b,offset)[0] if b else None
    return dict(valid_root=word(6,12),block_root=word(5,12),open_node_pool_length=word(7,8),open_ref_pool_length=word(8,8))


def summarize(calls):
    require(len(calls)>=3,'missing continuation triple')
    a,b,c=calls[:3]
    require(a['frame']==b['frame']==224 and c['frame']==225 and a['path']==c['path']!=b['path'] and
            [x['return'] for x in (a,b,c)]==[0xffffffff,0,1],'unexpected observed call sequence')
    stages=[dict(call=x['sequence'],phase=phase,fields=field_values(x[phase])) for x in (a,b,c) for phase in ('entry','exit')]
    return dict(calls=calls,stages=stages,
                intervening_changed_fields=[k for k,v in stages[2]['fields'].items() if v!=stages[3]['fields'][k]],
                intervening_exit_matches_next_entry=stages[3]['fields']==stages[4]['fields'],
                native_writer_instruction_identified=False)


def block_deltas(before,after):
    require(len(before)==len(after)==9,'complete boundary records required')
    result=[]
    for index,(a,b) in enumerate(zip(before,after)):
        left,right=bytes.fromhex(a['bytes']),bytes.fromhex(b['bytes'])
        same=a['address']==b['address'] and len(left)==len(right)
        result.append(dict(index=index,before_address=a['address'],after_address=b['address'],
                           same_extent=same,changed_byte_offsets=[i for i,(x,y) in enumerate(zip(left,right)) if x!=y] if same else None))
    return result


def require_boundary_gap(report):
    a,b,c=report['calls'][:3]
    snapshots=[a['exit'],b['entry'],b['exit'],c['entry']]
    values=[field_values(s) for s in snapshots]
    require(values[0]==dict(valid_root=0,block_root=0,open_node_pool_length=0,open_ref_pool_length=1),
            'first exit differs')
    require(all(values[i]['valid_root'] and values[i]['block_root'] for i in (1,2,3)),
            'later active roots are absent')
    expected=[{'valid_root','block_root'},
              {'valid_root','open_node_pool_length','open_ref_pool_length'},
              {'valid_root','block_root','open_ref_pool_length'}]
    for i,changed in enumerate(expected):
        require({k for k in values[i] if values[i][k]!=values[i+1][k]}==changed,'boundary attribution differs')
    require([(v['open_node_pool_length'],v['open_ref_pool_length']) for v in values]==[(0,1),(0,1),(1,0),(1,2)],
            'pool trajectory differs')
    require(all([r['address'] for r in s[2:]]==[r['address'] for r in snapshots[0][2:]] for s in snapshots[1:]),
            'active header identities changed')
    for call,identity in zip((a,b,c),((0,16),(0,22),(0,16))):
        for phase in ('entry','exit'):
            unit=bytes.fromhex(call[phase][1]['bytes'])
            require((unit[9],int.from_bytes(unit[10:12],'little'))==identity,'call owner/id differs')
    return [block_deltas(x,y) for x,y in zip(snapshots,snapshots[1:])]


def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('trace',type=Path);a=ap.parse_args()
    report=summarize(parse(records(a.trace)))
    report['bounded_gap_deltas']=require_boundary_gap(report)
    print(json.dumps(report,indent=2))
if __name__=='__main__':main()
