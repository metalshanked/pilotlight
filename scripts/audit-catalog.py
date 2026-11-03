"""Check catalog package identities against public registries without installing packages.

Maintainer tool only. Emits a proposal/evidence report; never rewrites the runtime catalog.
No source repository content or credentials are sent to registries.
"""
import argparse
import concurrent.futures
import datetime
import hashlib
import json
from pathlib import Path
import re
import urllib.parse
import urllib.request

HOSTS = {'pypi.org', 'registry.npmjs.org', 'repo.packagist.org'}
LIMIT = 8 * 1024 * 1024

class RegistryRedirects(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, response, code, message, headers, url):
        parsed = urllib.parse.urlsplit(url)
        if parsed.scheme != 'https' or parsed.hostname not in HOSTS or parsed.username or parsed.password:
            raise ValueError('unexpected registry redirect')
        return super().redirect_request(request, response, code, message, headers, url)

def coordinate_url(ecosystem, name):
    encoded = urllib.parse.quote(name, safe='')
    if ecosystem == 'pypi': return f'https://pypi.org/pypi/{encoded}/json'
    if ecosystem == 'npm': return f'https://registry.npmjs.org/{encoded}/latest'
    if ecosystem == 'composer':
        if not re.fullmatch(r'[a-z0-9_.-]+/[a-z0-9_.-]+', name): raise ValueError('invalid Composer coordinate')
        return f'https://repo.packagist.org/p2/{name}.json'
    return None

def normalize(ecosystem, name):
    return re.sub(r'[-_.]+', '-', name).lower() if ecosystem == 'pypi' else name

def interpret(ecosystem, name, payload):
    data = json.loads(payload)
    if ecosystem == 'pypi': data = data['info']
    if ecosystem == 'composer': data = data['packages'][name][0]
    if normalize(ecosystem, data['name']) != normalize(ecosystem, name): raise ValueError('registry identity mismatch')
    return {'registry_name': data['name'], 'observed_version': data['version'],
            'description': data.get('summary') if ecosystem == 'pypi' else data.get('description'),
            'deprecated': data.get('deprecated'), 'response_sha256': hashlib.sha256(payload).hexdigest()}

def audit(row):
    framework, ecosystem, name = row
    record = {'framework': framework, 'ecosystem': ecosystem, 'package': name}
    try:
        url = coordinate_url(ecosystem, name)
        if url is None: return dict(record, status='registry_adapter_unavailable')
        record['source'] = url
        request = urllib.request.Request(url, headers={'User-Agent': 'PilotLight-catalog-audit/0.1'})
        with urllib.request.build_opener(RegistryRedirects()).open(request, timeout=25) as response:
            payload = response.read(LIMIT + 1)
        if len(payload) > LIMIT: raise ValueError('registry response exceeds limit')
        return dict(record, status='identity_verified', **interpret(ecosystem, name, payload))
    except Exception as error:
        return dict(record, status='verification_failed', error=type(error).__name__)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--catalog', type=Path, default=Path(__file__).resolve().parents[1] / 'catalog/catalog.json')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--jobs', type=int, choices=range(1, 9), default=4)
    args = parser.parse_args()
    if args.output.exists(): parser.error('output exists; use a fresh audit filename')
    payload = args.catalog.read_bytes()
    if len(payload) > LIMIT: parser.error('catalog exceeds limit')
    catalog = json.loads(payload)
    rows = sorted((e['id'], eco, name) for e in catalog['entries'] for eco, names in e['packages'].items() for name in names)
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as pool: records = list(pool.map(audit, rows))
    report = {'schema_version': 1, 'checked_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'catalog_file_sha256': hashlib.sha256(payload).hexdigest(), 'records': records,
              'qualification': 'Registry identity checks do not qualify import paths, symbols, version compatibility or detection accuracy.'}
    with args.output.open('x', encoding='utf-8') as stream: json.dump(report, stream, indent=2); stream.write('\n')
    counts = {status: sum(r['status'] == status for r in records) for status in sorted({r['status'] for r in records})}
    print(json.dumps(counts))
    return int(any(r['status'] == 'verification_failed' for r in records))

if __name__ == '__main__': raise SystemExit(main())
