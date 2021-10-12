cls
rem set DATABASE_URL=sqlite://data/all.db
cd ..\frontend
del dist\*
trunk build
@REM trunk build --release
cd dist
powershell -Command "(Get-Content index.html) -replace '\"/', '\"asset/' -replace \"'/\", \"'asset/\" | Out-File -Encoding utf8 index.html"
cd ..
del ..\backend\src\resource\asset\*
move dist\index.html ..\backend\src\resource\page\
move dist\* ..\backend\src\resource\asset\
cargo b -vv