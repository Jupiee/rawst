@echo off

set URL=https://github.com/Jupiee/rawst/releases/latest/download/rawst-x86_64-pc-windows-msvc.zip
set DESTINATION=C:\Users\%USERNAME%\rawst

curl -L "%URL%" -o "rawst-x86_64-pc-windows-msvc.zip"

if %errorlevel% neq 0 (
    echo Failed to download the file.
    exit /b 1
)

echo Creating install directory...

if not exist "%DESTINATION%" (
    mkdir "%DESTINATION%"
)

powershell -Command "Expand-Archive -Path 'rawst-x86_64-pc-windows-msvc.zip' -DestinationPath '%DESTINATION%'"

if %errorlevel% neq 0 (
    echo Failed to extract the contents.
    exit /b 1
)

echo %PATH% | find /I "%DESTINATION%" >nul

if errorlevel 1 (
    echo Adding rawst to PATH...

    setx PATH "%PATH%;%DESTINATION%" >nul
)

echo Installation completed successfully.