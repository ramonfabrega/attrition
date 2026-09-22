import copy
import struct
import unittest
from defined_bytes import DefinedBytes
from defined_search_graph import GraphObserver
from absent_search_state import AbsentSearchObserver
from compare_search_graph import checked_graph,compare,derive_correspondence,require_required_field_agreement
from test_search_graph import fixture


def items():
    rows=[(k,o,a,d) for k,o,a,d in fixture() if k in (1,6,7)]
    k,o,a,d=rows[0];rows[0]=(k,o,a,bytes(20)+d[20:])
    return rows


def observe(rows,holes=(),observer=AbsentSearchObserver):
    def read(a,n):
        matches=[d for _,_,address,d in rows if address==a and len(d)==n]
        if len(matches)!=1:raise ValueError('missing/wrong record')
        return DefinedBytes(tuple(None if a+i in holes else v for i,v in enumerate(matches[0])))
    return observer(read,0x10000,223).run()


class AbsentTests(unittest.TestCase):
    def test_absent_is_explicit_and_does_not_invent_nodes(self):
        result=observe(items());self.assertTrue(result['complete']);checked_graph(result)
        self.assertEqual(len(result['records']),15)
        self.assertEqual(result['structural_report']['saved_containers'],'absent')
        self.assertFalse(result['structural_report']['recycler_targets_observed'])
        self.assertEqual({r['kind'] for r in result['records']},{1,6,7})
        self.assertFalse(observe(items(),observer=GraphObserver)['complete'])

    def test_present_or_partially_absent_roots_refuse(self):
        for mask in range(1,32):
            rows=items();k,o,a,d=rows[0];d=bytearray(d)
            for i in range(5):
                if mask & 1<<i:struct.pack_into('<I',d,i*4,0x20000+i*0x100)
            rows[0]=(k,o,a,bytes(d));result=observe(rows)
            self.assertFalse(result['complete']);self.assertIn('all five roots null',result['reason'])

    def test_missing_headers_arrays_and_invalid_extents_refuse(self):
        rows=items()
        for i in range(len(rows)):
            self.assertFalse(observe(rows[:i]+rows[i+1:])['complete'])
        for word,value in ((0,0),(0,0xfffffff0),(1,4097),(2,9)):
            bad=items();i=next(i for i,r in enumerate(bad) if r[:2]==(6,0));k,o,a,d=bad[i];d=bytearray(d)
            struct.pack_into('<I',d,word*4,value);bad[i]=(k,o,a,bytes(d))
            self.assertFalse(observe(bad)['complete'])

    def test_occupied_entries_required_unused_tail_explicit(self):
        rows=items();i=next(i for i,r in enumerate(rows) if r[:2]==(6,0));k,o,a,d=rows[i];d=bytearray(d)
        struct.pack_into('<I',d,8,1);rows[i]=(k,o,a,bytes(d))
        self.assertFalse(observe(rows,holes=(0x60000,))['complete'])
        result=observe(rows,holes=(0x60004,));self.assertTrue(result['complete']);checked_graph(result)
        self.assertEqual(result['undefined_bytes'],1);self.assertFalse(result['full_record_bytes_known'])
        for offset in (0,19,20,71):self.assertFalse(observe(rows,holes=(0x10104+offset,))['complete'])

    def test_whole_unit_and_unused_capacity_are_compared(self):
        left=observe(items());right=copy.deepcopy(left)
        right['records'][0]['values'][48]=1
        derived=derive_correspondence(left,right);result=compare(left,right,derived['pairs'])
        self.assertFalse(result['required_fields_agree'])
        with self.assertRaises(ValueError):require_required_field_agreement(result)
        right=copy.deepcopy(left);r=next(r for r in right['records'] if r['kind']==7);r['values'][0]=1
        result=compare(left,right,derive_correspondence(left,right)['pairs'])
        self.assertTrue(result['required_fields_agree']);self.assertFalse(result['known_bytes_agree'])
        right['structural_report']['saved_containers']='present'
        with self.assertRaises(ValueError):checked_graph(right)

    def test_extra_records_are_not_ignored(self):
        result=observe(items());result['records'].append(dict(kind=3,owner=0,address='0x30000',bytes=20,values=[0]*20))
        with self.assertRaises(ValueError):checked_graph(result)

if __name__=='__main__':unittest.main()
