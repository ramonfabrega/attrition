import struct
import unittest
import tempfile
from pathlib import Path
from test_search_graph import fixture as graph_fixture, encode as encode_graph
from restore_prefix import decode, pack, check, ENTRY, STOP


def fixture():
    before=(7,8,9,0x50004000,10,11,12,13,0x202)
    args=(0x12345678,0x140b8,0,1)
    deps=(0x11000,0x12000,0x13000,0x14000,2,4000^0x63637,8000^0x63637,125,0)
    after=(7,0,0x50003ffc,0x50003fe0,1,8000,4000,13,0x202)
    out=(args[1],4000,8000,0,1,0,before[1],before[2],*args)
    return pack(0x31545352,1,230,0x14000,*before,*args,*deps,*after,*out,75,1)


class RestoreTests(unittest.TestCase):
    def test_complete(self):
        c=decode(fixture());self.assertEqual(c['modes'],(75,1))
        self.assertEqual(c['out'][1:3],(4000,8000))

    def test_corrupt_handoff_fields(self):
        # Header, stack identity, saved register, coordinates, saving and ESP.
        for index in (0,1,15,16,29,35,36,37,38,39,40,41,42,43,48):
            raw=bytearray(fixture());value=struct.unpack_from('<I',raw,index*4)[0]
            struct.pack_into('<I',raw,index*4,value^0x400)
            with self.subTest(index=index), self.assertRaises(ValueError):decode(bytes(raw))

    def test_receipt_binding_and_missing_records(self):
        items=graph_fixture();k,o,_,d=items[0];items[0]=(k,o,0x14104,d)
        graph=bytearray(encode_graph(items));struct.pack_into('<II',graph,8,230,0x14000)
        def info(tag,*data):return (5,tag,*data,*([0]*(5-len(data))),230)
        rows=[info(136,0,1,64),info(12,0x283770),(2,230,7,0,0,0,0,230),
              info(160,1,ENTRY,STOP,0x682f30,0),info(150,0,0x14000,5,2,len(graph)),
              info(161,0x14000,0,1,0x140b8,2),info(162,0,0x14000,196,196,75),
              (7,0,0xe85e40,0x140b8,48,0,0,230),(8,0,1,0,0,0,0,230)]
        with tempfile.TemporaryDirectory() as folder:
            p=Path(folder);(p/'restore-prefix.bin').write_bytes(fixture())
            (p/'search-graph.bin').write_bytes(graph)
            header=pack(0x544e4f52,2,0x400000,0,0,0,0,0)
            def write(rs):(p/'rontrace.log').write_bytes(header+b''.join(pack(*r) for r in rs))
            write(rows);self.assertEqual(check(p)[0]['native_astar_result'],1)
            for i in range(len(rows)):
                write(rows[:i]+rows[i+1:])
                with self.subTest(i=i), self.assertRaises(ValueError):check(p)
            for tag in (151,163):
                write(rows+[info(tag,1)])
                with self.assertRaises(ValueError):check(p)

    def test_framing(self):
        for raw in (b'',fixture()[:-1],fixture()+b'x'):
            with self.assertRaises(ValueError):decode(raw)


if __name__=='__main__':unittest.main()
