import struct
import subprocess
import tempfile
import unittest
from pathlib import Path
from restore_poststate import decode_post, check_receipt, compare, require_agreement
from test_restore_prefix import fixture

ROOT=Path(__file__).resolve().parent


def packet():
    unit=bytearray(344);unit[10]=1
    struct.pack_into('<3I',unit,0xb8,0x20000,2,1)
    return (struct.pack('<8I',0x31505352,1,230,0x14000,216,32,0x8d5,0x688faa)+fixture(2)+
            struct.pack('<9I',0,0,0,0x50003ff8,0,0,0,8,0)+unit+bytes(range(32)))


def replay(c):
    return dict(payload_sha256='bound',last=dict(returned=True,eax=8,unit_path_observation=dict(
        unit_address=hex(c['unit']),**{k:c[k] for k in ('unit_bytes','path_address','capacity','length','path_slot_bytes')})))


class PoststateTests(unittest.TestCase):
    def test_c_producer_matches_python_and_failure_controls(self):
        with tempfile.TemporaryDirectory() as temp:
            binary=Path(temp)/'test';out=Path(temp)/'packet'
            subprocess.run(['clang','-std=gnu11','-Wall','-Wextra','-Wno-unused-function',
                            str(ROOT/'test_live_restore_poststate.c'),'-o',str(binary)],check=True)
            subprocess.run([str(binary),str(out)],check=True)
            self.assertEqual(out.read_bytes(),packet())
            self.assertEqual(decode_post(out.read_bytes(),fixture(2))['length'],1)

    def test_extent_identity_registers_and_prefix_refuse(self):
        raw=packet()
        for offset,value in ((0,0),(4,2),(8,231),(12,0),(16,196),(20,16),(24,0xffffffff),
                             (28,0),(248+12,0),(248+32,0x200),(284+0xb8,0xfffffff0),
                             (284+0xbc,4097),(284+0xc0,3)):
            bad=bytearray(raw);struct.pack_into('<I',bad,offset,value)
            with self.subTest(offset=offset),self.assertRaises(ValueError):decode_post(bad,fixture(2))
        for bad in (raw[:-1],raw+b'x',raw[:100]):
            with self.assertRaises(ValueError):decode_post(bad,fixture(2))
        bad=bytearray(raw);bad[284+9]=1
        with self.assertRaises(ValueError):decode_post(bad,fixture(2))
        with self.assertRaises(ValueError):decode_post(raw,fixture())

    def test_receipt_requires_bound_completed_native_call(self):
        c=decode_post(packet(),fixture(2))
        rows=[(5,167,0,0,0,0,0,230),(7,0,0,0,0,0,0,230),(8,0,1,0,0,0,0,230),
              (5,181,0,0x14000,660,2,8,230)]
        check_receipt(rows,c)
        for bad in ([rows[0],rows[1],rows[3]], rows+rows[-1:], rows[1:], [rows[-1]]+rows[:-1],
                    rows+[(5,182,1,0,0,0,0,230)],rows[:2]+[rows[-1],rows[2]],
                    [rows[0],(*rows[1][:-1],231),*rows[2:]]):
            with self.assertRaises(ValueError):check_receipt(bad,c)

    def test_comparison_reports_pointer_and_inactive_slots_separately(self):
        c=decode_post(packet(),fixture(2));r=replay(c)
        self.assertTrue(compare(c,r,'bound')['all_path_slots_equal'])
        o=r['last']['unit_path_observation'];b=bytearray.fromhex(o['unit_bytes'])
        struct.pack_into('<I',b,0xb8,0x30000);o['unit_bytes']=b.hex();o['path_address']='0x30000'
        result=compare(c,r,'bound')
        self.assertTrue(result['unit_changed_byte_offsets']);self.assertEqual(result['unit_changes_outside_path_pointer'],[])
        slots=bytearray.fromhex(o['path_slot_bytes']);slots[31]^=1;o['path_slot_bytes']=slots.hex()
        result=compare(c,r,'bound');self.assertTrue(result['active_path_equal']);self.assertFalse(result['all_path_slots_equal'])
        self.assertEqual(result['path_changed_slot_indices'],[1])
        b[55]^=1;o['unit_bytes']=b.hex();self.assertEqual(compare(c,r,'bound')['unit_changes_outside_path_pointer'],[55])
        r['last']['eax']=7;self.assertFalse(compare(c,r,'bound')['outer_return_equal'])
        with self.assertRaises(ValueError):compare(c,r,'different payload')
        r['last']['returned']=False
        with self.assertRaises(ValueError):compare(c,r,'bound')

    def test_agreement_assertion_rejects_each_measured_disagreement(self):
        c=decode_post(packet(),fixture(2));r=replay(c);good=compare(c,r,'bound')
        require_agreement(good)
        for key,value in (('outer_return_equal',False),('native_length',0),('native_capacity',1),
                          ('unit_changes_outside_path_pointer',[55]),('active_path_equal',False),
                          ('all_path_slots_equal',False)):
            with self.subTest(key=key),self.assertRaises(ValueError):require_agreement({**good,key:value})
        # Explicit relocation alone remains accepted; no other offset is masked.
        require_agreement({**good,'unit_changed_byte_offsets':[184,185,186,187]})

    def test_return_modes_are_observed_bound_and_compared(self):
        raw=bytearray(packet());struct.pack_into('<I',raw,4,3);raw+=struct.pack('<2I',300,0)
        c=decode_post(raw,fixture(2));self.assertEqual(c['return_modes'],[300,0])
        rows=[(5,167,0,0,0,0,0,230),(7,0,0,0,0,0,0,230),(8,0,1,0,0,0,0,230),
              (5,195,0x14000,300,0,668,8,230),(5,181,0,0x14000,668,2,8,230)]
        check_receipt(rows,c)
        for bad in (rows[:3]+rows[4:],rows[:3]+rows[3:4]*2+rows[4:],
                    rows[:3]+[(5,195,0x14000,300,1,668,8,230)]+rows[4:],
                    [rows[3]]+rows[:3]+rows[4:],rows[:3]+rows[4:]+rows[3:4],
                    rows[:1]+rows[3:4]+rows[1:3]+rows[4:]):
            with self.assertRaises(ValueError):check_receipt(bad,c)
        for bad in (raw[:-1],raw[:-8],raw+b'x'):
            with self.assertRaises(ValueError):decode_post(bad,fixture(2))
        r=replay(c)
        with self.assertRaises(ValueError):require_agreement(compare(c,r,'bound'))
        r['last']['return_modes']=[300,0];require_agreement(compare(c,r,'bound'))
        r['last']['return_modes']=[300,1]
        with self.assertRaises(ValueError):require_agreement(compare(c,r,'bound'))

    def test_empty_and_capacity_difference(self):
        raw=bytearray(packet()[:628]);struct.pack_into('<I',raw,20,0);struct.pack_into('<3I',raw,284+0xb8,0,0,0)
        c=decode_post(raw,fixture(2));r=replay(c)
        self.assertTrue(compare(c,r,'bound')['all_path_slots_equal'])
        nonempty=decode_post(packet(),fixture(2));r=replay(nonempty)
        result=compare(c,r,'bound');self.assertEqual(result['path_changed_slot_indices'],[0,1]);self.assertFalse(result['active_path_equal'])


if __name__=='__main__':unittest.main()
