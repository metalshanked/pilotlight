"""Maintainer-only packaging; the scanner itself requires no Python runtime."""
import argparse
import hashlib
from pathlib import Path
import tarfile
import tomllib
import zipfile
from public_source import audit, audit_binary

TARGETS = {
    'x86_64-unknown-linux-musl', 'aarch64-unknown-linux-musl',
    'x86_64-apple-darwin', 'aarch64-apple-darwin', 'x86_64-pc-windows-msvc',
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target', required=True, choices=sorted(TARGETS))
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    if not args.binary.is_file():
        parser.error('binary does not exist; build and smoke-test it first')
    audit_binary(args.binary)
    root = Path(__file__).resolve().parent.parent
    public_files, findings = audit(root)
    if findings:
        parser.error('publication audit failed; run python scripts/public_source.py')
    version = tomllib.loads((root / 'Cargo.toml').read_text(encoding='utf-8'))['package']['version']
    windows = args.target.endswith('windows-msvc')
    prefix = f'pilotlight-{version}-{args.target}'
    args.output.mkdir(parents=True, exist_ok=True)
    archive = args.output / (prefix + ('.zip' if windows else '.tar.gz'))
    files = [(args.binary, 'pilotlight.exe' if windows else 'pilotlight')]
    # Include the audited sources and notices alongside the standalone executable.
    # Documentation links and maintainer reproduction commands remain usable.
    files.extend((path, path.relative_to(root).as_posix()) for path in public_files)
    if windows:
        with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED) as bundle:
            for path, name in files:
                bundle.write(path, f'{prefix}/{name}')
    else:
        # Exclusive creation prevents silently replacing a previously produced artifact.
        with archive.open('xb') as stream, tarfile.open(fileobj=stream, mode='w:gz') as bundle:
            for path, name in files:
                info = bundle.gettarinfo(str(path), f'{prefix}/{name}')
                info.uid = info.gid = 0
                info.uname = info.gname = ''
                info.mode = 0o755 if name == 'pilotlight' else 0o644
                with path.open('rb') as source:
                    bundle.addfile(info, source)
    checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_name(archive.name + '.sha256').write_text(f'{checksum}  {archive.name}\n')
    print(archive)


if __name__ == '__main__':
    main()
