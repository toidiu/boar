#!/bin/bash
set -e

# git pull;
# cargo build;

sudo ./target/debug/boar -d 5mb -c 5 --delay-ms 200 --rate-mbit 50;
# sudo ./target/debug/boar -d 5mb -c 100 --delay-ms 100 --rate-mbit 50;
# sudo ./target/debug/boar -d 50mb -c 100 --delay-ms 50 --rate-mbit 50;
# sudo ./target/debug/boar -d 50mb -c 100 --delay-ms 25 --rate-mbit 50;
# sudo ./target/debug/boar -d 500mb -c 1 --delay-ms 10 --rate-mbit 50;
