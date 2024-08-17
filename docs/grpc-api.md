# Public Grpc API for zkm-prover service

## General API Information
* The following base endpoints are available.
    * **https://152.32.186.45:20002**

## Proto Link

Please refer to [Stage Proto](../service/proto/src/proto/stage/v1/stage.proto).

## Status Codes

* `SUCCESS` The operation has been successful.
* `UNSPECIFIED` There was an internal timeout during the operation, which can be queried later through `GetStatusRequest`.
* `COMPUTING` The task is currently being processed.
* `INVALID_PARAMETER` Input parameter error, and `error_message` may be helpful.
* `INTERNAL_ERROR` Task execution failed due to internal reasons, please try again later.
* `SPLIT_ERROR` Task execution failed due to execute `elf`, please check `elf` file.
* `PROVE_ERROR` Task execution failed due to prove.
* `AGG_ERROR` Task execution failed due to aggregate.
* `FINAL_ERROR` Task execution failed due to generate snark proof.
  **UNKNOWN** and could have been a success.

## General Info on Limits

proving service only provide services to whitelist users, be sure to use the correct signature.

## GenerateProof

### GenerateProofRequest
**Parameters:**
Name | Type | Mandatory | Description
------------ | ------------ | ------------ | ------------
chain_id | UINT64 | NO |
timestamp | UINT64 | NO | Current timestamp.
proof_id | STRING | YES | Generate a unique ID using UUID.
elf_data | BYTES | YES | Executable files under MIPS architecture.
block_data | VECTOR | NO | When use minigeth required.
block_no | UINT64 | NO | When use minigeth required.
seg_size | UINT32 | NO | Segment size[65536, 262144].
args | STRING | NO | ARGS for `elf_data`.
signature | STRING | YES | Signature.
public_input_stream | BYTES | NO | Public input, Will be passed as the first parameter to the `elf_data`.
private_input_stream | BYTES | NO | private input, Will be passed as the second parameter to the `elf_data`.
execute_only | BOOL | NO | Default false.


### GenerateProofResponse

Name | Type | Mandatory | Description
------------ | ------------ | ------------ | ------------
status | UINT32 | YES | Status Codes.
error_message | STRING | NO |
proof_id | STRING | YES | Request.proof_id.
proof_url | STRING | YES | After the task is completed, you can download the snark proof from this URL.
stark_proof_url | STRING | YES | After the task is completed, you can download the stark proof from this URL.
solidity_verifier_url | STRING | YES | After the task is completed, you can download the verifier's contract from this URL.
output_stream | BYTES | NO | Guest program output.

## GetStatus

### GetStatusRequest
**Parameters:**
Name | Type | Mandatory | Description
------------ | ------------ | ------------ | ------------
proof_id | STRING | YES | Proof id to be queried.

### GetStatusResponse

Name | Type | Mandatory | Description
------------ | ------------ | ------------ | ------------
proof_id | STRING | YES | Request.proof_id.
status | UINT32 | YES | Status Codes.
proof_with_public_inputs | BYTES | NO | Proof of binary data.
proof_url | STRING | YES | After the task is completed, you can download the snark proof from this URL.
stark_proof_url | STRING | YES | After the task is completed, you can download the stark proof from this URL.
solidity_verifier_url | STRING | YES | After the task is completed, you can download the verifier's contract from this URL.
output_stream | BYTES | NO | Guest program output.