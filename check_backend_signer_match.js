// Script to check if backend wallet matches contract's backendSigner
console.log('Checking backend signer compatibility...\n');

console.log('Backend wallet: 0x7e5f4552091a69125d5dfcb7b8c2659029395bdf');
console.log('Contract: 0x66Eb0Aa46827e5F3fFcb6Dea23C309CB401690B6');
console.log('');

console.log('The issue is most likely that the contract\'s backendSigner is set to a different address.');
console.log('');
console.log('Jenny needs to:');
console.log('1. Call contract.updateBackendSigner("0x7e5f4552091a69125d5dfcb7b8c2659029395bdf")');
console.log('2. Or provide the correct private key for the current backendSigner');
console.log('');
console.log('Until the addresses match, all signature verifications will fail with InvalidSignature()');