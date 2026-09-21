#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Validate a delegation context sidecar and regenerate its whole coordinate table.

This validates captured inputs; it does not yet execute the resumed search or
interpret the opaque FXSAVE image. Missing context is never synthesized.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
from coord_table_probe import initialize
from replay_capsule import digest, require
from restore_prefix import decode, check
from search_census import records

FIXED_BYTES, TABLE_MAX = 792, 4096*192
UNIT_BYTES = 0x158


def decode_context(raw, prefix_raw):
    require(len(raw)>=FIXED_BYTES,'truncated context')
    h=struct.unpack_from('<16I',raw)
    magic,version,frame,unit,prefix_bytes,table_bytes,teb,origin,center=h[:9]
    tib=h[9:]
    require(magic==0x31585452 and version in (1,2) and prefix_bytes==216,'unsupported context')
    fixed=FIXED_BYTES+(UNIT_BYTES if version==2 else 0)
    require(0<table_bytes<=TABLE_MAX and table_bytes%192==0 and
            len(raw)==fixed+table_bytes,'invalid context extent')
    require(raw[64:280]==prefix_raw,'context belongs to a different prefix packet')
    prefix=decode(prefix_raw)
    require(prefix['version']==2 and (frame,unit)==(prefix['frame'],prefix['unit']),
            'context event identity differs')
    require(0x10000<=teb<=2**32-28 and tib[6]==teb and
            tib[2]<prefix['after'][3] and prefix['after'][3]+48<=tib[1],
            'invalid thread bounds or self pointer')
    require(0x10000<=origin and center==origin+table_bytes//2 and origin+table_bytes<2**32,
            'invalid coordinate allocation pointers')
    require(teb+28<=origin or origin+table_bytes<=teb,'thread and table overlap')
    unit_data=raw[792:fixed] if version==2 else None
    if unit_data is not None:
        require(unit==prefix['deps'][3] and unit+UNIT_BYTES<2**32 and
                unit_data[9]==prefix['stack'][2] and
                struct.unpack_from('<H',unit_data,10)[0]==prefix['stack'][3] and
                unit_data[16:24]==struct.pack('<2I',*prefix['deps'][5:7]),
                'unit identity or coordinates differ')
        require((unit+UNIT_BYTES<=teb or teb+28<=unit) and
                (unit+UNIT_BYTES<=origin or origin+table_bytes<=unit), 'unit overlaps context')
    return {'version':version,'fixed_bytes':fixed,'unit_data':unit_data,'frame':frame,'unit':unit,'teb':teb,'tib':tib,'origin':origin,'center':center,
            'size':table_bytes//192,'table_bytes':table_bytes,'prefix':prefix,
            'fxsave':raw[280:792],'table':raw[fixed:]}


def compare_table(image, context):
    report, generated=initialize(image,context['size'])
    require(generated==context['table'],'live table differs from original initializer')
    return report


def check_receipt(rows,c):
    expected=(0,c['unit'],c['fixed_bytes']+c['table_bytes'],c['table_bytes'],c['teb'],c['frame'])
    require([r[2:] for r in rows if r[:2]==(5,164)]==[expected],'missing/mismatched context receipt')
    require(sum(r[:2]==(5,162) for r in rows)==1,'missing/duplicate delegation receipt')
    require(not any(r[:2]==(5,163) for r in rows),'collector failure receipt')
    delegation=next(i for i,r in enumerate(rows) if r[:2]==(5,162))
    receipt=next(i for i,r in enumerate(rows) if r[:2]==(5,164))
    following=next((i for i,r in enumerate(rows[delegation+1:],delegation+1) if r[:2]==(7,0)),len(rows))
    require(following<len(rows),'missing following native call')
    require(delegation<receipt<following,'context receipt outside delegation event')


def check_unit_graph(c,graph_raw):
    if c['unit_data'] is None:return
    from search_graph import validate as validate_graph
    _,items,_=validate_graph(graph_raw)
    unit_records=[(address,data) for kind,owner,address,data in items if (kind,owner)==(1,0)]
    require(unit_records==[(c['unit']+0x104,c['unit_data'][0x104:0x14c])],
            'unit graph changed between entry and delegation')


def validate(install, directory):
    prefix,graph=check(directory)
    c=decode_context((directory/'restore-context.bin').read_bytes(),
                     (directory/'restore-prefix.bin').read_bytes())
    binding=json.loads((directory/'capsule-image.json').read_text())['sha256']
    require(binding['source']==digest(install/'riseofnations.exe') and
            binding['tracer']==digest(directory/'rontrace.dll') and
            binding['traced']==digest(directory/'riseofnations_trace.exe'),'context image binding differs')
    check_receipt(list(records(directory/'rontrace.log')),c)
    check_unit_graph(c,(directory/'search-graph.bin').read_bytes())
    table=compare_table(install/'riseofnations.exe',c)
    return {'frame':c['frame'],'owner':prefix['stack'][2],'id':prefix['stack'][3],
            'table':table,'thread_prefix_bytes':28,'fxsave_bytes':len(c['fxsave']),
            'fxsave_sha256':hashlib.sha256(c['fxsave']).hexdigest(),
            'context_version':c['version'],'unit_bytes':len(c['unit_data'] or b''),
            'graph_records':graph['records'],'resumption_executed':False,
            'limit':'FXSAVE is retained but not interpreted or loaded; only FS offset zero is presently modeled'}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install',type=Path);parser.add_argument('directory',type=Path)
    args=parser.parse_args()
    print(json.dumps(validate(args.install,args.directory),indent=2))


if __name__=='__main__':main()
