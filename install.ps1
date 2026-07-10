#Requires -Version 5.1

$ErrorActionPreference = "Stop"

$Repo = "tsegaye27/bible-cli"
$BinaryName = "bible-cli"
$InstallDir = Join-Path $env:LOCALAPPDATA "bible-cli"

Write-Host "----------------------------------------"
Write-Host "  Installing Bible CLI (Amharic)        "
Write-Host "----------------------------------------"

$Arch = $env:PROCESSOR_ARCHITECTURE
switch ($Arch) {
    "AMD64" { $ArchSuffix = "x86_64" }
    "ARM64" { $ArchSuffix = "aarch64" }
    default {
        Write-Error "Unsupported architecture: $Arch"
        exit 1
    }
}

$DownloadUrl = "https://github.com/$Repo/releases/latest/download/${BinaryName}-windows-${ArchSuffix}.exe"
$InstallPath = Join-Path $InstallDir "$BinaryName.exe"

Write-Host "Checking for latest release for windows-$ArchSuffix..."
Write-Host "Downloading from GitHub: $DownloadUrl"

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $InstallPath -UseBasicParsing
    Write-Host "Download successful."
}
catch {
    Write-Error "Download failed. Check your internet or GitHub URL."
    exit 1
}

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    $NewPath = if ($UserPath) { "$UserPath;$InstallDir" } else { $InstallDir }
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    $env:Path = "$env:Path;$InstallDir"
    Write-Host "Added $InstallDir to your user PATH."
}

Write-Host "----------------------------------------"
Write-Host "Successfully installed Bible CLI!"
Write-Host "You can now run it by typing: $BinaryName"
Write-Host "----------------------------------------"
Write-Host "Font Recommendations for Amharic:"
Write-Host "----------------------------------------"
Write-Host "1. Abyssinica SIL (Highly Recommended)"
Write-Host "2. Noto Sans Ethiopic"
Write-Host "3. Kawsar"
Write-Host "4. FiraGO"
Write-Host ""
Write-Host "Set your terminal font to one of these"
Write-Host "to see Amharic characters correctly."
Write-Host "----------------------------------------"
Write-Host ""
Write-Host "Note: Open a new terminal window if $BinaryName is not found."
