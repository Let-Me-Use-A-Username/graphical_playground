@echo off
cd /d "%~dp0"
echo Starting Game...

set GAME_EXE=
if exist "graphical_playground.exe" set GAME_EXE=graphical_playground.exe
if exist "target\release\graphical_playground.exe" set GAME_EXE=target\release\graphical_playground.exe
if exist "target\debug\graphical_playground.exe" set GAME_EXE=target\debug\graphical_playground.exe

if "%GAME_EXE%"=="" (
    echo Error: Game executable not found in any expected location!
    pause
    exit /b 1
)

echo Found executable...

if not exist "assets" (
    echo Error: Assets folder not found!
    echo Please ensure all game files are present.
    pause
    exit /b 1
)

echo Found Assets...

"%GAME_EXE%"