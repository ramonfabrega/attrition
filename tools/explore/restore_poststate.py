#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Validate a paired native post-return sidecar; compare every unit/path byte.

Packet-derived JSON belongs outside git. A path-pointer relocation is reported
separately, never a blanket mask for arbitrary pointer-shaped differences.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
from replay_capsule import require
from restore_prefix import decode
from memory_payload import validate as validate_payload
from search_census import records

FIXED, UNIT, CAP = 628, 344, 4096


def decode_post(raw, prefix_raw):
    require(len(raw) >= FIXED, 'truncated post-state')
    magic,version,frame,unit,prefix_bytes,path_bytes,mask,boundary=struct.unpack_from('<8I',raw)
    require((magic,version,prefix_bytes,mask,boundary)==(0x31505352,1,216,0x8d5,0x688faa),
            'unsupported post-state')
    require(raw[32:248]==prefix_raw,'post-state belongs to another prefix')
    p=decode(prefix_raw)
    require(p['version']==2 and (frame,unit)==(p['frame'],p['unit']), 'post-state event differs')
    registers=struct.unpack_from('<9I',raw,248)
    require(registers[3]==p['after'][3]+24 and not registers[8]&~mask,'post-return register boundary differs')
    unit_data=raw[284:FIXED]
    require(unit_data[9]==p['stack'][2] and struct.unpack_from('<H',unit_data,10)[0]==p['stack'][3],
            'post-state unit identity differs')
    pointer,capacity,length=struct.unpack_from('<3I',unit_data,0xb8)
    require(capacity<=CAP and length<=capacity and path_bytes==capacity*16 and len(raw)==FIXED+path_bytes,
            'invalid post-state path extent')
    require(not capacity or (pointer>=0x10000 and pointer+path_bytes<2**32),'invalid post-state path address')
    return dict(frame=frame,unit=unit,registers=list(registers),unit_bytes=unit_data.hex(),
                path_address=hex(pointer),capacity=capacity,length=length,path_slot_bytes=raw[FIXED:].hex(),
                path_sha256=hashlib.sha256(raw[FIXED:]).hexdigest(),packet_bytes=len(raw))


def check_receipt(rows,c):
    expected=(0,c['unit'],c['packet_bytes'],c['capacity'],c['registers'][7],c['frame'])
    require([r[2:] for r in rows if r[:2]==(5,181)]==[expected], 'post-state receipt missing or mismatched')
    require(not any(r[:2] in ((5,163),(5,182)) for r in rows), 'post-state collector failure')
    pre=[i for i,r in enumerate(rows) if r[:2]==(5,167)]
    post=next(i for i,r in enumerate(rows) if r[:2]==(5,181))
    require(len(pre)==1 and pre[0]<post, 'post-state not after one pre-payload receipt')
    depth=0; calls=0
    for r in rows[pre[0]+1:post]:
        if r[:2] in ((7,0),(8,0)):
            require(r[-1]==c['frame'],'native call frame differs from post-state')
        if r[:2]==(7,0): depth+=1;calls+=1
        if r[:2]==(8,0):
            depth-=1;require(depth>=0,'unmatched A* return before post-state')
    require(calls>0 and depth==0,'post-state precedes native A* return')


def compare(c, replay, payload_sha256):
    require(replay.get('payload_sha256')==payload_sha256,'replay belongs to another payload')
    last=replay['last']; require(last.get('returned') is True,'replay did not return')
    o=last['unit_path_observation']
    require(int(o['unit_address'],16)==c['unit'],'replay unit differs')
    a,b=bytes.fromhex(c['unit_bytes']),bytes.fromhex(o['unit_bytes'])
    require(len(a)==len(b)==UNIT,'invalid replay unit extent')
    x,y=bytes.fromhex(c['path_slot_bytes']),bytes.fromhex(o['path_slot_bytes'])
    require(0<=o['length']<=o['capacity']<=CAP and len(y)==o['capacity']*16,
            'invalid replay path extent')
    require(struct.unpack_from('<3I',b,0xb8)==(int(o['path_address'],16),o['capacity'],o['length']),
            'replay header disagrees with observation')
    changed=[i for i in range(UNIT) if a[i]!=b[i]]
    # Keep every differing slot, including inactive slots and absent slots.
    slots=[i for i in range(max(c['capacity'],o['capacity'])) if x[i*16:(i+1)*16]!=y[i*16:(i+1)*16]]
    return dict(outer_return_equal=c['registers'][7]==last['eax'],
                native_length=c['length'],replay_length=o['length'],native_capacity=c['capacity'],replay_capacity=o['capacity'],
                unit_changed_byte_offsets=changed,
                unit_changes_outside_path_pointer=[i for i in changed if not 0xb8<=i<0xbc],
                native_path_address=c['path_address'],replay_path_address=o['path_address'],
                path_changed_slot_indices=slots,all_path_slots_equal=x==y,
                active_path_equal=c['length']==o['length'] and x[:c['length']*16]==y[:o['length']*16],
                full_native_state_compared=False)


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install',type=Path);ap.add_argument('directory',type=Path)
    ap.add_argument('--replay',type=Path)
    args=ap.parse_args()
    payload=validate_payload(args.install,args.directory)
    c=decode_post((args.directory/'restore-poststate.bin').read_bytes(),(args.directory/'restore-prefix.bin').read_bytes())
    check_receipt(list(records(args.directory/'rontrace.log')),c)
    result=dict(payload_sha256=payload['sha256'],native_poststate=c)
    if args.replay: result['comparison']=compare(c,json.loads(args.replay.read_text()),payload['sha256'])
    print(json.dumps(result,indent=2))


if __name__=='__main__':main()
