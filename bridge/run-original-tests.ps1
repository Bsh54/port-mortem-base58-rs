# Builds the Rust C-ABI shim and runs the original mr-tron/base58 Go test files,
# unmodified, against the Rust port through cgo.
#
# Requirements: Rust (GNU toolchain), Go, and a C compiler (MSYS2 mingw-w64 gcc).
$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$repo = Split-Path -Parent $here

$gcc = 'C:\msys64\mingw64\bin\gcc.exe'
if (-not (Test-Path $gcc)) { throw "mingw-w64 gcc not found at $gcc (install: pacman -S mingw-w64-x86_64-gcc)" }

$env:Path = "$env:USERPROFILE\.cargo\bin;C:\Program Files\Go\bin;C:\msys64\mingw64\bin;$here\target\release;$env:Path"
$env:CGO_ENABLED = '1'
$env:CC = $gcc

Write-Host '==> building Rust cdylib (b58bridge)'
Push-Location $here
cargo build --release
Pop-Location

Write-Host '==> running the original Go test suite against the Rust port'
Push-Location "$here\gotests"
go test -v -count=1 .
Pop-Location
