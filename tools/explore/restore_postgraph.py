#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Validate a paired native post-graph and compare a retained model observation.

Generated packets/reports contain original-derived bytes and belong outside git.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
from defined_bytes import DefinedBytes
from defined_search_graph import GraphObserver
from compare_search_graph import checked_graph,derive_correspondence,compare,require_required_field_agreement
from search_graph import validate,parse
from restore_poststate import decode_post,check_receipt,compare as compare_unit,require_agreement
from memory_payload import validate as validate_payload
from search_census import records
from replay_capsule import require
from payload_replay import IMAGE_SHA256


def decode_graph(raw,post_raw,prefix_raw):
    require(32<=len(raw)<=32+628+65536+32+256*1024,'postgraph extent outside bound')
    magic,version,frame,unit,post_size,graph_size,elapsed,boundary=struct.unpack_from('<8I',raw)
    require((magic,version,boundary)==(0x31504752,1,0x688faa) and elapsed<2000,'unsupported postgraph or deadline')
    require(post_size==len(post_raw) and len(raw)==32+post_size+graph_size,'postgraph framing differs')
    require(raw[32:32+post_size]==post_raw,'postgraph belongs to another post-state')
    post=decode_post(post_raw,prefix_raw)
    require((frame,unit)==(post['frame'],post['unit']),'postgraph event differs')
    binary=raw[32+post_size:];validate(binary);h,items=parse(binary)
    require((h[2],h[3])==(frame,unit),'embedded graph event differs')
    index={a:(k,o,d) for k,o,a,d in items}
    require(index[unit+0x104][2]==bytes.fromhex(post['unit_bytes'])[0x104:0x14c],
            'graph unit differs from paired post-state')
    def read(a,n):
        require(a in index and len(index[a][2])==n,'missing native graph record')
        return DefinedBytes(tuple(index[a][2]))
    graph=GraphObserver(read,unit,frame).run();graph['schema']='native-defined-search-graph-v1'
    checked_graph(graph)
    require(len(graph['records'])==len(items),'native graph records omitted')
    return dict(schema='native-postgraph-v1',post=post,graph=graph,elapsed_ms=elapsed,
                packet_bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest())


def check_graph_receipt(rows,native):
    post=native['post'];check_receipt(rows,post)
    expected=(5,186,0,post['unit'],native['packet_bytes'],len(native['graph']['records']),native['elapsed_ms'],post['frame'])
    require([r for r in rows if r[:2]==(5,186)]==[expected],'postgraph receipt missing or mismatched')
    require(not any(r[:2]==(5,187) for r in rows),'postgraph collector failure')
    start=next(i for i,r in enumerate(rows) if r[:2]==(5,181))
    end=next(i for i,r in enumerate(rows) if r[:2]==(5,186))
    require(start<end and not any(r[:2] in ((7,0),(8,0)) for r in rows[start+1:end]),
            'postgraph not immediately after the paired return observation')


def compare_native(native,replay,payload_sha):
    require(replay['image_sha256']==IMAGE_SHA256,'unsupported replay image')
    require(replay['payload_sha256']==payload_sha,'graph replay belongs to another payload')
    experiment=replay['prepared_experiments'];witness=experiment['native_unit_path']['replay']
    unit_comparison=compare_unit(native['post'],witness,payload_sha);require_agreement(unit_comparison)
    require(experiment['graph_repeats'] is True and experiment['observer_read_only'] is True and
            experiment['control_restored'] is True,'model observation checks absent')
    graph=experiment['graph'];require(graph['schema']=='model-defined-search-graph-v1','model graph required')
    root=next(r for r in graph['records'] if (r['kind'],r['owner'])==(1,0))
    observed=witness['last']['unit_path_observation']
    require(int(root['address'],16)==native['post']['unit']+0x104 and
            root['values']==list(bytes.fromhex(observed['unit_bytes'])[0x104:0x14c]),
            'model graph unit differs from paired replay observation')
    derivation=derive_correspondence(native['graph'],graph)
    result=compare(native['graph'],graph,derivation['pairs'])
    result.update(schema='native-model-graph-comparison-v1',native_compared=True,
                  native_packet_sha256=native['sha256'],payload_sha256=payload_sha,
                  unit_path_comparison=unit_comparison,derivation=derivation,
                  full_native_state_compared=False)
    return result


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install',type=Path);ap.add_argument('directory',type=Path);ap.add_argument('--replay',type=Path,required=True)
    ap.add_argument('--require-required-fields',action='store_true');args=ap.parse_args()
    d=args.directory
    native=decode_graph((d/'restore-postgraph.bin').read_bytes(),(d/'restore-poststate.bin').read_bytes(),
                        (d/'restore-prefix.bin').read_bytes())
    check_graph_receipt(list(records(d/'rontrace.log')),native)
    payload=validate_payload(args.install,d);replay=json.loads(args.replay.read_text())
    result=compare_native(native,replay,payload['sha256'])
    print(json.dumps(result,indent=2))
    if args.require_required_fields:require_required_field_agreement(result)


if __name__=='__main__':main()
