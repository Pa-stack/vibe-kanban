# Adds $HOME/.cargo/bin to PATH for current session and persists to User PATH.
$ErrorActionPreference = 'Stop'
$CargoBin = Join-Path $HOME ".cargo\bin"

if (-not (Test-Path -LiteralPath $CargoBin)) {
    Write-Error "Cargo bin not found at $CargoBin. Ensure rustup installed."
    exit 1
}

# Add to current session PATH if missing
$pathParts = $env:Path -split ';'
if (-not ($pathParts -contains $CargoBin)) {
    $env:Path = "$CargoBin;$env:Path"
    [Environment]::SetEnvironmentVariable('Path', $env:Path, 'Process') | Out-Null
    Write-Host "Prepended $CargoBin to current session PATH."
} else {
    Write-Host "Cargo bin already present in current session PATH."
}

# Persist to User PATH if missing
$userPath = [Environment]::GetEnvironmentVariable('Path','User')
if ($null -eq $userPath) { $userPath = '' }
$userParts = $userPath -split ';'
if (-not ($userParts -contains $CargoBin)) {
    $newUserPath = if ([string]::IsNullOrEmpty($userPath)) { $CargoBin } else { "$CargoBin;$userPath" }
    [Environment]::SetEnvironmentVariable('Path', $newUserPath, 'User')
    Write-Host "Added $CargoBin to User PATH. Restart terminals to apply globally."
} else {
    Write-Host "Cargo bin already present in User PATH."
}

# Print versions via absolute path (works regardless of PATH)
& (Join-Path $CargoBin 'cargo.exe') --version
& (Join-Path $CargoBin 'rustc.exe') --version
