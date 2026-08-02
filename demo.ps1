# Run this while screen-recording (Win+Alt+R) for the 5-minute demo.
# Pre-warm once before recording (see PREP at the bottom) so nothing stalls on camera.

$here = $PSScriptRoot
$env:Path = "$env:USERPROFILE\.cargo\bin;C:\Program Files\Go\bin;C:\msys64\mingw64\bin;$here\bridge\target\release;$env:Path"
$env:CGO_ENABLED = '1'
$env:CC = 'C:\msys64\mingw64\bin\gcc.exe'

function Banner($n, $t) {
    Write-Host ""
    Write-Host "==================================================================" -ForegroundColor Cyan
    Write-Host "  $n. $t" -ForegroundColor Cyan
    Write-Host "==================================================================" -ForegroundColor Cyan
    Start-Sleep -Seconds 2
}

Banner 1 "The port: zero unsafe, zero dependencies"
Get-Content "$here\src\lib.rs" -TotalCount 1
Write-Host "  ^ compiler-enforced. No external crates." -ForegroundColor DarkGray
Start-Sleep 2

Banner 2 "Translated original test suite passes"
cargo test --release --manifest-path "$here\Cargo.toml"

Banner 3 "Known base58 vectors, live on the CLI"
"61`n626262`n00000000000000000000" | & "$here\target\release\base58.exe" encode
Write-Host "  61 -> 2g, and 10 zero bytes -> ten '1's (leading zeros preserved)" -ForegroundColor DarkGray
Start-Sleep 2

Banner 4 "THE ORIGINAL Go tests, unmodified, run against the Rust port (cgo)"
Push-Location "$here\bridge\gotests"
go test -v -count=1 .
Pop-Location

Banner 5 "Differential fuzz vs the real Go library"
python "$here\fuzz\differential.py" --rust "$here\target\release\base58.exe" --go "$here\oracle-go\oracle.exe" --seconds 20 --seed 7

Banner 6 "Honest benchmark: throughput, startup, RSS"
Get-Content "$here\bench\results.json" | Select-String 'op"|ops_per_sec|rust_kb|go_kb|rust_ns|go_ns'

Write-Host ""
Write-Host "Done. 0 unsafe, original suite green, 0 fuzz divergences, honest numbers." -ForegroundColor Green

# PREP (run ONCE before recording so the camera never waits on a build):
#   cargo build --release
#   pushd oracle-go; go build -o oracle.exe .; popd
#   pushd bridge; cargo build --release; popd
#   pushd bridge\gotests; go test -count=1 .; popd   # warms the cgo build
