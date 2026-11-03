"""Query OSV for every public registry package in Cargo.lock; no source or tokens sent.

Requires Python 3.11+ and network access. Service failures fail the check.
"""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import tomllib
import urllib.request


def check(lockfile):
    data = lockfile.read_bytes()
    packages = tomllib.loads(data.decode('utf-8'))['package']
    dependencies = [p for p in packages if p.get('source') == 'registry+https://github.com/rust-lang/crates.io-index']
    unsupported = [p['name'] for p in packages if p.get('source') and p not in dependencies]
    if unsupported:
        raise ValueError('Unreviewed dependency sources: ' + ', '.join(unsupported))
    results = []
    for start in range(0, len(dependencies), 100):
        batch = dependencies[start:start + 100]
        pending = [(p, {'package': {'ecosystem': 'crates.io', 'name': p['name']}, 'version': p['version']}) for p in batch]
        while pending:
            request = urllib.request.Request('https://api.osv.dev/v1/querybatch',
                data=json.dumps({'queries': [q for _, q in pending]}).encode(),
                headers={'Content-Type': 'application/json', 'User-Agent': 'pilotlight-dependency-audit'})
            with urllib.request.urlopen(request, timeout=60) as response:
                result = json.load(response)['results']
            if len(result) != len(pending):
                raise ValueError('Incomplete advisory service response')
            more = []
            for (package, query), item in zip(pending, result):
                if item.get('vulns'):
                    results.append({'package': package['name'], 'version': package['version'],
                                    'advisories': sorted(v['id'] for v in item['vulns'])})
                if item.get('next_page_token'):
                    more.append((package, {**query, 'page_token': item['next_page_token']}))
            pending = more
    return {'checked_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'service': 'https://api.osv.dev/v1/querybatch',
            'lock_sha256': hashlib.sha256(data).hexdigest(),
            'packages_checked': len(dependencies), 'findings': results}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--lockfile', type=Path, default=Path(__file__).resolve().parent.parent / 'Cargo.lock')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    report = check(args.lockfile)
    encoded = json.dumps(report, indent=2) + '\n'
    if args.output:
        args.output.write_text(encoded, encoding='utf-8')
    print(encoded)
    raise SystemExit(1 if report['findings'] else 0)
