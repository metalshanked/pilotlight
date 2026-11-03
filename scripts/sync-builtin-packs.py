"""Synchronize first-party pack definitions into complete catalog snapshots.
Run explicitly after editing packs. Does not download or execute target code.
"""
from pathlib import Path
import json
root=Path(__file__).resolve().parents[1]
entries={}
for name in ['agents','mcp']:
    pack=json.loads((root/'packs'/f'{name}.json').read_text(encoding='utf-8'))
    for rule in pack['rules']:
        entry=rule['entry'];assert entry['id'] not in entries
        assert not entry.get('api_references'),'Seed references belong to the runtime snapshot'
        entries[entry['id']]=entry
for name in ['overlay','catalog']:
    path=root/'catalog'/f'{name}.json';doc=json.loads(path.read_text(encoding='utf-8'))
    missing=set(entries)-{e['id'] for e in doc['entries']}
    for i,old in enumerate(doc['entries']):
        if old['id'] in entries:
            updated=dict(entries[old['id']])
            if old.get('api_references'):updated['api_references']=old['api_references']
            doc['entries'][i]=updated
    doc['entries'].extend(entries[k] for k in sorted(missing))
    path.write_text(json.dumps(doc,indent=2)+'\n',encoding='utf-8')
    print(name,len(doc['entries']),'entries')
