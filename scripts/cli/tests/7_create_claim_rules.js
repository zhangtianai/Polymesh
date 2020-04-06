// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
const assert = require("assert");
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");

// Sets the default exit code to fail unless the script runs successfully
process.exitCode = 1;

async function main() {
  
  const api = await reqImports.createApi();

  const testEntities = await reqImports.initMain(api);

  let master_keys = await reqImports.generateKeys( api, 5, "master" );

  let signing_keys = await reqImports.generateKeys( api, 5, "signing" );

  await reqImports.distributePoly( api, master_keys.concat(signing_keys), reqImports.transfer_amount, testEntities[0] );

  await reqImports.blockTillPoolEmpty(api);

  let issuer_dids = await reqImports.createIdentities( api, master_keys, testEntities[0] );

  await reqImports.addSigningKeys( api, master_keys, issuer_dids, signing_keys );

  await reqImports.authorizeJoinToIdentities( api, master_keys, issuer_dids, signing_keys );

  await reqImports.issueTokenPerDid( api, master_keys, issuer_dids, reqImports.prepend );

  await createClaimRules( api, master_keys, issuer_dids, reqImports.prepend );

  if (reqImports.fail_count > 0) {
    console.log("Failed");
  } else {
    console.log("Passed");
    process.exitCode = 0;
  }

  process.exit();
}

async function createClaimRules(api, accounts, dids, prepend) {
    
  for (let i = 0; i < dids.length; i++) {
    const ticker = `token${prepend}${i}`.toUpperCase();
    assert( ticker.length <= 12, "Ticker cannot be longer than 12 characters");
    
    let senderRules = reqImports.senderRules1(accounts[i].address);
    let receiverRules = reqImports.receiverRules1(accounts[i].address);

    
    let nonceObj = {nonce: reqImports.nonces.get(accounts[i].address)};
    const transaction = await api.tx.generalTm.addActiveRule(ticker, senderRules, receiverRules);
    const result = await reqImports.sendTransaction(transaction, accounts[i], nonceObj);  
    const passed = result.findRecord('system', 'ExtrinsicSuccess');
    if (passed) reqImports.fail_count--;
     
    reqImports.nonces.set( accounts[i].address, reqImports.nonces.get(accounts[i].address).addn(1));
  }
}

main().catch(console.error);
