const ethers = require('ethers');

// Test both signature formats to see which one the contract expects
async function testSignatureFormats() {
    // Test data from Jenny's parameters
    const voucherId = '0xadeea4c8e0c60f95c97fe102e11d8b1c5d1ddd9d58bbd63f65e45abbc0e3f98b';
    const recipient = '0x9d47330f73336cedb75695dd0391ada2c6be529d';
    const deadline = 1721506060; // From Jenny's test
    const contractAddress = '0x66Eb0Aa46827e5F3fFcb6Dea23C309CB401690B6';
    const chainId = 42161;

    // Create message hash as contract expects: abi.encodePacked(voucherId, recipient, deadline, address(this), block.chainid)
    const encoded = ethers.utils.solidityPack(
        ['bytes32', 'address', 'uint256', 'address', 'uint256'],
        [voucherId, recipient, deadline, contractAddress, chainId]
    );
    
    const messageHash = ethers.utils.keccak256(encoded);
    console.log('Message hash:', messageHash);
    console.log('Expected hash from Jenny:', '0x95b41309318e7cd7de7f73b9c74bddea4c8bef4fd04b6ed1dab4e20dc5cd0ddd');
    console.log('Hashes match:', messageHash === '0x95b41309318e7cd7de7f73b9c74bddea4c8bef4fd04b6ed1dab4e20dc5cd0ddd');
    
    // Test private key (from your backend - this is just for testing)
    const privateKey = process.env.WALLET_PRIVATE_KEY || '0x...';  // You'll need to provide this
    const wallet = new ethers.Wallet(privateKey);
    
    // Format 1: EIP-191 (Ethereum Signed Message)
    const eip191Signature = await wallet.signMessage(ethers.utils.arrayify(messageHash));
    console.log('\nEIP-191 signature:', eip191Signature);
    
    // Format 2: Raw signature (direct signing of hash)
    const rawSignature = await wallet._signingKey().signDigest(messageHash);
    const rawSigString = ethers.utils.joinSignature(rawSignature);
    console.log('Raw signature:', rawSigString);
    console.log('Expected signature from Jenny:', '0x4de6e93b75f5b8daaf53ff6f4002b9b3d64fda7fb50e9ff3abf31c4f8b4e41ec0a7f41a0b3bd0e7b91a8f0b6efd0e0b6b4c2b4a3b4a3b4a3b4a3b4a3b4a3b1c');
    
    // Verify both signatures recover to correct address
    const eip191Recovered = ethers.utils.verifyMessage(ethers.utils.arrayify(messageHash), eip191Signature);
    const rawRecovered = ethers.utils.recoverAddress(messageHash, rawSigString);
    
    console.log('\nWallet address:', wallet.address);
    console.log('EIP-191 recovered:', eip191Recovered);
    console.log('Raw recovered:', rawRecovered);
    console.log('EIP-191 matches wallet:', eip191Recovered.toLowerCase() === wallet.address.toLowerCase());
    console.log('Raw matches wallet:', rawRecovered.toLowerCase() === wallet.address.toLowerCase());
}

testSignatureFormats().catch(console.error);