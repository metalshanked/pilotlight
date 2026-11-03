"""Build a portable release with developer paths remapped out of Rust diagnostics.

Requires the target Rust toolchain and native C/C++ toolchain already installed.
"""
import argparse
import os
from pathlib import Path
import subprocess
from public_source import audit_binary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target', required=True)
    parser.add_argument('--duckdb', action='store_true')
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    env = dict(os.environ)
    cargo_home = Path(env.get('CARGO_HOME', str(Path.home() / '.cargo'))).resolve()
    # Encoded flags preserve spaces in paths. Do not import unreviewed ambient flags
    # into distributable builds; ordinary cargo build remains available to developers.
    flags = []
    native_flags = []
    for path, public in [(root, '/pilotlight'), (cargo_home, '/cargo')]:
        for spelling in sorted({str(path), path.as_posix()}):
            flags.append(f'--remap-path-prefix={spelling}={public}')
        if args.target.endswith('windows-msvc'):
            native_flags.append(f'/pathmap:{path}={public}')
        else:
            native_flags.append(f'-ffile-prefix-map={path}={public}')
    if args.target.endswith('windows-msvc'):
        flags.extend(['-C', 'target-feature=+crt-static'])
        # MSVC pathmap covers debug paths; trimfile also covers __FILE__ in
        # native assertions. Verify the resulting bytes below rather than
        # assuming every compiler honors these switches.
        try:
            prefix = os.path.commonpath([root, cargo_home])
        except ValueError:
            prefix = str(cargo_home)
        native_flags.append('/d1trimfile:' + prefix.rstrip('\\/') + os.sep)
    env['CARGO_ENCODED_RUSTFLAGS'] = '\x1f'.join(flags)
    # Tree-sitter's native assertion messages also embed __FILE__ paths.
    import shlex
    env['CC_SHELL_ESCAPED_FLAGS'] = '1'
    for name in ('CFLAGS', 'CXXFLAGS'):
        env[f'{name}_{args.target.replace("-", "_")}'] = shlex.join(native_flags)
    command = ['cargo', 'build', '--release', '--locked', '--target', args.target]
    if args.duckdb:
        command.extend(['--features', 'duckdb-catalog'])
    subprocess.run(command, cwd=root, env=env, check=True)
    binary = root / 'target' / args.target / 'release' / ('pilotlight.exe' if args.target.endswith('windows-msvc') else 'pilotlight')
    # Native-library paths can evade Rust remapping; fail closed before packaging.
    data = binary.read_bytes().lower()
    for path in (root, cargo_home, Path.home()):
        for spelling in {str(path), path.as_posix()}:
            for encoding in ('utf-8', 'utf-16-le'):
                if spelling.lower().encode(encoding) in data:
                    raise SystemExit('Release binary still contains a local build path; do not distribute it.')
    audit_binary(binary)
    print('Release build and local-path check passed.')


if __name__ == '__main__':
    main()
