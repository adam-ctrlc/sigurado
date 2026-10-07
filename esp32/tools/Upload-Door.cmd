@echo off
title Upload to the door node
echo.
echo   Uploading the firmware to the door node.
echo   This board only reads fingers, so the reset is harmless.
echo.
echo   To send a single-component test sketch instead:
echo     Upload-Door.cmd fingerprint
echo.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\Upload-Board.ps1" -Node door -Sketch "%~1"
pause
