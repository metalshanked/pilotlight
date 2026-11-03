"""Allowlisted source packaging and local privacy checks (Python 3.11+, no network).

This is a publication guard, not a replacement for a dedicated secret scanner.
Diagnostics contain locations and rule names, never matched values.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import zipfile

ROOT_FILES = {
    '.gitattributes', '.gitignore', 'Cargo.toml', 'Cargo.lock', 'build.rs',
    'README.md', 'LICENSE', 'CONTRIBUTING.md', 'SECURITY.md',
    'THIRD_PARTY_NOTICES.md',
}
FOLDERS = {'.github', 'src', 'tests', 'catalog', 'packs', 'examples', 'docs',
           'benchmarks', 'scripts', 'licenses'}
EXCLUDED_DIRS = {'__pycache__', '.git', '.venv', 'node_modules', 'target',
                 '.idea', '.vscode', '.codex', '.agents', '.codebase-memory'}
EXCLUDED_SUFFIXES = {'.duckdb', '.wal', '.pyc', '.pyo'}
FORBIDDEN_SUFFIXES = {'.exe', '.dll', '.pdb', '.pem', '.key', '.p12', '.pfx',
                      '.keystore', '.zip', '.gz', '.7z'}
PATTERNS = {
    'developer-home-path': re.compile(r'(?i)(?:[a-z]:[\\/](?:users|documents and settings)[\\/]|/(?:home|Users)/)[^\s"\'<>]+'),
    'credential-url': re.compile(r'https?://[^\s/"\'<>]+:[^\s/"\'<>]+@'),
    'github-token': re.compile(r'\b(?:gh[pousr]_[A-Za-z0-9]{36,255}|github_pat_[A-Za-z0-9_]{40,255})\b'),
    'aws-access-key': re.compile(r'\b(?:AKIA|ASIA)[A-Z0-9]{16}\b'),
    'private-key-body': re.compile(r'-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----\s+[A-Za-z0-9+/=]{32,}'),
}


def audit_binary(path):
    # Check ASCII and UTF-16 strings; never emit the matched path itself.
    data = path.read_bytes().lower().replace(b'\x00', b'')
    if re.search(rb'(?:[a-z]:[\\/]users[\\/]|/(?:home|users)/)[^\s/\\]+', data):
        raise ValueError('binary contains a developer home path; use the release build wrapper')


def source_files(root):
    result = []
    for name in sorted(ROOT_FILES | FOLDERS):
        base = root / name
        if base.is_symlink():
            raise ValueError(f'symlink in publication scope: {name}')
        if name in ROOT_FILES:
            if not base.is_file():
                raise ValueError(f'required source file missing: {name}')
            result.append(base)
            continue
        if not base.is_dir():
            raise ValueError(f'required source directory missing: {name}')
        # walk without following symlinks; reject them rather than dereference.
        import os
        for parent, dirs, names in os.walk(base, followlinks=False):
            for child in dirs + names:
                path = Path(parent) / child
                if path.is_symlink():
                    raise ValueError(f'symlink in publication scope: {path.relative_to(root)}')
            dirs[:] = [d for d in dirs if d not in EXCLUDED_DIRS]
            result.extend(Path(parent) / n for n in names
                          if Path(n).suffix.lower() not in EXCLUDED_SUFFIXES)
    return sorted(result)


def audit(root, check_git=False):
    files = source_files(root)
    findings = []
    for path in files:
        relative = path.relative_to(root).as_posix()
        name = path.name.lower()
        if (path.suffix.lower() in FORBIDDEN_SUFFIXES or
                (name.startswith('.env') and name != '.env.example') or
                name in {'credentials.json', '.netrc', '.npmrc', '.pypirc', 'id_rsa', 'id_ed25519'} or
                name.startswith('secrets.')):
            findings.append({'path': relative, 'rule': 'private-or-binary-file'})
            continue
        data = path.read_bytes()
        if len(data) > 8 * 1024 * 1024:
            findings.append({'path': relative, 'rule': 'oversized-source-file'})
            continue
        try:
            text = data.decode('utf-8')
        except UnicodeDecodeError:
            findings.append({'path': relative, 'rule': 'non-utf8-source'})
            continue
        for rule, pattern in PATTERNS.items():
            for match in pattern.finditer(text):
                findings.append({'path': relative, 'line': text.count('\n', 0, match.start()) + 1, 'rule': rule})
    if check_git:
        # A tracked ignored file still appears here; ignores cannot hide staged secrets.
        result = subprocess.run(['git', '-C', str(root), 'ls-files', '-z', '--cached',
                                 '--others', '--exclude-standard'], check=True, capture_output=True)
        candidates = {p.decode('utf-8') for p in result.stdout.split(b'\0') if p}
        allowed = {p.relative_to(root).as_posix() for p in files}
        findings.extend({'path': p, 'rule': 'outside-publication-scope'} for p in sorted(candidates - allowed))
    return files, findings


def package(root, output):
    files, findings = audit(root)
    if findings:
        raise ValueError(json.dumps(findings))
    manifest = {p.relative_to(root).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in files}
    output.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(output, 'x', zipfile.ZIP_DEFLATED) as bundle:
        for path in files:
            info = zipfile.ZipInfo('pilotlight/' + path.relative_to(root).as_posix(), (2026, 1, 1, 0, 0, 0))
            info.create_system = 3
            info.external_attr = 0o100644 << 16
            bundle.writestr(info, path.read_bytes(), compress_type=zipfile.ZIP_DEFLATED)
    output.with_suffix('.manifest.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    output.with_suffix(output.suffix + '.sha256').write_text(hashlib.sha256(output.read_bytes()).hexdigest() + '  ' + output.name + '\n', encoding='utf-8')
    return len(files)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument('--check-git', action='store_true')
    parser.add_argument('--output', type=Path, help='Create a new source ZIP after successful checks')
    args = parser.parse_args()
    root = args.root.resolve()
    files, findings = audit(root, args.check_git)
    print(json.dumps({'files_checked': len(files), 'findings': findings}, indent=2))
    if findings:
        raise SystemExit(1)
    if args.output:
        package(root, args.output)


if __name__ == '__main__':
    main()
