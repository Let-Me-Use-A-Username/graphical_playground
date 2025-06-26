@echo off
SET EXEPATH=Geometrical/

mkdir "%EXEPATH%"
copy "target\release\graphical_playground.exe" "%EXEPATH%"
copy "run.bat" "%EXEPATH%"
xcopy "assets" "%EXEPATH%assets\" /E /I /H /Y
copy "README.md" "%EXEPATH%"

REM powershell -command "Compress-Archive -Path '%EXEPATH%' -DestinationPath 'geometrical.zip' -Force"

echo Distribution package created: geometrical.zip
pause