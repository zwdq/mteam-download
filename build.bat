@echo off
REM M-Team Query Tool Build Script
title M-Team Build Tool

echo.
echo ====================================================
echo    M-Team Query Tool - Build Script
echo ====================================================
echo.

REM Check Python
python --version >nul 2>&1
if errorlevel 1 (
    echo Error: Python not found
    echo Please install Python from: https://www.python.org/
    pause
    exit /b 1
)

echo Python detected
echo.

echo Building...
echo.
python build.py

if exist "dist\mteam-query.exe" (
    echo.
    echo ====================================================
    echo Build Success!
    echo ====================================================
    echo.
    echo Output: dist\mteam-query.exe
    echo Config: config.yaml
    echo.
    echo Usage:
    echo   1. Copy dist\mteam-query.exe to any folder
    echo   2. Copy config.yaml to same folder
    echo   3. Run mteam-query.exe
    echo.
    echo ====================================================
) else (
    echo.
    echo ====================================================
    echo Build Failed
    echo ====================================================
)

echo.
pause
