@echo off

set URL=https://github.com/Jupiee/rawst/releases/latest/download/rawst-x86_64-pc-windows-msvc.zip
set DESTINATION=C:\Users\%USERNAME%\AppData\Local\Microsoft\WindowsApps

curl -L "%URL%" -o "rawst-x86_64-pc-windows-msvc.zip"

if %errorlevel% neq 0 (
    echo Failed to download the file.
    exit /b 1
)

powershell -Command "Expand-Archive -Path 'rawst-x86_64-pc-windows-msvc.zip' -DestinationPath '%DESTINATION%'"

if %errorlevel% neq 0 (
    echo Failed to extract the contents.
    exit /b 1
)

echo Installation completed successfully.