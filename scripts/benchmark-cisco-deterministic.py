"""Run Cisco's unmodified deterministic ScanPipeline on fixed local cases.
LLM analysis is excluded; public model metadata may be fetched when allowed.
Target code is never imported.
"""
import argparse, hashlib, json, logging, socket, sys, time
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--source',required=True);p.add_argument('--kb',required=True);p.add_argument('--corpus',required=True);p.add_argument('--output',required=True);p.add_argument('--allow-metadata-network',action='store_true')
a=p.parse_args();source=Path(a.source).resolve();corpus=Path(a.corpus).resolve();out=Path(a.output).resolve();out.mkdir(parents=True,exist_ok=False)
sys.path.insert(0,str(source/'aibom/src'))
from aibom import cache_paths
cache_paths.default_cache_root=lambda: out.parent/"cisco-metadata-cache"
from aibom.scan_pipeline import ScanPipeline
from aibom.scanners.base import scanner_timings
network=[]
real_connect=socket.socket.connect
real_connect_ex=socket.socket.connect_ex
def local(address):
    return isinstance(address,tuple) and address[0] in ('127.0.0.1','::1','localhost')
def guarded_connect(sock,address):
    if local(address): return real_connect(sock,address)
    network.append('network_attempt')
    if a.allow_metadata_network: return real_connect(sock,address)
    raise OSError('External network disabled in deterministic comparison')
def guarded_connect_ex(sock,address):
    if local(address): return real_connect_ex(sock,address)
    network.append('network_attempt')
    if a.allow_metadata_network: return real_connect_ex(sock,address)
    raise OSError('External network disabled in deterministic comparison')
socket.socket.connect=guarded_connect;socket.socket.connect_ex=guarded_connect_ex
manifest=json.loads((corpus/'cases.json').read_text(encoding='utf-8'))
source_files=[{"path":str(f.relative_to(corpus)).replace('\\','/'),"sha256":hashlib.sha256(f.read_bytes()).hexdigest()} for f in sorted((corpus/'cases').rglob('*')) if f.is_file()]
rows=[]
for c in manifest['cases']:
    root=corpus/'cases'/c['id']; log=[];network.clear()
    class Capture(logging.Handler):
        def emit(self,record):log.append({'level':record.levelname,'message':record.getMessage()})
    capture=Capture();logging.getLogger().addHandler(capture)
    started=time.perf_counter();error=None;raw=[];status='ok'
    try:
        result=ScanPipeline([str(root)],kb_path=str(Path(a.kb).resolve()),llm_config=None,include_code_snippets=False).run()
        raw=[item.model_dump(mode='json') for item in result.components]
        assert result.agentic_status=='skipped'
        if any(x['level'] in ['ERROR','CRITICAL'] for x in log) or (network and not a.allow_metadata_network):status='incomplete'
    except Exception as exc:error=str(exc);status='failed'
    finally:logging.getLogger().removeHandler(capture)
    (out/(c['id']+'.json')).write_text(json.dumps({'components':raw,'status':status,'error':error,'logs':log,'network_attempts':len(network),'scanners':dict(scanner_timings)},indent=2),encoding='utf-8')
    count=sum(item['component_type']==c['target_type'] for item in raw)
    rows.append({'case':c['id'],'target_type':c['target_type'],'expected_present':c['expected_present'],'observed_count':count,'status':status,'elapsed_seconds':time.perf_counter()-started})
    (out/'outcomes.json').write_text(json.dumps(rows,indent=2),encoding='utf-8')
    print(c['id'],status,count,flush=True)
print('Completed',len(rows),'cases')

(out/"run.json").write_text(json.dumps({"mode":"ScanPipeline llm_config=None; unmodified scanner logic","metadata_network_allowed":a.allow_metadata_network,"kb_sha256":hashlib.sha256(Path(a.kb).read_bytes()).hexdigest(),"corpus_sha256":hashlib.sha256((corpus/"cases.json").read_bytes()).hexdigest(),"source_files":source_files,"cache_files":[{"path":str(f.relative_to(out.parent)),"sha256":hashlib.sha256(f.read_bytes()).hexdigest()} for f in (out.parent/"cisco-metadata-cache").rglob("*") if f.is_file()]},indent=2),encoding="utf-8")

assert source_files==[{"path":str(f.relative_to(corpus)).replace("\\","/"),"sha256":hashlib.sha256(f.read_bytes()).hexdigest()} for f in sorted((corpus/"cases").rglob("*")) if f.is_file()], "Corpus changed during run"
