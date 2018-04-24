var Web3 = require("web3");
var fs = require("fs");
// Connect to our local node
var web3 = new Web3(new Web3.providers.HttpProvider("http://localhost:8545"));
// Setup default account
web3.eth.defaultAccount = "0x004ec07d2329997267ec62b4166639513386f32e";
// Unlock account
web3.eth.personal.unlockAccount(web3.eth.defaultAccount, "user");
// read JSON ABI
var abi = JSON.parse(fs.readFileSync("./target/json/TokenInterface.json"));
// convert Wasm binary to hex format
var codeHex = '0x' + fs.readFileSync("./target/pwasm_tutorial_contract.wasm").toString('hex');

var TokenContract = new web3.eth.Contract(abi, { data: codeHex, from: web3.eth.defaultAccount });
// Will create TokenContract with `totalSupply` = 10000000 and print a result
TokenContract.deploy({data: codeHex, arguments: [10000000]}).send({from: web3.eth.defaultAccount}).then((a) => console.log(a));
