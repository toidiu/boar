A testing utility for measuring networking protocols.

## Platform Support
**Linux**
- Network simulation is created with tc (i.e. netem and htb).

**macOS**
- Due to lack of tc support on macOS, there is not network simulation and the
results are less interesting. However, development is still possible.


## Run

```
cargo build;

// tc/netem requires sudo permission
sudo ./target/debug/boar
```

## Debug

```
// kill all `http3` process
sudo pkill http3; ps aux | grep http

// run a command on the virtual namespace `ns_s1`
sudo ip netns exec ns_s1 sh -c "echo hi"

// run quiche-client on the virtual namespace `ns_c1`
sudo ip netns exec ns_c1 sh -c "RUST_LOG=info ../quiche/target/release/quiche-client https://test.com/stream-bytes/5000000 --no-verify --connect-to  10.55.10.1:9999 --idle-timeout 1"
```
