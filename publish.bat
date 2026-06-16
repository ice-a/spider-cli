@echo off
REM Reptool 发布脚本 (Windows)
REM 用法: publish.bat [version]

set VERSION=%1
if "%VERSION%"=="" (
    for /f "tokens=2 delims== " %%a in ('findstr /r "^version" Cargo.toml') do set VERSION=%%~a
    set VERSION=%VERSION:"=%
)

echo === 发布 Reptool v%VERSION% ===

echo [1/4] 构建 release...
cargo build --release
if errorlevel 1 exit /b 1

echo [2/4] 准备 npm 包...
if exist target\release\reptool.exe copy target\release\reptool.exe npm\bin\reptool.exe
if exist target\release\reptool.exe copy target\release\reptool.exe npm\bin\reptool-win-x64.exe

echo [3/4] 打包...
if exist reptool-win-x64.zip del reptool-win-x64.zip
if exist target\release\reptool.exe powershell -command "Compress-Archive -Path 'target\release\reptool.exe' -DestinationPath 'reptool-win-x64.zip'"

echo [4/4] 发布 npm...
cd npm
npm publish
cd ..

echo === 发布完成 ===
echo 用户安装: npm install -g reptool@%VERSION%
