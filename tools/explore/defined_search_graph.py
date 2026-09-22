"""Model-only structural graph observation, retaining every undefined byte.

Never emits a native binary graph or claims full-record/native agreement.
Unspecified BR-node tails and unoccupied recycler slots can remain undefined;
all other record bytes and every traversed field must be defined.
"""
from defined_bytes import DefinedBytes
from search_graph import POOLS,words,validate_items
from replay_capsule import require


class GraphObserver:
    def __init__(self,read,unit,frame):
        require(type(unit) is int and 0x10000<=unit<=2**32-344 and unit%4==0,'invalid graph unit')
        require(type(frame) is int and 0<=frame<2**32,'invalid graph frame')
        self.read,self.unit,self.frame=read,unit,frame
        self.rows=[];self.seen=set();self.payloads=[];self.total=32;self.used=False

    def record(self,kind,owner,address,size,required=None):
        require(len(self.rows)<4096 and self.total+16+size<=256*1024,'graph extent cap')
        require(type(address) is int and address>=0x10000 and address%4==0 and
                0<size<=16384 and address+size<=2**32,'invalid graph record extent')
        require(all(address+size<=a or a+len(d)<=address for _,_,a,d in self.rows),'overlapping graph records')
        data=self.read(address,size)
        require(isinstance(data,DefinedBytes) and len(data)==size,'invalid graph observation')
        self.rows.append((kind,owner,address,data));self.total+=16+size
        try:bytes(data[:size if required is None else required])
        except ValueError as error:raise ValueError(f'undefined required bytes in kind {kind}/{owner} at {address:#x}') from error
        return data

    def payload(self,address):
        require(address!=0,'null PathNode')
        if address not in self.payloads:
            require(len(self.payloads)<1024,'PathNode cap');self.payloads.append(address)

    def collect(self):
        unit=words(self.record(1,0,self.unit+0x104,72))
        for owner in range(5):
            header=words(self.record(2,owner,unit[owner],28 if owner in (0,4) else 24))
            pending=[header[3]] if header[3] else []
            for address in pending:
                require(address not in self.seen and len(self.seen)<2048,'cyclic/shared tree node or cap')
                self.seen.add(address)
                plain=owner in (0,4)
                node=self.record(3,owner,address,20 if plain else 24,20 if plain else 22)
                w=words(node[:20]);pending.extend(a for a in w[:2] if a)
                live=plain or node[21]==0
                if owner==0 or owner==2 and live:self.payload(w[3])
                if owner==4:self.record(5,owner,w[3],108)
        for address in self.payloads:
            w=words(self.record(4,0,address,36))
            if w[8]:self.payload(w[8])
        for i,address in enumerate(POOLS):
            w=words(self.record(6,i,address,16))
            require(w[2]<=w[1]<=4096,'invalid pool bounds')
            if w[1]:self.record(7,i,w[0],w[1]*4,w[2]*4)
        header=(0x31475352,1,self.frame,self.unit,len(self.rows),self.total,len(self.seen),len(self.payloads))
        report,_,_=validate_items(header,self.rows,partial=True)
        report['equivalent_full_packet_bytes']=report.pop('serialized_bytes')
        return report

    def run(self):
        require(not self.used,'graph observer is single-use')
        self.used=True
        report=None;error=None
        try:report=self.collect()
        except ValueError as exc:error=str(exc)
        records=[dict(kind=k,owner=o,address=hex(a),bytes=len(d),values=d.json()) for k,o,a,d in self.rows]
        unknown=sum(v is None for r in records for v in r['values'])
        return dict(schema='model-defined-search-graph-v1',complete=error is None,reason=error,
                    structural_report=report,undefined_bytes=unknown,
                    full_record_bytes_known=error is None and unknown==0,
                    native_compared=False,records=records)
