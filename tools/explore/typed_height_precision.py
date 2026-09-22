"""Measure information lost by six-decimal projection of retained float32 bits.

This is analysis arithmetic, not simulation arithmetic. A late snapshot does
not prove the initial value of a corner, even when its printed value agrees.
"""
import argparse,hashlib,json,math,struct
from fractions import Fraction
from pathlib import Path
from typed_state import require


def value(word):return struct.unpack('<f',struct.pack('<I',word))[0]

def measure(raw):
    require(raw and len(raw)%4==0 and len(raw)<=4*1000000,'invalid height extent')
    ambiguous=[];different=[]
    for index,(word,) in enumerate(struct.iter_unpack('<I',raw)):
        x=value(word);require(math.isfinite(x),'nonfinite height')
        printed=format(x,'.6f');rational=Fraction(printed)
        center=struct.unpack('<I',struct.pack('<f',float(printed)))[0]
        candidates=[b for b in (center-1,center,center+1) if 0<=b<2**32 and math.isfinite(value(b))]
        nearest=min(candidates,key=lambda b:(abs(Fraction(value(b))-rational),b&1))
        if nearest!=word:different.append(index)
        if any(0<=b<2**32 and format(value(b),'.6f')==printed for b in (word-1,word+1)):
            ambiguous.append(dict(index=index,bits=word,projected_print=printed,nearest_decimal_bits=nearest))
    return dict(schema='typed-height-precision-v1',corners=len(raw)//4,sha256=hashlib.sha256(raw).hexdigest(),
                ambiguous_corners=len(ambiguous),nearest_decimal_reconstruction_differs=len(different),
                ambiguous=ambiguous,different_indices=different,initial_exact_bits_established=False,
                projection='IEEE float32 promoted exactly to binary64; nearest-even six-decimal formatting; exact rational distance to candidate float32 values')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('state',type=Path)
    a=p.parse_args();state=json.loads(a.state.read_text());h=state['heights'];raw=bytes.fromhex(h['exact_float32_le_hex'])
    require(len(raw)==h['count']*4 and h['count']==h['width']*h['height'],'height shape differs')
    require(hashlib.sha256(raw).hexdigest()==h['sha256'],'height bits differ')
    print(json.dumps(dict(frame=state['snapshot']['frame'],**measure(raw)),indent=2))
