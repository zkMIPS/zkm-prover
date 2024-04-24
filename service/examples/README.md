# Examples

## Hello world

* Compile the Go code to MIPS

Write your own hello.go, and compile with

```
$ GOOS=linux GOARCH=mips GOMIPS=softfloat go build hello.go -o /tmp/zkm/test/hello_world
```

* Adjust parameter request stage service

```
$ export RUST_LOG=info
$ cargo run --release --example stage
```

## [Minigeth](https://github.com/zkMIPS/cannon-mips)

* Compile the minigeth_mips Please use Golang1.20 version

```
$ git clone https://github.com/zkMIPS/cannon-mips
$ cd cannon-mips && make minigeth_mips
$ cp mipsevm/minigeth /tmp/zkm/test/
```

* Download the block and place it in the corresponding directory

```
$ mkdir -p /tmp/cannon
$ export BASEDIR=/tmp/cannon; minigeth/go-ethereum 13284491
$ mkdir -p /tmp/zkm/test/0_13284491
$ cp -R /tmp/cannon/0_13284491 /tmp/zkm/test
```

* Adjust parameter request stage service

```
$ export RUST_LOG=info
$ ELF_PATH=/tmp/zkm/test/minigeth BLOCK_NO=13284491 BLOCK_PATH=/tmp/zkm/test/0_13284491 SEG_SIZE=262144 cargo run --release --example stage
```


# Deployment 

* Generate config toml

```
$ cd ../config
$ bash gen_config.sh
```

And you can view more detail in [gen_config](../config/README.md).

* Use S3 to store your data

We provide the support of s3, and you can enable s3 support by setting base_dir to a s3 path such as `s3://{bucket}/{object}`.

Besides, you may need to configure some s3 configurations in your environment variables as below or read more details in [s3-configuration](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-configure.html).

```
$ export AWS_ACCESS_KEY_ID="test"
$ export AWS_SECRET_ACCESS_KEY="test"
$ export AWS_DEFAULT_REGION="us-east-1"
$ export AWS_ENDPOINT_URL="{{ENDPOINT_URL}}"
```

* Compile zkm-prover

```
cargo build --release
```

* Start prover_server.

```
# use prover1_tls.toml and prover2_tls.toml instead if tls is enabled
$ RUST_LOG=info ./target/release/service --config ./service/config/prover1.toml
$ RUST_LOG=info ./target/release/service --config ./service/config/prover2.toml
```

* Start stage_server.

```
# use stage_tls.toml instead if tls is enabled
RUST_LOG=info ./target/release/service --config ./service/config/stage.toml --stage
```

* Start example stage

```
# set CA_CERT_PATH, CERT_PATH and KEY_PATH if tls is enabled
RUST_LOG=info cargo run --release --example stage
```
