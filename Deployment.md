# Introduction

This document provides detailed instructions for the deployment of our software product. It aims to ensure a smooth and successful installation process, minimizing any potential errors or issues.

# Pre-requisites

Before beginning the deployment process, please ensure that the following requirements are met:

Operating System: [Linux]
Hardware Requirements: [Greater than 50G memory]
Software Dependencies: [Rust and golang compilation environment]

# Deployment Steps

Follow the steps outlined below to deploy the software:

Step 1: Compile

```
git clone https://github.com/zkMIPS/zkm-prover.git
cd zkm-prover
cargo build --release
```

Step 2: Configure stage and prover

stage
```
# Replace it with your IP address and port
addr = "0.0.0.0:50000"
# All prover node 
prover_addrs = ["192.168.0.1:50000", "192.168.0.2:50000"]
# Snark node
snark_addrs = ["192.168.0.3:50051"]
# The NFS file system path must be used, and all node configurations must be the same
base_dir = "/tmp/zkm/test/test_proof"
```

prover
```
# Replace it with your IP address and port
addr = "0.0.0.0:50000"
prover_addrs = []
snark_addrs = []
# The NFS file system path must be used, and all node configurations must be the same
base_dir = "/tmp/zkm/test/test_proof"
```

Step 3: Start the service

You need one stage server and multiple prover servers.

Start stage server like this:
```
export RUST_LOG=info; nohup ./target/release/service --config ./service/config/stage.toml > stage.out &
```
Now you can see the service-related logs in stage.out.

Start all prover servers on the corresponding nodes.
```
export RUST_LOG=info; nohup ./target/release/service --config ./service/config/prover.toml > prover.out &
```

# Post-deployment Considerations

After successful deployment, please consider the following points:

Check the program process exist.

Check the base_dir the same for all.

Check the base_dir located within the NFS file system path.