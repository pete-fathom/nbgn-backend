const ethers = require('ethers');

async function checkVoucher() {
    const provider = new ethers.providers.JsonRpcProvider('https://arb1.arbitrum.io/rpc');
    const contractAddress = '0x66Eb0Aa46827e5F3fFcb6Dea23C309CB401690B6';
    
    const abi = ['function vouchers(bytes32) view returns (address creator, uint256 amount, bool claimed)'];
    const contract = new ethers.Contract(contractAddress, abi, provider);
    
    const voucherId = '0xee878f76d46f61ccec0a4eddbaf5027640cdea816ab5767a7d5a947ebee5ecba';
    const result = await contract.vouchers(voucherId);
    
    console.log('Voucher:', voucherId);
    console.log('Creator:', result.creator);
    console.log('Amount:', result.amount.toString());
    console.log('Claimed:', result.claimed);
    console.log('Is creator zero?', result.creator === ethers.constants.AddressZero);
}

checkVoucher().catch(console.error);