@echo off
setlocal

if /i "%~1"=="--clip" goto :clip
if /i "%~1"=="--internal" goto :run

:run
echo =========================================
echo Running MiniRust CI Pipeline...
echo =========================================
echo.

echo [1/5] Checking formatting...
cargo fmt --all -- --check
if %errorlevel% neq 0 goto :error

echo.
echo [2/5] Running cargo check...
cargo check --workspace --locked
if %errorlevel% neq 0 goto :error

echo.
echo [3/5] Running clippy...
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
if %errorlevel% neq 0 goto :error

echo.
echo [4/5] Running tests...
echo (Make sure Docker Desktop is running for the database integration tests)
cargo test --workspace --locked --all-targets
if %errorlevel% neq 0 goto :error

echo.
echo [5/5] Building workspace...
cargo build --workspace --locked
if %errorlevel% neq 0 goto :error

echo.
echo =========================================
echo SUCCESS: All CI checks passed!
echo =========================================
if /i not "%~1"=="--internal" pause
exit /b 0

:clip
set "CI_LOG=%TEMP%\minirust-ci-%RANDOM%.log"
call "%~f0" --internal > "%CI_LOG%" 2>&1
set "CI_EXIT=%errorlevel%"

type "%CI_LOG%"

if "%CI_EXIT%"=="0" (
    echo.
    echo =========================================
    echo SUCCESS: CI passed. Nothing copied to clipboard.
    echo =========================================
    del "%CI_LOG%" >nul 2>&1
    exit /b 0
)

echo.
echo =========================================
echo ERROR: CI failed. Copying output to clipboard...
echo =========================================
clip < "%CI_LOG%"
if %errorlevel% neq 0 (
    echo ERROR: Failed to copy CI output to clipboard.
    echo Make sure the Windows "clip" command is available.
    del "%CI_LOG%" >nul 2>&1
    exit /b %CI_EXIT%
)

echo CI output copied to clipboard.
del "%CI_LOG%" >nul 2>&1
exit /b %CI_EXIT%

:error
echo.
echo =========================================
echo ERROR: Pipeline failed at the current step.
echo Please check the error messages above.
echo =========================================
if /i not "%~1"=="--internal" pause
exit /b %errorlevel%
