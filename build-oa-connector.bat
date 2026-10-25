@echo off
setlocal EnableExtensions
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\build-oa-connector.ps1" %*
exit /b %errorlevel%
