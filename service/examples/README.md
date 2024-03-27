# Examples

## Tools

* Compile the Go code to MIPS

Write your own hello.go, and compile with

```
GOOS=linux GOARCH=mips GOMIPS=softfloat go build hello.go -o /tmp/zkm/test/hello_world
```

Download the block and place it in the corresponding directory [minigeth](https://github.com/zkMIPS/cannon-mips).

```
$ mkdir -p /tmp/cannon
$ export BASEDIR=/tmp/cannon; minigeth/go-ethereum 13284491
$ mkdir -p /tmp/zkm/test/0_13284491
$ cp /tmp/cannon/0_13284491/input /tmp/zkm/test/0_13284491
```

* Compile zkm-prover

```
cargo build --release
```

* Start prover_server.

```
# use prover1_tls.toml and prover2_tls.toml instead if tls is enabled
$ ./target/release/service --config ./service/config/prover1.toml
$ ./target/release/service --config ./service/config/prover2.toml
```

* Start stage_server.

```
# use stage_tls.toml instead if tls is enabled
./target/release/service --config ./service/config/stage.toml
```

* Start example stage

```
# set CA_CERT_PATH, CERT_PATH and KEY_PATH if tls is enabled
cargo run --release --example stage
```


