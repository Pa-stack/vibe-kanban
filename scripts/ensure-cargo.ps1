#requires -Version 5.1

# ensure-cargo.ps1 — Idempotent Rust (rustup/cargo) installer and validator for Windows
# - Fails fast (non-zero on error) and requires zero manual clicks (winget silent install)
# - Prepends $HOME\.cargo\bin to session and User PATH safely (case-insensitive de-dupe)
# - Installs and sets stable-x86_64-pc-windows-msvc toolchain via rustup
# - Prints cargo/rustc versions and rustup show; optional smoke test compiles and runs "ok"
# Usage: .\scripts\ensure-cargo.ps1 [-SkipSmokeTest]

param(
    [switch]$SkipSmokeTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message"
}

function Write-Warn2 {
    param([string]$Message)
    Write-Warning "[WARN] $Message"
}

function Fail {
    param([string]$Message, [int]$Code = 1)
    Write-Error "[FAIL] $Message"
    exit $Code
}

function Join-PathSafe {
    param([string]$a, [string]$b)
    return [System.IO.Path]::Combine($a, $b)
}

function Get-HomePath {
    if ($env:HOME -and $env:HOME.Trim()) { return $env:HOME }
    elseif ($env:USERPROFILE -and $env:USERPROFILE.Trim()) { return $env:USERPROFILE }
    else { Fail 'Unable to determine home directory (HOME/USERPROFILE not set).' }
}

function Convert-PathList {
    param([string[]]$Items)
    $clean = @()
    foreach ($p in $Items) {
        if (-not $p) { continue }
        $t = $p.Trim()
        if (-not $t) { continue }
        if ($clean -notcontains $t) { $clean += $t }
    }
    # Case-insensitive de-dupe while preserving first occurrence
    $seen = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::OrdinalIgnoreCase)
    $result = New-Object System.Collections.Generic.List[string]
    foreach ($t in $clean) {
        if ($seen.Add($t)) { $result.Add($t) }
    }
    return $result
}

function Set-SessionPathPrepend {
    param([string]$Dir)
    $parts = @(($env:Path -split ';'))
    $parts = Convert-PathList -Items $parts
    # Remove existing matches (case-insensitive)
    $parts = @($parts | Where-Object { $_ -and (-not [string]::Equals($_, $Dir, [StringComparison]::OrdinalIgnoreCase)) })
    $env:Path = ($Dir + ';' + ($parts -join ';')).TrimEnd(';')
    Write-Info "Session PATH updated; ensured '$Dir' is first."
}

function Set-UserPathPrepend {
    param([string]$Dir)
    $userPathRaw = [Environment]::GetEnvironmentVariable('Path', 'User')
    $userParts = @()
    if ($userPathRaw) { $userParts = $userPathRaw -split ';' }
    $userParts = Convert-PathList -Items $userParts
    $userParts = @($userParts | Where-Object { $_ -and (-not [string]::Equals($_, $Dir, [StringComparison]::OrdinalIgnoreCase)) })
    $new = ($Dir + ';' + ($userParts -join ';')).TrimEnd(';')
    [Environment]::SetEnvironmentVariable('Path', $new, 'User')
    Write-Info "User PATH persisted; ensured '$Dir' is first (System PATH untouched)."
}

function Exec {
    param(
        [Parameter(Mandatory=$true)][string]$FilePath,
        [string[]]$CmdArgs,
        [switch]$AllowFail
    )
    Write-Info ("Executing: {0} {1}" -f $FilePath, ($CmdArgs -join ' '))
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $FilePath
    if ($CmdArgs) { $psi.Arguments = ($CmdArgs -join ' ') }
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    $p = New-Object System.Diagnostics.Process
    $p.StartInfo = $psi
    [void]$p.Start()
    $stdout = $p.StandardOutput.ReadToEnd()
    $stderr = $p.StandardError.ReadToEnd()
    $p.WaitForExit()
    if (-not $AllowFail -and $p.ExitCode -ne 0) {
        Write-Host $stdout
        Write-Host $stderr
        Fail ("Command failed with exit code {0}: {1} {2}" -f $p.ExitCode, $FilePath, ($CmdArgs -join ' '))
    }
    return [PSCustomObject]@{ Code=$p.ExitCode; StdOut=$stdout; StdErr=$stderr }
}

# 1) Preflight
Write-Info "Preflight: environment details"
$os = [System.Environment]::OSVersion.VersionString
$is64 = [System.Environment]::Is64BitOperatingSystem
Write-Host ("OS: {0} (64-bit={1})" -f $os, $is64)
Write-Host ("PowerShell: {0}" -f $PSVersionTable.PSVersion)
Write-Host "PATH entries:"; ($env:Path -split ';' | ForEach-Object { '  - ' + $_ })

$winget = (Get-Command winget.exe -ErrorAction SilentlyContinue)?.Source
if (-not $winget) { Fail 'winget.exe not found. Please install App Installer from Microsoft Store to get winget.' }
Write-Info "winget found at: $winget"

$userHome = Get-HomePath
$cargoBin = Join-PathSafe $userHome '.cargo\bin'
$rustupExe = Join-PathSafe $cargoBin 'rustup.exe'

# 2) Install rustup (if needed)
if (-not (Test-Path -Path $rustupExe -PathType Leaf)) {
    Write-Info 'rustup not found; installing via winget (silent, no prompts)...'
    $wingetArgs = @('install','-e','--id','Rustlang.Rustup','--accept-package-agreements','--accept-source-agreements','--silent')
    # winget sometimes returns non-zero if already installed; rely on post-check too
    $res = Exec -FilePath $winget -CmdArgs $wingetArgs -AllowFail
    if ($res.Code -ne 0) { Write-Warn2 ("winget returned {0}, continuing to verify install" -f $res.Code) }
    Start-Sleep -Seconds 2
}

if (-not (Test-Path -Path $rustupExe -PathType Leaf)) {
    Fail ("rustup was not found after installation attempt at: {0}" -f $rustupExe)
}
Write-Info "rustup located: $rustupExe"

# 3) PATH (session + persistent)
Set-SessionPathPrepend -Dir $cargoBin
Set-UserPathPrepend -Dir $cargoBin

# 4) Toolchain sanity
$toolchain = 'stable-x86_64-pc-windows-msvc'
Exec -FilePath $rustupExe -CmdArgs @('toolchain','install',$toolchain)
Exec -FilePath $rustupExe -CmdArgs @('default',$toolchain)

$cargoCmd = Get-Command cargo.exe -ErrorAction SilentlyContinue
$rustcCmd = Get-Command rustc.exe -ErrorAction SilentlyContinue
if (-not $cargoCmd -or -not $rustcCmd) {
    Fail 'cargo/rustc not found on PATH even after update. Close and reopen your terminal, then re-run this script.'
}

Write-Info 'Printing tool versions:'
& $cargoCmd.Source -V | Write-Host
& $rustcCmd.Source -V | Write-Host
$rustupShow = Exec -FilePath $rustupExe -CmdArgs @('show')
if ($rustupShow.StdOut) { $rustupShow.StdOut.TrimEnd() | Write-Host }
if ($rustupShow.StdErr) { $rustupShow.StdErr.TrimEnd() | Write-Host }

# 5) Optional smoke test
if (-not $SkipSmokeTest) {
    Write-Info 'Running quick smoke test (cargo run should print "ok")'
    $tmpRoot = [System.IO.Path]::GetTempPath()
    $tmpDir = Join-PathSafe $tmpRoot ("ensure-cargo-" + [Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null
    Push-Location $tmpDir
    try {
        # Create a minimal cargo package deterministically
        $srcDir = Join-PathSafe $tmpDir 'src'
        if (-not (Test-Path $srcDir)) { New-Item -ItemType Directory -Path $srcDir | Out-Null }
        $cargoToml = Join-PathSafe $tmpDir 'Cargo.toml'
        @(
            '[package]',
            'name = "ensure_cargo_smoke"',
            'version = "0.0.0"',
            'edition = "2021"',
            '',
            '[dependencies]'
        ) | Set-Content -Path $cargoToml -Encoding UTF8

        $mainRs = Join-PathSafe $srcDir 'main.rs'
        @(
            'fn main() {',
            '    println!("ok");',
            '}'
        ) | Set-Content -Path $mainRs -Encoding UTF8

        $runOut = & $cargoCmd.Source run -q 2>&1
        $code = $LASTEXITCODE
        if ($code -ne 0 -or -not ($runOut -join "`n") -match "\bok\b") {
            Write-Warn2 'Smoke test failed. Last 100 log lines:'
            ($runOut | Select-Object -Last 100) | ForEach-Object { Write-Host $_ }
            Fail 'Rust toolchain is installed but cargo run failed.'
        }
        Write-Info 'Smoke test passed.'
    }
    finally {
        Pop-Location
        try { Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue } catch { }
    }
} else {
    Write-Info 'Skipping smoke test as requested.'
}

# 6) Summary
Write-Info 'Summary:'
Write-Host 'where cargo:'
try { & where.exe cargo | ForEach-Object { '  ' + $_ } } catch { Write-Warn2 'where.exe cargo failed' }
Write-Host 'where rustc:'
try { & where.exe rustc | ForEach-Object { '  ' + $_ } } catch { Write-Warn2 'where.exe rustc failed' }

Write-Host ('Effective cargo path: {0}' -f $cargoCmd.Source)
Write-Info 'All checks completed successfully.'
exit 0
