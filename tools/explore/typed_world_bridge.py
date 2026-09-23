"""Compare typed world-grid referents against every WORLD logger block.

WorldData::log_data@006b6080 defines grid extents and array order. Layout and
signedness come from PDB; no pointer extent or allocation liveness is inferred.
"""
import argparse,collections,hashlib,json,re
from pathlib import Path
from typed_fields import Decoder
from typed_state import Payload,Types,require
from typed_state_pilots import select


class ArrayBytes:
    def __init__(self,address,data):self.address=address;self.data=data
    def read(self,address,size):
        offset=address-self.address
        require(0<=offset and offset+size<=len(self.data),'array read outside extent')
        return self.data[offset:offset+size]


def decode_array(types,memory,pointer,count,path,suffix=''):
    require(pointer['status']=='pointer' and pointer['value']!=0,'missing grid pointer')
    require(type(count) is int and 0<count<=1000000,'invalid grid count')
    stride=Decoder(types,memory).size(pointer['pointee'])
    require(0<stride and stride*count<=16*1024*1024,'grid byte budget exceeded')
    base=pointer['value'];cached=ArrayBytes(base,memory.read(base,stride*count));result=[]
    for index in range(count):
        d=Decoder(types,cached);name=f'{path}[{index}]';d.decode(pointer['pointee'],base+index*stride,name)
        row=select(d.rows,name+suffix)
        require(row['status']=='value' and type(row.get('value')) is int,'grid element not a decoded integer')
        result.append(row)
    return result


def grid_plan(state,types,memory):
    rows=state['roots']['world']['rows'];sizes={}
    for name,x,y in (('size','xs','ys'),('tile_size','tile_xs','tile_ys'),
                     ('fog_size','fog_xs','fog_ys'),('reg_size','reg_xs','reg_ys')):
        fields=[select(rows,'world.'+f) for f in (name,x,y)]
        require(all(r['status']=='value' and type(r.get('value')) is int and r['value']>0 for r in fields),'invalid grid dimensions')
        n,nx,ny=[r['value'] for r in fields];require(n==nx*ny,'grid dimensions disagree');sizes[name]=n
    plan={}
    for field,size,key,suffix in (
        ('tdata','tile_size','tdata[scan].mask','.mask'),
        ('wcoord_seen','size','wcoord_seen[scan]',''),
        ('seen','fog_size','seen[scan]',''),('seen2','fog_size','seen2[scan]',''),
        ('seen3','fog_size','seen3[scan]','')):
        plan[key]=decode_array(types,memory,select(rows,'world.'+field),sizes[size],'world.'+field,suffix)
    pointers={}
    for row in rows:
        path=re.sub(r'::<base:[0-9a-f]+>','',row['path'])
        match=re.fullmatch(r'world.danger\[([0-9]+)\]',path)
        if match:
            i=int(match[1]);require(i not in pointers,'duplicate danger pointer');pointers[i]=row
    require(pointers and sorted(pointers)==list(range(len(pointers))),'missing danger pointer extent')
    plan['danger[who][scan]']=[r for owner in range(len(pointers)) for r in
        decode_array(types,memory,pointers[owner],sizes['reg_size'],f'world.danger[{owner}]')]
    return plan



def cell_plan(state,types,memory):
    rows=state['roots']['world']['rows'];pointer=select(rows,'world.wdata')
    count=select(rows,'world.size')['value']
    require(pointer['status']=='pointer' and pointer['value']!=0,'missing cell pointer')
    require(type(count) is int and 0<count<=1000000,'invalid cell count')
    stride=Decoder(types,memory).size(pointer['pointee']);base=pointer['value']
    require(0<stride and stride*count<=16*1024*1024,'cell byte budget exceeded')
    cached=ArrayBytes(base,memory.read(base,stride*count));cells=[]
    for index in range(count):
        name=f'world.wdata[{index}]';d=Decoder(types,cached)
        d.decode(pointer['pointee'],base+index*stride,name);fields={}
        for row in d.rows:
            match=re.fullmatch(re.escape(name)+r'\.([A-Za-z_][A-Za-z_0-9]*)',row['path'])
            if not match:continue
            field=match[1]
            require(field not in fields,'duplicate cell field')
            fields[field]=row
        cells.append(fields)
    return cells


# WData::log_data@006af7e0: flags immediately precedes goods; the remaining
# numeric fields occur once before was_seen. Land and flag-name text and the
# pointed collision mask are deliberately outside this scalar projection.
CELL_FIELDS=('flags','goods','who','who2','region','region2','val','land_sub',
             'light','blocked','bad','solid','down','down_who','was_seen')


def apply_cells(report,cells):
    require(cells,'empty cell plan');blocks=collections.defaultdict(list);errors=[];summaries=[]
    for row in report['rows']:
        if row['logger_path']=='GAME/FRAME/WORLD':blocks[row['owner_block_line']].append(row)
    for block,rows in blocks.items():
        starts=[i for i,r in enumerate(rows) if r['key']=='goods'];pairs=[]
        try:
            require(len(starts)==len(cells),'cell record cardinality differs')
            for index,start in enumerate(starts):
                require(start>0 and rows[start-1]['key']=='flags','missing cell flags before goods')
                end=starts[index+1]-1 if index+1<len(starts) else len(rows)
                segment=rows[start-1:end]
                # Stop at the cell terminator, leaving later flattened containers alone.
                termini=[i for i,r in enumerate(segment) if r['key']=='was_seen']
                require(len(termini)==1,'missing or duplicate cell terminator')
                segment=segment[:termini[0]+1]
                numeric=[r for r in segment if r['key'] in CELL_FIELDS]
                require([r['key'] for r in numeric]==list(CELL_FIELDS),'cell field sequence differs')
                for row in numeric:
                    # Existing WORLD scalar comparison marked all flags ambiguous.
                    require(row['status'] in ('unmapped','ambiguous_logger_ownership'),'cell would overwrite classified occurrence')
                    c=cells[index].get(row['key'])
                    require(c is not None and c['status']=='value' and type(c.get('value')) is int,'unreadable cell field')
                    pairs.append((row,c,int(row['value']),index))
        except ValueError as e:
            errors.append(dict(block=block,reason=str(e)));continue
        counts=collections.Counter()
        for row,c,value,index in pairs:
            status='exact_integer_match' if value==c['value'] else 'integer_mismatch';counts[status]+=1
            row.update(status=status,pdb_path=c['path'],address=c['address'],decoded=c['value'],printed=value,
                       cell_index=index,logger_projection='WData::log_data@006af7e0 ordered scalar record')
        summaries.append(dict(block=block,status_counts=dict(counts)))
    if not blocks:errors.append(dict(reason='no WORLD records'))
    status=dict(collections.Counter(r['status'] for r in report['rows']))
    require(sum(status.values())==report['observable_occurrences'],'cell comparison lost an occurrence')
    report.update(status_counts=status,matched_occurrences=status.get('exact_integer_match',0)+status.get('logger_transform_match',0),
                  world_cell_bridge=dict(blocks=summaries,errors=errors,distinct_cells=len(cells),
                                         scalar_fields_per_cell=len(CELL_FIELDS),referents_anchored=False),logger_parity_established=False)
    report['comparison_scope']+='; ordered world-cell scalar records, excluding land labels and collision-mask referents'
    return report

def apply(report,state,plan):
    require(report['snapshot_sha256']==state['snapshot']['sha256'],'snapshot identity differs')
    blocks=collections.defaultdict(list);errors=[];summaries=[]
    for row in report['rows']:
        if row['logger_path']=='GAME/FRAME/WORLD':blocks[row['owner_block_line']].append(row)
    require(plan and all(plan.values()),'empty grid plan')
    for block,rows in blocks.items():
        counts=collections.Counter()
        for key,storage in plan.items():
            printed=[r for r in rows if r['key']==key]
            require(all(r['status']=='unmapped' for r in printed),'grid would overwrite classified occurrence')
            try:
                require(len(printed)==len(storage),'logger/grid cardinality differs')
                values=[int(r['value']) for r in printed]
            except ValueError as e:
                for row in printed:row['status']='world_grid_unresolved'
                errors.append(dict(block=block,key=key,reason=str(e)));continue
            for index,(row,c,value) in enumerate(zip(printed,storage,values)):
                status='exact_integer_match' if value==c['value'] else 'integer_mismatch';counts[status]+=1
                row.update(status=status,pdb_path=c['path'],address=c['address'],decoded=c['value'],printed=value,
                           grid_occurrence=index,logger_projection='WorldData::log_data@006b6080 grid order')
        summaries.append(dict(block=block,status_counts=dict(counts)))
    if not blocks:errors.append(dict(reason='no WORLD records'))
    status=dict(collections.Counter(r['status'] for r in report['rows']))
    require(sum(status.values())==report['observable_occurrences'],'grid comparison lost an occurrence')
    report.update(status_counts=status,matched_occurrences=status.get('exact_integer_match',0)+status.get('logger_transform_match',0),
                  world_grid_bridge=dict(blocks=summaries,errors=errors,distinct_grid_values=sum(map(len,plan.values())),
                                         referents_anchored=False),logger_parity_established=False)
    report['comparison_scope']+='; explicit tile-mask, visibility and danger grids in each WORLD block'
    return report


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for name in ('state','comparison','types','payload'):p.add_argument(name,type=Path)
    a=p.parse_args();state=json.loads(a.state.read_text())
    require(hashlib.sha256(a.types.read_bytes()).hexdigest()==state['types_sha256'],'type export differs')
    with a.payload.open('rb') as f:require(hashlib.file_digest(f,'sha256').hexdigest()==state['snapshot']['sha256'],'payload differs')
    types=Types(json.loads(a.types.read_text()));memory=Payload(a.payload,state['snapshot'])
    try:
        plan=grid_plan(state,types,memory)
        report=apply(json.loads(a.comparison.read_text()),state,plan)
        print(json.dumps(apply_cells(report,cell_plan(state,types,memory)),indent=2))
    finally:memory.close()
