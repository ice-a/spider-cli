@echo off
REM Reptool npm 发布脚本 (Windows)
REM 用法: npm-publish.bat [version]

set VERSION=%1
if "%VERSION%"=="" (
    for /f "tokens=2 delims== " %%a in ('findstr /r "^version" Cargo.toml') do set VERSION=%%~a
    set VERSION=%VERSION:"=%
)

echo === 发布 Reptool npm 包 v%VERSION% ===

echo [1/5] 构建当前平台...
cargo build --release
if errorlevel 1 exit /b 1

echo [2/5] 准备平台包...
if not exist npm\win32-x64\bin mkdir npm\win32-x64\bin
copy target\release\reptool.exe npm\win32-x64\bin\

echo [3/5] 更新版本号...
cd npm\win32-x64
call npm version %VERSION% --no-git-tag-version 2>nul
cd ..\..
cd npm\reptool
call npm version %VERSION% --no-git-tag-version 2>nul
cd ..\..

echo [4/5] 更新 optionalDependencies...
cd npm\reptool
node -e "const fs=require('fs');const p=JSON.parse(fs.readFileSync('package.json','utf8'));for(const d in p.optionalDependencies)p.optionalDependencies[d]='%VERSION%';fs.writeFileSync('package.json',JSON.stringify(p,null,2)+'\n');"
cd ..\..

echo [5/5] 发布到 npm...
cd npm

echo 发布 reptool-win32-x64@%VERSION%...
cd win32-x64
call npm publish --access public
cd ..

echo 发布 reptool@%VERSION%...
cd reptool
call npm publish --access public
cd ..

cd ..

echo.
echo === 发布完成 ===
echo npm: https://www.npmjs.com/package/reptool
echo 用户安装: npm install -g reptool@%VERSION%