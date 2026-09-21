"""Authored inventory ranges and the exact C producer; no real memory payload."""
from pathlib import Path
import struct
import subprocess
import tempfile
import unittest
from memory_inventory import decode,classify,check_receipts,HEADER,MAX_RECORDS
from test_restore_prefix import fixture
from test_restore_context import unit_packet
from restore_context import decode_context

ROOT=Path(__file__).resolve().parent


class InventoryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        with tempfile.TemporaryDirectory() as folder:
            p=Path(folder);(p/'prefix').write_bytes(fixture(2))
            subprocess.run(['cc','-Wall','-Wextra','-Werror','-Wno-int-to-pointer-cast',
                            str(ROOT/'test_live_memory_inventory.c'),'-o',str(p/'test')],check=True)
            subprocess.run([str(p/'test'),str(p/'prefix'),str(p/'inventory')],check=True)
            cls.raw=(p/'inventory').read_bytes()

    def test_complete_and_classification(self):
        c=decode(self.raw,fixture(2));raw,_=unit_packet();context=decode_context(raw,fixture(2))
        report=classify(c,context);categories=report['categories']
        self.assertEqual(report['inventory_records'],14)
        self.assertEqual(categories['main_image_data']['bytes'],0xbff000)
        self.assertEqual(categories['private_data']['bytes'],0x12000)
        self.assertEqual(categories['observer_image']['bytes'],0x10000)
        self.assertEqual(categories['observer_stack']['bytes'],0x8000)
        self.assertEqual(categories['guard']['bytes'],0x1000)
        self.assertEqual(report['candidate_bytes'],0xc11000)
        self.assertEqual(sum(v['bytes'] for v in categories.values()),c['end']-c['begin'])
        self.assertFalse(report['memory_payload_captured'])

    def test_header_and_framing_refusals(self):
        for i,value in ((0,0),(1,2),(2,231),(3,1),(4,0),(5,0),(6,3),(7,32),
                        (8,MAX_RECORDS+1),(9,2),(10,2000),(11,0),(12,0),(13,0),(14,1),(15,1)):
            raw=bytearray(self.raw);struct.pack_into('<I',raw,i*4,value)
            with self.subTest(field=i),self.assertRaises(ValueError):decode(raw,fixture(2))
        for raw in (b'',self.raw[:-1],self.raw+b'x'):
            with self.assertRaises(ValueError):decode(raw,fixture(2))
        with self.assertRaises(ValueError):decode(self.raw,fixture())

    def test_ranges_refuse_gaps_overlap_and_bad_lengths(self):
        for row,index,value in ((1,0,0x11000),(1,0,0x21000),(0,3,0),(0,3,4097),
                                (0,3,0xffffffff),(0,4,0)):
            raw=bytearray(self.raw);struct.pack_into('<I',raw,HEADER+28*row+4*index,value)
            with self.subTest(row=row,index=index),self.assertRaises(ValueError):decode(raw,fixture(2))
        raw=bytearray(self.raw[:-28]);struct.pack_into('<I',raw,32,13)
        with self.assertRaises(ValueError):decode(raw,fixture(2))

    def test_receipt_order_and_failure(self):
        c=decode(self.raw,fixture(2));receipt=(5,165,0,c['unit'],len(self.raw),14,0,230)
        begin=(5,164,0,c['unit'],1328,192,0x16000,230);call=(7,0,0,0,0,0,0,230)
        check_receipts([begin,receipt,call],c)
        for rows in ([begin,call],[receipt,call],[receipt,begin,call],[begin,call,receipt],
                     [begin,receipt],[begin,receipt,receipt,call],
                     [begin,receipt,call,(5,166,2,0,0,0,0,230)]):
            with self.subTest(rows=rows),self.assertRaises(ValueError):check_receipts(rows,c)
        for i in range(2,8):
            bad=list(receipt);bad[i]^=1
            with self.subTest(field=i),self.assertRaises(ValueError):check_receipts([begin,tuple(bad),call],c)

    def test_undefined_free_fields_do_not_become_candidates(self):
        c=decode(self.raw,fixture(2));rows=list(c['rows']);r=list(rows[1]);self.assertEqual(r[4],0x10000)
        r[1]=0x400000;r[2]=4;r[5]=4;r[6]=0x20000;rows[1]=tuple(r)
        context=decode_context(unit_packet()[0],fixture(2))
        self.assertEqual(classify(c,context),classify({**c,'rows':rows},context))

    def test_observer_requires_distinct_image(self):
        c=decode(self.raw,fixture(2));context=decode_context(unit_packet()[0],fixture(2))
        for observer in (0x400000,0x2000000):
            with self.subTest(observer=observer),self.assertRaises(ValueError):classify({**c,'observer':observer},context)


if __name__=='__main__':unittest.main()
