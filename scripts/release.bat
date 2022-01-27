cls
rem set DATABASE_URL=sqlite://data/all.db
cd ..\frontend
del /S /Q dist\*
rmdir /S /Q dist
trunk build --release
cd dist
powershell -Command "(Get-Content index.html) -replace '\"/', '\"/asset/' -replace \"'/\", \"'/asset/\" | Out-File -Encoding utf8 index.html"
cd ..
del /S/Q ..\backend\src\resource\asset\*
move dist\index.html ..\backend\src\resource\page\
xcopy /E dist\* ..\backend\src\resource\asset\
cd ..\backend
@REM cargo b -vv
cargo build --release