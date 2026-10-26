@echo off
setlocal EnableExtensions
cd /d "%~dp0"
rem ============================================================
rem  build-all.bat - build both Linux releases for Ubuntu in order:
rem    1. build-backend.bat   (Rust    -> dist\line-linux)
rem    2. build-frontend.bat  (Next.js -> dist\frontend-linux)
rem  Stops at the first failure. Requires Docker Desktop (Linux containers).
rem  Usage: build-all.bat
rem ============================================================

echo === [1/2] Backend: build-backend.bat ===
call "%~dp0build-backend.bat"
if errorlevel 1 goto :fail_backend

echo.
echo === [2/2] Frontend: build-frontend.bat ===
call "%~dp0build-frontend.bat"
if errorlevel 1 goto :fail_frontend

echo.
echo === Build complete ===
set "BACKEND_TAG="
set "FRONTEND_TAG="
if exist "%~dp0dist\line-linux\release.tag" set /p BACKEND_TAG=<"%~dp0dist\line-linux\release.tag"
if exist "%~dp0dist\frontend-linux\frontend.tag" set /p FRONTEND_TAG=<"%~dp0dist\frontend-linux\frontend.tag"
echo   backend : %BACKEND_TAG%
echo   frontend: %FRONTEND_TAG%
echo.
echo Next: rungit.bat "commit message"
echo Then on Ubuntu: bash ./loadgit.sh main ^&^& bash ./deploy-all.sh main
exit /b 0

:fail_backend
echo.
echo [ERROR] build-backend.bat failed. Frontend was not built.
exit /b 1

:fail_frontend
echo.
echo [ERROR] build-frontend.bat failed. The backend release in dist\line-linux is new.
exit /b 1
