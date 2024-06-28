#!/bin/bash 

cargo build 
cp ./target/debug/olsync $HOME/.local/bin/olsync
