@echo off
setlocal EnableExtensions
title frontend - Ubuntu backend
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\run-frontend.ps1" %*
exit /b %errorlevel%
