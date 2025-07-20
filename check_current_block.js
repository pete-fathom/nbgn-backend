const ethers = require('ethers');

async function checkCurrentBlock() {
    const provider = new ethers.providers.JsonRpcProvider('https://arb1.arbitrum.io/rpc');
    const currentBlock = await provider.getBlockNumber();
    console.log('Current block:', currentBlock);
    console.log('Last indexed block: 359777755');
    console.log('Behind by:', currentBlock - 359777755, 'blocks');
}

checkCurrentBlock().catch(console.error);