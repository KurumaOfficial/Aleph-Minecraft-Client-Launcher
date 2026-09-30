<#
.SYNOPSIS
    Fetches the latest successful cloud build and refreshes the Desktop shortcut.
.DESCRIPTION
    1. Finds the newest successful CI run on main via the public GitHub API
       (the token is reused from the local git credential helper, never typed).
    2. Downloads artifact 'amc-launcher-windows-x64-debug' into <workspace>/dist.
       The download is skipped when the stamp file already matches the run id.
    3. (Re)creates the "Aleph Launcher" shortcut on the Desktop pointing at
       dist/amc-launcher.exe.
.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts/fetch-build.ps1
#>
[CmdletBinding()]
param(
    [string]$Repo = "KurumaOfficial/Aleph-Minecraft-Client-Launcher",
    [string]$WorkflowName = "CI",
    [string]$ArtifactName = "amc-launcher-windows-x64-debug",
    [string]$DistDir = "",
    [string]$ShortcutName = "Aleph Launcher"
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($DistDir)) {
    $DistDir = Join-Path (Split-Path -Parent $RepoRoot) "dist"
}

function Get-GitHubToken {
    $env:GIT_TERMINAL_PROMPT = "0"
    $req = "protocol=https`nhost=github.com`n"
    $lines = $req | git credential fill 2>$null
    foreach ($line in $lines) {
        if ($line -like "password=*") { return $line.Substring(9) }
    }
    throw "No GitHub credential in the git credential helper. Push once (git push) first."
}

function Invoke-GitHubApi([string]$Url) {
    $headers = @{
        Authorization = "Bearer $script:Token"
        Accept        = "application/vnd.github+json"
    }
    return Invoke-RestMethod -Uri $Url -Headers $headers -TimeoutSec 60
}

$script:Token = Get-GitHubToken
if ([string]::IsNullOrWhiteSpace($script:Token)) { throw "Empty GitHub token." }

# 1. Latest successful CI run on main.
$runsUrl = "https://api.github.com/repos/$Repo/actions/runs?branch=main&status=success&per_page=10"
$run = (Invoke-GitHubApi $runsUrl).workflow_runs |
    Where-Object { $_.name -eq $WorkflowName } |
    Select-Object -First 1
if (-not $run) { throw "No successful '$WorkflowName' runs on main yet." }
$runId = $run.id
$runSha = $run.head_sha.Substring(0, 7)
Write-Host "Latest successful run: $WorkflowName #$($run.run_number) ($runSha, id=$runId)"

# 2. Download (skip when the stamp matches).
$stampFile = Join-Path $DistDir ".last-successful-run"
$stamp = ""
if (Test-Path -LiteralPath $stampFile) { $stamp = (Get-Content -Raw -LiteralPath $stampFile).Trim() }
$exePath = Join-Path $DistDir "amc-launcher.exe"
if ($stamp -eq "$runId" -and (Test-Path -LiteralPath $exePath)) {
    Write-Host "dist/ is already at run $runId - download skipped."
} else {
    $artifact = (Invoke-GitHubApi "https://api.github.com/repos/$Repo/actions/runs/$runId/artifacts").artifacts |
        Where-Object { $_.name -eq $ArtifactName -and -not $_.expired } |
        Select-Object -First 1
    if (-not $artifact) { throw "Artifact '$ArtifactName' not found or expired on run $runId." }
    if (-not (Test-Path -LiteralPath $DistDir)) { New-Item -ItemType Directory -Path $DistDir | Out-Null }
    $zip = Join-Path ([IO.Path]::GetTempPath()) ("amc-build-" + $runId + ".zip")
    $dlHeaders = @{ Authorization = "Bearer $script:Token" }
    Invoke-WebRequest -Uri $artifact.archive_download_url -Headers $dlHeaders -OutFile $zip -TimeoutSec 300
    Expand-Archive -LiteralPath $zip -DestinationPath $DistDir -Force
    Remove-Item -Force -LiteralPath $zip
    if (-not (Test-Path -LiteralPath $exePath)) { throw "Downloaded artifact has no amc-launcher.exe." }
    Set-Content -LiteralPath $stampFile -Value "$runId" -NoNewline
    $size = (Get-Item -LiteralPath $exePath).Length
    Write-Host ("Downloaded amc-launcher.exe ({0:N0} bytes)." -f $size)
}

# 3. Desktop shortcut.
$shell = New-Object -ComObject WScript.Shell
$lnkPath = Join-Path ([Environment]::GetFolderPath("Desktop")) ($ShortcutName + ".lnk")
$lnk = $shell.CreateShortcut($lnkPath)
$lnk.TargetPath = $exePath
$lnk.WorkingDirectory = $DistDir
$icon = Join-Path $RepoRoot "assets/icon.ico"
$lnk.IconLocation = $(if (Test-Path -LiteralPath $icon) { "$icon,0" } else { "$exePath,0" })
$lnk.Description = "Aleph Minecraft Client Launcher (cloud build $WorkflowName run $runId, $runSha)"
$lnk.Save()
Write-Host "Shortcut refreshed: $lnkPath -> $exePath"
