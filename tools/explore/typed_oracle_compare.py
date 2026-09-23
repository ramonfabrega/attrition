"""Reproduce the partial typed-state/logger comparison from a retained capture.

Writes generated state outside Git: complete comparison (gzip), coverage, and
provenance/timing. Exit zero means analysis completed, never full parity.
"""
import argparse,gzip,hashlib,json,time
from pathlib import Path
from frame_state import experiment
from typed_fields import Decoder
from typed_state import Payload,Types,require
from typed_logger_inventory import extract
from typed_logger_compare import compare
from typed_leader_bridge import apply as leader_bridge
from typed_leader_projection import apply as leader_projection
from typed_world_bridge import grid_plan,cell_plan,apply as world_bridge,apply_cells
from typed_coverage_report import summarize
from typed_guy_bridge import decode_figures,apply as guy_bridge


def write_json(path,value):
    path.write_text(json.dumps(value,indent=2)+'\n')


def run(install,capture,export,plan_path,output):
    # Refuse destinations within any Git checkout, including ignored directories.
    # Install-derived reports remain external even when a caller forgets that rule.
    require(not any((p/'.git').exists() for p in (output.resolve(),*output.resolve().parents)),
            'generated output must be outside Git worktrees')
    output.mkdir(parents=True,exist_ok=False);start=time.monotonic()
    manifest=dict(schema='typed-oracle-reproduction-v1',status='running',whole_frame_parity=False,
                  install=str(install.resolve()),capture=str(capture.resolve()),export=str(export.resolve()),
                  plan_sha256=hashlib.sha256(plan_path.read_bytes()).hexdigest(),timings_seconds={})
    source=Path(__file__).parent
    manifest['authored_tool_sha256']={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(source.glob('*.py'))
                                     if p.name.startswith(('typed_','frame_'))}
    write_json(output/'manifest.json',manifest)
    def stage(name,fn):
        before=time.monotonic();value=fn();manifest['timings_seconds'][name]=round(time.monotonic()-before,6);return value
    try:
        state=stage('decode_state',lambda:experiment(install,capture,export,json.loads(plan_path.read_text())))
        types=Types(json.loads((export/'types.json').read_text()));frame=state['snapshot']['frame']
        lines=stage('extract_logger_frame',lambda:extract(capture/'gamelog.txt',frame))
        report=stage('unit_world_scalars',lambda:compare(lines,frame,state,types));del lines
        memory=Payload(capture/'frame-snapshot.bin',state['snapshot'])
        def decode(pointer):
            d=Decoder(types,memory);d.decode(pointer['pointee'],pointer['value'],'encrypted');return d.rows
        try:
            stage('leader_encrypted',lambda:leader_bridge(report,state,decode))
            stage('leader_projections',lambda:leader_projection(report,state))
            stage('world_grids',lambda:world_bridge(report,state,grid_plan(state,types,memory)))
            stage('world_cells',lambda:apply_cells(report,cell_plan(state,types,memory)))
            stage('guy_records',lambda:guy_bridge(report,state,decode_figures(state,types,memory)))
        finally:memory.close()
        coverage=stage('coverage_accounting',lambda:summarize(state,report))
        write_json(output/'coverage.json',coverage)
        before=time.monotonic()
        with gzip.open(output/'comparison.json.gz','wt',compresslevel=1) as stream:json.dump(report,stream)
        manifest['timings_seconds']['write_comparison']=round(time.monotonic()-before,6)
        manifest.update(status='completed_partial_comparison',snapshot_sha256=state['snapshot']['sha256'],
                        types_sha256=state['types_sha256'],frame=frame,frame_text_sha256=report['frame_text_sha256'],
                        matched_occurrences=report['matched_occurrences'],observable_occurrences=report['observable_occurrences'],
                        mismatch_occurrences=report['status_counts'].get('integer_mismatch',0),
                        bridge_errors={key:report[key]['errors'] for key in ('leader_bridge','leader_projection','world_grid_bridge','world_cell_bridge','guy_bridge')})
    except Exception as error:
        manifest.update(status='failed',error=str(error));raise
    finally:
        manifest['elapsed_seconds']=round(time.monotonic()-start,6);write_json(output/'manifest.json',manifest)
    return manifest


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for name in ('install','capture','export','plan','output'):p.add_argument(name,type=Path)
    a=p.parse_args();print(json.dumps(run(a.install,a.capture,a.export,a.plan,a.output),indent=2))
