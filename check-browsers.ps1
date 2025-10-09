# Browser Detection Check Script
# This script checks which browsers are installed and where

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  ripVID Browser Detection Check" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$browsersFound = 0

# Check Chrome
Write-Host "Checking for Google Chrome..." -ForegroundColor Yellow
$chromePaths = @(
    "$env:ProgramFiles\Google\Chrome\Application\chrome.exe",
    "${env:ProgramFiles(x86)}\Google\Chrome\Application\chrome.exe",
    "$env:LOCALAPPDATA\Google\Chrome\Application\chrome.exe"
)

foreach ($path in $chromePaths) {
    if (Test-Path $path) {
        Write-Host "  ✓ FOUND: $path" -ForegroundColor Green
        $browsersFound++
        break
    }
}
if (-not (Test-Path $chromePaths[0]) -and -not (Test-Path $chromePaths[1]) -and -not (Test-Path $chromePaths[2])) {
    Write-Host "  ✗ Chrome not found" -ForegroundColor Red
}

Write-Host ""

# Check Edge
Write-Host "Checking for Microsoft Edge..." -ForegroundColor Yellow
$edgePaths = @(
    "$env:ProgramFiles\Microsoft\Edge\Application\msedge.exe",
    "${env:ProgramFiles(x86)}\Microsoft\Edge\Application\msedge.exe"
)

foreach ($path in $edgePaths) {
    if (Test-Path $path) {
        Write-Host "  ✓ FOUND: $path" -ForegroundColor Green
        $browsersFound++
        break
    }
}

# Try where command for Edge
try {
    $whereEdge = where.exe msedge.exe 2>$null
    if ($whereEdge) {
        Write-Host "  ✓ FOUND via 'where': $whereEdge" -ForegroundColor Green
        if ($browsersFound -eq 0) { $browsersFound++ }
    }
} catch {
    # Ignore
}

if (-not (Test-Path $edgePaths[0]) -and -not (Test-Path $edgePaths[1]) -and -not $whereEdge) {
    Write-Host "  ✗ Edge not found" -ForegroundColor Red
}

Write-Host ""

# Check Firefox
Write-Host "Checking for Mozilla Firefox..." -ForegroundColor Yellow
$firefoxPaths = @(
    "$env:ProgramFiles\Mozilla Firefox\firefox.exe",
    "${env:ProgramFiles(x86)}\Mozilla Firefox\firefox.exe",
    "$env:LOCALAPPDATA\Mozilla Firefox\firefox.exe"
)

foreach ($path in $firefoxPaths) {
    if (Test-Path $path) {
        Write-Host "  ✓ FOUND: $path" -ForegroundColor Green
        $browsersFound++
        break
    }
}
if (-not (Test-Path $firefoxPaths[0]) -and -not (Test-Path $firefoxPaths[1]) -and -not (Test-Path $firefoxPaths[2])) {
    Write-Host "  ✗ Firefox not found" -ForegroundColor Red
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan

if ($browsersFound -gt 0) {
    Write-Host "  ✓ RESULT: $browsersFound browser(s) detected" -ForegroundColor Green
    Write-Host "  Browser cookies should work!" -ForegroundColor Green
} else {
    Write-Host "  ✗ RESULT: No browsers detected" -ForegroundColor Red
    Write-Host ""
    Write-Host "  To download Facebook/Instagram videos with cookies," -ForegroundColor Yellow
    Write-Host "  please install one of these browsers:" -ForegroundColor Yellow
    Write-Host "    - Google Chrome: https://www.google.com/chrome/" -ForegroundColor White
    Write-Host "    - Microsoft Edge: Pre-installed on Windows 10/11" -ForegroundColor White
    Write-Host "    - Mozilla Firefox: https://www.mozilla.org/firefox/" -ForegroundColor White
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Press any key to exit..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
