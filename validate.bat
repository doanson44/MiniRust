@echo off
setlocal

set "CLIP=0"
if /I "%~1"=="--clip" set "CLIP=1"

set "LOG=%TEMP%\minirust-validate-%RANDOM%.log"

call :run "cargo fmt --check" "cargo fmt --check"
if errorlevel 1 goto :failed

call :run "cargo check --all-targets --all-features" "cargo check --all-targets --all-features"
if errorlevel 1 goto :failed

call :run "cargo test --all-features" "cargo test --all-features"
if errorlevel 1 goto :failed

call :run "cargo clippy --all-targets --all-features -- -D warnings" "cargo clippy --all-targets --all-features -- -D warnings"
if errorlevel 1 goto :failed

del "%LOG%" >nul 2>&1
echo.
echo Validation passed.
exit /b 0

:failed
if "%CLIP%"=="1" (
    type "%LOG%" | clip
    echo.
    echo Validation failed. Output copied to clipboard.
) else (
    echo.
    echo Validation failed.
)
del "%LOG%" >nul 2>&1
exit /b 1

:run
echo [%~1]
%~2 >"%LOG%" 2>&1
set "EXIT_CODE=%ERRORLEVEL%"
type "%LOG%"
exit /b %EXIT_CODE%
