import struct
import unittest
from unicorn import UC_MEM_READ
from defined_bytes import DefinedBytes,observe_bytes
from defined_search_graph import GraphObserver
from search_graph import validate
from test_search_graph import fixture,encode
from test_modeled_lifetime import runner,allocation,call_free,ENTRY,REGS,ARENA,OBJ
from payload_replay import fingerprint


def observe_fixture(items,holes=()):
    def read(a,n):
        row=next((r for r in items if r[2]==a and len(r[3])==n),None)
        if row is None:raise ValueError('missing record')
        return DefinedBytes(tuple(None if a+i in holes else v for i,v in enumerate(row[3])))
    return GraphObserver(read,0x10000,223).run()


class DefinedObservationTests(unittest.TestCase):
    def test_unknown_bytes_are_not_read_and_guest_still_refuses(self):
        r=runner(allocation()+b'\xc6\x40\x01\xab\xc3');r.run(ENTRY,REGS)
        # Poison backing bytes without initializing them in the model.
        r.uc.mem_write(ARENA,b'Z');r.uc.mem_write(ARENA+2,b'ZZ')
        before=fingerprint(r,None);calls=[];original=r.uc.mem_read
        def read(a,n):
            calls.append((a,n));self.assertEqual((a,n),(ARENA+1,1));return original(a,n)
        r.uc.mem_read=read
        observed=observe_bytes(r,ARENA,4)
        self.assertEqual(observed.json(),[None,171,None,None]);self.assertEqual(calls,[(ARENA+1,1)])
        r.uc.mem_read=original
        self.assertEqual(before,fingerprint(r,None))
        self.assertEqual(bytes(observed[1:2]),b'\xab')
        with self.assertRaises(ValueError):bytes(observed)
        with self.assertRaises(ValueError):observed[0]
        with self.assertRaisesRegex(ValueError,'uninitialized'):r.access(r.uc,UC_MEM_READ,ARENA,4,0,None)

    def test_allocation_extent_retirement_code_and_missing_bytes_refuse(self):
        r=runner(allocation()+b'\xc3');r.run(ENTRY,REGS)
        for a,n in [(ARENA,5),(ARENA+4,1),(ARENA-1,2),(ENTRY,1),(0xdead0000,1),(OBJ+7,2),
                    (OBJ,0),(OBJ,16385),(0xffffffff,2)]:
            with self.subTest(address=a,size=n),self.assertRaises(ValueError):observe_bytes(r,a,n)
        r=runner(call_free()+b'\xc3');r.run(ENTRY,REGS)
        with self.assertRaisesRegex(ValueError,'retired'):observe_bytes(r,OBJ,8)
        r=runner(allocation()+call_free(ARENA,10)+b'\xc3');r.run(ENTRY,REGS)
        with self.assertRaisesRegex(ValueError,'retired'):observe_bytes(r,ARENA,4)

    def test_complete_fixture_matches_existing_validator(self):
        result=observe_fixture(fixture());expected=validate(encode(fixture()))[0]
        expected['equivalent_full_packet_bytes']=expected.pop('serialized_bytes')
        self.assertTrue(result['complete']);self.assertTrue(result['full_record_bytes_known'])
        self.assertEqual(result['structural_report'],expected)
        self.assertEqual(result['undefined_bytes'],0);self.assertFalse(result['native_compared'])

    def test_padding_and_unused_pool_capacity_stay_explicit(self):
        holes={0x30100+22,0x30100+23,0x60000+31}
        result=observe_fixture(fixture(),holes)
        self.assertTrue(result['complete']);self.assertFalse(result['full_record_bytes_known'])
        self.assertEqual(result['undefined_bytes'],3)
        node=next(r for r in result['records'] if r['address']=='0x30100')
        self.assertEqual(node['values'][-2:],[None,None])
        # A defined value in the same position is retained, never masked.
        items=fixture();i=next(i for i,r in enumerate(items) if r[:2]==(3,1))
        k,o,a,d=items[i];items[i]=(k,o,a,d[:-2]+b'\xfe\xff')
        result=observe_fixture(items);node=next(r for r in result['records'] if r['address']=='0x30100')
        self.assertEqual(node['values'][-2:],[254,255])

    def test_every_required_node_byte_refuses_if_unknown(self):
        for offset in range(22):
            result=observe_fixture(fixture(),{0x30100+offset})
            with self.subTest(offset=offset):
                self.assertFalse(result['complete']);self.assertIn('undefined required',result['reason'])
                self.assertEqual(result['undefined_bytes'],1)
        items=fixture();i=next(i for i,r in enumerate(items) if r[:2]==(6,0))
        k,o,a,d=items[i];items[i]=(k,o,a,d[:8]+struct.pack('<I',1)+d[12:])
        self.assertFalse(observe_fixture(items,{0x60000})['complete'])
        for address in (0x10104,0x20000,0x40000,0x50000):
            self.assertFalse(observe_fixture(fixture(),{address})['complete'])

    def test_missing_records_cycles_aliases_and_identity_fail(self):
        items=fixture()
        for i in range(len(items)):
            self.assertFalse(observe_fixture(items[:i]+items[i+1:])['complete'])
        for kind,owner,off,value in [(3,0,0,0x30000),(3,0,8,99),(3,1,12,0x30400),
                                     (3,2,12,0x40000),(4,0,32,0x40000),(6,6,8,9)]:
            bad=items.copy();i=next(i for i,r in enumerate(bad) if r[:2]==(kind,owner))
            k,o,a,d=bad[i];data=bytearray(d);struct.pack_into('<I',data,off,value);bad[i]=(k,o,a,bytes(data))
            self.assertFalse(observe_fixture(bad)['complete'])

    def test_failure_is_single_use_and_cannot_authorize_retry(self):
        def absent(a,n):raise ValueError('absent')
        observer=GraphObserver(absent,0x10000,0)
        self.assertFalse(observer.run()['complete'])
        with self.assertRaisesRegex(ValueError,'single-use'):observer.run()

    def test_value_and_record_bounds(self):
        for values in ([None],(True,),(256,),(-1,)):
            with self.assertRaises(ValueError):DefinedBytes(values)
        for address in (0,0x10001,2**32):
            with self.assertRaises(ValueError):GraphObserver(None,address,0)
        observer=GraphObserver(lambda a,n:DefinedBytes((0,)*n),0x10000,0)
        observer.total=256*1024
        with self.assertRaises(ValueError):observer.record(1,0,0x10104,72)
