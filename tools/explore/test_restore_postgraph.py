import copy
import struct
import subprocess
import tempfile
import unittest
from pathlib import Path
from test_search_graph import fixture as graph_fixture,encode
from test_restore_poststate import packet,replay,ROOT
from test_restore_prefix import fixture as prefix_fixture
from test_native_limit import limit_packet,receipts
from restore_poststate import decode_post
from restore_postgraph import decode_graph,check_graph_receipt,compare_native,IMAGE_SHA256
from compare_search_graph import require_required_field_agreement


def fixtures(limit=False):
    post,prefix=limit_packet() if limit else (packet(),prefix_fixture(2))
    post=bytearray(post);items=graph_fixture()
    k,o,a,d=items[0];items[0]=(k,o,0x14104,d);post[284+0x104:284+0x14c]=d
    graph=bytearray(encode(items));struct.pack_into('<2I',graph,8,230,0x14000)
    envelope=struct.pack('<8I',0x31504752,1,230,0x14000,len(post),len(graph),0,0x688faa)+post+graph
    return bytes(envelope),bytes(post),prefix,bytes(graph)


def rows(native,limit=False):
    p=native['post']
    r=receipts() if limit else [(5,167,0,0,0,0,0,230),(7,0,0,0,0,0,0,230),
        (8,0,1,0,0,0,0,230),(5,181,0,0x14000,p['packet_bytes'],2,8,230)]
    return r+[(5,186,0,0x14000,native['packet_bytes'],len(native['graph']['records']),0,230)]


def model(native):
    witness=replay(native['post']);witness['native_intervention']=native['post']['intervention']
    graph=copy.deepcopy(native['graph']);graph['schema']='model-defined-search-graph-v1'
    return dict(payload_sha256='bound',image_sha256=IMAGE_SHA256,prepared_experiments=dict(native_unit_path=dict(replay=witness),
        graph=graph,graph_repeats=True,observer_read_only=True,control_restored=True))


class PostgraphTests(unittest.TestCase):
    def test_c_producer_packets_and_failure_controls(self):
        for limit in (False,True):
            raw,post,prefix,graph=fixtures(limit)
            with tempfile.TemporaryDirectory() as tmp:
                d=Path(tmp);(d/'post').write_bytes(post);(d/'graph').write_bytes(graph)
                subprocess.run(['clang','-std=gnu11','-Wall','-Wextra','-Wno-unused-function',
                    '-Wno-int-to-void-pointer-cast',*(['-DRON_RESTORE_LIMIT95'] if limit else []),
                    str(ROOT/'test_live_restore_postgraph.c'),'-o',str(d/'test')],check=True)
                subprocess.run([str(d/'test'),str(d/'graph'),str(d/'post'),str(d/'out')],check=True)
                actual=(d/'out').read_bytes()
            # Collector order is allowed to differ; all bytes/records must agree after validation.
            a,b=decode_graph(actual,post,prefix),decode_graph(raw,post,prefix)
            self.assertEqual(a['graph'],b['graph']);self.assertEqual(a['post'],b['post'])

    def test_envelope_and_pairing_refusals(self):
        raw,post,prefix,_=fixtures(True)
        for offset in range(0,32,4):
            bad=bytearray(raw);bad[offset]^=1
            # An elapsed millisecond is valid; force the deadline for this word.
            if offset==24:struct.pack_into('<I',bad,24,2000)
            with self.subTest(offset=offset),self.assertRaises(ValueError):decode_graph(bad,post,prefix)
        for bad in (raw[:-1],raw+b'x',raw[:31]):
            with self.assertRaises(ValueError):decode_graph(bad,post,prefix)
        bad=bytearray(raw);bad[32+300]^=1
        with self.assertRaises(ValueError):decode_graph(bad,post,prefix)
        bad=bytearray(raw);bad[32+len(post)+48+40]^=1
        with self.assertRaises(ValueError):decode_graph(bad,post,prefix)
        with self.assertRaises(ValueError):decode_graph(raw,fixtures()[1],prefix)

    def test_receipt_order_identity_and_failures(self):
        for limit in (False,True):
            raw,post,prefix,_=fixtures(limit);n=decode_graph(raw,post,prefix);r=rows(n,limit)
            check_graph_receipt(r,n)
            for bad in (r[:-1],r+r[-1:],r[-1:]+r[:-1],r+[(5,187,1,0,0,0,0,230)],
                        r[:-1]+[(7,0,0,0,0,0,0,230)]+r[-1:]):
                with self.assertRaises(ValueError):check_graph_receipt(bad,n)
            for i in range(2,8):
                bad=r.copy();row=list(bad[-1]);row[i]^=1;bad[-1]=tuple(row)
                with self.assertRaises(ValueError):check_graph_receipt(bad,n)

    def test_native_model_comparison_and_required_payload_mutant(self):
        raw,post,prefix,_=fixtures(True);n=decode_graph(raw,post,prefix);m=model(n)
        result=compare_native(n,m,'bound');require_required_field_agreement(result)
        self.assertTrue(result['native_compared']);self.assertFalse(result['full_native_state_compared'])
        with self.assertRaises(ValueError):compare_native(n,m,'wrong')
        for key in ('graph_repeats','observer_read_only','control_restored'):
            bad=copy.deepcopy(m);bad['prepared_experiments'][key]=False
            with self.assertRaises(ValueError):compare_native(n,bad,'bound')
        bad=copy.deepcopy(m)
        row=next(r for r in bad['prepared_experiments']['graph']['records'] if r['kind']==4)
        row['values'][0]^=1
        result=compare_native(n,bad,'bound')
        with self.assertRaises(ValueError):require_required_field_agreement(result)
        bad=copy.deepcopy(m);bad['prepared_experiments']['native_unit_path']['replay']['last']['eax']=9
        with self.assertRaises(ValueError):compare_native(n,bad,'bound')

    def test_native_known_tail_does_not_fill_model_unknowns(self):
        raw,post,prefix,_=fixtures();n=decode_graph(raw,post,prefix);m=model(n)
        from defined_search_graph import GraphObserver
        from defined_bytes import DefinedBytes
        records=m['prepared_experiments']['graph']['records']
        index={int(r['address'],16):r for r in records}
        def read(a,size):
            row=index[a];v=row['values'].copy()
            if row['kind']==3 and row['owner'] in (1,2,3):v[22:24]=[None,None]
            return DefinedBytes(tuple(v))
        m['prepared_experiments']['graph']=GraphObserver(read,0x14000,230).run()
        result=compare_native(n,m,'bound');require_required_field_agreement(result)
        self.assertEqual(len(result['unknown_byte_positions']),6)
        self.assertFalse(result['definedness_equal']);self.assertFalse(result['all_bytes_known'])
        bad=copy.deepcopy(m);bad['image_sha256']='wrong'
        with self.assertRaises(ValueError):compare_native(n,bad,'bound')
        bad=copy.deepcopy(m)
        row=next(r for r in bad['prepared_experiments']['graph']['records'] if r['kind']==1)
        row['values'][40]^=1
        with self.assertRaises(ValueError):compare_native(n,bad,'bound')
