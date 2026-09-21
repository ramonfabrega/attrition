"""Authored context packets, C/Python agreement and install-dependent table check."""
import struct
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from restore_context import decode_context, check_receipt, compare_table, check_unit_graph
from test_restore_prefix import fixture

ROOT=Path(__file__).resolve().parent


def packet():
    prefix=fixture(2)
    words=(0x31585452,1,230,0x14000,216,192,0x16000,0x2000000,0x2000060,
           0xffffffff,0x50008000,0x50000000,0,0,0,0x16000)
    # FX bytes are opaque authored data, not a valid hardware state claim.
    return struct.pack('<16I',*words)+prefix+bytes(512)+struct.pack('<48i',*(i//3 for i in range(-24,24)))


def unit_packet():
    from test_search_graph import fixture as graph_fixture, encode
    raw=bytearray(packet());struct.pack_into('<I',raw,4,2)
    unit=bytearray(0x158);unit[10]=1
    struct.pack_into('<2I',unit,16,4000^0x63637,8000^0x63637)
    items=graph_fixture();kind,owner,_,data=items[0]
    items[0]=(kind,owner,0x14104,data);unit[0x104:0x14c]=data
    graph=bytearray(encode(items));struct.pack_into('<II',graph,8,230,0x14000)
    return bytes(raw[:792]+unit+raw[792:]),bytes(graph)


class ContextTests(unittest.TestCase):
    def test_decode(self):
        c=decode_context(packet(),fixture(2))
        self.assertEqual((c['size'],len(c['fxsave'])),(1,512))
        self.assertEqual(c['tib'][0],0xffffffff)

    def test_unit_packet_identity_and_graph(self):
        raw,graph=unit_packet();c=decode_context(raw,fixture(2));check_unit_graph(c,graph)
        self.assertEqual((c['version'],len(c['unit_data']),c['fixed_bytes']),(2,344,1136))
        for offset in (9,10,11,16,20):
            bad=bytearray(raw);bad[792+offset]^=1
            with self.subTest(offset=offset),self.assertRaisesRegex(ValueError,'identity or coordinates'):
                decode_context(bad,fixture(2))
        bad=bytearray(raw);bad[792+0x144]^=1
        with self.assertRaisesRegex(ValueError,'graph changed'):
            check_unit_graph(decode_context(bad,fixture(2)),graph)
        with self.assertRaises(ValueError):decode_context(raw[:-344],fixture(2))
        rows=[(5,162,0,0x14000,216,216,75,230),
              (5,164,0,0x14000,1328,192,0x16000,230),(7,0,0,0,0,0,0,230)]
        check_receipt(rows,c)
        wrong=[rows[0],(5,164,0,0x14000,984,192,0x16000,230),rows[2]]
        with self.assertRaises(ValueError):check_receipt(wrong,c)

    def test_reject_malformed(self):
        for index,value in ((0,0),(1,2),(2,231),(3,0),(4,196),(5,0),(5,193),
                            (5,786624),(6,0),(7,0),(7,0xfffffff0),(8,0),
                            (10,0x50003fe0),(11,0x50003fe0),(15,0)):
            raw=bytearray(packet());struct.pack_into('<I',raw,index*4,value)
            with self.subTest(index=index,value=value),self.assertRaises(ValueError):
                decode_context(raw,fixture(2))
        for raw in (b'',packet()[:-1],packet()+b'x'):
            with self.assertRaises(ValueError):decode_context(raw,fixture(2))
        with self.assertRaises(ValueError):decode_context(packet(),fixture())
        raw=bytearray(packet());struct.pack_into('<II',raw,28,0x16000,0x16060)
        with self.assertRaisesRegex(ValueError,'overlap'):decode_context(raw,fixture(2))

    def test_receipts(self):
        c=decode_context(packet(),fixture(2))
        start=(5,162,0,0x14000,216,216,75,230)
        receipt=(5,164,0,0x14000,984,192,0x16000,230)
        call=(7,0,0,0,0,0,0,230)
        check_receipt([start,receipt,call],c)
        for rows in ([start,call],[receipt,call],[receipt,start,call],
                     [start,call,receipt],[start,receipt,receipt,call],
                     [start,receipt],[start,receipt,call,(5,163,1,0,0,0,0,230)]):
            with self.subTest(rows=rows),self.assertRaises(ValueError):check_receipt(rows,c)
        for i in range(2,8):
            bad=list(receipt);bad[i]^=1
            with self.subTest(field=i),self.assertRaises(ValueError):check_receipt([start,tuple(bad),call],c)

    @unittest.skipUnless(os.environ.get('RON_CONTEXT_IMAGE'), 'owned executable not supplied')
    def test_original_table_and_corruption(self):
        image=Path(os.environ['RON_CONTEXT_IMAGE'])
        c=decode_context(packet(),fixture(2))
        self.assertEqual(compare_table(image,c)['entries_checked'],48)
        c['table']=bytes([c['table'][0]^1])+c['table'][1:]
        with self.assertRaisesRegex(ValueError,'differs'):
            compare_table(image,c)

    @unittest.skipUnless(os.environ.get('RON_CONTEXT_IMAGE'), 'owned executable not supplied')
    def test_authored_unit_context_replays_entry(self):
        from resume_frontier import run
        raw,graph=unit_packet();c=decode_context(raw,fixture(2))
        c['table']=struct.pack('<19200i',*(i//3 for i in range(-9600,9600)))
        c['center']=c['origin']+38400
        # All memory here is authored. This checks the v2 region assembly, not
        # a claim that the live search has crossed the old unit-header boundary.
        report=run(Path(os.environ['RON_CONTEXT_IMAGE']),c,graph,(0xe85e40,0x140b8,48,0),repeats=2)
        self.assertEqual(report['prefix_instructions'],50)
        self.assertNotEqual(report['frontier_pc'],'0x6837c8')
        self.assertFalse(report['full_resumption_returned'])

    def test_exact_c_collector(self):
        with tempfile.TemporaryDirectory() as directory:
            p=Path(directory)
            subprocess.run(['cc','-Wall','-Wextra','-Werror','-Wno-pointer-to-int-cast',
                            '-Wno-int-to-pointer-cast',str(ROOT/'test_live_restore_probe.c'),
                            '-o',str(p/'test')],check=True)
            subprocess.run([str(p/'test'),str(p/'prefix'),str(p/'context')],check=True)
            c=decode_context((p/'context').read_bytes(),(p/'prefix').read_bytes())
            self.assertEqual((c['frame'],c['table_bytes'],c['version']),(224,192,2))
            self.assertEqual(len(c['unit_data']),344)
            self.assertEqual(c['table'],packet()[792:])


if __name__=='__main__':unittest.main()
