@echo off
setlocal enabledelayedexpansion

REM Build release script for things-cost (Windows)
REM This script builds the project for Windows platforms

echo [INFO] Building things-cost for Windows...

REM Check if we're in the project root
if not exist "Cargo.toml" (
    echo [ERROR] Please run this script from the project root directory
    exit /b 1
)

REM Create output directory
if not exist "dist" mkdir dist

REM Get version from Cargo.toml
for /f "tokens=2 delims==" %%i in ('findstr "^version =" Cargo.toml') do (
    set VERSION=%%i
)
set VERSION=%VERSION:"=%
echo [INFO] Building version: %VERSION%

REM Build targets
set TARGETS=x86_64-pc-windows-msvc

REM Build for each target
for %%t in (%TARGETS%) do (
    echo [INFO] Building for target: %%t

    REM Install target if not installed
    rustup target list | findstr "%%t (installed)" >nul
    if errorlevel 1 (
        echo [INFO] Installing target: %%t
        rustup target add %%t
    )

    REM Build the project
    cargo build --release --target %%t
    if !errorlevel! neq 0 (
        echo [ERROR] Build failed for %%t
        exit /b 1
    )

    echo [INFO] Build successful for %%t

    REM Create archive
    set ARCHIVE_NAME=things-cost-v%VERSION%-%%t.zip

    cd target\%%t\release
    "C:\Program Files\7-Zip\7z.exe" a ..\..\..\dist\%ARCHIVE_NAME% things-cost.exe >nul
    cd ..\..\..

    echo [INFO] Created archive: dist\%ARCHIVE_NAME%

    REM Generate SHA256 checksum
    cd dist
    certutil -hashfile %ARCHIVE_NAME% SHA256 > %ARCHIVE_NAME%.sha256
    cd ..

    echo [INFO] Created checksum: dist\%ARCHIVE_NAME%.sha256
)

echo [INFO] All builds completed successfully!
echo [INFO] Output files are in: dist\

echo [INFO] Created files:
dir dist\