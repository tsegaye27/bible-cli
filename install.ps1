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

# Hibur Mono: a monospaced Ethiopic font (SIL Open Font License 1.1), so
# Amharic letters fit the terminal grid instead of overlapping.
# Set BIBLE_CLI_NO_FONT=1 to skip.
$FontInstalled = $false
if ($env:BIBLE_CLI_NO_FONT -ne "1") {
    try {
        $FontDir = Join-Path $env:LOCALAPPDATA "Microsoft\Windows\Fonts"
        New-Item -ItemType Directory -Force -Path $FontDir | Out-Null
        $FontPath = Join-Path $FontDir "HiburMono-Regular.ttf"
        Write-Host "Downloading the Hibur Mono font for Amharic..."
        Invoke-WebRequest -Uri "https://raw.githubusercontent.com/google/fonts/main/ofl/hiburmono/HiburMono-Regular.ttf" -OutFile $FontPath -UseBasicParsing
        # Per-user registration: no admin rights needed (Windows 10 1809+).
        New-ItemProperty -Force -Path "HKCU:\Software\Microsoft\Windows NT\CurrentVersion\Fonts" -Name "Hibur Mono Regular (TrueType)" -Value $FontPath | Out-Null
        $FontInstalled = $true
    }
    catch {
        Write-Warning "Could not install the font. Bible CLI still works; see the font notes below."
    }
}

Write-Host "----------------------------------------"
Write-Host "Successfully installed Bible CLI!"
Write-Host "You can now run it by typing: $BinaryName"
Write-Host "----------------------------------------"
if ($FontInstalled) {
    Write-Host "Installed the Hibur Mono font."
    Write-Host "In Windows Terminal: Settings > Profiles > Appearance > Font face,"
    Write-Host "choose 'Hibur Mono' so Amharic letters line up without overlapping."
} else {
    Write-Host "For Amharic, set your terminal font to one that covers"
    Write-Host "Ethiopic, ideally the monospaced Hibur Mono:"
    Write-Host "  https://fonts.google.com/specimen/Hibur+Mono"
    Write-Host "Other options: Noto Sans Ethiopic, Abyssinica SIL."
}
Write-Host "----------------------------------------"
Write-Host ""
Write-Host "Note: Open a new terminal window if $BinaryName is not found."
