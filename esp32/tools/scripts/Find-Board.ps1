<#
.SYNOPSIS
    Finds the COM port for the door or cabinet node and says why it picked it.

.DESCRIPTION
    Scores every serial port by USB vendor and product ID, then cross checks
    with arduino-cli when it is available. Bluetooth ports are always excluded,
    which matters on a laptop: a paired phone leaves two or three COM ports
    behind that look like boards to anything that only counts ports.

    The two nodes are the same hardware, so USB alone cannot tell the door from
    the cabinet. With one board plugged in there is nothing to confuse and it is
    taken as whichever node you asked for. With both plugged in you are asked
    once, and the answer is remembered in nodes.json against the board's USB
    instance id, so it keeps up when Windows renumbers the ports.

    Scores and reasons are printed rather than hidden. Trust a "confirmed"
    line; treat a "likely" line as a guess worth checking.

.PARAMETER Node
    door or cabinet.

.PARAMETER Quiet
    Print only the port name, for use in scripts:
        $port = .\Find-Board.ps1 -Node door -Quiet

.PARAMETER Forget
    Drop what was remembered and ask again next time.

.EXAMPLE
    .\Find-Board.ps1 -Node door
    .\Find-Board.ps1 -Node cabinet -Quiet
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('door', 'cabinet')]
    [string]$Node,

    [switch]$Quiet,

    [switch]$Forget
)

$ErrorActionPreference = 'Stop'

$MemoryPath = Join-Path $PSScriptRoot 'nodes.json'

# Every ESP32 DevKit reaches the PC through one of these. Recognising the chip
# is a positive identification, not a guess at whatever is left over.
$VendorNames = @{
    '10C4' = 'Silicon Labs CP210x bridge'
    '1A86' = 'WCH CH340/CH9102 bridge'
    '0403' = 'FTDI bridge'
    '303A' = 'Espressif (native USB)'
}

function Get-SerialPorts {
    $entries = @()
    Get-CimInstance Win32_PnPEntity -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -match '\((COM\d+)\)' } |
        ForEach-Object {
            $port = if ($_.Name -match '\((COM\d+)\)') { $Matches[1] } else { $null }
            if (-not $port) { return }

            $vendorId = $null
            $productId = $null
            if ($_.DeviceID -match 'VID_([0-9A-F]{4}).*PID_([0-9A-F]{4})') {
                $vendorId = $Matches[1]
                $productId = $Matches[2]
            }

            $entries += [pscustomobject]@{
                Port      = $port
                Name      = $_.Name
                Vid       = $vendorId
                Pid       = $productId
                HwId      = $_.DeviceID
                Bluetooth = $_.DeviceID -like 'BTHENUM*'
            }
        }
    $entries | Where-Object { -not $_.Bluetooth }
}

function Get-ArduinoCliPorts {
    if (-not (Get-Command arduino-cli -ErrorAction SilentlyContinue)) { return @{} }
    $map = @{}
    try {
        $json = arduino-cli board list --format json 2>$null | ConvertFrom-Json
        foreach ($p in $json.detected_ports) {
            $label = ($p.matching_boards | ForEach-Object { $_.name }) -join ', '
            if ($label) { $map[$p.port.address] = $label }
        }
    } catch { }
    return $map
}

function Get-Memory {
    if (-not (Test-Path $MemoryPath)) { return @{} }
    try {
        $json = Get-Content $MemoryPath -Raw | ConvertFrom-Json
        $map = @{}
        foreach ($p in $json.PSObject.Properties) { $map[$p.Name] = $p.Value }
        return $map
    } catch { return @{} }
}

function Save-Memory($map) {
    $ordered = [ordered]@{}
    foreach ($key in ($map.Keys | Sort-Object)) { $ordered[$key] = $map[$key] }
    $ordered | ConvertTo-Json -Depth 4 | Set-Content -Path $MemoryPath -Encoding UTF8
}

function Remember($node, $entry) {
    $map = Get-Memory

    # One board cannot be both nodes. Claiming it for this one releases it from
    # the other, which is what happens when somebody flashes the door firmware
    # and then the cabinet firmware onto the same board.
    foreach ($other in @($map.Keys)) {
        if ($other -ne $node -and $map[$other].hwid -eq $entry.HwId) {
            $map.Remove($other)
        }
    }

    $map[$node] = [pscustomobject]@{
        hwid = $entry.HwId
        port = $entry.Port
        name = $entry.Name
    }
    Save-Memory $map
}

function Score-Candidate {
    param($Entry, $CliLabel, [string]$Target, $Memory, [int]$BoardCount)

    $score = 0
    $reasons = @()

    # What was remembered wins over anything the USB layer can say, because the
    # USB layer genuinely cannot tell two identical DevKits apart.
    if ($Memory.ContainsKey($Target) -and $Memory[$Target].hwid -eq $Entry.HwId) {
        $score += 100
        $reasons += "remembered as the $Target node from an earlier run"
    }

    # Only rule a board out for belonging to the other node when there is
    # another board it could be. With one board on the bench, being told it was
    # the door last time is no reason to refuse to flash the cabinet onto it.
    if ($BoardCount -gt 1) {
        foreach ($other in @('door', 'cabinet')) {
            if ($other -eq $Target) { continue }
            if ($Memory.ContainsKey($other) -and $Memory[$other].hwid -eq $Entry.HwId) {
                $score -= 100
                $reasons += "this one is remembered as the $other node"
            }
        }
    }

    if ($Entry.Vid -eq '303A') {
        $score += 40
        $reasons += "USB VID 303A, $($VendorNames[$Entry.Vid])"
    }
    if ($Entry.Vid -in @('10C4', '1A86', '0403')) {
        $score += 40
        $reasons += "$($VendorNames[$Entry.Vid]), what a DevKit v1 uses"
    }
    if ($CliLabel) {
        $score += 10
        $reasons += "arduino-cli identifies it as '$CliLabel'"
    }
    if ($CliLabel -match 'Mega|Uno|Nano') {
        $score -= 100
        $reasons += "arduino-cli says this is a '$CliLabel', not an ESP32"
    }
    if ($Entry.Vid -in @('2341', '2A03')) {
        $score -= 100
        $reasons += "USB VID $($Entry.Vid) is an Arduino board, not an ESP32"
    }

    [pscustomobject]@{
        Port    = $Entry.Port
        Name    = $Entry.Name
        HwId    = $Entry.HwId
        Entry   = $Entry
        Score   = $score
        Reasons = $reasons
    }
}

if ($Forget) {
    $map = Get-Memory
    if ($map.ContainsKey($Node)) {
        $map.Remove($Node)
        Save-Memory $map
        if (-not $Quiet) { Write-Host "  forgot the $Node node." -ForegroundColor Gray }
    }
    exit 0
}

$ports = @(Get-SerialPorts)
if ($ports.Count -eq 0) {
    if (-not $Quiet) {
        Write-Warning "No USB serial port found. Is the board plugged in?"
        Write-Host "  Bluetooth ports do not count and are not listed." -ForegroundColor DarkGray
        Write-Host "  A charge-only USB lead looks exactly like this, and so does" -ForegroundColor DarkGray
        Write-Host "  a clone board with no CH340 driver installed." -ForegroundColor DarkGray
    }
    exit 1
}

$cli = Get-ArduinoCliPorts
$memory = Get-Memory

# How many of these look like an ESP32 at all. This decides whether there is
# anything to disambiguate, so it has to be counted before scoring.
$boardCount = @($ports | Where-Object { $_.Vid -in @('10C4', '1A86', '0403', '303A') }).Count

$ranked = @($ports |
    ForEach-Object {
        Score-Candidate -Entry $_ -CliLabel $cli[$_.Port] -Target $Node `
            -Memory $memory -BoardCount $boardCount
    } |
    Sort-Object Score -Descending)

$plausible = @($ranked | Where-Object { $_.Score -gt 0 })

if ($plausible.Count -eq 0) {
    if (-not $Quiet) {
        Write-Warning "No ESP32 found. Ports seen:"
        foreach ($r in $ranked) { Write-Host ("  {0}  {1}" -f $r.Port, $r.Name) -ForegroundColor DarkGray }
    }
    exit 1
}

# Two identical DevKits and nothing remembered: this is the one case USB cannot
# settle, so ask rather than pick one and be wrong half the time.
$ambiguous = $plausible.Count -gt 1 -and $plausible[0].Score -eq $plausible[1].Score

if ($ambiguous) {
    if ($Quiet) {
        exit 1
    }

    Write-Host ""
    Write-Host "  Two boards are plugged in and both look like an ESP32." -ForegroundColor Yellow
    Write-Host "  Which one is the $Node node? Unplug the other if you are not sure." -ForegroundColor Yellow
    Write-Host "  This is asked once and then remembered." -ForegroundColor DarkGray
    Write-Host ""
    for ($i = 0; $i -lt $plausible.Count; $i++) {
        Write-Host ("   [{0}] {1}  {2}" -f ($i + 1), $plausible[$i].Port, $plausible[$i].Name)
    }
    Write-Host ""

    $answer = Read-Host "  number"
    $index = 0
    if (-not [int]::TryParse($answer, [ref]$index) -or $index -lt 1 -or $index -gt $plausible.Count) {
        Write-Host ""
        Write-Host "  Not one of the choices." -ForegroundColor Red
        Write-Host ""
        exit 1
    }

    $best = $plausible[$index - 1]
    Remember $Node $best.Entry
    $best.Reasons = @("you picked it, and it is now remembered as the $Node node")
} else {
    $best = $plausible[0]

    # One board and nothing remembered: take it, and remember it so that adding
    # the second board later does not make both nodes ambiguous.
    if ($plausible.Count -eq 1 -and -not $memory.ContainsKey($Node)) {
        Remember $Node $best.Entry
        $best.Reasons += "the only ESP32 plugged in, remembered as the $Node node"
    }
}

if ($Quiet) {
    Write-Output $best.Port
    exit 0
}

$confidence = if ($best.Score -ge 100) { 'confirmed' } else { 'likely' }
$colour = if ($best.Score -ge 100) { 'Green' } else { 'Yellow' }

Write-Host ""
Write-Host ("  {0} : {1}  ({2})" -f $Node.ToUpper(), $best.Port, $confidence) -ForegroundColor $colour
Write-Host ("  {0}" -f $best.Name) -ForegroundColor Gray
foreach ($reason in $best.Reasons) { Write-Host "    - $reason" -ForegroundColor DarkGray }

if ($ranked.Count -gt 1) {
    Write-Host ""
    Write-Host "  other ports:" -ForegroundColor DarkGray
    foreach ($r in $ranked) {
        if ($r.Port -eq $best.Port) { continue }
        Write-Host ("    {0}  {1}" -f $r.Port, $r.Name) -ForegroundColor DarkGray
    }
}

Write-Host ""
Write-Host "  flash the firmware:" -ForegroundColor Cyan
Write-Host ("    tools\Upload-{0}.cmd" -f (Get-Culture).TextInfo.ToTitleCase($Node))
Write-Host "  watch it run:" -ForegroundColor Cyan
Write-Host ("    tools\Monitor-{0}.cmd" -f (Get-Culture).TextInfo.ToTitleCase($Node))
Write-Host ""
exit 0
