"""Inventory every line of one complete logger frame without claiming decoding.

Scope is generated from the dump, not a hand-maintained key list. Indentation
is retained as evidence; ambiguous field ownership is never resolved by fiat.
"""
import argparse,collections,hashlib,json,re
from pathlib import Path
from typed_state import require


def extract(path,frame,max_lines=2000000,max_bytes=128*1024*1024):
    before=path.stat();found=0;active=False;closed=False;lines=[];size=0
    with path.open('rb') as f:
        for number,raw in enumerate(f,1):
            match=re.fullmatch(rb'\s*BEGIN FRAME ([0-9]+)\s*',raw)
            if match:
                if active:closed=True;active=False
                if int(match[1])==frame:
                    found+=1;require(found==1,'duplicate requested frame');active=True
                continue
            if active:
                size+=len(raw);require(len(lines)<max_lines and size<=max_bytes,'frame budget exceeded')
                lines.append((number,raw.decode('utf-8').rstrip('\r\n')))
    after=path.stat()
    require((before.st_size,before.st_mtime_ns)==(after.st_size,after.st_mtime_ns),'logger changed while reading')
    require(found==1 and closed,'missing or incomplete requested frame')
    require(any(line.strip() for _,line in lines),'empty requested frame')
    return lines


def inventory(lines,frame):
    rows=[];structures=[];stack=[];counts=collections.Counter();blank=0
    for number,line in lines:
        text=line.strip()
        if not text:blank+=1;continue
        depth=len(line)-len(line.lstrip(' '))
        if text.startswith('BEGIN '):
            while stack and stack[-1]['depth']>=depth:stack.pop()
            block=dict(line=number,depth=depth,label=text[6:])
            structures.append(block);stack.append(block);continue
        if text.startswith('END '):
            # Retain rather than interpreting an unexpected end marker as a key.
            structures.append(dict(line=number,depth=depth,label=text,kind='end_marker'));continue
        key,separator,value=text.partition(' ')
        ownership='ambiguous_indentation' if stack and depth<=stack[-1]['depth'] else 'nested_indentation'
        while stack and stack[-1]['depth']>=depth:stack.pop()
        status='unmapped' if separator else 'unclassified_text'
        counts[key]+=1
        rows.append(dict(line=number,depth=depth,key=key,value=value,raw=line,
                         enclosing_labels=[b['label'] for b in stack],owner_block_line=stack[-1]['line'] if stack else None,
                         logger_path='GAME/FRAME/'+ '/'.join(re.sub(r' [0-9]+$','',b['label']) for b in stack),
                         ownership=ownership,status=status))
    require(rows,'frame has no observable fields')
    require(len(rows)+len(structures)+blank==len(lines),'line accounting differs')
    digest=hashlib.sha256('\n'.join(line for _,line in lines).encode()).hexdigest()
    return dict(schema='typed-logger-inventory-v1',frame=frame,frame_text_sha256=digest,
                total_lines=len(lines),structural_lines=len(structures),blank_lines=blank,
                observable_occurrences=len(rows),distinct_tokens=len(counts),token_counts=dict(counts),
                rows=rows,structures=structures,matched_occurrences=0,logger_parity_established=False,
                all_printed_lines_accounted=True,semantic_ownership_complete=False)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('gamelog',type=Path);p.add_argument('frame',type=int)
    a=p.parse_args();print(json.dumps(inventory(extract(a.gamelog,a.frame),a.frame),indent=2))
