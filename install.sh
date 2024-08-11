#!/bin/bash 

GREEN='\033[1;32m'
YELLOW='\033[1;33m'
CYAN='\033[1;36m'
NC='\033[0m'

# Where olsync binary will be moved
binpath=$HOME/.local/bin

# Install olsync
cd olsync
cargo build --release
mkdir -p $binpath
cp ./target/release/olsync $binpath/olsync
cd ..

# Install olsync-rs-socketio-client
cd socketio-client
pipx install .
pipx runpip olsync-rs-socketio-client install -r requirements.txt

# After installation
echo -e "\n${GREEN}olsync has been installed successfuly!${NC}"
echo -e "\n${YELLOW}Make sure to add $binpath to your PATH.${NC}"
echo -e "\n${YELLOW}You can check whether everything went fine by trying ${CYAN}olsync --help${NC}"
