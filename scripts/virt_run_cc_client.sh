#!/bin/bash
set -e

source scripts/virt_common.sh

# ========= args
SIZE_MB="${1:-10m}"
NS_NAME="${2:-$NS_C1}"
SERVER_IP="${3:-$IP_S1_EG}"
PORT="${4:-$PORT_S1}"
LOOPS=1

echo "Connection to port: $PORT";

if [[ -z "$NS_NAME" ]]; then
  echo "Must provide a namespace" 1>&2
  exit 1
fi

NS="ip netns exec $NS_NAME"
echo "Running client in ns: $NS_NAME"
echo "Request payload size: $SIZE_MB"
echo "Connecting to server ip: $SERVER_IP"
echo "Connection to port: $PORT";
# ========= args

# Build binary
cargo build --package cc-client

# Use `cdn-cgi/dummy/file/1m` to avoid delays caused by the mock http-bin
# connection
echo "Starting download...";



for i in $(seq 1 $LOOPS);
do
    # Use `cdn-cgi/dummy/file/1m` to avoid delays caused by the mock http-bin
    # connection
    sudo $NS ./target/debug/cc-client --addr $SERVER_IP:$PORT --url https://dummy.test.com/cdn-cgi/dummy/file/${SIZE_MB}?key=DUMMY_KEY --num-downloads 1 --http3
done
