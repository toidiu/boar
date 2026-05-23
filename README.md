A testing utility for measuring networking protocols.

Demo:
- Go to [boar.toidiu.com](boar.toidiu.com)
- Load the reports in `sample_report` folder (load entire folder)

## Platform Support
**Linux**
Network simulation is created with tc (i.e. netem and htb).

```
sudo apt update
sudo apt install iproute2 ethtool
```

Or run it in containers: see [DOCKER.md](DOCKER.md).

**macOS**
Due to lack of tc support on macOS, there is not network simulation and the
results are less interesting. However, development is still possible.


## Getting started

```
git clone git@github.com:toidiu/boar.git
git submodule update --init
```

### Simulator

Build and run:
```
cargo test
cargo build

// Run simulation
// tc/netem requires sudo permission
sudo ./target/debug/boar
```

### Viewer

The standalone web app for viewing the reports.

Find the hosted version at: [boar.toidiu.com](boar.toidiu.com)

Prerequisites:
```
rustup target add wasm32-unknown-unknown
cargo install trunk

sudo apt install npm
```

Build and run:
```
cd Viewer
npm install
trunk build

trunk serve
```

### Debug

```
// kill all `http3` process
sudo pkill http3; ps aux | grep http

// run a command on the virtual namespace `ns_s1`
sudo ip netns exec ns_s1 sh -c "echo hi"

// run quiche-client on the virtual namespace `ns_c1`
sudo ip netns exec ns_c1 sh -c "RUST_LOG=info ../quiche/target/release/quiche-client https://test.com/stream-bytes/5000000 --no-verify --connect-to  10.55.10.1:9999 --idle-timeout 1"
```

## Tasks
- [x] render report output as a html page
- [ ] allow for running multiple server binaries
  - [ ] compare stats between different builds
- [ ] if there is no startup exit then there will be no data.. handle parsing

