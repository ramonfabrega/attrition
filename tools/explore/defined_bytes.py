"""Read-only model observations: undefined bytes have no invented value.

This does not relax guest access checks. Unknown bytes are reported without
reading their backing storage; required fields still require every byte.
"""
from dataclasses import dataclass
from unicorn import UC_MEM_READ
from replay_capsule import require


@dataclass(frozen=True)
class DefinedBytes:
    values: tuple

    def __post_init__(self):
        require(isinstance(self.values,tuple) and all(v is None or type(v) is int and 0<=v<=255
                                                    for v in self.values),'invalid defined-byte values')

    def __len__(self):return len(self.values)

    def __getitem__(self,key):
        if isinstance(key,slice):return DefinedBytes(self.values[key])
        value=self.values[key]
        require(value is not None,f'undefined required byte at offset {key}')
        return value

    def __bytes__(self):
        require(None not in self.values,'undefined required field bytes')
        return bytes(self.values)

    def json(self):return list(self.values)


def observe_bytes(runner,address,size):
    require(type(address) is int and type(size) is int and 0x10000<=address<2**32 and
            0<size<=16384 and address+size<=2**32,'invalid observation extent')
    require(not any(address<a+n and address+size>a for a,n in getattr(runner,'retired',())),
            'observation overlaps retired object')
    result=[]
    while len(result)<size:
        at=address+len(result);region=runner.region(at,1)
        require(region is not None and not region.executable,'observation needs declared data')
        count=min(size-len(result),region.address+len(region.data)-at)
        if hasattr(runner,'arena_address') and at<runner.arena_address+runner.arena_size and at+count>runner.arena_address:
            require(any(a['address']<=at and at+count<=a['address']+a['size'] for a in runner.allocations),
                    'observation outside modeled allocation')
        known=[not region.scratch or a in runner.initialized for a in range(at,at+count)]
        start=0
        while start<count:
            end=start+1
            while end<count and known[end]==known[start]:end+=1
            if known[start]:
                runner.access(runner.uc,UC_MEM_READ,at+start,end-start,0,None)
                result.extend(runner.uc.mem_read(at+start,end-start))
            else:result.extend([None]*(end-start))
            start=end
    return DefinedBytes(tuple(result))
