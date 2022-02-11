$ErrorActionPreference = "Stop"
$global:latestReleaseRequestUrl = "https://api.github.com/repos/maidsafe/safe_network/releases/latest"
$global:vcRedistUrl = "https://download.microsoft.com/download/9/3/F/93FCF1E7-E6A4-478B-96E7-D4B285925B00/vc_redist.x64.exe"
$global:installPath = Join-Path -Path $env:USERPROFILE -ChildPath ".safe\cli"
$global:safeBinPath = Join-Path -Path $installPath -ChildPath "safe.exe"
$global:pathModified = $false
$global:vcRedistInstalled = $false

function InstallVcppRedist {
    $local:filename = $vcRedistUrl -split '/' | Select-Object -Last 1
    $local:exePath = Join-Path -Path $env:TEMP -ChildPath $filename
    $list = Get-ItemProperty HKLM:\Software\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\* `
        | Where-Object { $_.DisplayName -like "Microsoft Visual C++*" } | Select-Object DisplayName
    if ($list) {
        echo "Visual C++ Redistributable is already installed"
        return        
    }
    echo "Downloading Visual C++ Redistributable installer to $exePath"
    wget $vcRedistUrl -outfile $exePath
    echo "Running Visual C++ Redistributable installer as Administrator"
    $p = Start-Process $exePath `
        -ArgumentList "/install /quiet /norestart" -wait -PassThru -Verb RunAs
    if ($p.ExitCode -ne 0) {
        echo "Visual C++ Redistributable wasn't installed successfully"
        exit 1
    }
    $global:vcRedistInstalled = $true
    echo "Visual C++ Redistributable installed"
    echo "Removing temporary $exePath file"
    Remove-Item $exePath
}

function ConfigureInstallPath {
    if (!(Test-Path $installPath)) {
        New-Item -ItemType Directory -Path $installPath
    }
    $local:currentPaths = [Environment]::GetEnvironmentVariable(
        'Path', [EnvironmentVariableTarget]::User) -split ';'
    if ($currentPaths -notcontains $installPath) {
        $global:pathModified = $true
        echo "Adding CLI install path to user PATH variable"
        $currentPaths = $currentPaths + $installPath | where { $_ }
        [Environment]::SetEnvironmentVariable(
            'Path', $currentPaths -join ';', [EnvironmentVariableTarget]::User)
    }
}

function DownloadAndUnpackSafe {
    if (Test-Path $safeBinPath) {
        $proceed = Read-Host `
            "A safe binary was already detected at $safeBinPath. Would you like to overwrite? [y/n]"
        if (($proceed -ne "y") -or ($proceed -ne "Y")) {
            return
        }
    }
    echo "Obtaining latest version of safe"
    $local:response = Invoke-RestMethod -Uri $latestReleaseRequestUrl
    $version = $response.tag_name -split '-' | Select-Object -Last 1
    echo "Latest version is $version"
    $local:windowsAsset = $response.assets `
        | Where-Object { $_.name -match "sn_cli-.*-x86_64-pc-windows-msvc.zip" }
    $local:zipFileName = $windowsAsset.browser_download_url -split '/' | Select-Object -Last 1
    $local:archivePath = Join-Path -Path $env:TEMP -ChildPath $zipFileName
    echo "Downloading $zipFileName to $archivePath"
    wget $windowsAsset.browser_download_url -outfile $archivePath
    echo "Unpacking safe binary to $installPath"
    Expand-Archive -Path $archivePath -DestinationPath $installPath -Force
    echo "Removing temporary $archivePath file"
    Remove-Item $archivePath
}

InstallVcppRedist
ConfigureInstallPath
DownloadAndUnpackSafe
if ($pathModified -and !$vcRedistInstalled) {
    echo "Restart your Powershell session to use safe"
}
if ($vcRedistInstalled) {
    echo "Visual C++ Redistributable installation requires a restart"
}
