const ethers = require('ethers');

async function checkBackendSigner() {
    const provider = new ethers.providers.JsonRpcProvider('https://arb1.arbitrum.io/rpc');
    const contractAddress = '0x66Eb0Aa46827e5F3fFcb6Dea23C309CB401690B6';
    
    const abi = ['function backendSigner() view returns (address)'];
    const contract = new ethers.Contract(contractAddress, abi, provider);
    
    const backendSigner = await contract.backendSigner();
    console.log('Contract backendSigner:', backendSigner);
    
    // Check if backend is running and get its address
    try {
        const response = await fetch('http://localhost:8080/api/health');
        if (response.ok) {
            console.log('Backend is running');
            // You'd need to add an endpoint to get the wallet address
        } else {
            console.log('Backend not responding');
        }
    } catch (e) {
        console.log('Backend not running');
    }
}

checkBackendSigner().catch(console.error);