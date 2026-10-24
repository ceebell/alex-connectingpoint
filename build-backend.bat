@echo off
setlocal EnableExtensions
cd /d "%~dp0"
rem Compile the Rust Linux/amd64 release locally for Ubuntu Docker.
call "%~dp0build-line.bat" %*
exit /b %errorlevel%
