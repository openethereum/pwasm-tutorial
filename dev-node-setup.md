## Development node setup

Wasm support isn't enabled by default and needs to be specified in a "chainspec" file. `wasmActivationTransition` param sets a block number Wasm support should be activated. This is a sample "development chain" spec with Wasm enabled (based on https://paritytech.github.io/wiki/Private-development-chain):

[Source](https://github.com/paritytech/pwasm-tutorial/tree/master/wasm-dev-chain.json)
```json
{
    "name": "DevelopmentChain",
    "engine": {
        "instantSeal": null
    },
    "params": {
        "wasmActivationTransition": "0x01",
        "gasLimitBoundDivisor": "0x0400",
        "accountStartNonce": "0x0",
        "maximumExtraDataSize": "0x20",
        "minGasLimit": "0x1388",
        "networkID" : "0x11"
    },
    "genesis": {
        "seal": {
            "generic": "0x0"
        },
        "difficulty": "0x20000",
        "author": "0x0000000000000000000000000000000000000000",
        "timestamp": "0x00",
        "parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "extraData": "0x",
        "gasLimit": "0x5B8D80"
    },
    "accounts": {
        "0000000000000000000000000000000000000001": { "balance": "1", "builtin": { "name": "ecrecover", "pricing": { "linear": { "base": 3000, "word": 0 } } } },
        "0000000000000000000000000000000000000002": { "balance": "1", "builtin": { "name": "sha256", "pricing": { "linear": { "base": 60, "word": 12 } } } },
        "0000000000000000000000000000000000000003": { "balance": "1", "builtin": { "name": "ripemd160", "pricing": { "linear": { "base": 600, "word": 120 } } } },
        "0000000000000000000000000000000000000004": { "balance": "1", "builtin": { "name": "identity", "pricing": { "linear": { "base": 15, "word": 3 } } } },
        "0x004ec07d2329997267ec62b4166639513386f32e": { "balance": "1606938044258990275541962092341162602522202993782792835301376" }
    }
}
```
Run node:
```bash
parity --chain ./wasm-dev-chain.json --jsonrpc-apis=all
```

Among with other things we've set balance for `0x004ec07d2329997267ec62b4166639513386f32e` account on which behalf we'll run transactions (such as deploy). This should add an above account to the keychain:

```bash
curl --data '{"jsonrpc":"2.0","method":"parity_newAccountFromPhrase","params":["user", "user"],"id":0}' -H "Content-Type: application/json" -X POST localhost:8545
```
Should output something like:
```json
{"jsonrpc":"2.0","result":"0x004ec07d2329997267ec62b4166639513386f32e","id":0}
```
