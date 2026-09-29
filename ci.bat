@echo off
setlocal EnableExtensions EnableDelayedExpansion

set "CI_IGNORE_TESTS=0"
if /i "%~1"=="--ignore-test" set "CI_IGNORE_TESTS=1"
if /i "%~1"=="--ignore-tests" set "CI_IGNORE_TESTS=1"
if /i "%~2"=="--ignore-test" set "CI_IGNORE_TESTS=1"
if /i "%~2"=="--ignore-tests" set "CI_IGNORE_TESTS=1"

if /i "%~1"=="--clip" goto :clip
if /i "%~1"=="--internal" goto :run

:run
if /i "%~1"=="--internal" if not defined CI_STEP_LOG (
    set "CI_STEP_LOG=%TEMP%\minirust-ci-step-%RANDOM%.log"
)

echo =========================================
echo Running MiniRust CI Pipeline...
echo =========================================

echo.
echo [1/5] Checking formatting...
if /i "%~1"=="--internal" (
    cargo fmt --all -- --check > "%CI_STEP_LOG%" 2>&1
    set "CI_STEP_EXIT=!errorlevel!"
    type "%CI_STEP_LOG%"
) else (
    cargo fmt --all -- --check
    set "CI_STEP_EXIT=!errorlevel!"
)
if not "!CI_STEP_EXIT!"=="0" goto :error

echo.
echo [2/5] Running cargo check...
if /i "%~1"=="--internal" (
    cargo check --workspace --locked > "%CI_STEP_LOG%" 2>&1
    set "CI_STEP_EXIT=!errorlevel!"
    type "%CI_STEP_LOG%"
) else (
    cargo check --workspace --locked
    set "CI_STEP_EXIT=!errorlevel!"
)
if not "!CI_STEP_EXIT!"=="0" goto :error

echo.
echo [3/5] Running clippy...
if /i "%~1"=="--internal" (
    cargo clippy --workspace --all-targets --all-features --locked -- -D warnings > "%CI_STEP_LOG%" 2>&1
    set "CI_STEP_EXIT=!errorlevel!"
    type "%CI_STEP_LOG%"
) else (
    cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
    set "CI_STEP_EXIT=!errorlevel!"
)
if not "!CI_STEP_EXIT!"=="0" goto :error

echo.
if "!CI_IGNORE_TESTS!"=="1" (
    echo [4/5] Skipping tests...
    set "CI_STEP_EXIT=0"
) else (
    echo [4/5] Running tests...
    echo (Make sure Docker Desktop is running for database integration tests)
    if /i "%~1"=="--internal" (
    cargo test --workspace --locked --all-targets -- --test-threads=1 --nocapture > "%CI_STEP_LOG%" 2>&1
    set "CI_STEP_EXIT=!errorlevel!"
    type "%CI_STEP_LOG%"
) else (
        cargo test --workspace --locked --all-targets -- --test-threads=1 --nocapture
        set "CI_STEP_EXIT=!errorlevel!"
    )
    call :cleanup_docker
)
if not "!CI_STEP_EXIT!"=="0" goto :error

echo.
echo [5/5] Building workspace...
if /i "%~1"=="--internal" (
    cargo build --workspace --locked > "%CI_STEP_LOG%" 2>&1
    set "CI_STEP_EXIT=!errorlevel!"
    type "%CI_STEP_LOG%"
) else (
    cargo build --workspace --locked
    set "CI_STEP_EXIT=!errorlevel!"
)
if not "!CI_STEP_EXIT!"=="0" goto :error

echo.
call :cleanup_docker
echo.
echo =========================================
echo SUCCESS: All CI checks passed!
echo =========================================
if /i not "%~1"=="--internal" pause
exit /b 0

:clip
set "CI_STEP_LOG=%TEMP%\minirust-ci-step-%RANDOM%.log"
if "!CI_IGNORE_TESTS!"=="1" (
    call "%~f0" --internal --ignore-tests
) else (
    call "%~f0" --internal
)
set "CI_EXIT=%errorlevel%"

if "%CI_EXIT%"=="0" (
    echo.
    echo =========================================
    echo SUCCESS: CI passed. Nothing copied to clipboard.
    echo =========================================
    del "%CI_STEP_LOG%" >nul 2>&1
    exit /b 0
)

echo.
echo =========================================
echo ERROR: CI failed. Copying failed step output to clipboard...
echo =========================================
if exist "%CI_STEP_LOG%" (
    clip < "%CI_STEP_LOG%"
    if errorlevel 1 (
        echo ERROR: Failed to copy CI errors to clipboard.
        echo Make sure the Windows "clip" command is available.
        del "%CI_STEP_LOG%" >nul 2>&1
        exit /b %CI_EXIT%
    )
    echo CI errors copied to clipboard.
    del "%CI_STEP_LOG%" >nul 2>&1
) else (
    echo ERROR: Failed to find step error log.
)
exit /b %CI_EXIT%

:error
call :cleanup_docker
set "CI_EXIT=!CI_STEP_EXIT!"
echo.
echo =========================================
echo ERROR: Pipeline failed at the current step.
echo Please check the error messages above.
echo =========================================
if /i not "%~1"=="--internal" pause
exit /b %CI_EXIT%

:cleanup_docker
set "CONTAINERS_CLEANED=0"
for /f "tokens=*" %%i in ('docker ps -q --filter "label=org.testcontainers.managed-by=testcontainers" 2^>nul') do (
    docker rm -f %%i >nul 2>&1
    set "CONTAINERS_CLEANED=1"
)
if "!CONTAINERS_CLEANED!"=="1" (
    echo [Docker] Cleaned up test containers.
)
goto :eof
