#!/bin/bash 

# Install olsync
cd olsync
cargo build 
cp ./target/debug/olsync $HOME/.local/bin/olsync
cd ..

# Install olsync-rs-socketio-client
cd socketio-client
pipx install .
pipx runpip olsync-rs-socketio-client install -r requirements.txt
