#!/bin/bash

# You should provide some variable to use this config bash
provers=("localhost:50001" "localhost:50002")
stage="localhost:50000"
snarks=("localhost:50051")
tls=false
base_dir="/tmp/zkm/test/test_proof"

# Generate tls certs
if [ "$tls" = true ]; then
    IFS=':' read -r host port <<< "$stage"
    cd ./../../tools/certs
    bash certgen.sh --cn stage --ssl-dns $host
    rm -rf stage.csr
    id=1
    for prover in "${provers[@]}"; do
        prover_name="prover${id}"
        IFS=':' read -r host port <<< "$prover"
        bash certgen.sh --cn $prover_name --ssl-dns $host
        rm -rf ${prover_name}.csr
        ((id++))
    done
    rm -rf ca.srl
    cd -
fi

# Generate stage toml
# Read templeta content first
if [ "$tls" = true ]; then
    stage_template_content=$(cat stage_tls_template.toml)
else
    stage_template_content=$(cat stage_template.toml)
fi
stage_config="$stage_template_content"
stage_config="${stage_config//\{\{addr\}\}/${stage}}"
# generate prover addrs
prover_addrs=""
for prover in "${provers[@]}"; do
    if [ -z "$result" ]; then
        prover_addrs="$prover"
    else
        prover_addrs="$prover_addrs, \"$prover\""
    fi
done
stage_config="${stage_config//\{\{prover_addrs\}\}/\"${prover_addrs}\"}"
# generate snark addrs
snark_addrs=""
for snark in "${snarks[@]}"; do
    if [ -z "$result" ]; then
        snark_addrs="$snark"
    else
        snark_addrs="$prover_addrs, \"$snark\""
    fi
done
stage_config="${stage_config//\{\{snark_addrs\}\}/\"${snark_addrs}\"}"
if [ "$tls" = true ]; then
    echo "$stage_config" > stage_tls.toml 
else
    echo "$stage_config" > stage.toml 
fi

# Generate provers toml
# Read templeta content first
if [ "$tls" = true ]; then
    prover_template_content=$(cat prover_tls_template.toml)
else
    prover_template_content=$(cat prover_template.toml)
fi

id=1
for prover in "${provers[@]}"; do
    if [ "$tls" = true ]; then
        prover_path="prover${id}_tls.toml"
    else
        prover_path="prover${id}.toml"
    fi
    IFS=':' read -r host port <<< "$prover"
    prover_config="$prover_template_content"
    addr="0.0.0.0:${port}"
    prover_config="${prover_config//\{\{addr\}\}/${addr}}"
    prover_config="${prover_config//\{\{prover_addrs\}\}/\"${addr}\"}"
    prover_config="${prover_config//\{\{base_dir\}\}/${base_dir}}"
    prover_config="${prover_config//\{\{prover_name\}\}/prover${id}}"
    if [ "$tls" = true ]; then
        echo "$prover_config" > "prover${id}_tls.toml"
    else
        echo "$prover_config" > "prover${id}.toml"
    fi
    ((id++))
done
