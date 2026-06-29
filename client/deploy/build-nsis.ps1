$ErrorActionPreference = 'Stop'

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$clientDir = Resolve-Path (Join-Path $scriptDir '..')
$repoRoot = Resolve-Path (Join-Path $clientDir '..')
$fingerprintPath = Join-Path $repoRoot 'deploy/assets/tls/server.crt.sha256'
$packageLockPath = Join-Path $clientDir 'package-lock.json'
$distDir = Join-Path $clientDir 'dist'

function Assert-Command {
  param([Parameter(Mandatory = $true)][string]$Name)

  if ($null -eq (Get-Command $Name -ErrorAction SilentlyContinue)) {
    throw "Required command '$Name' was not found in PATH."
  }
}

Assert-Command node
Assert-Command npm

if (-not (Test-Path $fingerprintPath)) {
  throw "TLS fingerprint file not found: $fingerprintPath"
}

if (-not (Test-Path $packageLockPath)) {
  throw "package-lock.json not found: $packageLockPath"
}

$fingerprint = (Get-Content -Raw -Path $fingerprintPath).Trim().ToLowerInvariant()
if ($fingerprint -notmatch '^[0-9a-f]{64}$') {
  throw "TLS fingerprint must be 64 lowercase hex characters: $fingerprintPath"
}

Push-Location $clientDir
try {
  $env:USB_CONTROL_CERT_FINGERPRINT = $fingerprint

  npm ci
  if ($LASTEXITCODE -ne 0) {
    throw "npm ci failed with exit code $LASTEXITCODE"
  }

  npm run build
  if ($LASTEXITCODE -ne 0) {
    throw "npm run build failed with exit code $LASTEXITCODE"
  }

  npm run dist
  if ($LASTEXITCODE -ne 0) {
    throw "npm run dist failed with exit code $LASTEXITCODE"
  }

  $installer = Get-ChildItem -Path $distDir -Filter 'USB-Control-Setup-*.exe' |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1

  if ($null -eq $installer) {
    throw "NSIS installer was not generated in $distDir"
  }

  $hash = Get-FileHash -Algorithm SHA256 -Path $installer.FullName
  $hashFile = "$($installer.FullName).sha256"
  "$($hash.Hash.ToLowerInvariant())  $($installer.Name)" | Set-Content -Encoding ascii -Path $hashFile

  Write-Host "Installer: $($installer.FullName)"
  Write-Host "SHA256:    $($hash.Hash.ToLowerInvariant())"
  Write-Host "Checksum:  $hashFile"
}
finally {
  Remove-Item Env:\USB_CONTROL_CERT_FINGERPRINT -ErrorAction SilentlyContinue
  Pop-Location
}
