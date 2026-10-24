@echo off
setlocal EnableExtensions
title build - release binaries
cd /d "%~dp0"

rem ============================================================
rem  build.bat - build Rust axum (release) + Next.js (standalone)
rem  Keeps ONLY the built output in .\dist :
rem     dist\backend\backend.exe
rem     dist\frontend\            (server.js + .next\static + public)
rem  Usage: build.bat
rem ============================================================

set "ROOT=%~dp0"
set "DIST=%~dp0dist"
set "BIN_NAME=backend"

where cargo >nul 2>nul
if errorlevel 1 goto :no_cargo
where npm >nul 2>nul
if errorlevel 1 goto :no_npm
if not exist "%ROOT%backend\Cargo.toml" goto :no_backend
if not exist "%ROOT%frontend\package.json" goto :no_frontend

echo === Clean dist ===
rem Do not delete Linux releases created by build-line.bat.
if exist "%DIST%\backend" rmdir /s /q "%DIST%\backend"
if exist "%DIST%\frontend" rmdir /s /q "%DIST%\frontend"
mkdir "%DIST%\backend"
mkdir "%DIST%\frontend"

rem ---------- [1/2] backend ----------
echo.
echo === [1/2] Build Rust axum (release) ===
pushd "%ROOT%backend"
cargo build --release
set "RC=%errorlevel%"
popd
if not "%RC%"=="0" goto :fail

if not exist "%ROOT%backend\target\release\%BIN_NAME%.exe" goto :no_binary
copy /y "%ROOT%backend\target\release\%BIN_NAME%.exe" "%DIST%\backend\" >nul
if errorlevel 1 goto :fail

rem ---------- [2/2] frontend ----------
echo.
echo === [2/2] Build Next.js (standalone) ===
pushd "%ROOT%frontend"
if exist package-lock.json (set "NPM_CMD=ci") else (set "NPM_CMD=install")
call npm %NPM_CMD%
if errorlevel 1 goto :fail_pop
call npm run build
if errorlevel 1 goto :fail_pop
popd

if not exist "%ROOT%frontend\.next\standalone\server.js" goto :no_standalone

rem robocopy exit codes 0-7 = success, 8 or more = failure
robocopy "%ROOT%frontend\.next\standalone" "%DIST%\frontend" /E /NFL /NDL /NJH /NJS /NP >nul
if errorlevel 8 goto :fail
robocopy "%ROOT%frontend\.next\static" "%DIST%\frontend\.next\static" /E /NFL /NDL /NJH /NJS /NP >nul
if errorlevel 8 goto :fail
if exist "%ROOT%frontend\public" robocopy "%ROOT%frontend\public" "%DIST%\frontend\public" /E /NFL /NDL /NJH /NJS /NP >nul
if errorlevel 8 goto :fail

echo.
echo ============================================================
echo  BUILD OK  -  output in: %DIST%
echo.
echo  Run backend  : dist\backend\backend.exe
echo  Run frontend : node dist\frontend\server.js   (set PORT / HOSTNAME as needed)
echo ============================================================
exit /b 0

rem ---------- errors ----------
:fail_pop
popd
:fail
echo.
echo [ERROR] Build failed.
exit /b 1

:no_cargo
echo [ERROR] cargo not found in PATH. Install Rust from https://rustup.rs
exit /b 1

:no_npm
echo [ERROR] npm not found in PATH. Install Node.js LTS from https://nodejs.org
exit /b 1

:no_backend
echo [ERROR] backend\Cargo.toml not found. See prompts\00-bootstrap.md
exit /b 1

:no_frontend
echo [ERROR] frontend\package.json not found. See prompts\00-bootstrap.md
exit /b 1

:no_binary
echo [ERROR] backend\target\release\%BIN_NAME%.exe not found. The Cargo package name must be "%BIN_NAME%".
exit /b 1

:no_standalone
echo [ERROR] frontend\.next\standalone\server.js not found. Set output: "standalone" in frontend\next.config.ts
exit /b 1
