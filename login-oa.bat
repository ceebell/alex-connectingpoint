@echo off
setlocal EnableExtensions
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\login-oa.ps1" %*
exit /b %errorlevel%
