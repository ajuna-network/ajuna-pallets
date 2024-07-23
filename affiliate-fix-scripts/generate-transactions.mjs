// Import
import { ApiPromise, WsProvider } from '@polkadot/api';
import fs from 'fs';

// Read file
var data = fs.readFileSync('affiliatee-chains.csv').toString().trim().replace(/"/g, '');
var rows = data.split('\n');

var account_dict = {};

rows.forEach(row => {
    var accounts = row.split(',');
    var affiliatee = accounts[0];
    var chain = accounts.slice(1, -1);

    account_dict[affiliatee] = chain;
});

// Construct
const wsProvider = new WsProvider('wss://bajun.api.onfinality.io/public-ws');
const api = await ApiPromise.create({ provider: wsProvider });

console.log(api.genesisHash.toHex());

var force_set_list = [];

var batch_hex_list = [];

for (var acc in account_dict) {
    const call = api.tx.awesomeAvatars.forceSetAffiliateeState(acc, account_dict[acc]);
    force_set_list.push(call);

    if (force_set_list.length == 100) {
        const batch = api.tx.utility.batchAll(force_set_list);
        batch_hex_list.push(batch.toHex());
        force_set_list = []
    }
}

if (force_set_list.length > 0) {
    const batch = api.tx.utility.batchAll(force_set_list);
    batch_hex_list.push(batch.toHex());
}

fs.writeFileSync('encoded-call.txt', batch_hex_list.join('\n'));

process.exit(0)