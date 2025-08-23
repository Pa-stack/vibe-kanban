# Fail fast and strict
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Write-Host "[INFO] Starting VS Code cargo fixer..."

# ========== 1) Diagnostics ==========
try {
    Write-Host "[INFO] PowerShell version: $($PSVersionTable.PSVersion)"
    Write-Host "[INFO] Is 64-bit process: $([Environment]::Is64BitProcess)"
    Write-Host "[INFO] Edition: $($PSVersionTable.PSEdition)"
    Write-Host "[INFO] HOME: $HOME"
    $cargoBin = Join-Path $HOME '.cargo\bin'
    Write-Host "[INFO] cargo bin (expected): $cargoBin"

    Write-Host "[INFO] Current session PATH:"
    Write-Host $env:Path

    $gcCargo = Get-Command cargo.exe -ErrorAction SilentlyContinue
    if ($gcCargo) {
        Write-Host "[INFO] Get-Command cargo.exe => $($gcCargo.Path)"
    } else {
        Write-Host "[INFO] Get-Command cargo.exe => <not found>"
    }

    $wcBefore = & where.exe cargo 2>&1
    if ($LASTEXITCODE -eq 0) { Write-Host "[INFO] where.exe cargo =>`n$wcBefore" } else { Write-Host "[INFO] where.exe cargo => <not found>" }
} catch {
    Write-Host "[FAIL] Diagnostics failed: $($_.Exception.Message)"; exit 1
}

# Helpers
function Join-UniquePaths {
    param([string[]]$segments)
    $list = New-Object System.Collections.Generic.List[string]
    $set  = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::OrdinalIgnoreCase)
    foreach ($s in $segments) {
        if (-not $s) { continue }
        $t = $s.Trim().TrimEnd(';')
        if ([string]::IsNullOrWhiteSpace($t)) { continue }
        if ($set.Add($t)) { [void]$list.Add($t) }
    }
    return ($list -join ';')
}

function Test-PathSegment {
    param([string[]]$segments, [string]$probe)
    $p = $probe.Trim().TrimEnd('\',';')
    foreach ($seg in $segments) {
        if (-not $seg) { continue }
        $s = $seg.Trim().TrimEnd('\',';')
        if ($s -ieq $p) { return $true }
    }
    return $false
}

# ========== 2) Ensure rustup/cargo bin on User PATH and session PATH ==========
try {
    if (-not (Test-Path -LiteralPath $cargoBin)) {
        New-Item -ItemType Directory -Path $cargoBin -Force | Out-Null
        Write-Host "[INFO] Ensured directory exists: $cargoBin"
    } else {
        Write-Host "[INFO] Directory exists: $cargoBin"
    }

    # Update User PATH (prepend if missing, de-dupe, do not change System PATH)
    $userPathRaw = [Environment]::GetEnvironmentVariable('Path','User')
    $userSegs = if ([string]::IsNullOrEmpty($userPathRaw)) { @() } else { $userPathRaw -split ';' }
    if (-not (Test-PathSegment -segments $userSegs -probe $cargoBin)) {
        $newUserPath = Join-UniquePaths (@($cargoBin) + $userSegs)
        [Environment]::SetEnvironmentVariable('Path', $newUserPath, 'User')
        Write-Host "[INFO] Updated User PATH (prepended cargo bin)."

        # Best-effort broadcast to update env for new processes
        try {
            Add-Type -Namespace Native -Name Win32 -MemberDefinition @"
using System;
using System.Runtime.InteropServices;
public static class Win32 {
    [DllImport("user32.dll", SetLastError=true, CharSet=CharSet.Auto)]
    public static extern IntPtr SendMessageTimeout(IntPtr hWnd, int Msg, IntPtr wParam, string lParam, int flags, int timeout, out IntPtr lpdwResult);
}
"@ -ErrorAction SilentlyContinue
            $result = [IntPtr]::Zero
            [void][Native.Win32]::SendMessageTimeout([IntPtr]0xffff, 0x1A, [IntPtr]::Zero, 'Environment', 0, 1000, [ref]$result)
            Write-Host "[INFO] Broadcasted environment change (WM_SETTINGCHANGE)."
        } catch { Write-Host "[INFO] Skipped env broadcast: $($_.Exception.Message)" }
    } else {
        Write-Host "[INFO] User PATH already contains cargo bin."
    }

    # Update session PATH (prepend if missing, de-dupe)
    $sessSegs = if ([string]::IsNullOrEmpty($env:Path)) { @() } else { $env:Path -split ';' }
    if (-not (Test-PathSegment -segments $sessSegs -probe $cargoBin)) {
        $env:Path = Join-UniquePaths (@($cargoBin) + $sessSegs)
        Write-Host "[INFO] Updated session PATH (prepended cargo bin)."
    } else {
        $env:Path = Join-UniquePaths $sessSegs  # de-dupe any accidental dupes
        Write-Host "[INFO] Session PATH already contains cargo bin."
    }

    $wcAfter = & where.exe cargo 2>&1
    if ($LASTEXITCODE -eq 0) { Write-Host "[INFO] where.exe cargo (after PATH update) =>`n$wcAfter" } else { Write-Host "[INFO] where.exe cargo (after PATH update) => <not found>" }
} catch {
    Write-Host "[FAIL] PATH update failed: $($_.Exception.Message)"; exit 1
}

# ========== 3) VS Code workspace override ==========
try {
    $cwd = Get-Location
    $vscodeDir = Join-Path $cwd '.vscode'
    $settingsPath = Join-Path $vscodeDir 'settings.json'
    if (-not (Test-Path -LiteralPath $vscodeDir)) {
        New-Item -ItemType Directory -Path $vscodeDir -Force | Out-Null
        Write-Host "[INFO] Created .vscode directory: $vscodeDir"
    }

    $settingsObj = @{}
    if (Test-Path -LiteralPath $settingsPath) {
        try {
            $jsonRaw = Get-Content -LiteralPath $settingsPath -Raw -Encoding UTF8
            $settingsObj = $jsonRaw | ConvertFrom-Json -ErrorAction Stop
        } catch {
            Write-Host "[INFO] Existing settings.json is invalid JSON; starting fresh."
            $settingsObj = @{}
        }
    }

    function Set-TopJsonKey {
        param($obj, [string]$key, $value)
        $prop = $obj.PSObject.Properties[$key]
        if ($prop) { $prop.Value = $value } else { $obj | Add-Member -NotePropertyName $key -NotePropertyValue $value -Force }
    }

    # Build PATH value for VS Code terminals: <CARGO_BIN>;<EXISTING_PATH>, with de-dupe for safety
    $existingPath = $env:Path  # current session PATH captured by the script
    $pathForVSCode = Join-UniquePaths (@($cargoBin) + ($existingPath -split ';'))

    # Ensure inheritEnv = true
    Set-TopJsonKey -obj $settingsObj -key 'terminal.integrated.inheritEnv' -value $true

    # Merge terminal.integrated.env.windows preserving existing keys
    $envWindows = @{}
    $envProp = $settingsObj.PSObject.Properties['terminal.integrated.env.windows']
    if ($envProp -and $envProp.Value) {
        foreach ($p in $envProp.Value.PSObject.Properties) { $envWindows[$p.Name] = $p.Value }
    }
    $envWindows['PATH'] = $pathForVSCode
    Set-TopJsonKey -obj $settingsObj -key 'terminal.integrated.env.windows' -value $envWindows

    $tmp = "$settingsPath.tmp"
    $settingsObj | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $tmp -Encoding UTF8
    Move-Item -LiteralPath $tmp -Destination $settingsPath -Force
    Write-Host "[INFO] Wrote workspace settings: $settingsPath"
} catch {
    Write-Host "[FAIL] Failed to write .vscode/settings.json: $($_.Exception.Message)"; exit 1
}

# ========== 4) Optional MSVC linker injection (non-fatal) ==========
try {
    $whereLink = & where.exe link 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[INFO] 'link' not found; attempting MSVC tools discovery via vswhere..."
        $vswhereCandidates = @()
        if ($env:ProgramFiles) { $vswhereCandidates += (Join-Path $env:ProgramFiles 'Microsoft Visual Studio\Installer\vswhere.exe') }
        if (${env:ProgramFiles(x86)}) { $vswhereCandidates += (Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe') }
        $vswhereCandidates += 'vswhere.exe'
        $vswhereExe = $null
        foreach ($cand in $vswhereCandidates) {
            try {
                $null = & $cand -? 2>$null
                if ($LASTEXITCODE -in 0,1) { $vswhereExe = $cand; break }
            } catch { }
        }
        if ($vswhereExe) {
            $installPath = & $vswhereExe -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
            if ($LASTEXITCODE -eq 0 -and $installPath) {
                $msvcToolsRoot = Join-Path $installPath 'VC\Tools\MSVC'
                if (Test-Path -LiteralPath $msvcToolsRoot) {
                    $latest = Get-ChildItem -LiteralPath $msvcToolsRoot -Directory | Sort-Object Name -Descending | Select-Object -First 1
                    if ($latest) {
                        $msvcBin = Join-Path $latest.FullName 'bin\Hostx64\x64'
                        if (Test-Path -LiteralPath $msvcBin) {
                            $env:Path = Join-UniquePaths (@($msvcBin) + ($env:Path -split ';'))
                            Write-Host "[INFO] injected MSVC bin: $msvcBin"
                        }
                    }
                }
            }
        } else {
            Write-Host "[INFO] vswhere.exe not found; skipping MSVC injection."
        }
    } else {
        Write-Host "[INFO] 'link' is available: $whereLink"
    }
} catch {
    Write-Host "[INFO] MSVC injection skipped due to error: $($_.Exception.Message)"
}

# ========== 5) Verification ==========
try {
    # Find a child PowerShell to run diagnostics in a fresh process
    $childExe = $null
    $pwshFromHome = Join-Path $PSHOME 'pwsh.exe'
    if (Test-Path -LiteralPath $pwshFromHome) { $childExe = $pwshFromHome }
    elseif (Get-Command pwsh -ErrorAction SilentlyContinue) { $childExe = (Get-Command pwsh).Source }
    else { $childExe = (Get-Command powershell -ErrorAction Stop).Source }

    $childCmd = @(
        "$ErrorActionPreference='SilentlyContinue'",
        "if ((Get-Command cargo.exe -ErrorAction SilentlyContinue) -and (Get-Command rustc.exe -ErrorAction SilentlyContinue)) { & cargo -V; & rustc -V; exit 0 }",
        "Write-Output 'cargo or rustc missing'",
        "exit 2"
    ) -join '; '

    $childOut = & $childExe -NoLogo -NoProfile -Command $childCmd 2>&1
    Write-Host "[INFO] Child shell check output:`n$childOut"
    if ($LASTEXITCODE -eq 0) {
        Write-Host "[PASS] cargo and rustc resolved in a fresh child shell."
        exit 0
    } else {
        Write-Host "[FAIL] cargo still not found in a fresh child shell."
        Write-Host "Close and reopen VS Code, then start a NEW integrated terminal. The workspace settings also force PATH to include cargo."
        exit 1
    }
} catch {
    Write-Host "[FAIL] Verification step failed: $($_.Exception.Message)"; exit 1
}
