#!/bin/bash
# author:FlyingBlackShark
clear
cd ../frontend
rm -rf dist
trunk build --release
cd dist
sed -i  's?"/?"/asset/?g' index.html
sed -i  "s?'/?'/asset/?g" index.html
cd ..
rm -rf ../backend/src/resource/asset/*
mv dist/index.html ../backend/src/resource/page/
cp -r dist/* ../backend/src/resource/asset/
cd ../backend
cargo build --release