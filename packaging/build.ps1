$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Packaging = Join-Path $Root "packaging"
$Stage = Join-Path $Packaging "stage"
$Dist = Join-Path $Root "dist"
$CargoToml = Join-Path $Root "Cargo.toml"
$InstallerMode = if ($env:INSTALLER) { $env:INSTALLER.ToLower() } else { "both" }

$versionLine = Get-Content $CargoToml | Where-Object { $_ -match '^\s*version\s*=' } | Select-Object -First 1
$Version = ($versionLine -replace '.*"([^"]+)".*', '$1')

Write-Host "==> cargo build --release"
Push-Location $Root
cargo build --release
Pop-Location

$exeSrc = Join-Path $Root "target\release\suimi-gpui.exe"
if (-not (Test-Path $exeSrc)) { throw "未找到 $exeSrc" }

$portableName = "随密-GPUI-$Version-win64-绿色版"
$portableDir = Join-Path $Dist $portableName
if (Test-Path $Dist) { Remove-Item $Dist -Recurse -Force }
New-Item -ItemType Directory -Path $portableDir -Force | Out-Null

Copy-Item $exeSrc (Join-Path $portableDir "随密-GPUI.exe") -Force
Copy-Item (Join-Path $Stage "启动.bat") $portableDir -Force
Copy-Item (Join-Path $Stage "使用说明.txt") $portableDir -Force

$zipPath = Join-Path $Dist "$portableName.zip"
Push-Location $Dist
tar -a -cf $zipPath (Split-Path $portableDir -Leaf)
Pop-Location
Write-Host "已生成: $zipPath"

$installerStage = Join-Path $Packaging "installer-stage"
if (Test-Path $installerStage) { Remove-Item $installerStage -Recurse -Force }
New-Item -ItemType Directory -Path $installerStage -Force | Out-Null
Copy-Item (Join-Path $portableDir "*") $installerStage -Recurse -Force

function Find-Makensis {
    @(
        "${env:ProgramFiles(x86)}\NSIS\makensis.exe",
        "$env:ProgramFiles\NSIS\makensis.exe"
    ) | Where-Object { Test-Path $_ } | Select-Object -First 1
}

function Find-Iscc {
    @(
        "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
        "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
    ) | Where-Object { Test-Path $_ } | Select-Object -First 1
}

$setupExe = Join-Path $Dist "随密-GPUI-$Version-win64-setup.exe"
$builtInstaller = $false

if ($InstallerMode -in @("nsis", "both")) {
    $makensis = Find-Makensis
    if ($makensis) {
        Write-Host "==> NSIS: $makensis"
        $nsi = Join-Path $Packaging "nsis\setup.nsi"
        $setupAscii = Join-Path $Dist "Suimi-GPUI-$Version-win64-setup.exe"
        & $makensis "/INPUTCHARSET" "UTF8" "/DVERSION=$Version" "/DSTAGE_DIR=$installerStage" "/DOUTFILE=$setupAscii" $nsi
        Move-Item -Force $setupAscii $setupExe
        Write-Host "已生成安装包 (NSIS): $setupExe"
        $builtInstaller = $true
    }
}

if (-not $builtInstaller -and $InstallerMode -in @("inno", "both")) {
    $iscc = Find-Iscc
    if ($iscc) {
        $iss = Join-Path $Packaging "inno\setup.iss"
        & $iscc $iss "/DMyAppVersion=$Version" "/DStageDir=$installerStage" "/DOutputDir=$Dist"
        Write-Host "已生成安装包 (Inno): $setupExe"
        $builtInstaller = $true
    }
}

if (-not $builtInstaller -and $InstallerMode -ne "none") {
    Write-Host "未生成安装包。安装 NSIS 或 Inno 后重试。" -ForegroundColor Yellow
}

Write-Host "完成。输出目录: $Dist"
