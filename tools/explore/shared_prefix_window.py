"""Compare validated retained delegation payloads with their native A* entries.

Uses temporary symlink views for existing validators, not copies of broad game
payloads. Selected-unit identity comes from the packet, not the previous unit
still referenced by PathFinderData before callee setup. JSON belongs outside Git.
"""
import argparse,json,struct,tempfile,time
from contextlib import contextmanager
from pathlib import Path
from second_restore_packet import FILES,COMMON,project,validate_pair
from search_census import records
from shared_search import parse,block_deltas,field_values,SIZES,require_boundary_gap,summarize
from restore_context import decode_context
from restore_prefix import execute as verify_wrapper
from memory_payload import validate
from payload_replay import CapturedPages
from memory_inventory import decode as inventory
from replay_capsule import require,digest


@contextmanager
def packet_view(source,rows,index):
    source=source.resolve()
    with tempfile.TemporaryDirectory(prefix='shared-prefix-') as tmp:
        directory=Path(tmp);stem='second-' if index else ''
        for name in FILES:(directory/name).symlink_to(source/(stem+name))
        for name in COMMON:
            if name!='capsule-image.json':(directory/name).symlink_to(source/name)
        # Existing validators require this locally derived image binding. Use
        # the capture runner's retained hashes; validators recompute identities.
        receipt=json.loads((source/'receipt.json').read_text())['sha256']
        binding=dict(source=receipt['riseofnations.exe'],tracer=receipt['rontrace.dll'],traced=receipt['riseofnations_trace.exe'])
        (directory/'capsule-image.json').write_text(json.dumps({'sha256':binding}))
        raw=(source/'rontrace.log').read_bytes()
        require(raw[32:]==b''.join(struct.pack('<8I',*r) for r in rows),'source trace changed')
        (directory/'rontrace.log').write_bytes(raw[:32]+b''.join(struct.pack('<8I',*r) for r in project(rows,index)))
        yield directory


def extract_headers(pages,unit):
    def read(address,size):
        value=pages.read(address,size)
        require(value is not None and len(value)==size,'missing complete boundary record')
        return value
    pf=read(0xe85e80,136);pointers=struct.unpack_from('<5I',pf)
    addresses=[0xe85e80,unit,*pointers,0xc8d9b0,0xc8d820]
    blocks=[]
    for i,(address,size) in enumerate(zip(addresses,SIZES)):
        require(address or 2<=i<=6,'missing required boundary address')
        data=pf if i==0 else read(address,size) if address else b''
        blocks.append(dict(address=address,bytes=data.hex()))
    return blocks


def require_unchanged_window(before,after):
    deltas=block_deltas(before,after)
    require(deltas[0]['same_extent'],'PathFinderData identity differs')
    require(all(d['same_extent'] and d['changed_byte_offsets']==[] for d in deltas[1:]),
            'unit, active header or recycler changed before A*')
    return deltas


def experiment(install,source):
    started=time.monotonic();trace_hash=digest(source/'rontrace.log');rows=list(records(source/'rontrace.log'))
    validate_pair(source,rows);calls=parse(rows);require_boundary_gap(summarize(calls));out=[]
    for index,call in ((0,calls[0]),(1,calls[2])):
        with packet_view(source,rows,index) as directory:
            report=validate(install,directory)
            prefix=(directory/'restore-prefix.bin').read_bytes()
            context=decode_context((directory/'restore-context.bin').read_bytes(),prefix)
            require((context['frame'],context['unit']+0xb8)==(call['frame'],call['path']),
                    'delegation belongs to another call')
            inv=inventory((directory/'memory-inventory.bin').read_bytes(),prefix)
            with (directory/'memory-payload.bin').open('rb') as stream:
                blocks=extract_headers(CapturedPages(stream,report['spans'],inv['rows']),context['unit'])
            require(bytes.fromhex(blocks[1]['bytes'])==context['unit_data'],'selected unit differs from context anchor')
            wrapper=verify_wrapper(install/'riseofnations.exe',context['prefix'],repeats=2)
            out.append(dict(packet_index=index,call_sequence=call['sequence'],frame=context['frame'],
                            payload_sha256=report['sha256'],delegation_blocks=blocks,
                            delegation_fields=field_values(blocks),astar_fields=field_values(call['entry']),
                            deltas=require_unchanged_window(blocks,call['entry']),wrapper_prefix=wrapper,
                            atomic_snapshot=False,transient_writes_excluded=False))
    require(digest(source/'rontrace.log')==trace_hash,'source trace changed during comparison')
    return dict(schema='shared-prefix-window-v1',source_trace_sha256=trace_hash,windows=out,
                scope='No net unit/header change from selected delegation to A* entry; not absence of transient writes or a global writer claim',
                seconds=time.monotonic()-started)


def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('install',type=Path);ap.add_argument('capture',type=Path)
    a=ap.parse_args();print(json.dumps(experiment(a.install,a.capture),indent=2))
if __name__=='__main__':main()
