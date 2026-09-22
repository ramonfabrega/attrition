import struct
import unittest
from bounded_call import Region,BoundedCall
from path_intervention import request,input_patch,execute,parse_trial,LIMIT

UNIT,PATH,ENTRY,STOP=0x200000,0x210000,0x100000,0x100100


def regions(capacity=2,length=1):
    unit=bytearray(344);struct.pack_into('<3I',unit,0xb8,PATH,capacity,length)
    return [Region('unit',UNIT,bytes(unit),writable=True),
            Region('path',PATH,struct.pack('<8i',100,200,0,1,300,400,48,2),writable=True),
            Region('modes',LIMIT,struct.pack('<2I',75,1),writable=True)]


class InterventionTests(unittest.TestCase):
    def test_exact_one_word_and_original_bytes_unchanged(self):
        rs=regions();inputs,patch=input_patch(rs,UNIT,'x',101)
        self.assertEqual(patch,dict(field='x',address=hex(PATH),before=100,after=101,changed=True))
        self.assertEqual(inputs['path'][4:],rs[1].data[4:]);self.assertEqual(request(rs,UNIT)[1]['x'],100)
        self.assertFalse(input_patch(rs,UNIT,'x',100)[1]['changed'])
        self.assertEqual(request(regions(length=2),UNIT)[0],PATH+16)
        self.assertEqual(input_patch(regions(length=2),UNIT,'tolerance',49)[1]['address'],hex(PATH+24))

    def test_invalid_or_missing_input_refuses(self):
        for cap,length in ((0,0),(2,0),(1,2),(4097,1),(3,3)):
            with self.assertRaises(ValueError):request(regions(cap,length),UNIT)
        for field,value in (('flags',1),('x',-1),('x',2**31),('x',True)):
            with self.assertRaises(ValueError):input_patch(regions(),UNIT,field,value)
        for changed in (Region('path',PATH,bytes(32)),Region('path',PATH,bytes(32),writable=True,scratch=True)):
            with self.assertRaises(ValueError):input_patch([regions()[0],changed],UNIT,'x',1)

    def test_reset_restores_control_after_changed_and_refused_inputs(self):
        # Load x, store it in the unit, then return by jump to the declared stop.
        code=b'\xa1'+struct.pack('<I',PATH)+b'\xa3'+struct.pack('<I',UNIT)+b'\xe9'+struct.pack('<i',STOP-(ENTRY+15))
        r=BoundedCall([*regions(),Region('code',ENTRY,code,executable=True)],STOP,budget=20)
        regs=[0]*9;regs[8]=2
        before=execute(r,regs,entry=ENTRY);self.assertTrue(before['returned'])
        patched,_=input_patch(r.regions,UNIT,'x',101)
        after=execute(r,regs,patched,ENTRY);self.assertEqual(after['eax'],101)
        self.assertNotEqual(before['fingerprint'],after['fingerprint'])
        self.assertEqual(before['fingerprint'],execute(r,regs,entry=ENTRY)['fingerprint'])
        # A source address outside the declared input is still refused.
        badcode=b'\xa1'+struct.pack('<I',PATH+32)+b'\xe9'+struct.pack('<i',STOP-(ENTRY+10))
        bad=BoundedCall([*regions(),Region('code',ENTRY,badcode,executable=True)],STOP,budget=20)
        refused=execute(bad,regs,entry=ENTRY);self.assertFalse(refused['returned'])
        self.assertIn('undeclared access',refused['reason'])

    def test_same_runner_control_recovers_after_partial_write_and_refusal(self):
        code=(b'\xa1'+struct.pack('<I',PATH)+b'\xa3'+struct.pack('<I',UNIT)+
              b'\x83\xf8\x64\x74\x05\xa1'+struct.pack('<I',PATH+32)+
              b'\xe9'+struct.pack('<i',STOP-(ENTRY+25)))
        r=BoundedCall([*regions(),Region('code',ENTRY,code,executable=True)],STOP,budget=20)
        regs=[0]*9;regs[8]=2
        baseline=execute(r,regs,entry=ENTRY)
        patched,_=input_patch(r.regions,UNIT,'x',101)
        failed=execute(r,regs,patched,ENTRY)
        self.assertFalse(failed['returned']);self.assertIn('undeclared access',failed['reason'])
        self.assertNotEqual(baseline['fingerprint'],failed['fingerprint'])
        self.assertEqual(baseline['fingerprint'],execute(r,regs,entry=ENTRY)['fingerprint'])

    def test_limit_is_one_word_and_preserves_saving_mode(self):
        rs=regions();inputs,patch=input_patch(rs,UNIT,'limit',1)
        self.assertEqual(patch['before'],75);self.assertEqual(patch['address'],hex(LIMIT))
        self.assertEqual(struct.unpack('<2I',inputs['modes']),(1,1))
        self.assertEqual(struct.unpack('<2I',rs[2].data),(75,1))
        with self.assertRaises(ValueError):input_patch(rs,UNIT,'limit',4097)

    def test_trial_parse(self):
        self.assertEqual(parse_trial('tolerance:0x30'),('tolerance',48))
        with self.assertRaises(ValueError):parse_trial('flags:1')


if __name__=='__main__':unittest.main()
