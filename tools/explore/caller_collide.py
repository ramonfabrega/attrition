"""Execute one owned-image caller instruction; not a movement/world model.

The recorded restore caller returns to 0x5f7e29. Its preceding instruction at
0x5f7e05 increments UnitData.collide (PDB +0x88, short). Read code from the owned
image, retain the whole unit record, and require exactly that two-byte write.
"""
import hashlib
from pathlib import Path
from bounded_call import BoundedCall,Region
from replay_capsule import require,image_bytes,digest
from payload_replay import IMAGE_SHA256

ENTRY,STOP,RETURN=0x5f7e05,0x5f7e0c,0x5f7e29
UNIT_BYTES,OFFSET=344,0x88


class CallerCollide:
    def __init__(self,image,unit):
        image=Path(image)
        require(digest(image)==IMAGE_SHA256,'caller instruction image differs')
        require(type(unit) is int and 0x10000<=unit<2**32-UNIT_BYTES and unit%4==0,
                'invalid unit address')
        code=image_bytes(image,ENTRY,STOP-ENTRY)
        self.unit=unit
        self.code_sha256=hashlib.sha256(code).hexdigest()
        self.runner=BoundedCall([Region('code',ENTRY,code,executable=True),
                                Region('unit',unit,bytes(UNIT_BYTES),writable=True)],STOP,budget=1)

    def run(self,data):
        require(isinstance(data,bytes) and len(data)==UNIT_BYTES,'complete immutable unit required')
        regs=[0]*9;regs[4]=self.unit;regs[8]=2  # saved-image register order: EBX is slot 4.
        _,state,writes=self.runner.run(ENTRY,regs,{'unit':data})
        out=dict(state)['unit']
        require(len(writes)==1 and writes[0][:2]==(self.unit+OFFSET,2),'unexpected caller write extent')
        require(all(a==b for i,(a,b) in enumerate(zip(data,out)) if not OFFSET<=i<OFFSET+2),
                'caller changed another unit byte')
        return out,dict(entry=hex(ENTRY),stop=hex(STOP),code_sha256=self.code_sha256,
                        instructions=self.runner.instructions,write_offset=OFFSET,write_bytes=2,
                        before=int.from_bytes(data[OFFSET:OFFSET+2],'little'),
                        after=int.from_bytes(out[OFFSET:OFFSET+2],'little'))
