"""Record upstream's test-path policy for benchmark context, not accuracy labels."""
import argparse
import json
import sys
from pathlib import Path

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--work', type=Path, required=True)
p.add_argument('--cisco', type=Path, required=True)
p.add_argument('--commit', required=True)
a = p.parse_args()
sys.path.insert(0, str(a.cisco.resolve() / 'aibom/src'))
from aibom.scan_pipeline import _is_test_file

out = a.work / 'cisco-test-paths.json'
if out.exists():
    raise SystemExit('Output already exists')
paths = {}
for directory in sorted((a.work / 'sources').iterdir()):
    files = json.loads((directory / 'source-files.json').read_text(encoding='utf-8'))
    paths[directory.name] = [r['path'] for r in files if _is_test_file(str(directory / 'source' / r['path']))]
out.write_text(json.dumps({'cisco_commit': a.commit, 'policy': 'aibom.scan_pipeline._is_test_file',
                           'qualification': 'Path context only; consolidated components may retain test evidence alongside other occurrences.',
                           'repositories': paths}, indent=2), encoding='utf-8')
print('Classified test paths for', len(paths), 'repositories')
