import copy
import struct
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from payload_replay import native_return
from pathlib import Path
from restore_poststate import decode_post,check_receipt,compare,require_agreement
from test_restore_poststate import packet,replay,ROOT
from test_restore_prefix import fixture


def limit_packet():
    prefix=bytearray(fixture(2));struct.pack_into('<I',prefix,47*4,300)
    raw=bytearray(packet());struct.pack_into('<I',raw,4,2);raw[32:248]=prefix
    return bytes(raw)+struct.pack('<8I',1,0xe85ec0,300,95,1,95,300,3),bytes(prefix)


def receipts():
    return [(5,167,0,0,0,0,0,230),(5,183,0x14000,0xe85ec0,300,95,1,230),
            (7,0,0,0,0,0,0,230),(8,0,1,0,0,0,0,230),
            (5,184,0x14000,0xe85ec0,95,300,1,230),(5,181,0,0x14000,692,2,8,230)]


class NativeLimitTests(unittest.TestCase):
    def test_unmodified_replay_does_not_claim_altered_native_astar_comparison(self):
        with patch('payload_replay.records',return_value=iter(receipts())):
            self.assertIsNone(native_return(Path('/unused')))

    def test_actual_c_producer_and_failure_controls(self):
        raw,prefix=limit_packet()
        with tempfile.TemporaryDirectory() as temp:
            binary=Path(temp)/'test';out=Path(temp)/'packet'
            subprocess.run(['clang','-std=gnu11','-DRON_RESTORE_LIMIT95','-Wall','-Wextra',
                            '-Wno-unused-function',str(ROOT/'test_live_restore_poststate.c'),'-o',str(binary)],check=True)
            subprocess.run([str(binary),str(out)],check=True)
            self.assertEqual(out.read_bytes(),raw)
        self.assertEqual(decode_post(raw,prefix)['intervention']['after'],95)

    def test_provenance_cannot_be_silently_removed_or_changed(self):
        raw,prefix=limit_packet()
        for i in range(8):
            bad=bytearray(raw);bad[-32+i*4]^=1
            with self.subTest(word=i),self.assertRaises(ValueError):decode_post(bad,prefix)
        for bad in (raw[:-32],raw[:-1],raw+b'x'):
            with self.assertRaises(ValueError):decode_post(bad,prefix)
        with self.assertRaises(ValueError):decode_post(raw,fixture(2))
        bad=bytearray(raw);struct.pack_into('<I',bad,4,1)
        with self.assertRaises(ValueError):decode_post(bad,prefix)

    def test_receipt_order_identity_failure_and_unmodified_control(self):
        raw,prefix=limit_packet();c=decode_post(raw,prefix);rows=receipts();check_receipt(rows,c)
        for i in (1,4):
            for j in range(2,8):
                bad=copy.deepcopy(rows);r=list(bad[i]);r[j]^=1;bad[i]=tuple(r)
                with self.subTest(row=i,word=j),self.assertRaises(ValueError):check_receipt(bad,c)
            with self.assertRaises(ValueError):check_receipt(rows[:i]+rows[i+1:],c)
        for a,b in ((0,1),(1,2),(3,4),(4,5)):
            bad=rows.copy();bad[a],bad[b]=bad[b],bad[a]
            with self.assertRaises(ValueError):check_receipt(bad,c)
        for tag in (168,185):
            with self.assertRaises(ValueError):check_receipt(rows+[(5,tag,1,0,0,0,0,230)],c)
        control=decode_post(packet(),fixture(2));bad=rows.copy();bad[-1]=(5,181,0,0x14000,660,2,8,230)
        with self.assertRaises(ValueError):check_receipt(bad,control)

    def test_matching_output_alone_is_not_intervention_agreement(self):
        raw,prefix=limit_packet();c=decode_post(raw,prefix);r=replay(c)
        with self.assertRaises(ValueError):compare(c,r,'bound')
        r['native_intervention']=c['intervention'].copy()
        require_agreement(compare(c,r,'bound'))
        r['native_intervention']['after']=96
        with self.assertRaises(ValueError):compare(c,r,'bound')
        with self.assertRaises(ValueError):compare(decode_post(packet(),fixture(2)),r,'bound')
