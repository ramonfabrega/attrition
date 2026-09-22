"""Validate two explicitly bounded restore packets and project one for existing readers.

Projection removes only the other packet's collector receipts. It preserves all
native call/census events and startup metadata. Original trace provenance and
row indices remain in a manifest; output and game-derived bytes stay outside git.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import struct
from replay_capsule import require
from search_census import records
from restore_prefix import decode
from restore_poststate import decode_post,check_receipt

PACKET_TAGS={150,151,161,162,163,164,165,166,167,168,181,182,183,184,185,186,187}
FILES=('search-graph.bin','restore-prefix.bin','restore-context.bin',
       'memory-inventory.bin','memory-payload.bin','restore-poststate.bin')
COMMON=('capsule-image.json','rontrace.dll','riseofnations_trace.exe')


def intervals(rows):
    require(not any(r[0]==5 and r[1] in (151,163,166,168,182,185,187,192,193,194) for r in rows),
            'collector failure in paired trace')
    spans=[];active=None
    for i,r in enumerate(rows):
        if r[:2]==(5,190):
            require(active is None and len(spans)<2 and r[2]==len(spans) and r[6]==0,
                    'out-of-order/duplicate packet begin')
            require(r[3]>=0x10000 and r[4]<8 and r[5]<512,'invalid packet identity')
            active=(i,r)
        elif r[:2]==(5,191):
            require(active is not None,'packet end without begin')
            begin,b=active
            require((r[2],r[3],r[5],r[6],r[7])==(b[2],b[3],0,0,b[7]),'packet end identity differs')
            require(r[2]!=0 or r[4]==0xffffffff,'first packet did not suspend')
            spans.append((begin,i));active=None
        elif r[0]==5 and r[1] in PACKET_TAGS:
            require(active is not None,'orphan packet receipt')
    require(active is None and len(spans)==2,'two completed packets required')
    a,b=(rows[start] for start,_ in spans)
    require(a[3:6]==b[3:6] and a[7]<=b[7],'second unit identity or frame differs')
    return spans


def project(rows,index):
    require(type(index) is int and index in (0,1),'packet index must be zero or one')
    spans=intervals(rows);begin,end=spans[index]
    return [r for i,r in enumerate(rows) if not (r[0]==5 and r[1] in PACKET_TAGS) or begin<=i<=end]


def validate_pair(directory,rows):
    spans=intervals(rows);packets=[]
    for index,(begin,end) in enumerate(spans):
        stem='second-' if index else ''
        prefix=(directory/(stem+'restore-prefix.bin')).read_bytes()
        p=decode(prefix)
        post=decode_post((directory/(stem+'restore-poststate.bin')).read_bytes(),prefix)
        b,e=rows[begin],rows[end]
        require((p['unit'],*p['stack'][2:],p['frame'])==(*b[3:6],b[7]),'prefix marker identity differs')
        require(post['registers'][7]==e[4],'post return differs from packet end')
        require((post['intervention'] is not None)==(index==0),'intervention on wrong packet')
        require(tuple(p['modes'])==(300,1),'delegation modes differ')
        if index==0:
            unit=bytes.fromhex(post['unit_bytes'])
            require(all(struct.unpack_from('<5I',unit,0x104)),'first post-state has absent roots')
        check_receipt(project(rows,index),post)
        packets.append(post)
    return packets


def materialize(install,source,destination,index):
    # Validate both unit/path witnesses before creating any projection.
    rows=list(records(source/'rontrace.log'));packets=validate_pair(source,rows)
    selected=project(rows,index);spans=intervals(rows)
    require(not destination.exists(),'destination must not already exist')
    destination.mkdir(parents=True)
    (destination/'.incomplete').write_text('Projection validation has not finished.\n')
    stem='second-' if index else ''
    for name in FILES:shutil.copyfile(source/(stem+name),destination/name)
    for name in COMMON:shutil.copyfile(source/name,destination/name)
    original=(source/'rontrace.log').read_bytes()
    require(hashlib.sha256(original[32:]).digest()==hashlib.sha256(b''.join(struct.pack('<8I',*r) for r in rows)).digest(),
            'source trace changed during projection')
    (destination/'rontrace.log').write_bytes(original[:32]+b''.join(struct.pack('<8I',*r) for r in selected))
    from memory_payload import validate
    payload=validate(install,destination)
    copied_post=decode_post((destination/'restore-poststate.bin').read_bytes(),
                            (destination/'restore-prefix.bin').read_bytes())
    require(copied_post==packets[index],'post-state changed during projection')
    check_receipt(list(records(destination/'rontrace.log')),packets[index])
    manifest=dict(schema='restore-packet-projection-v1',packet_index=index,
                  source=str(source.resolve()),source_trace_sha256=hashlib.sha256(original).hexdigest(),
                  source_record_interval=list(spans[index]),source_records=len(rows),projected_records=len(selected),
                  native_calls_preserved=True,payload_sha256=payload['sha256'],
                  files={name:hashlib.sha256((destination/name).read_bytes()).hexdigest()
                         for name in (*FILES,*COMMON,'rontrace.log') if name!='memory-payload.bin'})
    (destination/'packet-projection.json').write_text(json.dumps(manifest,indent=2)+'\n')
    (destination/'.incomplete').unlink()
    return manifest


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install',type=Path);ap.add_argument('source',type=Path)
    ap.add_argument('destination',type=Path);ap.add_argument('--packet',type=int,choices=(0,1),required=True)
    args=ap.parse_args()
    print(json.dumps(materialize(args.install,args.source,args.destination,args.packet),indent=2))

if __name__=='__main__':main()
