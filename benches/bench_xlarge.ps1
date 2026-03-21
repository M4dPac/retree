<#
.SYNOPSIS
    Benchmark rtree on 1M files using hyperfine.
.DESCRIPTION
    Compares sequential / parallel / streaming modes, thread scaling,
    and output formats. Requires hyperfine.
    Install : winget install sharkdp.hyperfine
    Cleanup : Remove-Item -Recurse target\bench_trees
.EXAMPLE
    .\benches\bench_xlarge.ps1
#>

$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot
$treePath    = Join-Path $projectRoot "target\bench_trees\xlarge_1m"
$rtree       = Join-Path $projectRoot "target\release\rtree.exe"
$outFile     = Join-Path $projectRoot "target\bench_1m.md"

$r = "`"$rtree`""
$t = "`"$treePath`""

# -- Prerequisites ---------------------------------------------------

if (-not (Get-Command hyperfine -ErrorAction SilentlyContinue)) {
    Write-Host "hyperfine not found. Install:" -ForegroundColor Red
    Write-Host "  winget install sharkdp.hyperfine" -ForegroundColor White
    Write-Host "  # or: cargo install hyperfine"    -ForegroundColor White
    exit 1
}

# -- Build -----------------------------------------------------------

Write-Host "`nBuilding rtree (release)..." -ForegroundColor Cyan
Push-Location $projectRoot
$ErrorActionPreference = "Continue"
cargo build --release 2>&1 | Out-Null
$buildExit = $LASTEXITCODE
$ErrorActionPreference = "Stop"
if ($buildExit -ne 0) {
    Write-Host "Build failed." -ForegroundColor Red
    Pop-Location
    exit 1
}
Pop-Location
Write-Host "Build OK." -ForegroundColor Green

if (-not (Test-Path $rtree)) {
    Write-Host "Binary not found: $rtree" -ForegroundColor Red
    exit 1
}

# -- Ensure 1M tree exists -------------------------------------------

if (-not (Test-Path (Join-Path $treePath ".tree_ready"))) {
    Write-Host "`n1M tree not found. Creating (one-time, may take a while)..." -ForegroundColor Yellow
    Push-Location $projectRoot
    cargo bench --bench rtree_perf_xlarge -- "seq_plain"
    Pop-Location
    if (-not (Test-Path (Join-Path $treePath ".tree_ready"))) {
        Write-Host "Tree creation failed. Check that bench_trees are persisted in rtree_perf.rs." -ForegroundColor Red
        exit 1
    }
}

Write-Host "Tree found : $treePath" -ForegroundColor Green

# -- Helper : run one hyperfine section and append to $outFile -------

$tmpMd = Join-Path $env:TEMP "rtree_bench_section.md"

function Invoke-Section {
    param(
        [string]   $Title,
        [string[]] $HyperfineArgs
    )

    Write-Host "`n=== $Title ===" -ForegroundColor Cyan

    & hyperfine @HyperfineArgs --export-markdown $tmpMd
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Benchmark '$Title' failed." -ForegroundColor Red
        exit 1
    }

    "`n## $Title`n" | Add-Content -Path $outFile -Encoding UTF8
    Get-Content $tmpMd | Add-Content -Path $outFile -Encoding UTF8
}

# -- Init output file ------------------------------------------------

"# rtree benchmark - 1M files`n" | Set-Content -Path $outFile -Encoding UTF8

# -- Benchmark 1 : mode comparison -----------------------------------

Invoke-Section "Mode comparison (sequential / parallel / streaming)" @(
    "--warmup", "1", "--runs", "5",
    "--command-name", "sequential", "$r $t --noreport",
    "--command-name", "parallel",   "$r $t --noreport --parallel",
    "--command-name", "streaming",  "$r $t --noreport --streaming"
)

# -- Benchmark 2 : parallel thread scaling ---------------------------

Invoke-Section "Parallel thread scaling" @(
    "--warmup", "1", "--runs", "3",
    "--parameter-list", "threads", "1,2,4,8",
    "--command-name", "threads={threads}",
    "$r $t --noreport --parallel --threads {threads}"
)

# -- Benchmark 3 : output formats ------------------------------------

Invoke-Section "Output formats (text / JSON / XML)" @(
    "--warmup", "1", "--runs", "3",
    "--command-name", "text", "$r $t --noreport",
    "--command-name", "json", "$r $t --noreport -J",
    "--command-name", "xml",  "$r $t --noreport -X"
)

# -- Done ------------------------------------------------------------

Remove-Item $tmpMd -ErrorAction SilentlyContinue

Write-Host "`nResults saved to: $outFile" -ForegroundColor Gray
