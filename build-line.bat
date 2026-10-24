@echo off
setlocal EnableExtensions
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\build-line.ps1" %*
exit /b %errorlevel%
