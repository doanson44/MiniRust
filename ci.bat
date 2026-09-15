@echo off
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
pause
exit /b 0

:error
echo.
echo =========================================
echo ERROR: Pipeline failed at the current step.
echo Please check the error messages above.
echo =========================================
pause
exit /b %errorlevel%
