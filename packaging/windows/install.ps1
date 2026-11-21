# SecretPiece Windows installer (no admin rights required).
# Run from the extracted GUI archive: powershell -ExecutionPolicy Bypass -File install.ps1
$ErrorActionPreference = 'Stop'
$source = Split-Path -Parent $MyInvocation.MyCommand.Path
$target = Join-Path $env:LOCALAPPDATA 'Programs\SecretPiece'
New-Item -ItemType Directory -Force -Path $target | Out-Null
Get-ChildItem -Path $source -File | Where-Object { $_.Name -ne 'install.ps1' } |
    Copy-Item -Destination $target -Force

$exe = Join-Path $target 'secretpiece.exe'
if (-not (Test-Path $exe)) { $exe = Join-Path $target 'sp.exe' }
$link = Join-Path ([Environment]::GetFolderPath('Desktop')) 'SecretPiece.lnk'
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($link)
$shortcut.TargetPath = $exe
$shortcut.WorkingDirectory = $target
$shortcut.Description = 'SecretPiece encrypted vault'
$shortcut.Save()
Write-Host "SecretPiece installed to $target"
