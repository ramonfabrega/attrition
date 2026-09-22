import copy
import struct
import unittest
from defined_bytes import DefinedBytes
from defined_search_graph import GraphObserver
from compare_search_graph import compare,require_known_agreement,require_required_field_agreement,checked_graph,paired_graphs,derive_correspondence
from test_search_graph import fixture


def graph(items,unit=0x10000,holes=()):
    def read(a,n):
        data=next(d for _,_,address,d in items if address==a and len(d)==n)
        return DefinedBytes(tuple(None if a+i in holes else v for i,v in enumerate(data)))
    g=GraphObserver(read,unit,223).run()
    assert g['complete'],g['reason']
    return g


def relocate(items,delta=0x100000):
    # Authored fixture transformation independent of comparator policy. Pools
    # are globals; every other observed object moves. All payloads are explicit.
    mapping={a:(a if k==6 else a+delta) for k,o,a,d in items}
    out=[]
    for k,o,a,d in items:
        d=bytearray(d)
        offsets={1:[0,4,8,12,16],2:[12,16,20]+([] if o==3 else [0]),
                 3:[0,4,8]+([] if o==3 else [12]),4:[32],5:[],6:[0],7:[]}[k]
        for at in offsets:
            value=struct.unpack_from('<I',d,at)[0]
            struct.pack_into('<I',d,at,mapping.get(value,value))
        out.append((k,o,mapping[a],bytes(d)))
    return out,list(mapping.items())


def change(items,kind,owner,offset,value):
    out=items.copy();i=next(i for i,r in enumerate(out) if r[:2]==(kind,owner))
    k,o,a,d=out[i];data=bytearray(d);struct.pack_into('<I',data,offset,value);out[i]=(k,o,a,bytes(data));return out


class CorrespondenceTests(unittest.TestCase):
    def test_controlled_relocation_preserves_all_known_values(self):
        items=fixture();moved,pairs=relocate(items)
        result=compare(graph(items),graph(moved,0x110000),pairs)
        require_known_agreement(result)
        self.assertTrue(result['all_bytes_known']);self.assertFalse(result['literal_byte_values_agree'])
        self.assertTrue(result['relocated_pointer_fields']);self.assertFalse(result['native_compared'])

    def test_derived_mapping_matches_authored_relocation_and_ignores_values(self):
        items=fixture();moved,pairs=relocate(items)
        derivation=derive_correspondence(graph(items),graph(moved,0x110000))
        self.assertEqual(dict(derivation['pairs']),dict(pairs))
        changed=graph(change(moved,4,0,0,42),0x110000)
        self.assertEqual(derivation,derive_correspondence(graph(items),changed))
        self.assertFalse(compare(graph(items),changed,derivation['pairs'])['required_fields_agree'])
        # A valid but different parent relation cannot be paired silently.
        changed=items.copy();i=next(i for i,r in enumerate(changed) if r[2]==0x40100)
        k,o,a,d=changed[i];changed[i]=(k,o,a,d[:32]+bytes(4))
        with self.assertRaisesRegex(ValueError,'edge presence'):
            derive_correspondence(graph(items),graph(changed))

    def test_nulls_and_unknowns_are_explicit_not_zero_or_agreement(self):
        items=fixture();a=graph(items,holes={0x30116,0x30117})
        pairs=[(r[2],r[2]) for r in items]
        result=compare(a,graph(items),pairs);require_known_agreement(result)
        self.assertEqual(len(result['unknown_byte_positions']),2)
        self.assertFalse(result['all_bytes_known']);self.assertFalse(result['definedness_equal'])
        self.assertFalse(result['literal_byte_values_agree'])
        result=compare(a,a,pairs);self.assertTrue(result['definedness_equal'])
        self.assertEqual(len(result['unknown_byte_positions']),2)

    def test_topology_keys_payloads_flags_and_opaque_bytes_cannot_hide(self):
        items=fixture();a=graph(items);pairs=[(r[2],r[2]) for r in items]
        # These changes preserve basic structural validity, but must not compare equal.
        for k,o,offset,value in [(3,0,16,99),(4,0,0,42),(5,4,40,0x40000),(1,0,68,99),
                                 (2,0,4,99),(3,3,12,0x40000),(7,0,28,0x40000)]:
            b=graph(change(items,k,o,offset,value))
            with self.subTest(kind=k,owner=o,offset=offset):
                result=compare(a,b,pairs);self.assertFalse(result['known_bytes_agree'])
                with self.assertRaises(ValueError):require_known_agreement(result)
        # Parent/link corruption cannot get through graph revalidation.
        bad=copy.deepcopy(a);row=next(r for r in bad['records'] if (r['kind'],r['owner'])==(3,0))
        row['values'][0:4]=list(struct.pack('<I',0x30000))
        with self.assertRaises(ValueError):compare(a,bad,pairs)

    def test_address_shaped_scalars_remain_literal_under_relocation(self):
        items=fixture()
        for k,o,offset in [(3,3,12),(2,3,0),(3,1,16),(4,0,24),(1,0,20),(5,4,0),(7,0,0)]:
            original=change(items,k,o,offset,0x40000);moved,pairs=relocate(original)
            # Deliberately rewrite an integer/opaque/unused word as if a pointer.
            changed=change(moved,k,o,offset,0x140000)
            result=compare(graph(original),graph(changed,0x110000),pairs)
            with self.subTest(kind=k,owner=o,offset=offset):self.assertFalse(result['known_bytes_agree'])

    def test_optional_known_difference_is_reported_and_strict_assertion_fails(self):
        items=fixture();pairs=[(r[2],r[2]) for r in items]
        for k,o,offset,value in [(7,0,28,1),(3,1,20,0x10000)]:
            result=compare(graph(items),graph(change(items,k,o,offset,value)),pairs)
            require_required_field_agreement(result)
            self.assertFalse(result['known_bytes_agree']);self.assertTrue(result['differences'])
            self.assertTrue(all(not d['required'] for d in result['differences']))
            with self.assertRaises(ValueError):require_known_agreement(result)
        result=compare(graph(items),graph(change(items,3,1,20,1)),pairs)
        with self.assertRaises(ValueError):require_required_field_agreement(result)

    def test_wrong_valid_topology_and_correspondence_do_not_match(self):
        items=fixture();pairs=[(r[2],r[2]) for r in items]
        # The second PathNode may legitimately have no parent; that is still
        # a different observed graph and cannot be repaired by correspondence.
        changed=items.copy();i=next(i for i,r in enumerate(changed) if r[2]==0x40100)
        k,o,a,d=changed[i];changed[i]=(k,o,a,d[:32]+bytes(4))
        result=compare(graph(items),graph(changed),pairs)
        self.assertFalse(result['required_fields_agree'])
        # Swap the two same-typed PathNode pairs, keeping the map bijective.
        wrong=[(a,0x40100 if a==0x40000 else 0x40000 if a==0x40100 else b) for a,b in pairs]
        result=compare(graph(items),graph(items),wrong)
        self.assertFalse(result['required_fields_agree'])

    def test_cli_pair_binding_rejects_mixed_inputs(self):
        a=dict(payload_sha256='payload',image_sha256='image',prepared_experiments=dict(
            graph=graph(fixture()),native_unit_path=dict(replay=dict(payload_sha256='payload',native_intervention={'limit':95}))))
        self.assertEqual(paired_graphs(a,a),(a['prepared_experiments']['graph'],)*2)
        for field in ['payload','image','intervention','nested_payload']:
            b=copy.deepcopy(a)
            if field=='payload':b['payload_sha256']='other'
            if field=='image':b['image_sha256']='other'
            if field=='intervention':b['prepared_experiments']['native_unit_path']['replay']['native_intervention']=None
            if field=='nested_payload':b['prepared_experiments']['native_unit_path']['replay']['payload_sha256']='other'
            with self.subTest(field=field),self.assertRaises(ValueError):paired_graphs(a,b)

    def test_correspondence_must_cover_records_bijectively(self):
        a=graph(fixture());pairs=[(r[2],r[2]) for r in fixture()]
        for bad in [pairs[:-1],pairs+[pairs[0]],[(pairs[0][0],pairs[1][1]),*pairs[1:]],
                    [(pairs[0][0],0xdead0000),*pairs[1:]],[(pairs[1][0],pairs[0][1]),*pairs[1:]]]:
            with self.assertRaises(ValueError):compare(a,a,bad)

    def test_metadata_unknown_required_fields_and_record_tags_are_revalidated(self):
        a=graph(fixture());pairs=[(r[2],r[2]) for r in fixture()]
        for mutation in ['count','type','required','complete','extra']:
            b=copy.deepcopy(a)
            if mutation=='count':b['undefined_bytes']=1
            if mutation=='type':b['records'][0]['kind']=4
            if mutation=='required':b['records'][0]['values'][0]=None
            if mutation=='complete':b['complete']=False
            if mutation=='extra':b['records'].append(copy.deepcopy(b['records'][0]))
            with self.subTest(mutation=mutation),self.assertRaises(ValueError):compare(a,b,pairs)
        self.assertEqual(len(checked_graph(a)),len(fixture()))
