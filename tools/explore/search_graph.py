#!/usr/bin/env python3
"""Validate a bounded structural search snapshot. Reports belong outside git.

This closes tree edges and PathNode parents, not opaque CollBlock internals or
all engine state needed for resumption. No addresses are treated as code.
"""
import argparse
import json
from pathlib import Path
import struct
from search_census import records, require, scan

POOLS = (0xc8d810,0xc8d950,0xc8d860,0xc8d880,0xc8d9a0,0xc8d9b0,0xc8da70)


def words(data):
    return struct.unpack('<'+'I'*(len(data)//4), bytes(data))


def parse(raw):
    require(32 <= len(raw) <= 256*1024, 'graph size outside bound')
    h = words(raw[:32])
    require(h[:2] == (0x31475352,1) and h[5] == len(raw), 'bad graph header')
    require(0 < h[4] <= 4096 and h[6] <= 2048 and h[7] <= 1024, 'graph count outside bound')
    items, at = [], 32
    for _ in range(h[4]):
        require(at+16 <= len(raw), 'truncated graph record')
        kind, owner, address, size = words(raw[at:at+16]); at += 16
        require(1 <= kind <= 7 and address >= 0x10000 and address%4 == 0 and
                0 < size <= 16384 and size%4 == 0 and at+size <= len(raw) and
                address+size <= 2**32, 'invalid graph region')
        for _, _, a, data in items:
            require(address+size <= a or a+len(data) <= address, 'overlapping graph regions')
        items.append((kind,owner,address,raw[at:at+size])); at += size
    require(at == len(raw), 'trailing graph bytes')
    return h, items


def validate(raw):
    h, items = parse(raw)
    return validate_items(h,items)


def validate_items(h,items,*,partial=False):
    """Internal structural checks; callers separately enforce extent/framing.

    Partial model records retain undefined bytes. Only the explicitly sized
    node fields and occupied recycler entries are consumed in that mode.
    Native binary validation always uses complete records.
    """
    groups = {(k,o): [(a,d) for kind,owner,a,d in items if (kind,owner)==(k,o)]
              for k,o,_,_ in items}
    require(set(groups) <= {(1,0),(4,0)} | {(k,i) for k in (2,3,5) for i in range(5)} |
            {(k,i) for k in (6,7) for i in range(7)}, 'unknown owner/kind')
    def single(k,o,size):
        rows = groups.get((k,o),[])
        require(len(rows)==1 and len(rows[0][1])==size, f'missing/wrong singleton {k}/{o}')
        return rows[0]
    unit_address, unit_data = single(1,0,72)
    require(unit_address == h[3]+0x104, 'unit address mismatch')
    unit = words(unit_data)
    trees, all_nodes, active_nodes = [], {}, []
    for i in range(5):
        address, data = single(2,i,28 if i in (0,4) else 24)
        tree = words(data); require(address == unit[i], 'unit/container mismatch')
        nodes = dict(groups.get((3,i),[]))
        require(all(len(d)==(20 if i in (0,4) else 24) for d in nodes.values()), 'wrong tree node size')
        pending = [(tree[3],0)] if tree[3] else []
        visited, live = set(), {}
        while pending:
            pointer, parent = pending.pop()
            require(pointer in nodes and pointer not in visited, 'missing/cyclic/shared tree node')
            visited.add(pointer); n = words(nodes[pointer][:20] if partial else nodes[pointer])
            require(n[2] == parent, 'tree parent mismatch')
            if i in (0,4) or nodes[pointer][21] == 0:
                live[pointer] = n
            if n[0]: pending.append((n[0],pointer))
            if n[1]: pending.append((n[1],pointer))
        require(visited == set(nodes), 'unreachable tree node')
        require(len(live) == tree[2], 'logical container length mismatch')
        require(tree[4] == 0 or tree[4] in nodes, 'cursor outside tree')
        require(tree[5] == 0 or tree[5] in nodes, 'cursor parent outside tree')
        trees.append(tree); active_nodes.append(live); all_nodes.update(nodes)
    require(len(all_nodes)==h[6], 'node count mismatch')
    owned = [n[3] for i in (0,2) for n in active_nodes[i].values()]
    require(all(owned) and len(set(owned))==len(owned), 'duplicate/null owned PathNode')
    refs = [n[3] for n in active_nodes[1].values()]
    require(len(set(refs)) == len(refs) and set(refs)==set(active_nodes[0]), 'open/ref ownership mismatch')
    payloads = dict(groups.get((4,0),[]))
    require(len(payloads)==h[7] and all(len(d)==36 for d in payloads.values()), 'wrong PathNode set')
    closure = set()
    for pointer in owned:
        chain = set()
        while pointer:
            require(pointer in payloads and pointer not in chain, 'missing/cyclic PathNode parent')
            chain.add(pointer); closure.add(pointer); pointer = words(payloads[pointer])[8]
    require(closure == set(payloads), 'unreachable PathNode payload')
    blocks = dict(groups.get((5,4),[]))
    require(all(len(d)==108 for d in blocks.values()) and
            set(blocks)=={n[3] for n in active_nodes[4].values()}, 'wrong CollBlock payloads')
    require(not any(k==5 and o!=4 for k,o in groups), 'unexpected block owner')
    pool_data = []
    for i in range(7):
        address, data = single(6,i,16); p = words(data)
        require(address==POOLS[i] and p[2]<=p[1]<=4096 and (not p[1] or p[0]), 'invalid pool header')
        if p[1]:
            a,d = single(7,i,p[1]*4); require(a==p[0], 'pool array mismatch')
        else:
            require((7,i) not in groups, 'unexpected empty pool array'); d=b''
        if i==6:
            require(not set(owned).intersection((words(d[:p[2]*4]) if partial else words(d)[:p[2]])), 'owned PathNode is already recycled')
        pool_data.append((p,d))
    report = {'frame':h[2], 'records':h[4], 'serialized_bytes':h[5],
              'declared_bytes':sum(len(d) for _,_,_,d in items),
              'tree_nodes':h[6], 'logical_lengths':[t[2] for t in trees],
              'physical_lengths':[len(groups.get((3,i),[])) for i in range(5)],
              'owned_pathnodes':len(owned), 'parent_closure_pathnodes':len(payloads),
              'collblocks':len(blocks), 'pathnode_pool_available':pool_data[6][0][1]-pool_data[6][0][2]}
    return report, items, active_nodes


def check_trace(raw, trace):
    report, items, _ = validate(raw)
    rows=list(records(trace)); census=scan(iter(rows))
    require(census['events'], 'no natural suspension')
    e=census['events'][0]; h,_=parse(raw)
    receipt=[r for r in rows if r[:2]==(5,150)]
    require(not any(r[:2]==(5,151) for r in rows) and
            receipt==[(5,150,0,h[3],h[6],h[7],h[5],h[2])], 'missing/failed graph receipt')
    require(e['frame']==h[2] and e['unit']==h[3], 'wrong suspension owner/frame')
    for i in range(5):
        a,d=next((a,d) for k,o,a,d in items if (k,o)==(2,i)); w=words(d)
        require(e['containers'][i]==[a,w[2],w[3]], 'graph/census container mismatch')
    for i in range(7):
        w=next(words(d) for k,o,_,d in items if (k,o)==(6,i))
        require(e['pools'][i]==list(w[:3]), 'graph/census pool mismatch')
    return report


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('directory',type=Path); args=ap.parse_args()
    print(json.dumps(check_trace((args.directory/'search-graph.bin').read_bytes(),
                                args.directory/'rontrace.log'),indent=2))


if __name__=='__main__': main()
