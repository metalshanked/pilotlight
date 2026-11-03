"""Generate/check dependency notices from locked Cargo metadata (Python 3.11+).

Generation uses cached crate license texts; when absent it fetches license files
from the public GitHub commit recorded in the published crate's VCS metadata.
Review the generated texts and manifest before committing them.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tomllib
import urllib.error
import urllib.request

LICENSE_NAME = re.compile(r'(?i)(?:^|[._-])(?:licen[sc]e|copying|copyright|notice)(?:$|[._-])')


def generate(root):
    process = subprocess.run(['cargo', 'metadata', '--locked', '--all-features', '--format-version', '1'],
                             cwd=root, check=True, capture_output=True, encoding='utf-8')
    metadata = json.loads(process.stdout)
    entries = []
    for pkg in sorted(metadata['packages'], key=lambda x: (x['name'], x['version'])):
        if pkg['source'] is None:
            continue
        if pkg['source'] != 'registry+https://github.com/rust-lang/crates.io-index':
            raise ValueError('Only public crates.io dependencies are supported')
        crate = Path(pkg['manifest_path']).parent
        texts = [(p.relative_to(crate).as_posix(), p.read_bytes(), None) for p in sorted(crate.rglob('*'))
                 if p.is_file() and (LICENSE_NAME.search(p.name) or p.name == 'AUTHORS' or
                    any(part.lower() in {'licenses', 'licences'} for part in p.relative_to(crate).parts[:-1]))]
        if not texts:
            vcs = json.loads((crate / '.cargo_vcs_info.json').read_text(encoding='utf-8'))
            sha = vcs['git']['sha1']
            if not re.fullmatch('[a-f0-9]{40}', sha):
                raise ValueError('Invalid upstream commit')
            repo = pkg['repository'].removesuffix('.git').removeprefix('https://github.com/').rstrip('/')
            if not re.fullmatch(r'[\w.-]+/[\w.-]+', repo):
                raise ValueError('Manual license retrieval required for ' + pkg['name'])
            # OR licenses: retain one valid offered option when upstream ships several.
            for name in ['LICENSE', 'LICENSE-MIT', 'LICENSE-MIT.txt', 'LICENSE.txt', 'LICENSE.md',
                         'LICENSES/MIT.txt', 'LICENSE-MIT-0', 'COPYING', 'LICENSE-APACHE']:
                url = f'https://raw.githubusercontent.com/{repo}/{sha}/{name}'
                try:
                    with urllib.request.urlopen(url, timeout=30) as response:
                        body = response.read(1024 * 1024 + 1)
                    if len(body) > 1024 * 1024:
                        raise ValueError('Oversized upstream license')
                    texts.append((name, body, url))
                    break
                except urllib.error.HTTPError as error:
                    if error.code != 404:
                        raise
            if not texts:
                raise ValueError('License text missing for ' + pkg['name'])
        files = []
        for name, data, url in texts:
            data.decode('utf-8')  # Reject binary license candidates.
            target = root / 'licenses/dependencies' / (pkg['name'] + '-' + pkg['version']) / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
            files.append({'path': target.relative_to(root).as_posix(), 'sha256': hashlib.sha256(data).hexdigest(),
                          'source': url or f"https://crates.io/crates/{pkg['name']}/{pkg['version']}"})
        entries.append({'name': pkg['name'], 'version': pkg['version'], 'license_expression': pkg['license'],
                        'repository': pkg['repository'], 'files': files})
    document = {'scope': 'All locked Cargo dependencies, including optional features and platform alternatives. '
                         'Bundled native subdependencies may impose additional notices for optional builds.',
                'packages': entries}
    (root / 'licenses/dependencies.json').write_text(json.dumps(document, indent=2) + '\n', encoding='utf-8')
    check(root)


def check(root):
    lock = tomllib.loads((root / 'Cargo.lock').read_text(encoding='utf-8'))
    expected = {(p['name'], p['version']) for p in lock['package'] if p.get('source')}
    doc = json.loads((root / 'licenses/dependencies.json').read_text(encoding='utf-8'))
    actual = {(p['name'], p['version']) for p in doc['packages']}
    if actual != expected or len(actual) != len(doc['packages']):
        raise ValueError('Dependency notices do not match Cargo.lock; regenerate and review')
    count = 0
    for package in doc['packages']:
        if not package['license_expression'] or not package['files']:
            raise ValueError('Missing license metadata')
        for entry in package['files']:
            path = (root / entry['path']).resolve()
            if not path.is_relative_to((root / 'licenses/dependencies').resolve()):
                raise ValueError('Invalid license path')
            if hashlib.sha256(path.read_bytes()).hexdigest() != entry['sha256']:
                raise ValueError('License file checksum mismatch: ' + entry['path'])
            count += 1
    print(f'Checked {len(actual)} dependency notices and {count} license texts.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--generate', action='store_true', help='Refresh from Cargo metadata; may download missing public licenses')
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    generate(root) if args.generate else check(root)
