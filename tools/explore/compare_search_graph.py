#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Compare model graph records under an explicit, checked correspondence.

No pointer guessing, value-matching search, opaque-value normalization or native
fidelity claim. Undefined bytes remain separately reported. Generated output
belongs outside git. CLI inputs are paired defined_graph_replay reports.
"""
import argparse
import json
from pathlib import Path
from defined_bytes import DefinedBytes
from defined_search_graph import GraphObserver
from replay_capsule import require


def checked_graph(graph):
    require(graph.get('schema')=='model-defined-search-graph-v1' and graph.get('complete') is True,
            'complete defined graph required')
    rows=graph['records'];require(type(rows) is list and 0<len(rows)<=4096,'invalid record count')
    index={};total=32
    for row in rows:
        a=int(row['address'],16);n=row['bytes'];values=row['values']
        require(type(n) is int and 0<n<=16384 and type(values) is list and len(values)==n,'invalid record extent')
        total+=16+n;require(total<=256*1024,'graph extent cap')
        require(a not in index,'duplicate record address')
        require(type(row['kind']) is int and type(row['owner']) is int,'invalid record identity')
        index[a]=(row['kind'],row['owner'],DefinedBytes(tuple(values)))
    roots=[a for a,(k,o,_) in index.items() if (k,o)==(1,0)]
    require(len(roots)==1,'missing unit root')
    def read(a,n):
        require(a in index and len(index[a][2])==n,'missing or wrong-sized graph record')
        return index[a][2]
    rebuilt=GraphObserver(read,roots[0]-0x104,graph['structural_report']['frame']).run()
    require(rebuilt['complete'],str(rebuilt['reason']))
    identities={(int(r['address'],16),r['kind'],r['owner']) for r in rebuilt['records']}
    require(identities=={(a,k,o) for a,(k,o,_) in index.items()},'extra or wrongly typed graph records')
    require(rebuilt==graph,'graph metadata or observation differs from recomputation')
    return index


def word(data,offset):return int.from_bytes(bytes(data[offset:offset+4]),'little')


def pointer_offsets(kind,owner,index):
    # Typed fields from the owned PDB; ulong metrics and opaque CollBlocks
    # deliberately remain literal, even when their values resemble addresses.
    if kind==1:return (0,4,8,12,16)
    if kind==2:return (12,16,20) if owner==3 else (0,12,16,20)
    if kind==3:return (0,4,8) if owner==3 else (0,4,8,12)
    if kind==4:return (32,)
    if kind==6:return (0,)
    if kind==7:
        header=next(d for _,(k,o,d) in index.items() if (k,o)==(6,owner))
        return tuple(range(0,word(header,8)*4,4))
    return ()


def derive_correspondence(left,right):
    """Pair anchored roles and labeled ownership edges; never search by values."""
    a,b=checked_graph(left),checked_graph(right)
    require(left['structural_report']['frame']==right['structural_report']['frame'],'graph frames differ')
    mapping={};reverse={};pending=[];witness=[]
    def singleton(index,kind,owner):
        matches=[address for address,(k,o,_) in index.items() if (k,o)==(kind,owner)]
        require(len(matches)==1,'missing correspondence anchor');return matches[0]
    def bind(x,y,kind,owner,reason):
        require(bool(x)==bool(y),'corresponding edge presence differs')
        if not x:return
        require(x in a and y in b,'edge target is not observed')
        require(a[x][:2]==b[y][:2]==(kind,owner) and len(a[x][2])==len(b[y][2]),
                'edge target type/extent differs')
        if x in mapping:
            require(mapping[x]==y,'edge correspondence conflicts');return
        require(y not in reverse,'edge correspondence is not bijective')
        mapping[x]=y;reverse[y]=x;pending.append((x,y))
        witness.append(dict(left=hex(x),right=hex(y),reason=reason))
    bind(singleton(a,1,0),singleton(b,1,0),1,0,'unit saved-search root')
    for owner in range(7):
        bind(singleton(a,6,owner),singleton(b,6,owner),6,owner,f'recycler header {owner}')
    for x,y in pending:
        kind,owner,av=a[x];bv=b[y][2]
        def edge(offset,k,o,label):
            bind(word(av,offset),word(bv,offset),k,o,f'{x:#x}/{y:#x} {label}')
        if kind==1:
            for i in range(5):edge(i*4,2,i,f'saved tree {i}')
        elif kind==2:edge(12,3,owner,'tree root')
        elif kind==3:
            edge(0,3,owner,'left child');edge(4,3,owner,'right child')
            if owner in (1,2):require(av[21]==bv[21],'node removal state differs')
            if owner==0 or owner==2 and av[21]==0:edge(12,4,0,'owned PathNode')
            elif owner==1 and av[21]==0:edge(12,3,0,'referenced open node')
            elif owner==4:edge(12,5,4,'owned CollBlock')
        elif kind==4:edge(32,4,0,'PathNode parent')
        elif kind==6:
            require(word(av,4)==word(bv,4),'recycler capacity differs')
            if word(av,4):edge(0,7,owner,'recycler backing array')
    require(set(mapping)==set(a) and set(reverse)==set(b),'ownership walk did not pair every record')
    return dict(pairs=sorted(mapping.items()),witness=witness,
                method='anchored roles and labeled edges; no value-matching search')


def required_byte(kind,owner,offset,index):
    if kind==3 and owner not in (0,4):return offset<22
    if kind==7:
        header=next(d for _,(k,o,d) in index.items() if (k,o)==(6,owner))
        return offset<word(header,8)*4
    return True


def compare(left,right,pairs):
    a,b=checked_graph(left),checked_graph(right)
    require(left['structural_report']['frame']==right['structural_report']['frame'],'graph frames differ')
    require(type(pairs) is list and len(pairs)==len(a)==len(b),'complete record correspondence required')
    mapping={};reverse={}
    for x,y in pairs:
        require(type(x) is int and type(y) is int and x in a and y in b,'correspondence outside observed records')
        require(x not in mapping and y not in reverse,'correspondence is not bijective')
        require(a[x][:2]==b[y][:2] and len(a[x][2])==len(b[y][2]),'correspondence changes record type/extent')
        mapping[x]=y;reverse[y]=x
    require(set(mapping)==set(a) and set(reverse)==set(b),'incomplete correspondence')
    differences=[];unknown=[];relocated=[];raw_changes=0;known=0
    for x,y in sorted(mapping.items()):
        kind,owner,av=a[x];bv=b[y][2]
        av,bv=av.values,bv.values
        identity=dict(kind=kind,owner=owner,left_address=hex(x),right_address=hex(y))
        for i,(u,v) in enumerate(zip(av,bv)):
            if u is None or v is None:
                unknown.append(dict(**identity,offset=i,left_known=u is not None,right_known=v is not None))
            else:known+=1;raw_changes+=u!=v
        pointers=set(pointer_offsets(kind,owner,a))
        require(pointers==set(pointer_offsets(kind,owner,b)),'pointer field roles differ')
        consumed=set()
        for offset in sorted(pointers):
            u,v=word(a[x][2],offset),word(b[y][2],offset)
            consumed.update(range(offset,offset+4))
            expected=mapping.get(u,u)
            if expected!=v:
                differences.append(dict(**identity,offset=offset,bytes=4,required=True,reason='pointer correspondence differs',left=hex(u),right=hex(v)))
            elif u!=v:
                relocated.append(dict(**identity,offset=offset,left=hex(u),right=hex(v)))
        for i,(u,v) in enumerate(zip(av,bv)):
            if i not in consumed and u is not None and v is not None and u!=v:
                differences.append(dict(**identity,offset=i,bytes=1,required=required_byte(kind,owner,i,a),reason='literal byte differs',left=u,right=v))
    return dict(schema='model-graph-correspondence-v1',record_pairs=len(mapping),
                correspondence=[dict(left=hex(x),right=hex(y)) for x,y in sorted(mapping.items())],
                compared_known_bytes=known,unknown_byte_positions=unknown,
                definedness_equal=all(r['left_known']==r['right_known'] for r in unknown),
                all_bytes_known=not unknown,known_bytes_agree=not differences,
                required_fields_agree=not any(d['required'] for d in differences),
                literal_byte_values_agree=not unknown and raw_changes==0,
                raw_changed_known_bytes=raw_changes,relocated_pointer_fields=relocated,
                differences=differences,native_compared=False)


def require_known_agreement(result):
    require(result['known_bytes_agree'],'known graph values differ under correspondence')


def require_required_field_agreement(result):
    require(result['required_fields_agree'],'required graph fields differ under correspondence')


def paired_graphs(left,right):
    require(left['payload_sha256']==right['payload_sha256'],'replays belong to different payloads')
    require(left['image_sha256']==right['image_sha256'],'replay images differ')
    le,re=left['prepared_experiments'],right['prepared_experiments']
    require(le['native_unit_path']['replay']['native_intervention'] is not None and
            le['native_unit_path']['replay']['native_intervention']==re['native_unit_path']['replay']['native_intervention'],
            'intervention inputs differ')
    for doc,e in ((left,le),(right,re)):
        require(e['native_unit_path']['replay']['payload_sha256']==doc['payload_sha256'],'nested replay payload differs')
    return le['graph'],re['graph']


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('left',type=Path);ap.add_argument('right',type=Path)
    ap.add_argument('correspondence',type=Path,nargs='?',help='JSON list of [left-record-address, right-record-address] integers')
    ap.add_argument('--derive-correspondence',action='store_true',help='derive a bijection from anchored roles and labeled edges')
    ap.add_argument('--require-known-agreement',action='store_true')
    ap.add_argument('--require-required-fields',action='store_true',help='narrower assertion; still reports all known optional-byte differences')
    args=ap.parse_args();left=json.loads(args.left.read_text());right=json.loads(args.right.read_text())
    lg,rg=paired_graphs(left,right)
    if args.derive_correspondence:
        if args.correspondence is not None:ap.error('choose a supplied or derived correspondence')
        derivation=derive_correspondence(lg,rg);pairs=derivation['pairs']
    else:
        if args.correspondence is None:ap.error('supply correspondence or --derive-correspondence')
        derivation=None;pairs=json.loads(args.correspondence.read_text())
    result=compare(lg,rg,pairs)
    if derivation is not None:result['derivation']=derivation
    print(json.dumps(result,indent=2))
    if args.require_known_agreement:require_known_agreement(result)
    if args.require_required_fields:require_required_field_agreement(result)


if __name__=='__main__':main()
