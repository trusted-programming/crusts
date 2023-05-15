#!/bin/sh

git clone https://github.com/KomaEc/crown.git
cd crown
cargo build --release
cargo install --path .
cd ..
rm -rf crown 
