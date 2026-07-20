@echo off
setlocal
REM Repo-root shim so `kit` works without a global install.
set "KIT_ROOT=%~dp0"
set "KIT_BIN=%KIT_ROOT%packages\cli\dist\bin.js"

if not exist "%KIT_BIN%" (
  echo kit: CLI not built. Run: pnpm build
  exit /b 1
)

node "%KIT_BIN%" %*
exit /b %ERRORLEVEL%
