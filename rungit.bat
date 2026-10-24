@echo off
setlocal EnableExtensions
cd /d "%~dp0"

rem ============================================================
rem  rungit.bat - git add + commit + push
rem  Usage: rungit.bat "commit message"
rem         rungit.bat            (will ask for the message)
rem         rungit.bat --setup "repository-url"
rem         rungit.bat --check    (read-only preflight)
rem ============================================================

where git >nul 2>nul
if errorlevel 1 goto :no_git
if /i "%~1"=="--setup" goto :setup
if /i "%~1"=="--check" goto :check

call :preflight
if errorlevel 1 exit /b 1

set MSG=%*
if defined MSG set "MSG=%MSG:"=%"
if defined MSG goto :have_msg
set /p "MSG=Commit message: "
if not defined MSG goto :no_msg

:have_msg
echo.
echo --- changes ---
git status --short
echo ---------------

git add -A
if errorlevel 1 goto :fail

git diff --cached --quiet
if not errorlevel 1 goto :nothing_to_commit

git diff --cached --name-only | findstr /i /x /l "revising-log.md" >nul
if errorlevel 1 echo [WARN] revising-log.md is not part of this commit - did you log the change?

git commit -m "%MSG%"
if errorlevel 1 goto :fail
goto :push

:nothing_to_commit
echo Nothing new to commit - pushing existing commits only.

:push
for /f "delims=" %%b in ('git rev-parse --abbrev-ref HEAD') do set "BRANCH=%%b"
echo.
echo Pushing branch: %BRANCH%
git push -u origin "%BRANCH%"
if errorlevel 1 goto :fail

echo.
echo Done.
exit /b 0

:setup
if "%~2"=="" goto :setup_usage
if exist ".git" goto :setup_remote
git init -b main
if errorlevel 1 goto :fail

:setup_remote
git remote get-url origin >nul 2>nul
if not errorlevel 1 goto :remote_exists
git remote add origin "%~2"
if errorlevel 1 goto :fail
echo [OK] Local Git repository and origin are configured.
echo Next: rungit.bat --check
echo Then: rungit.bat "commit message"
exit /b 0

:check
call :preflight
if errorlevel 1 exit /b 1
echo [OK] Git repository and origin are configured. No files were staged or committed.
echo [INFO] Remote access is checked when you push.
git status --short
exit /b 0

:preflight
if not exist ".git" goto :not_repo
git rev-parse --is-inside-work-tree >nul 2>nul
if errorlevel 1 goto :not_repo
git remote get-url origin >nul 2>nul
if errorlevel 1 goto :no_remote
exit /b 0

:not_repo
echo [ERROR] This folder has no Git repository yet.
echo Run once: rungit.bat --setup "repository-url"
echo Get the repository URL from its GitHub or GitLab page.
exit /b 1

:no_remote
echo [ERROR] Git is initialized, but origin is not configured. Nothing was staged or committed.
echo Run once: rungit.bat --setup "repository-url"
echo Get the repository URL from its GitHub or GitLab page.
exit /b 1

:setup_usage
echo Usage: rungit.bat --setup "repository-url"
exit /b 1

:remote_exists
echo [ERROR] Origin is already configured. Existing settings were kept.
echo Use rungit.bat --check to inspect the local repository.
exit /b 1

:no_git
echo [ERROR] Git is not installed or not available in PATH.
exit /b 1

:no_msg
echo [ERROR] Commit message is required.
exit /b 1

:fail
echo.
echo [ERROR] git command failed - see the message above.
exit /b 1
