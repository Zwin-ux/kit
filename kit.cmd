@echo off
setlocal
REM Repo-root shim: Rust Control Room (1.0), not the Node 0.1 skill launcher.
set "KIT_ROOT=%~dp0"
set "KIT_BIN=%KIT_ROOT%target\release\kit.exe"
if not exist "%KIT_BIN%" set "KIT_BIN=%KIT_ROOT%target\debug\kit.exe"
if not exist "%KIT_BIN%" (
  echo kit: Rust binary not built.
  echo   cargo build -p kit-cli --release
  echo Then run: kit --demo
  exit /b 1
)
"%KIT_BIN%" %*
exit /b %ERRORLEVEL%
