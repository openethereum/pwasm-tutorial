var Web3 = require("web3");
var web3 = new Web3(new Web3.providers.HttpProvider("http://localhost:8540"));
var abiPath = "/Users/fro/parity/pwasm-token-example/compiled/TokenContract.json";
var codePath = "/Users/fro/parity/pwasm-token-example/compiled/token.wasm";
web3.eth.defaultAccount = "0x004ec07d2329997267ec62b4166639513386f32e";
var abi = JSON.parse(fs.readFileSync(abiPath));
var codeHex = '0x' + fs.readFileSync(codePath).toString('hex');
var token = new web3.eth.Contract(abi, { data: codeHex, from: web3.eth.defaultAccount });

token2.deploy({data: codeHex, arguments: [10000000]}).send({from: web3.eth.defaultAccount, gas: 411139000, gasPrice: '100000'}).then((a) => console.log(a));

var repoAbiPath = "/Users/fro/parity/pwasm-repo-contract/compiled/RepoContract.json";
var repoCodePath = "/Users/fro/parity/pwasm-repo-contract/compiled/repo.wasm";

var repoAbi = JSON.parse(fs.readFileSync(repoAbiPath));
var repoCodeHex = '0x' + fs.readFileSync(repoCodePath).toString('hex');
var Repo = new web3.eth.Contract(repoAbi, { data: repoCodeHex, from: web3.eth.defaultAccount });

Repo.deploy({data: repoCodeHex, arguments: ["0x32be343b94f860124dc4fee278fdcbd38c102d88", "0x32be343b94f860124dc4fee278fdcbd38c102d11", "0x32be343b94f860124dc4fee278fdcbd38c102d22", "0x32be343b94f860124dc4fee278fdcbd38c102d33", 10000, 10000, 1, 1000, 100000]}).send({from: web3.eth.defaultAccount, gas: 411139000, gasPrice: '100000'}).then((a) => console.log(a));
