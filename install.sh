#!/bin/bash 

# Install olsync
cargo build 
cp ./target/debug/olsync $HOME/.local/bin/olsync

# Install olsync-rs-socketio-client
cd socketio-client
pipx install .
pipx runpip olsync-rs-socketio-client install -r requirements.txt
