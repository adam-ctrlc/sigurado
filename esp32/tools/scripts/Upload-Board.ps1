<#
.SYNOPSIS
    Finds a node and uploads to it.

.DESCRIPTION
    Locates the port with Find-Board.ps1, then compiles and uploads the
    firmware in door/ or cabinet/ using arduino-cli.

    Pass -Sketch to send one of the single-component sketches in testing/
    instead of the real firmware. That is the way to check a sensor on its own
    before trusting the whole thing.

    Uploading resets the board. The door node only reads fingers, so a reset
    there is harmless. The cabinet node drives the solenoid: it releases the
    coil on reset and comes back locked, which is the safe direction, but the
    relay does click.

.PARAMETER Node
    door or cabinet.

.PARAMETER Sketch
    Name of a folder under testing/, to upload that instead of the firmware.

.PARAMETER Monitor
    Open the serial monitor after uploading.

.PARAMETER Port
    Skip auto-detection and use this port.

.PARAMETER UploadSpeed
    Baud for flashing. Defaults to 921600, dropping to 115200 on failure.

.EXAMPLE
    .\Upload-Board.ps1 -Node door
    .\Upload-Board.ps1 -Node cabinet -Monitor
    .\Upload-Board.ps1 -Node door -Sketch fingerprint -Monitor
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('door', 'cabinet')]
    [string]$Node,

    [string]$Sketch = '',

    [switch]$Monitor,

    [string]$Port = '',

    [int]$UploadSpeed = 921600
)

$ErrorActionPreference = 'Stop'

$esp32 = Resolve-Path (Join-Path $PSScriptRoot '..\..')
$finder = Join-Path $PSScriptRoot 'Find-Board.ps1'

# The DevKit v1 has no PSRAM or JTAG menu. Those only appear for the S3 boards,
# so there is nothing here to turn off.
$fqbnBase = 'esp32:esp32:esp32doit-devkit-v1'

function Fail($message) {
    Write-Host ""
    Write-Host "  $message" -ForegroundColor Red
    Write-Host ""
    exit 1
}

# ---- pick the sketch -----------------------------------------------------

if ($Sketch) {
    $sketchDir = Join-Path (Join-Path $esp32 'testing') $Sketch
    $label = "$Sketch test"

    if (-not (Test-Path $sketchDir)) {
        Write-Host ""
        Write-Host "  No test sketch called '$Sketch'. The ones there are:" -ForegroundColor Yellow
        Get-ChildItem (Join-Path $esp32 'testing') -Directory |
            ForEach-Object { Write-Host "    $($_.Name)" -ForegroundColor DarkGray }
        Fail "Pick one of those."
    }
} else {
    $sketchDir = Join-Path $esp32 $Node
    $label = "$Node firmware"

    if (-not (Test-Path (Join-Path $sketchDir "$Node.ino"))) {
        Fail "No sketch at $sketchDir\$Node.ino"
    }
}

if (-not (Get-Command arduino-cli -ErrorAction SilentlyContinue)) {
    Fail "arduino-cli is not on PATH. Install it with: winget install ArduinoSA.CLI"
}

# ---- warn about placeholders --------------------------------------------

$config = Join-Path $sketchDir 'src\Config.h'
if (Test-Path $config) {
    $text = Get-Content $config -Raw
    $left = @()
    if ($text -match 'YOUR_WIFI"') { $left += 'WIFI_SSID' }
    if ($text -match 'YOUR_WIFI_PASSWORD') { $left += 'WIFI_PASS' }
    if ($text -match 'PASTE_\w+_DEVICE_ID') { $left += 'DEVICE_ID' }
    if ($text -match 'PASTE_\w+_DEVICE_SECRET') { $left += 'DEVICE_SECRET' }

    if ($left.Count -gt 0) {
        Write-Host ""
        Write-Host ("  Still placeholders in Config.h: " + ($left -join ', ')) -ForegroundColor Yellow
        Write-Host "  It will flash and boot, but it cannot reach the server yet." -ForegroundColor DarkGray
    }
}

# ---- find the port -------------------------------------------------------

if (-not $Port) {
    Write-Host ""
    Write-Host "  looking for the $Node node ..." -ForegroundColor Cyan
    $Port = & $finder -Node $Node -Quiet

    if (-not $Port) {
        # Quiet mode stays silent when two identical boards are plugged in and
        # nothing has been remembered yet. Run it loud so it can ask.
        & $finder -Node $Node | Out-Null
        $Port = & $finder -Node $Node -Quiet
    }
    if (-not $Port) {
        Fail "No ESP32 found. Plug it in, or pass -Port COMx. Run Find-Board.cmd to see what is connected."
    }
}
Write-Host "  port: $Port" -ForegroundColor Green

# ---- upload --------------------------------------------------------------

Write-Host ""
Write-Host "  compiling $label ..." -ForegroundColor Cyan
arduino-cli compile --fqbn $fqbnBase $sketchDir
if ($LASTEXITCODE -ne 0) { Fail "Compile failed, nothing was uploaded." }

Write-Host ""
Write-Host "  uploading to $Port ..." -ForegroundColor Cyan
arduino-cli upload -p $Port --fqbn "$fqbnBase`:UploadSpeed=$UploadSpeed" $sketchDir

# 921600 is optimistic on a CH340 with a long lead, and dropping the speed fixes
# it far more often than any wiring change would.
if ($LASTEXITCODE -ne 0 -and $UploadSpeed -ne 115200) {
    Write-Host ""
    Write-Host "  failed at $UploadSpeed baud, trying 115200 ..." -ForegroundColor Yellow
    arduino-cli upload -p $Port --fqbn "$fqbnBase`:UploadSpeed=115200" $sketchDir
}

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "  Upload failed. Worth checking, in order:" -ForegroundColor Red
    Write-Host "    1. a monitor window still open and holding $Port" -ForegroundColor DarkGray
    Write-Host "    2. hold BOOT while it says 'Connecting...', then let go" -ForegroundColor DarkGray
    Write-Host "    3. a charge-only USB cable" -ForegroundColor DarkGray
    Write-Host "    4. wiring on GPIO 0, 2, 12 or 15 forcing the wrong boot mode" -ForegroundColor DarkGray
    Write-Host ""
    exit 1
}

Write-Host ""
Write-Host "  done" -ForegroundColor Green

if ($Monitor) {
    Write-Host ""
    Write-Host "  monitor, Ctrl+C to stop" -ForegroundColor Cyan
    Write-Host ""
    Start-Sleep -Milliseconds 1500
    arduino-cli monitor -p $Port --config baudrate=115200
}

Write-Host ""
exit 0
