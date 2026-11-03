"""Compare case/type presence, retaining all outputs; not an accuracy benchmark."""
import argparse, csv, hashlib, json, subprocess, time
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--pilotlight',required=True);p.add_argument('--corpus',required=True);p.add_argument('--cisco-results',required=True);p.add_argument('--output',required=True)
a=p.parse_args();corpus=Path(a.corpus).resolve();out=Path(a.output).resolve();out.mkdir(parents=True,exist_ok=False);exe=Path(a.pilotlight).resolve()
cisco=Path(a.cisco_results);cr=json.loads((cisco/'run.json').read_text(encoding='utf-8'));old={r['case']:r for r in json.loads((cisco/'outcomes.json').read_text(encoding='utf-8'))}
manifest=json.loads((corpus/'cases.json').read_text(encoding='utf-8'))
sources=[{'path':f.relative_to(corpus).as_posix(),'sha256':hashlib.sha256(f.read_bytes()).hexdigest()} for f in sorted((corpus/'cases').rglob('*')) if f.is_file()]
assert sources==cr['source_files'],'Source identity mismatch'
assert hashlib.sha256((corpus/'cases.json').read_bytes()).hexdigest()==cr['corpus_sha256']
rows=[]
for c in manifest['cases']:
    root=corpus/'cases'/c['id'];target=out/(c['id']+'.json');start=time.perf_counter()
    proc=subprocess.run([str(exe),'scan',str(root),'--format','json','--output',str(target)],capture_output=True,text=True,encoding='utf-8',timeout=90)
    if proc.returncode!=0: raise RuntimeError(proc.stderr)
    report=json.loads(target.read_text(encoding='utf-8'))
    count=sum(x['kind']==c['target_type'] for x in report['assets']);prior=old[c['id']]
    assert prior['status']=='ok',prior
    rows.append({'case':c['id'],'language':c['language'],'target_type':c['target_type'],'expected_present':c['expected_present'],'pilotlight_count':count,'cisco_count':prior['observed_count'],'pilotlight_matches_expectation':bool(count)==c['expected_present'],'cisco_matches_expectation':bool(prior['observed_count'])==c['expected_present'],'pilotlight_status':report['status'],'pilotlight_diagnostics':json.dumps(report['coverage']['diagnostics']),'pilotlight_seconds':time.perf_counter()-start,'cisco_seconds':prior['elapsed_seconds']})
    print(c['id'],count,prior['observed_count'],flush=True)
with (out/'comparison.csv').open('w',encoding='utf-8',newline='') as f:
    writer=csv.DictWriter(f,fieldnames=list(rows[0]));writer.writeheader();writer.writerows(rows)
summary={'cases':len(rows),'positive_probes':sum(r['expected_present'] for r in rows),'negative_probes':sum(not r['expected_present'] for r in rows),'pilotlight_positive_hits':sum(r['expected_present'] and r['pilotlight_count']>0 for r in rows),'cisco_positive_hits':sum(r['expected_present'] and r['cisco_count']>0 for r in rows),'pilotlight_negative_emissions':sum(not r['expected_present'] and r['pilotlight_count']>0 for r in rows),'cisco_negative_emissions':sum(not r['expected_present'] and r['cisco_count']>0 for r in rows),'qualification':'authored_development_case_type_presence; no independent labels or end-to-end LLM comparison','binary_sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'corpus_sha256':cr['corpus_sha256'],'source_files':sources,'rows':rows}
(out/'comparison.json').write_text(json.dumps(summary,indent=2),encoding='utf-8')
print(json.dumps({k:v for k,v in summary.items() if k not in ['rows','source_files']},indent=2))
