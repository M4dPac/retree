<#
.SYNOPSIS
    Measure peak memory usage of retree (sequential / streaming / parallel).
.DESCRIPTION
    Uses System.Diagnostics.Process.PeakWorkingSet64 — same value as
    "Peak Working Set" in Task Manager.
    Requires persistent bench trees in target\bench_trees\.
    Run 'cargo bench --bench retree_perf' first to create them.
.EXAMPLE
    .\benches\bench_memory.ps1
    .\benches\bench_memory.ps1 -Sizes 10k,100k
    .\benches\bench_memory.ps1 -Sizes 1m -Runs 5
#>

param(
    [string[]] $Sizes = @("10k", "100k", "1m"),
    [int]      $Runs  = 3
)

$ErrorActionPreference = "Stop"

# ── Paths ─────────────────────────────────────────────────────────────

$projectRoot = Split-Path -Parent $PSScriptRoot
$retree       = Join-Path $projectRoot "target\release\rt.exe"
$outFile     = Join-Path $projectRoot "target\bench_memory.md"

$treeMap = @{
    "100"  = @{ Path = "small_100";   Label = "100 files"  }
    "10k"  = @{ Path = "medium_10k";  Label = "10k files"  }
    "100k" = @{ Path = "large_100k";  Label = "100k files" }
    "1m"   = @{ Path = "xlarge_1m";   Label = "1M files"   }
}

# ── Build ─────────────────────────────────────────────────────────────

Write-Host "`nBuilding retree (release)..." -ForegroundColor Cyan
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

if (-not (Test-Path $retree)) {
    Write-Host "Binary not found: $retree" -ForegroundColor Red
    exit 1
}

Write-Host "Build OK." -ForegroundColor Green

# ── Validate requested sizes ──────────────────────────────────────────

foreach ($size in $Sizes) {
    if (-not $treeMap.ContainsKey($size)) {
        Write-Host "Unknown size: $size. Available: $($treeMap.Keys -join ', ')" -ForegroundColor Red
        exit 1
    }
    $marker = Join-Path $projectRoot "target\bench_trees\$($treeMap[$size].Path)\.tree_ready"
    if (-not (Test-Path $marker)) {
        Write-Host "Tree '$size' not found. Run first:" -ForegroundColor Red
        Write-Host "  cargo bench --bench retree_perf" -ForegroundColor Yellow
        exit 1
    }
}

# ── Memory measurement ────────────────────────────────────────────────

function Measure-RtreeMemory {
    param(
        [string]   $ExePath,
        [string]   $TreePath,
        [string[]] $ExtraArgs,
        [int]      $Runs
    )

    $peaks = @()

    for ($i = 0; $i -lt $Runs; $i++) {
        $psi = New-Object System.Diagnostics.ProcessStartInfo
        $psi.FileName               = $ExePath
        $psi.Arguments              = (@($TreePath, "--noreport") + $ExtraArgs) -join ' '
        $psi.UseShellExecute        = $false
        $psi.RedirectStandardOutput = $true
        $psi.RedirectStandardError  = $true
        $psi.CreateNoWindow         = $true

        $proc = [System.Diagnostics.Process]::Start($psi)

        # Read output asynchronously to prevent deadlock on large trees
        $proc.BeginOutputReadLine()
        $proc.BeginErrorReadLine()

        $peakBytes = [long] 0
        while (-not $proc.HasExited) {
            try {
                $proc.Refresh()
                if ($proc.PeakWorkingSet64 -gt $peakBytes) {
                    $peakBytes = $proc.PeakWorkingSet64
                }
            } catch {}
            Start-Sleep -Milliseconds 10
        }

        # Final read after exit
        try {
            $proc.Refresh()
            if ($proc.PeakWorkingSet64 -gt $peakBytes) {
                $peakBytes = $proc.PeakWorkingSet64
            }
        } catch {}

        $proc.WaitForExit()
        $exitCode = $proc.ExitCode
        $proc.Dispose()

        if ($exitCode -ne 0) {
            Write-Host "  retree exited with code $exitCode" -ForegroundColor Red
            return $null
        }

        $peaks += $peakBytes
    }

    return [PSCustomObject]@{
        AvgMB = [math]::Round(($peaks | Measure-Object -Average).Average / 1MB, 1)
        MinMB = [math]::Round(($peaks | Measure-Object -Minimum).Minimum / 1MB, 1)
        MaxMB = [math]::Round(($peaks | Measure-Object -Maximum).Maximum / 1MB, 1)
    }
}

# ── Modes ─────────────────────────────────────────────────────────────

$modes = @(
    @{ Name = "sequential"; Args = @()               }
    @{ Name = "streaming";  Args = @("--streaming")  }
    @{ Name = "parallel";   Args = @("--parallel")   }
)

# ── Run measurements ──────────────────────────────────────────────────

$allResults = @()

foreach ($size in $Sizes) {
    $info     = $treeMap[$size]
    $treePath = Join-Path $projectRoot "target\bench_trees\$($info.Path)"

    Write-Host "`n=== $($info.Label) ($Runs runs each) ===" -ForegroundColor Cyan

    foreach ($mode in $modes) {
        Write-Host "  $($mode.Name)..." -NoNewline -ForegroundColor White

        $result = Measure-RtreeMemory `
            -ExePath   $retree `
            -TreePath  $treePath `
            -ExtraArgs $mode.Args `
            -Runs      $Runs

        if ($null -eq $result) {
            Write-Host " FAILED" -ForegroundColor Red
            continue
        }

        Write-Host " $($result.AvgMB) MB  (min=$($result.MinMB), max=$($result.MaxMB))" `
            -ForegroundColor Green

        $allResults += [PSCustomObject]@{
            Size  = $info.Label
            Mode  = $mode.Name
            AvgMB = $result.AvgMB
            MinMB = $result.MinMB
            MaxMB = $result.MaxMB
        }
    }
}

# ── Summary table ─────────────────────────────────────────────────────

$sep = "=" * 65

Write-Host "`n$sep" -ForegroundColor Cyan
Write-Host "  PEAK MEMORY USAGE  (PeakWorkingSet64)" -ForegroundColor Cyan
Write-Host $sep -ForegroundColor Cyan

Write-Host ("{0,-14} {1,-14} {2,10} {3,10} {4,10}" -f `
    "Size", "Mode", "Avg MB", "Min MB", "Max MB") -ForegroundColor Yellow
Write-Host ("-" * 65)

$prevSize = ""
foreach ($r in $allResults) {
    if ($r.Size -ne $prevSize -and $prevSize -ne "") { Write-Host "" }
    $prevSize = $r.Size
    Write-Host ("{0,-14} {1,-14} {2,10} {3,10} {4,10}" -f `
        $r.Size, $r.Mode, $r.AvgMB, $r.MinMB, $r.MaxMB)
}

# ── Streaming savings ─────────────────────────────────────────────────

Write-Host "`n$sep" -ForegroundColor Cyan
Write-Host "  STREAMING SAVINGS vs SEQUENTIAL" -ForegroundColor Cyan
Write-Host $sep -ForegroundColor Cyan

foreach ($group in ($allResults | Group-Object Size)) {
    $seq = $group.Group | Where-Object { $_.Mode -eq "sequential" }
    $str = $group.Group | Where-Object { $_.Mode -eq "streaming"  }

    if ($seq -and $str -and $seq.AvgMB -gt 0) {
        $savedMB  = [math]::Round($seq.AvgMB - $str.AvgMB, 1)
        $savedPct = [math]::Round(($savedMB / $seq.AvgMB) * 100, 0)
        $arrow    = if ($savedPct -gt 0) { "▼" } else { "▲" }
        $color    = if ($savedPct -gt 0) { "Green" } else { "Yellow" }

        Write-Host ("  {0,-14} seq={1,6} MB   stream={2,6} MB   {3}{4}% ({5} MB)" -f `
            $group.Name, $seq.AvgMB, $str.AvgMB, `
            $arrow, [math]::Abs($savedPct), [math]::Abs($savedMB)) `
            -ForegroundColor $color
    }
}

# ── Export markdown ───────────────────────────────────────────────────

$md  = @("# retree benchmark - peak memory (PeakWorkingSet64)", "")
$md += "| Size | Mode | Avg MB | Min MB | Max MB |"
$md += "|------|------|-------:|-------:|-------:|"

foreach ($r in $allResults) {
    $md += "| $($r.Size) | $($r.Mode) | $($r.AvgMB) | $($r.MinMB) | $($r.MaxMB) |"
}

$md -join "`n" | Set-Content -Path $outFile -Encoding UTF8

Write-Host "`nResults saved to: $outFile" -ForegroundColor Gray
