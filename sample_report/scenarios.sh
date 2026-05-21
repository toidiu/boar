#!/bin/bash
set -e

# git pull;
# cargo build;

# Generate data
sudo ./target/debug/boar -d 50mb -c 100 --delay-ms 10 --rate-mbit 50;
sudo ./target/debug/boar -d 50mb -c 100 --delay-ms 25 --rate-mbit 50;
sudo ./target/debug/boar -d 50mb -c 100 --delay-ms 50 --rate-mbit 50;
sudo ./target/debug/boar -d 50mb -c 100 --delay-ms 100 --rate-mbit 50;
sudo ./target/debug/boar -d 50mb -c 100 --delay-ms 200 --rate-mbit 50;
