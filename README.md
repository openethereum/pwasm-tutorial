This tutorial cover steps how to compile contract written in Rust to Wasm module, pack it as constructor and deploy on the chain.

## Compilation

First we need `wasm32-unknown-unknown` target to be installed:

```
rustup update
rustup target add wasm32-unknown-unknown --toolchain nightly
```

Also we need to install a build tool from https://github.com/paritytech/wasm-utils which do pack Wasm into contract constructor and optimize size of the final wasm trowing away all code which is unused:

```
cargo install --git https://github.com/paritytech/wasm-utils wasm-build
```

Now let's compile a sample ERC-20 contract. Clone https://github.com/paritytech/pwasm-token-example somewhere and run:

```
cd pwasm-token-example
cargo build --release --target wasm32-unknown-unknown
```

It will produce a `token.wasm` under `./target/wasm32-unknown-unknown`. Thanks to the  [pwasm-abi crate](https://github.com/paritytech/pwasm-abi) it'll also generate the Solidity ABI for us and save as `TokenContract.json` under `./target/json` directory.

Now run:

```
wasm-build --target wasm32-unknown-unknown ./target token // "token" is the name of *wasm binary to find
```

`wasm-build` will look for compiled Wasm under the `./target/wasm32-unknown-unknown` dir and put the final wasm into the `./target/token.wasm`.

In order to successfully build the final deployable contract Wasm module should export 2 functions: `call` and `deploy` (see https://github.com/paritytech/pwasm-token-example/blob/master/src/token.rs).
 The `call` export used to call methods on the deployed contract while `deploy` should contain the initialization code. `wasm-build` optimise Wasm over `call` function, puts it's binary code it into static data segment, renames `deploy` export to `call` and makes it return a pointer to and length of that static data segment.

## Node setup

Let's deploy packed `token.wasm` on the chain. For purposes of tutorial we will setup a simple Development chain. You'll need to use a custom development chain spec with `params: {"wasm": true}` to enable the Wasm support in parity:

```json
{
	"name": "DevelopmentChain",
	"engine": {
		"instantSeal": { "params": {} }
	},
	"params": {
		"wasm": true,
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

Save it as `dev-wasm-spec.json` in the current directory and run:
```
parity --chain dev-wasm-spec.json --jsonrpc-apis all
```

Among with other things we've added an account `0x004ec07d2329997267ec62b4166639513386f32e` with some ETH to `dev-wasm-spec.json` to be able to make transactions. Now we need to add this account to the local keychain:

```
curl --data '{"jsonrpc":"2.0","method":"parity_newAccountFromPhrase","params":["user", "user"],"id":0}' -H "Content-Type: application/json" -X POST localhost:8545
```
Should output something like:
```
{"jsonrpc":"2.0","result":"0x004ec07d2329997267ec62b4166639513386f32e","id":0}
```

For future convenience we'll restart the node with unlocked account. To do so you need to pass an additional params: `--unlock 0x004ec07d2329997267ec62b4166639513386f32e` and `--password passwd` where `passwd` is a file containing an `0x004ec07d2329997267ec62b4166639513386f32e`'s password:
```
user
```

So now you can restart your node with:
```
parity --chain dev-wasm-spec.json --unlock 0x004ec07d2329997267ec62b4166639513386f32e --password passwd --jsonrpc-apis all
```

## Deployment

In order to connect to node and deploy our contract we'll use (NodeJS)[https://nodejs.org/en/] and (Web3 library)[https://github.com/ethereum/web3.js/]. Make sure you have installed either.

Now run `node` in the terminal. Step by step
```
var Web3 = require("web3");

// Connect to our local node
var web3 = new Web3(new Web3.providers.HttpProvider("http://localhost:8545"));

// Setup default account
web3.eth.defaultAccount = "0x004ec07d2329997267ec62b4166639513386f32e";

var abiPath = "/path/to/compiled/TokenContract.json";
var codePath = "/path/to/compiled/pwasm-token-example/compiled/token.wasm";

// Load an ABI object from JSON file
var abi = JSON.parse(fs.readFileSync(abiPath));

// Reads from codePath and converts in to hex format
var codeHex = '0x' + fs.readFileSync(codePath).toString('hex');

//
var Token = new web3.eth.Contract(abi, { data: codeHex, from: web3.eth.defaultAccount });

//
token2.deploy({data: codeHex, arguments: [10000000]}).send({from: web3.eth.defaultAccount, gas: 411139000, gasPrice: '100000'}).then((a) => console.log(a));

```
