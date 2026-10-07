@echo off
title Upload to the cabinet node
echo.
echo   Uploading the firmware to the cabinet node.
echo.
echo   This resets the board, so the solenoid relay will click. The coil
echo   releases on reset and the cabinet comes back locked, which is the
echo   safe direction, but do not have your hand in the way of the bolt.
echo.
echo   To send a single-component test sketch instead:
echo     Upload-Cabinet.cmd limit_switch
echo.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\Upload-Board.ps1" -Node cabinet -Sketch "%~1"
pause
