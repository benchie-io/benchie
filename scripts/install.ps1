#!/usr/bin/env pwsh

$ErrorActionPreference = 'Stop'

if ($v) {
  $Version = "${v}"
}
if ($args.Length -eq 1) {
  $Version = $args.Get(0)
}

$BenchieInstall = $env:BENCHIE_INSTALL
$BinDir = if ($BenchieInstall) {
  "$BenchieInstall\bin"
} else {
  "$Home\.benchie\bin"
}

$BenchieZip = "$BinDir\benchie.zip"
$BenchieExe = "$BinDir\benchie.exe"
$Target = 'x86_64-pc-windows-msvc'

# GitHub requires TLS 1.2
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$BenchieUri = if (!$Version) {
  "https://github.com/benchie-io/benchie/releases/latest/download/benchie-${Target}.zip"
} else {
  "https://github.com/benchie-io/benchie/releases/download/${Version}/benchie-${Target}.zip"
}

if (!(Test-Path $BinDir)) {
  New-Item $BinDir -ItemType Directory | Out-Null
}

Invoke-WebRequest $BenchieUri -OutFile $BenchieZip -UseBasicParsing

if (Get-Command Expand-Archive -ErrorAction SilentlyContinue) {
  Expand-Archive $BenchieZip -Destination $BinDir -Force
} else {
  if (Test-Path $BenchieExe) {
    Remove-Item $BenchieExe
  }
  Add-Type -AssemblyName System.IO.Compression.FileSystem
  [IO.Compression.ZipFile]::ExtractToDirectory($BenchieZip, $BinDir)
}

Remove-Item $BenchieZip

$User = [EnvironmentVariableTarget]::User
$Path = [Environment]::GetEnvironmentVariable('Path', $User)
if (!(";$Path;".ToLower() -like "*;$BinDir;*".ToLower())) {
  [Environment]::SetEnvironmentVariable('Path', "$Path;$BinDir", $User)
  $Env:Path += ";$BinDir"
}

Write-Output "benchie was installed successfully to $BenchieExe"
Write-Output "Run 'benchie --help' to get started"