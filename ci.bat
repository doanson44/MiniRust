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
if /i "%~1"=="--internal" (
    cargo fmt --all -- --check > "%CI_STEP_LOG%" 2>&1
    type "%CI_STEP_LOG%"
) else (
    cargo fmt --all -- --check
)
if %errorlevel% neq 0 goto :error

echo.
echo [2/5] Running cargo check...
if /i "%~1"=="--internal" (
    cargo check --workspace --locked > "%CI_STEP_LOG%" 2>&1
    type "%CI_STEP_LOG%"
) else (
    cargo check --workspace --locked
)
if %errorlevel% neq 0 goto :error

echo.
echo [3/5] Running clippy...
if /i "%~1"=="--internal" (
    cargo clippy --workspace --all-targets --all-features --locked -- -D warnings > "%CI_STEP_LOG%" 2>&1
    type "%CI_STEP_LOG%"
) else (
    cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
)
if %errorlevel% neq 0 goto :error

echo.
echo [4/5] Running tests...
echo (Make sure Docker Desktop is running for the database integration tests)
if /i "%~1"=="--internal" (
    cargo test --workspace --locked --all-targets > "%CI_STEP_LOG%" 2>&1
    type "%CI_STEP_LOG%"
) else (
    cargo test --workspace --locked --all-targets
)
if %errorlevel% neq 0 goto :error

echo.
echo [5/5] Building workspace...
if /i "%~1"=="--internal" (
    cargo build --workspace --locked > "%CI_STEP_LOG%" 2>&1
    type "%CI_STEP_LOG%"
) else (
    cargo build --workspace --locked
)
if %errorlevel% neq 0 goto :error

echo.
echo =========================================
echo SUCCESS: All CI checks passed!
echo =========================================
if /i not "%~1"=="--internal" pause
exit /b 0

:clip
set "CI_LOG=%TEMP%\minirust-ci-%RANDOM%.log"
set "CI_ERR=%TEMP%\minirust-ci-%RANDOM%.err"
set "CI_STEP_LOG=%TEMP%\minirust-ci-step-%RANDOM%.log"
call "%~f0" --internal > "%CI_LOG%" 2> "%CI_ERR%"
set "CI_EXIT=%errorlevel%"

type "%CI_LOG%"
type "%CI_ERR%"

if "%CI_EXIT%"=="0" (
    echo.
    echo =========================================
    echo SUCCESS: CI passed. Nothing copied to clipboard.
    echo =========================================
    del "%CI_LOG%" >nul 2>&1
    del "%CI_ERR%" >nul 2>&1
    exit /b 0
)

echo.
echo =========================================
echo ERROR: CI failed. Copying failed step output to clipboard...
echo =========================================
if exist "%CI_STEP_LOG%" (
    clip < "%CI_STEP_LOG%"
) else (
    type "%CI_ERR%" | clip
)
if errorlevel 1 (
    echo ERROR: Failed to copy CI errors to clipboard.
    echo Make sure the Windows "clip" command is available.
    del "%CI_LOG%" >nul 2>&1
    del "%CI_ERR%" >nul 2>&1
    del "%CI_STEP_LOG%" >nul 2>&1
    exit /b %CI_EXIT%
)

echo CI errors copied to clipboard.
del "%CI_LOG%" >nul 2>&1
del "%CI_ERR%" >nul 2>&1
del "%CI_STEP_LOG%" >nul 2>&1
exit /b %CI_EXIT%

:error
set "CI_EXIT=%errorlevel%"
echo.
echo =========================================
echo ERROR: Pipeline failed at the current step.
echo Please check the error messages above.
echo =========================================
if /i not "%~1"=="--internal" pause
if /i "%~1"=="--internal" (
    type "%CI_STEP_LOG%"
)
exit /b %CI_EXIT%
