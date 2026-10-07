<#
.SYNOPSIS
    Opens the serial monitor on the door or cabinet node.

.DESCRIPTION
    Locates the port with Find-Board.ps1 and hands it to arduino-cli monitor at
    115200, which is what both sketches call Serial.begin with.

    The monitor holds the port open. Close it with Ctrl+C before uploading, or
    the upload cannot claim the port and fails part way through.

.PARAMETER Node
    door or cabinet.

.PARAMETER Port
    Skip auto-detection and use this port.

.PARAMETER Baud
    Defaults to 115200, matching the firmware.

.EXAMPLE
    .\Monitor-Board.ps1 -Node door
    .\Monitor-Board.ps1 -Node cabinet -Port COM7
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('door', 'cabinet')]
    [string]$Node,

    [string]$Port = '',

    [int]$Baud = 115200
)

$ErrorActionPreference = 'Stop'

$finder = Join-Path $PSScriptRoot 'Find-Board.ps1'

function Fail($message) {
    Write-Host ""
    Write-Host "  $message" -ForegroundColor Red
    Write-Host ""
    exit 1
}

if (-not (Get-Command arduino-cli -ErrorAction SilentlyContinue)) {
    Fail "arduino-cli is not on PATH. Install it with: winget install ArduinoSA.CLI"
}

if (-not $Port) {
    Write-Host ""
    Write-Host "  looking for the $Node node ..." -ForegroundColor Cyan
    $Port = & $finder -Node $Node -Quiet

    if (-not $Port) {
        & $finder -Node $Node | Out-Null
        $Port = & $finder -Node $Node -Quiet
    }
    if (-not $Port) {
        Fail "No ESP32 found. Plug it in, or pass -Port COMx. Run Find-Board.cmd to see what is connected."
    }
}

Write-Host "  port: $Port" -ForegroundColor Green
Write-Host ""
Write-Host "  monitor at $Baud, Ctrl+C to stop" -ForegroundColor Cyan
Write-Host "  Type to send. The test sketches take single-letter commands." -ForegroundColor DarkGray
Write-Host ""

arduino-cli monitor -p $Port --config baudrate=$Baud

Write-Host ""
exit 0
