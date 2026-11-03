"""Fixed public source replay. Never imports/builds target code; retain every outcome."""
import argparse, collections, hashlib, json, os, shutil, subprocess, sys, tarfile, time, urllib.request
from pathlib import Path, PurePosixPath
from concurrent.futures import ThreadPoolExecutor, as_completed

def sha(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(1048576),b''):h.update(b)
 return h.hexdigest()
def write(p,v):p.write_text(json.dumps(v,indent=2),encoding='utf-8')
def inventory(root):
 return [{'path':p.relative_to(root).as_posix(),'bytes':p.stat().st_size,'sha256':sha(p)} for p in sorted(root.rglob('*')) if p.is_file()]
def prepare(a):
 base=Path(a.work).resolve();base.mkdir(parents=True,exist_ok=True);records=[]
 for manifest in a.corpus:
  data=json.loads(Path(manifest).read_text(encoding='utf-8'));group=data['corpus_id']
  for item in data['repositories']:records.append({**item,'group':group})
 for i,r in enumerate(records,1):
  d=base/'sources'/f'{i:03}';d.mkdir(parents=True,exist_ok=True);meta=d/'source.json'
  if meta.exists():
   old=json.loads(meta.read_text());assert old['repository']==r['repository'] and old['commit']==r['commit']
   if old['status']=='ready':continue
   raise RuntimeError('Previous failed acquisition preserved; use a new directory')
  start=time.perf_counter();out={**r,'index':i};root=d/'source';root.mkdir();archive=d/'source.tar.gz'
  try:
   parts=r['repository'].removeprefix('https://github.com/').split('/');assert len(parts)==2 and len(r['commit'])==40
   url=f"https://codeload.github.com/{'/'.join(parts)}/tar.gz/{r['commit']}"
   with urllib.request.urlopen(urllib.request.Request(url,headers={'User-Agent':'PilotLight-fixed-source-replay'}),timeout=90) as response,archive.open('wb') as f:
    total=0
    while b:=response.read(1048576):
     total+=len(b)
     if total>536870912:raise ValueError('Compressed archive exceeds 512 MiB')
     f.write(b)
   omitted=[];seen=set();expanded=0;entries=0
   with tarfile.open(archive,'r:gz') as tar:
    for member in tar:
     entries+=1
     if entries>100000:raise ValueError('Archive entry limit')
     parts=PurePosixPath(member.name).parts
     if len(parts)<2:continue
     rel=PurePosixPath(*parts[1:])
     if rel.is_absolute() or any(p in ['.','..'] or ':' in p or '\\' in p for p in rel.parts):raise ValueError('Unsafe archive path')
     target=(root/str(rel)).resolve()
     if not target.is_relative_to(root):raise ValueError('Archive path escaped source')
     if member.issym() or member.islnk():omitted.append(str(rel));continue
     if member.isdir():target.mkdir(parents=True,exist_ok=True);continue
     if not member.isfile():raise ValueError('Special archive member')
     key=str(rel).casefold()
     if key in seen:raise ValueError('Case-colliding or duplicate path')
     seen.add(key);expanded+=member.size
     if expanded>2147483648:raise ValueError('Expanded archive exceeds 2 GiB')
     target.parent.mkdir(parents=True,exist_ok=True)
     with tar.extractfile(member) as src,target.open('wb') as dest:shutil.copyfileobj(src,dest)
   files=inventory(root);write(d/'source-files.json',files)
   out.update(status='ready',archive_sha256=sha(archive),source_inventory_sha256=sha(d/'source-files.json'),files=len(files),bytes=sum(x['bytes'] for x in files),omitted_links=omitted)
  except Exception as e:out.update(status='acquisition_failed',error=str(e))
  out['acquisition_seconds']=time.perf_counter()-start;write(meta,out);print(i,r['repository'],out['status'],flush=True)
 write(base/'corpus.json',records)

def worker(a):
 source=Path(a.source).resolve();out=Path(a.output).resolve();sys.path.insert(0,str(Path(a.cisco).resolve()/'aibom/src'))
 import faulthandler
 faulthandler.dump_traceback_later(60,repeat=True)
 from aibom import cache_paths
 cache_paths.default_cache_root=lambda:Path(a.work).resolve()/'metadata-cache'
 from aibom.scan_pipeline import ScanPipeline
 import logging
 warnings=[]
 class Capture(logging.Handler):
  def emit(self,r):
   if r.levelno>=logging.WARNING:warnings.append({'level':r.levelname,'message':r.getMessage()})
 logging.getLogger().addHandler(Capture())
 start=time.perf_counter()
 result=ScanPipeline([str(source)],kb_path=str(Path(a.kb).resolve()),llm_config=None,include_code_snippets=False,progress_callback=lambda event: print(json.dumps(event,default=str),flush=True)).run()
 assert result.agentic_status=='skipped'
 write(out,{'components':[c.model_dump(mode='json') for c in result.components],'relationships':[r.model_dump(mode='json') for r in result.relationships],'warnings':warnings,'timings':[vars(t) for t in result.timings],'pipeline_seconds':time.perf_counter()-start,'agentic_status':result.agentic_status})

def run(a):
 base=Path(a.work).resolve();destination=base/a.run;destination.mkdir(exist_ok=False)
 script=Path(__file__).resolve();rows=[]
 write(destination/'run.json',{'scanner':a.scanner,'mode':'Cisco deterministic ScanPipeline; no LLM' if a.scanner=='cisco' else 'Pilot Light default types; jobs=4; max file=2 MiB','timeout_seconds':a.timeout,'corpus':json.loads((base/'corpus.json').read_text()),'engine_sha256':sha(Path(a.exe)) if a.exe else None,'cisco_commit':a.commit,'cisco_kb_sha256':sha(Path(a.kb)) if a.scanner=='cisco' else None,'workers':a.workers,'harness_sha256':sha(script),'timing_qualification':'shared-workstation replay wall time including source integrity checks; no controlled performance or accuracy claim'})
 def replay(directory):
  meta=json.loads((directory/'source.json').read_text());root=directory/'source';out=destination/(directory.name+'.json');row={k:meta[k] for k in ['index','repository','commit','group','status']};start=time.perf_counter()
  if meta['status']=='ready':
   expected=json.loads((directory/'source-files.json').read_text());assert inventory(root)==expected,'Source changed before replay'
   row['source_inventory_sha256']=meta['source_inventory_sha256'];row['source_omitted_links']=len(meta['omitted_links'])
   command=[sys.executable,str(script),'worker','--work',str(destination/'cache'/directory.name),'--source',str(root),'--output',str(out),'--cisco',a.cisco,'--kb',a.kb] if a.scanner=='cisco' else [a.exe,'scan',str(root),'--format','json','--output',str(out),'--jobs','4','--max-file-bytes','2097152']
   try:
    with (destination/(directory.name+'.log')).open('w',encoding='utf-8') as log:proc=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,timeout=a.timeout)
    row['exit_code']=proc.returncode;row['status']='failed' if proc.returncode else 'completed'
    if proc.returncode==0:
     data=json.loads(out.read_text(encoding='utf-8'));items=data['components'] if a.scanner=='cisco' else data['assets'];key='component_type' if a.scanner=='cisco' else 'kind'
     row['by_type']=dict(collections.Counter(x[key] for x in items));row['findings']=len(items)
     if a.scanner=='cisco':
      row['warnings']=len(data['warnings'])
      if data['warnings']:row['status']='completed_with_warnings'
     else:row['status']=data['status'];row['coverage']=data['coverage']
   except subprocess.TimeoutExpired:row['status']='timed_out'
   except Exception as e:row['status']='failed';row['error']=str(e)
   if inventory(root)!=expected:row['status']='source_changed'
  row['process_wall_seconds']=time.perf_counter()-start;return row
 with ThreadPoolExecutor(max_workers=a.workers) as executor:
  futures=[executor.submit(replay,directory) for directory in sorted((base/'sources').iterdir())]
  for future in as_completed(futures):
   row=future.result();rows.append(row);rows.sort(key=lambda r:r['index']);write(destination/'outcomes.json',rows);print(a.run,row['index'],row['repository'],row['status'],row.get('findings'),round(row['process_wall_seconds'],2),flush=True)
 write(destination/'summary.json',{'repositories':len(rows),'outcomes':dict(collections.Counter(r['status'] for r in rows))})

if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('mode',choices=['prepare','run','worker']);p.add_argument('--work',required=True);p.add_argument('--corpus',action='append');p.add_argument('--scanner',choices=['cisco','pilotlight']);p.add_argument('--run');p.add_argument('--exe');p.add_argument('--cisco');p.add_argument('--commit');p.add_argument('--kb');p.add_argument('--source');p.add_argument('--output');p.add_argument('--timeout',type=int,default=300);p.add_argument('--workers',type=int,choices=range(1,5),default=1);a=p.parse_args()
 {'prepare':prepare,'run':run,'worker':worker}[a.mode](a)
