@echo off
title Find Sigurado boards
echo.
echo   Looking for the ESP32 nodes and saying why each port was picked.
echo   Bluetooth ports are ignored, so only real boards are listed.
echo.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\Find-Board.ps1" -Node door
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\Find-Board.ps1" -Node cabinet
pause
