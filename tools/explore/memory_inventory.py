#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Validate a metadata-only address-space inventory and estimate candidate bytes.

This neither reads a running process nor treats accessible memory as a coherent
simulation snapshot. Inventory and numerical reports belong outside git.
"""
import argparse
from collections import defaultdict
import json
from pathlib import Path
import struct
from replay_capsule import require
from restore_context import validate as validate_context, decode_context
from search_census import records

HEADER, RECORD, MAX_RECORDS = 280, 28, 8192


def decode(raw,prefix_raw):
    require(len(raw)>=HEADER and len(prefix_raw)==216,'truncated memory inventory/prefix')
    h=struct.unpack_from('<16I',raw)
    magic,version,frame,unit,begin,end,page,record_bytes,count,status,elapsed,cursor,main,observer,cap,ms=h
    require((magic,version,record_bytes,cap,ms)==(0x31494d52,1,RECORD,MAX_RECORDS,2000),
            'unsupported memory inventory')
    require(count<=cap and len(raw)==HEADER+count*RECORD,'invalid inventory extent')
    require(raw[64:HEADER]==prefix_raw,'inventory belongs to another prefix')
    pf=struct.unpack_from('<4I',prefix_raw)
    require((frame,unit)==pf[2:4],'inventory event differs')
    require(status==0,f'incomplete inventory: status={status}, cursor={cursor:x}, count={count}')
    require(0x10000<=begin<end<2**32 and 0<page<=65536 and not page&(page-1) and
            begin%page==end%page==0 and cursor==end and elapsed<ms,
            'invalid inventory bounds or elapsed time')
    rows=[];at=begin
    for pos in range(HEADER,len(raw),RECORD):
        r=struct.unpack_from('<7I',raw,pos)
        base,allocation,allocation_protect,size,state,protect,kind=r
        require(base==at and size>0 and base+size<2**32 and size%page==0 and base%page==0 and
                state in (0x1000,0x2000,0x10000),'inventory gap/overlap or invalid range')
        require(at<end,'extra inventory range')
        at=min(end,base+size);rows.append(r)
    require(at==end,'inventory does not cover declared bounds')
    require(main==0x400000 and begin<=observer<end,'invalid image identity/address')
    return {'frame':frame,'unit':unit,'begin':begin,'end':end,'page':page,
            'elapsed_ms':elapsed,'main':main,'observer':observer,'rows':rows,'bytes':len(raw)}


def classify(c,context):
    rows=c['rows'];observer_row=next(r for r in rows if r[0]<=c['observer']<r[0]+r[3])
    require(observer_row[4]==0x1000 and observer_row[6]==0x1000000 and observer_row[1]!=c['main'],
            'observer is not a distinct committed image')
    observer_allocation=observer_row[1]
    totals=defaultdict(lambda:{'ranges':0,'bytes':0});allocations=defaultdict(int)
    for base,allocation,ap,size,state,protect,kind in rows:
        size=min(base+size,c['end'])-base
        if state==0x10000:label='free'
        elif state==0x2000:label='reserved'
        elif protect&0x100:label='guard'
        elif protect&0xff not in (2,4,8,0x20,0x40,0x80):label='not_readable'
        elif allocation==observer_allocation:label='observer_image'
        elif base<context['tib'][1] and base+size>context['tib'][2]:label='observer_stack'
        elif protect&0xff in (0x20,0x40,0x80):label='executable'
        elif kind==0x1000000 and allocation==c['main']:label='main_image_data'
        elif kind==0x20000:label='private_data'
        elif kind==0x1000000:label='other_image_data'
        elif kind==0x40000:label='mapped_data'
        else:label='unknown_type'
        totals[label]['ranges']+=1;totals[label]['bytes']+=size
        if label in ('main_image_data','private_data'):allocations[allocation]+=size
    candidate=sum(totals[k]['bytes'] for k in ('main_image_data','private_data'))
    roots={'world_pointer_slot':0xc06188,'unit':context['unit'],
           'unit_type':struct.unpack_from('<I',context['unit_data'],24)[0],
           'coordinate_table':context['origin'],'thread':context['teb']}
    root_ranges={name:next(({'base':hex(r[0]),'size':r[3],'allocation':hex(r[1]),
                            'state':hex(r[4]),'protect':hex(r[5]),'type':hex(r[6])}
                           for r in rows if r[0]<=address<r[0]+r[3]),None)
                 for name,address in roots.items()}
    return {'frame':c['frame'],'inventory_records':len(rows),'inventory_bytes':c['bytes'],
            'enumeration_ms':c['elapsed_ms'],'address_begin':hex(c['begin']),'address_end':hex(c['end']),
            'categories':dict(totals),'candidate_bytes':candidate,
            'fits_experimental_caps_mib':{str(m):candidate<=m*1024**2 for m in (128,256,512,1024)},
            'largest_candidate_allocations':[{'base':hex(a),'bytes':n} for a,n in
                                            sorted(allocations.items(),key=lambda p:(-p[1],p[0]))[:10]],
            'root_ranges':root_ranges,'memory_payload_captured':False,
            'limit':'Candidate private data may contain assets; this is an inventory, not an atomic snapshot'}


def check_receipts(rows,c):
    expected=(0,c['unit'],c['bytes'],len(c['rows']),c['elapsed_ms'],c['frame'])
    require([r[2:] for r in rows if r[:2]==(5,165)]==[expected],'missing/mismatched inventory receipt')
    require(not any(r[:2]==(5,166) for r in rows),'inventory failure receipt')
    starts=[i for i,r in enumerate(rows) if r[:2]==(5,164)]
    require(len(starts)==1,'missing/duplicate context receipt')
    finish=next(i for i,r in enumerate(rows) if r[:2]==(5,165))
    following=next((i for i,r in enumerate(rows[starts[0]+1:],starts[0]+1) if r[:2]==(7,0)),len(rows))
    require(starts[0]<finish<following<len(rows),'inventory outside delegation event')


def validate(install,directory):
    validate_context(install,directory)
    context=decode_context((directory/'restore-context.bin').read_bytes(),(directory/'restore-prefix.bin').read_bytes())
    require(context['version']==2,'inventory needs the full unit context')
    c=decode((directory/'memory-inventory.bin').read_bytes(),(directory/'restore-prefix.bin').read_bytes())
    check_receipts(list(records(directory/'rontrace.log')),c)
    return classify(c,context)


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install',type=Path);ap.add_argument('directory',type=Path)
    args=ap.parse_args();print(json.dumps(validate(args.install,args.directory),indent=2))


if __name__=='__main__':main()
