@echo off
setlocal

echo [1/4] cargo fmt --check
cargo fmt --check
if errorlevel 1 exit /b 1

echo [2/4] cargo check --all-targets --all-features
cargo check --all-targets --all-features
if errorlevel 1 exit /b 1

echo [3/4] cargo test --all-features
cargo test --all-features
if errorlevel 1 exit /b 1

echo [4/4] cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings
if errorlevel 1 exit /b 1

echo.
echo Validation passed.
exit /b 0
