param(
    [string]$ExePath = (Join-Path $PSScriptRoot "target\release\zenith-sensor-worker.exe"),
    [int]$DurationSec = 20,
    [int]$IntervalMs = 200,
    [double]$LimitMB = 10
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $ExePath)) {
    Write-Error "worker exe not found: $ExePath (build with: cargo build --release)"
    exit 2
}

$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = (Resolve-Path -LiteralPath $ExePath).Path
$psi.Arguments = "--mock"
$psi.UseShellExecute = $false
$psi.CreateNoWindow = $true
$psi.RedirectStandardInput = $true

$proc = [System.Diagnostics.Process]::Start($psi)

$samples = [System.Collections.Generic.List[double]]::new()
$stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
try {
    while ($stopwatch.Elapsed.TotalSeconds -lt $DurationSec -and -not $proc.HasExited) {
        Start-Sleep -Milliseconds $IntervalMs
        $proc.Refresh()
        if (-not $proc.HasExited) {
            $samples.Add($proc.WorkingSet64 / 1MB)
        }
    }
} finally {
    if (-not $proc.HasExited) {
        $proc.Kill()
        $proc.WaitForExit()
    }
}

if ($samples.Count -eq 0) {
    Write-Error "no samples collected; worker exited before measurement"
    exit 3
}

$idle = $samples[0]
$peak = ($samples | Measure-Object -Maximum).Maximum
$steadyCount = [Math]::Min(5, $samples.Count)
$steady = ($samples[($samples.Count - $steadyCount)..($samples.Count - 1)] | Measure-Object -Average).Average
$pass = $peak -lt $LimitMB

"idle_rss_mb={0:F2} peak_rss_mb={1:F2} steady_rss_mb={2:F2} limit_mb={3:F2} pass={4}" -f $idle, $peak, $steady, $LimitMB, $pass
if (-not $pass) { exit 1 }
