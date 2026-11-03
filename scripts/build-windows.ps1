param([switch]$DuckDB)
$ErrorActionPreference = 'Stop'
Push-Location (Join-Path $PSScriptRoot '..')
try {
    $buildArgs = @('scripts/build-release.py', '--target', 'x86_64-pc-windows-msvc')
    if ($DuckDB) { $buildArgs += '--duckdb' }
    & python @buildArgs
    if ($LASTEXITCODE -ne 0) { throw 'Build failed' }
    Write-Host 'Binary: target/x86_64-pc-windows-msvc/release/pilotlight.exe'
} finally {
    Pop-Location
}
