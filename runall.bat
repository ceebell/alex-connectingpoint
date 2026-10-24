@echo off
setlocal EnableExtensions
title runall - dev launcher

rem PowerShell handles paths, config checks and optional services.
rem ExecutionPolicy Bypass applies only to this launcher process.
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\runall.ps1" %*
exit /b %errorlevel%
