# Compile

```
git clone https://github.com/zkMIPS/zkm-prover.git
cd zkm-prover
cargo build --release
```

# Deployment

## Deployment Prover

You can only deploy the provider service for other stage services to call

Configure
```
# Replace it with your IP address and port
addr = "0.0.0.0:50000"
prover_addrs = []
snark_addrs = []
# The NFS file system path / S3 must be used, and all node configurations must be the same
base_dir = "/tmp/zkm/test/test_proof"
```

Start
```
export RUST_LOG=info; nohup ./target/release/service --config ./service/config/prover.toml > prover.out &
```

## Deployment Stage

Configure
```
# Replace it with your IP address and port
addr = "0.0.0.0:50000"
# All prover node 
prover_addrs = ["192.168.0.1:50000", "192.168.0.2:50000"]
# Snark node
snark_addrs = ["192.168.0.3:50051"]
# The NFS file system path / S3 must be used, and all node configurations must be the same
base_dir = "/tmp/zkm/test/test_proof"
```

Start
```
export RUST_LOG=info; nohup ./target/release/service --config ./service/config/stage.toml --stage > stage.out &
```