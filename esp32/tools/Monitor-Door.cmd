@echo off
title Monitor the door node
echo.
echo   Serial monitor on the door node at 115200.
echo   Ctrl+C closes it and frees the port for uploading.
echo.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\Monitor-Board.ps1" -Node door
pause
