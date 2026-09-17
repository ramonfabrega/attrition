import struct
import unittest
from wow64_fault import evidence


def authored_image(displacement=0x1234):
    data=bytearray(0x300);data[:2]=b'MZ';struct.pack_into('<I',data,60,0x80)
    data[0x80:0x84]=b'PE\0\0';struct.pack_into('<HH',data,0x84,0x8664,1)
    struct.pack_into('<H',data,0x94,0xf0);struct.pack_into('<H',data,0x98,0x20b)
    struct.pack_into('<4I',data,0x190,0x100,0x1000,0x100,0x200)
    data[0x210:0x216]=b'\x8b\x15'+struct.pack('<i',displacement)
    return data


class Wow64FaultTests(unittest.TestCase):
    def test_two_effective_addresses_are_not_a_mode_verdict(self):
        report=evidence(authored_image(),0x100000,0x101010,0x1234)
        self.assertTrue(report['access_matches_i386'])
        self.assertFalse(report['access_matches_x86_64'])
        self.assertEqual(report['x86_64_effective_address'],'0x10224a')
        self.assertFalse(report['active_cpu_mode_established'])

    def test_signed_displacement_and_64bit_match(self):
        report=evidence(authored_image(-16),0x100000,0x101010,0x101006)
        self.assertTrue(report['access_matches_x86_64'])
        self.assertEqual(report['i386_effective_address'],'0xfffffff0')

    def test_refuse_wrong_opcode_truncation_and_unbacked_address(self):
        bad=authored_image();bad[0x210]=0x90
        for data,pc in ((bad,0x101010),(authored_image()[:0x214],0x101010),(authored_image(),0x102000)):
            with self.assertRaises(ValueError):evidence(data,0x100000,pc,0)


if __name__=='__main__':unittest.main()
