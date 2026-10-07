@echo off
title Monitor the cabinet node
echo.
echo   Serial monitor on the cabinet node at 115200.
echo   Ctrl+C closes it and frees the port for uploading.
echo.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\Monitor-Board.ps1" -Node cabinet
pause
