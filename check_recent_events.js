const Web3 = require('web3');

async function checkRecentEvents() {
    const web3 = new Web3('https://arb1.arbitrum.io/rpc');
    const contractAddress = '0x66Eb0Aa46827e5F3fFcb6Dea23C309CB401690B6';
    
    // Get current block
    const currentBlock = await web3.eth.getBlockNumber();
    console.log('Current block:', currentBlock);
    
    // Look for VoucherCreated events in last 50 blocks
    const fromBlock = currentBlock - 50;
    const logs = await web3.eth.getPastLogs({
        address: contractAddress,
        fromBlock: fromBlock,
        toBlock: currentBlock
    });
    
    console.log(`Found ${logs.length} events from block ${fromBlock} to ${currentBlock}`);
    
    // VoucherCreated signature: keccak256("VoucherCreated(bytes32,address,uint256)")
    const voucherCreatedSig = '0x' + require('crypto').createHash('sha3-256').update('VoucherCreated(bytes32,address,uint256)').digest('hex');
    console.log('VoucherCreated signature:', voucherCreatedSig);
    
    for (const log of logs) {
        console.log('\nEvent:');
        console.log('  Block:', log.blockNumber);
        console.log('  Topic0:', log.topics[0]);
        console.log('  Tx:', log.transactionHash);
        
        if (log.topics[0] === voucherCreatedSig) {
            console.log('  -> This is a VoucherCreated event!');
            console.log('  -> Voucher ID:', log.topics[1]);
            console.log('  -> Creator:', '0x' + log.topics[2].slice(26));
        }
    }
}

checkRecentEvents().catch(console.error);