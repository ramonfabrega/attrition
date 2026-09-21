"""Owned-install/capture checks; no original bytes are test fixtures in git."""
import os
from pathlib import Path
import unittest
from restore_context import decode_context, validate as validate_context
from search_census import records
from resume_frontier import run, make_runner, registers, verify_entry, native_call, ENTRY, OUTPUTS


@unittest.skipUnless(os.environ.get('RON_RESUME_INSTALL') and os.environ.get('RON_RESUME_CAPTURE'),
                     'owned install and captured context not supplied')
class FrontierTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        install=Path(os.environ['RON_RESUME_INSTALL']);d=Path(os.environ['RON_RESUME_CAPTURE'])
        validate_context(install,d)
        cls.image=install/'riseofnations.exe'
        cls.context=decode_context((d/'restore-context.bin').read_bytes(),(d/'restore-prefix.bin').read_bytes())
        cls.graph=(d/'search-graph.bin').read_bytes()
        cls.expected=native_call(list(records(d/'rontrace.log')),cls.context)

    def test_live_entry_and_next_refusal(self):
        r=run(self.image,self.context,self.graph,self.expected)
        self.assertEqual((r['prefix_instructions'],r['frontier_attempted_instructions']),
                         (50,73 if self.context['version']==1 else 91))
        if self.context['version']==2:self.assertEqual(r['missing_address'],'0xc06188')
        self.assertFalse(r['full_resumption_returned'])

    def test_each_missing_output_and_thread(self):
        for name in ('seh',*(f'output_{a:x}' for a in OUTPUTS)):
            with self.subTest(name=name),self.assertRaises(ValueError):
                make_runner(self.image,self.context,self.graph,omit=(name,)).run(ENTRY,registers(self.context))

    def test_wrong_native_arguments(self):
        r=make_runner(self.image,self.context,self.graph);r.run(ENTRY,registers(self.context))
        for i in range(4):
            wrong=list(self.expected);wrong[i]^=1
            with self.subTest(i=i),self.assertRaisesRegex(ValueError,'entry differs'):
                verify_entry(r,self.context,tuple(wrong))

    def test_wrong_working_outputs(self):
        r=make_runner(self.image,self.context,self.graph)
        for address in OUTPUTS:
            r.run(ENTRY,registers(self.context))
            original=bytes(r.uc.mem_read(address,4))
            r.uc.mem_write(address,bytes([original[0]^1])+original[1:])
            with self.subTest(address=address),self.assertRaisesRegex(ValueError,'output differs'):
                verify_entry(r,self.context,self.expected)


if __name__=='__main__':unittest.main()
