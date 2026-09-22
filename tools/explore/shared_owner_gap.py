"""Identify retained pathing owners outside the bounded native A* census.

Uses validated local payloads only. Derived records and JSON belong outside Git.
An owner transition is not attribution of tree/recycler writes to that owner.
"""
import argparse,json,struct
from pathlib import Path
from shared_prefix_window import packet_view
from shared_search import parse,require_boundary_gap,summarize
from second_restore_packet import validate_pair
from search_census import records
from memory_payload import validate
from memory_inventory import decode as inventory
from payload_replay import CapturedPages
from replay_capsule import require,digest


def read_owner(pages):
    def read(address,size):
        require(0x10000<=address<address+size<=2**32,'invalid owner record address')
        data=pages.read(address,size)
        require(data is not None and len(data)==size,'incomplete owner record')
        return data
    pf=read(0xe85e80,136)
    address=struct.unpack_from('<I',pf,20)[0]
    unit=read(address,344);owner=unit[9];uid=struct.unpack_from('<H',unit,10)[0]
    require(owner<8,'invalid owner')
    registry_address=0xc0aeb4+owner*28
    registry=read(registry_address,16);length,capacity,increment,slots=struct.unpack('<4I',registry)
    require(uid<length<=capacity<=32768,'invalid unit registry extent')
    slot=read(slots+uid*4,4)
    require(struct.unpack('<I',slot)[0]==address,'previous owner no longer occupies registry slot')
    return dict(address=address,owner=owner,id=uid,unit_bytes=unit.hex(),
                registry_address=registry_address,registry_bytes=registry.hex(),slot_bytes=slot.hex())


def require_gap(calls,observations):
    require(len(observations)==2,'both delegation owners required')
    a,b,c=calls[:3]
    previous=observations[1]
    require(previous['address']!=b['path']-0xb8,'no owner transition after intervening A*')
    require(all(previous['address']!=x['path']-0xb8 for x in (a,b,c)),
            'previous owner is represented in the selected A* triple')
    return dict(intervening_astar_unit=b['path']-0xb8,
                next_delegation_previous_unit=previous['address'],
                owner_transition_observed=True,writer_instruction_identified=False,
                shared_header_writes_attributed=False,atomic_snapshot=False)


def experiment(install,source):
    before=digest(source/'rontrace.log');rows=list(records(source/'rontrace.log'))
    validate_pair(source,rows);calls=parse(rows);require_boundary_gap(summarize(calls))
    observations=[]
    for index in (0,1):
        with packet_view(source,rows,index) as view:
            report=validate(install,view)
            inv=inventory((view/'memory-inventory.bin').read_bytes(),(view/'restore-prefix.bin').read_bytes())
            with (view/'memory-payload.bin').open('rb') as stream:
                owner=read_owner(CapturedPages(stream,report['spans'],inv['rows']))
            observations.append(dict(packet_index=index,frame=report['frame'],payload_sha256=report['sha256'],**owner))
    gap=require_gap(calls,observations)
    require(digest(source/'rontrace.log')==before,'source trace changed')
    return dict(schema='shared-owner-gap-v1',source_trace_sha256=before,observations=observations,
                astar_units=[dict(sequence=c['sequence'],frame=c['frame'],address=c['path']-0xb8) for c in calls],gap=gap)


def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('install',type=Path);ap.add_argument('capture',type=Path)
    a=ap.parse_args();print(json.dumps(experiment(a.install,a.capture),indent=2))
if __name__=='__main__':main()
