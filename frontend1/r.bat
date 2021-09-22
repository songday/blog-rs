call b.bat
copy /Y src\asset\* pkg
miniserve ./pkg --index index.html