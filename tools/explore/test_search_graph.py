import struct
import unittest
from search_graph import validate, POOLS


def w(*values): return struct.pack('<'+'I'*len(values),*values)


def fixture():
    # Five one-node containers; refs points to the open node, closed owns a
    # second PathNode whose parent is the first. All memory is authored.
    items=[(1,0,0x10104,w(*(0x20000+i*0x100 for i in range(5)),*([0]*13)))]
    for i in range(5):
        size=7 if i in (0,4) else 6
        items.append((2,i,0x20000+i*0x100,w(0,0,1,0x30000+i*0x100,0,0,*([0]*(size-6)))))
        data=(0x40000,0x30000,0x40100,42,0x50000)[i]
        items.append((3,i,0x30000+i*0x100,w(0,0,0,data,7,*([0]*(size-5 if i not in (0,4) else 0)))))
    items.extend([(4,0,0x40000,w(*([0]*9))), (4,0,0x40100,w(*([0]*8),0x40000)),
                  (5,4,0x50000,bytes(108))])
    for i,a in enumerate(POOLS):
        items.extend([(6,i,a,w(0x60000+i*0x100,8,0,0)),(7,i,0x60000+i*0x100,bytes(32))])
    return items


def encode(items):
    raw=b''.join(w(k,o,a,len(d))+d for k,o,a,d in items)
    return w(0x31475352,1,223,0x10000,len(items),32+len(raw),5,2)+raw


class GraphTests(unittest.TestCase):
    def test_complete(self):
        r,_,_=validate(encode(fixture()))
        self.assertEqual(r['owned_pathnodes'],2)
        self.assertEqual(r['physical_lengths'],[1]*5)

    def test_every_missing_region(self):
        items=fixture()
        for i in range(len(items)):
            with self.subTest(i=i), self.assertRaises(ValueError):
                validate(encode(items[:i]+items[i+1:]))

    def test_cycles_parent_refs_and_duplicate_ownership(self):
        for kind,owner,off,value in [(3,0,0,0x30000),(3,0,8,99),(3,1,12,0x30400),
                                     (3,2,12,0x40000),(4,0,32,0x40000),(6,6,8,9)]:
            items=fixture(); i=next(i for i,r in enumerate(items) if r[:2]==(kind,owner))
            k,o,a,d=items[i]; data=bytearray(d); struct.pack_into('<I',data,off,value)
            items[i]=(k,o,a,bytes(data))
            with self.subTest(kind=kind,owner=owner,off=off), self.assertRaises(ValueError):
                validate(encode(items))

    def test_removed_node_is_not_a_live_reference(self):
        items=fixture()
        # Keep a physical tombstone in refs with a deliberately invalid data
        # pointer; only active refs own a relationship to an open node.
        j=next(i for i,r in enumerate(items) if r[:2]==(3,1))
        k,o,a,d=items[j]; data=bytearray(d)
        struct.pack_into('<I',data,0,0x31000);items[j]=(k,o,a,bytes(data))
        items.append((3,1,0x31000,w(0,0,a,0xdeadbeef,0,0x100)))
        raw=bytearray(encode(items));struct.pack_into('<I',raw,24,6)
        r,_,_=validate(bytes(raw))
        self.assertEqual(r['physical_lengths'][1],2)
        self.assertEqual(r['logical_lengths'][1],1)

    def test_framing_and_overlap(self):
        raw=encode(fixture())
        for bad in (b'',raw[:-1],raw+b'x',bytes(32),raw[:16]+w(5000)+raw[20:]):
            with self.assertRaises(ValueError): validate(bad)
        items=fixture();k,o,a,d=items[-1];items[-1]=(k,o,items[-2][2],d)
        with self.assertRaises(ValueError): validate(encode(items))


if __name__=='__main__': unittest.main()
