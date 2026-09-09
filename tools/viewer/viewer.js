'use strict';
(() => {
 const $ = id => document.getElementById(id);
 let data;
 try {
  data=JSON.parse($('replay-data').textContent);
  if(data.schema!==1 || !Array.isArray(data.frames) || !data.frames.length) throw Error('Unsupported or empty replay data.');
  for(const f of data.frames) if(new Set(f.units.map(u=>u.id)).size!==f.units.length) throw Error('Duplicate unit identity in export.');
  for(const f of data.frames) for(const u of f.units) for(const p of [u.original,u.rust,...u.originalPath,...u.rustPath]) if(p && (!Array.isArray(p)||p.length!==2||!p.every(Number.isSafeInteger))) throw Error('Unsafe or malformed coordinate in export.');
 } catch(e){$('app').hidden=true;$('error').hidden=false;$('error').textContent=e.message;return;}
 const canvas=$('map'),ctx=canvas.getContext('2d');
 let index=0,selected=null,camera={x:0,y:0,scale:1},bounds,w=1,h=1,drag=null,playing=null,hits=[];
 const frame=()=>data.frames[index];
 const unit=()=>frame().units.find(u=>u.id===selected);
 const status=u=>!u.scope?'Outside player comparison scope':!u.original?'Rust only':!u.rust?'Original only':u.original[0]!==u.rust[0]||u.original[1]!==u.rust[1]?'Position disagreement':'Position agrees';
 const issue=u=>u.scope&&status(u)!=='Position agrees';
 const screen=p=>[(p[0]-camera.x)*camera.scale+w/2,(p[1]-camera.y)*camera.scale+h/2];
 const world=p=>[(p[0]-w/2)/camera.scale+camera.x,(p[1]-h/2)/camera.scale+camera.y];
 function fit(){camera={x:(bounds[0]+bounds[2])/2,y:(bounds[1]+bounds[3])/2,scale:Math.min(w/Math.max(1,bounds[2]-bounds[0]),h/Math.max(1,bounds[3]-bounds[1]))*.9};draw();}
 function resize(){const r=canvas.getBoundingClientRect();w=r.width;h=r.height;const d=window.devicePixelRatio||1;canvas.width=w*d;canvas.height=h*d;ctx.setTransform(d,0,0,d,0,0);draw();}
 function line(a,b,color,dashed=false){const p=screen(a),q=screen(b);ctx.strokeStyle=color;ctx.setLineDash(dashed?[5,4]:[]);ctx.beginPath();ctx.moveTo(...p);ctx.lineTo(...q);ctx.stroke();ctx.setLineDash([]);}
 function draw(){
  ctx.clearRect(0,0,w,h);ctx.fillStyle='#eaf0f6';ctx.fillRect(0,0,w,h);hits=[];
  const step=2**Math.ceil(Math.log2(75/camera.scale));const low=world([0,0]),high=world([w,h]);
  ctx.lineWidth=1;ctx.font='10px sans-serif';ctx.fillStyle='#71889a';
  for(let x=Math.ceil(low[0]/step)*step;x<high[0];x+=step){line([x,low[1]],[x,high[1]],'#d5e0eb');ctx.fillText(String(x),screen([x,0])[0]+3,14);}
  for(let y=Math.ceil(low[1]/step)*step;y<high[1];y+=step){line([low[0],y],[high[0],y],'#d5e0eb');ctx.fillText(String(y),4,screen([0,y])[1]-3);}
  const p=screen([0,0]),q=screen(data.world);ctx.strokeStyle='#9bb0c5';ctx.strokeRect(p[0],p[1],q[0]-p[0],q[1]-p[1]);
  const chosen=unit();if(chosen&&$('paths').checked){for(const [pos,path,color] of [[chosen.original,chosen.originalPath,'#2065c0'],[chosen.rust,chosen.rustPath,'#b94d22']]){if(!pos)continue;let prev=pos;for(const p of path){line(prev,p,color,true);const s=screen(p);ctx.fillStyle=color;ctx.fillRect(s[0]-2,s[1]-2,4,4);prev=p;}}}
  for(const u of frame().units){
   if($('only').checked&&!issue(u)&&u.id!==selected)continue;
   const chosen=u.id===selected;
   if(u.original&&u.rust&&issue(u)){ctx.lineWidth=chosen?2:1;line(u.original,u.rust,chosen?'#775397':'#bd7f77');}
   for(const [pos,original] of [[u.original,true],[u.rust,false]]){
    if(!pos)continue;const [x,y]=screen(pos);const r=chosen?7:4;
    if(chosen){ctx.beginPath();ctx.arc(x,y,12,0,Math.PI*2);ctx.strokeStyle='#775397';ctx.lineWidth=1;ctx.stroke();}
    ctx.strokeStyle=u.scope?(original?'#2065c0':'#b94d22'):'#7a8794';ctx.fillStyle=ctx.strokeStyle;ctx.lineWidth=1.5;
    ctx.beginPath();if(original)ctx.arc(x,y,r,0,Math.PI*2);else{ctx.moveTo(x,y-r-1);ctx.lineTo(x+r+1,y);ctx.lineTo(x,y+r+1);ctx.lineTo(x-r-1,y);ctx.closePath();}
    if(original)ctx.fill();else ctx.stroke();hits.push({x,y,id:u.id});
    if(chosen){ctx.fillStyle='#183047';ctx.font='12px sans-serif';ctx.fillText(u.id,x+15,y-10);}
   }
  }
 }
 function provenance(){return JSON.stringify({capture:data.capture,sourceBytes:data.sourceBytes,revision:data.revision,siblings:data.siblings,trace:data.trace,recording:data.recording,seedInputs:data.seedInputs,guyInputs:data.guyInputs,frame:frame().n,sourceFrameIndex:frame().index,seedInstalledThisTick:frame().seedInstalledThisTick,rngAfterHarnessTick:frame().rng,notes:data.notes,applied:data.applied,reproduce:data.reproduce},null,2);}
 function inspect(){
  const u=unit();$('unit-title').textContent=u?`Unit ${u.id}`:'Unit absent on this frame';
  $('status').textContent=u?`${status(u)}. Path vertices run from stack top toward the goal. Empty paths or orders can mean they were not logged.`:'Choose a unit present in this frame.';
  $('coordinates').replaceChildren();for(const [axis,i] of [['x',0],['y',1]]){const tr=document.createElement('tr');const vals=[axis,u?.original?.[i]??'—',u?.rust?.[i]??'—',u?.original&&u?.rust?u.rust[i]-u.original[i]:'—'];for(const v of vals){const td=document.createElement('td');td.textContent=v;tr.append(td);}$('coordinates').append(tr);}
  const mode=$('record').value;$('details').textContent=mode==='report'?frame().report:mode==='provenance'?provenance():u?.[mode]??'This unit is not present in the exported frame.';
  $('focus').disabled=!u?.original&&!u?.rust;draw();
 }
 function link(){const p=new URLSearchParams({index:String(index),unit:selected??'',record:$('record').value});return '#'+p.toString();}
 function render(){
  const f=frame();$('slider').value=index;$('frame-label').replaceChildren();const title=document.createElement('span');title.textContent=`Frame ${f.n}`;const detail=document.createElement('small');detail.textContent=`${index+1} of ${data.frames.length} exported records`; $('frame-label').append(title,detail);
  $('previous').disabled=index===0;$('next').disabled=index===data.frames.length-1;$('compared').textContent=f.compared;$('mismatch').textContent=f.positionMismatches;
  $('coverage').textContent=`${f.issues} total reported issues · ${f.orderCompared} order lists compared`;
  $('unit').replaceChildren();for(const u of f.units){const o=document.createElement('option');o.value=u.id;o.textContent=`${u.id} · ${status(u)}`;$('unit').append(o);}
  if(!selected)selected=f.units.find(issue)?.id??f.units[0]?.id??null;
  if(selected&&!unit()){const o=document.createElement('option');o.value=selected;o.textContent=`${selected} · Absent on this frame`;$('unit').prepend(o);}
  $('unit').value=selected??'';inspect();
 }
 function move(i){index=Math.max(0,Math.min(data.frames.length-1,i));render();}
 function stop(){if(playing)clearInterval(playing);playing=null;$('play').textContent='Play';}
 function play(){if(playing)return stop();if(index===data.frames.length-1)move(0);playing=setInterval(()=>{if(index===data.frames.length-1)stop();else move(index+1);},180);$('play').textContent='Pause';}
 function restore(){const p=new URLSearchParams(location.hash.slice(1));index=Math.max(0,Math.min(data.frames.length-1,Number(p.get('index')??data.focusIndex??0)||0));index=Math.floor(index);selected=p.get('unit')||null;if(['originalRecord','rustRecord','report','provenance'].includes(p.get('record')))$('record').value=p.get('record');render();}
 bounds=[0,0,data.world[0],data.world[1]];for(const f of data.frames)for(const u of f.units)for(const p of [u.original,u.rust])if(p){bounds[0]=Math.min(bounds[0],p[0]);bounds[1]=Math.min(bounds[1],p[1]);bounds[2]=Math.max(bounds[2],p[0]);bounds[3]=Math.max(bounds[3],p[1]);}
 $('source').textContent=data.capture.split(/[\\/]/).pop()+` · ${data.revision}`;
 $('assistance').textContent=data.seedInputs?'Assisted replay inputs configured':'No RNG reseeding configured';
 $('scope').textContent=`Capture-derived setup. ${data.seedInputs} RNG seed records; ${data.guyInputs} figure-state records. Recording: ${data.recording==='Not supplied'?'not supplied':'supplied'}; trace: ${data.trace==='Not supplied'?'not supplied':'supplied'}. Map colors distinguish implementations, not players. Terrain and border layers are not exported.`;
 const failureNote=data.notes.find(n=>n.startsWith('Failure selected by test:'));if(failureNote)$('scope').textContent+=' '+failureNote;
 $('slider').max=data.frames.length-1;data.frames.forEach((f,i)=>{if(f.issues){const mark=document.createElement('i');mark.style.left=`${i/Math.max(1,data.frames.length-1)*100}%`;$('marks').append(mark);}});
 $('slider').oninput=()=>{stop();move(Number($('slider').value));};$('previous').onclick=()=>move(index-1);$('next').onclick=()=>move(index+1);$('play').onclick=play;
 $('next-issue').onclick=()=>{stop();for(let k=1;k<=data.frames.length;k++){const i=(index+k)%data.frames.length;if(data.frames[i].issues){move(i);selected=frame().units.find(issue)?.id??selected;render();return;}}$('feedback').textContent='No reported issues in this window';};
 $('unit').onchange=()=>{selected=$('unit').value;inspect();};$('record').onchange=inspect;$('only').onchange=draw;$('paths').onchange=draw;$('fit').onclick=fit;
 $('focus').onclick=()=>{const u=unit(),p=u?.original??u?.rust;if(p){camera.x=p[0];camera.y=p[1];camera.scale=Math.max(camera.scale,Math.min(w,h)/3000);draw();}};
 $('link').onclick=async()=>{location.hash=link();try{await navigator.clipboard.writeText(location.href);$('feedback').textContent='Link copied';}catch{$('feedback').textContent='Selection saved in URL';}};
 $('repro').onclick=()=>{const text=provenance()+'\n\nExport command (choose a new output path):\n'+data.reproduce+'\n';const url=URL.createObjectURL(new Blob([text],{type:'text/plain'}));const a=document.createElement('a');a.href=url;a.download=`replay-frame-${frame().n}-unit-${(selected??'none').replace('/','-')}.txt`;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);};
 canvas.onpointerdown=e=>{canvas.setPointerCapture(e.pointerId);drag={x:e.clientX,y:e.clientY,cx:camera.x,cy:camera.y,moved:false};};
 canvas.onpointermove=e=>{if(!drag)return;const dx=e.clientX-drag.x,dy=e.clientY-drag.y;if(Math.hypot(dx,dy)>4)drag.moved=true;camera.x=drag.cx-dx/camera.scale;camera.y=drag.cy-dy/camera.scale;draw();};
 canvas.onpointerup=e=>{if(drag&&!drag.moved){const rect=canvas.getBoundingClientRect();const nearest=hits.map(p=>({...p,d:Math.hypot(p.x-e.clientX+rect.left,p.y-e.clientY+rect.top)})).sort((a,b)=>a.d-b.d)[0];if(nearest?.d<18){selected=nearest.id;render();}}drag=null;};canvas.onpointercancel=()=>{drag=null;};
 canvas.addEventListener('wheel',e=>{e.preventDefault();const r=canvas.getBoundingClientRect(),p=[e.clientX-r.left,e.clientY-r.top],before=world(p);camera.scale=Math.max(.0001,Math.min(20,camera.scale*Math.exp(-e.deltaY*.001)));const after=world(p);camera.x+=before[0]-after[0];camera.y+=before[1]-after[1];draw();},{passive:false});
 document.addEventListener('keydown',e=>{if(['INPUT','SELECT','BUTTON','TEXTAREA'].includes(document.activeElement.tagName))return;if(e.key==='ArrowRight'){e.preventDefault();move(index+1);}if(e.key==='ArrowLeft'){e.preventDefault();move(index-1);}if(e.code==='Space'){e.preventDefault();play();}});
 window.addEventListener('hashchange',restore);new ResizeObserver(resize).observe(canvas);resize();fit();restore();
})();
