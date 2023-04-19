#!/bin/sh

git clone https://github.com/KomaEc/crown.git
cd crown
cargo install --path .
cd ..
rm -rf crown 
export LD_LIBRARY_PATH=$(rustc --print sysroot)/lib
