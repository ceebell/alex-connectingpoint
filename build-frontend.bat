@echo off
setlocal EnableExtensions
cd /d "%~dp0"
rem Build the Next.js Linux/amd64 standalone release locally.
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\build-frontend.ps1" %*
exit /b %errorlevel%
