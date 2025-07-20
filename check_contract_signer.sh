#!/bin/bash

# Check contract's backendSigner using cast
echo "Backend wallet address: 0x7e5f4552091a69125d5dfcb7b8c2659029395bdf"

# Try to call backendSigner() function
cast call 0x66Eb0Aa46827e5F3fFcb6Dea23C309CB401690B6 "backendSigner()" --rpc-url https://arb1.arbitrum.io/rpc

echo ""
echo "If the addresses don't match, Jenny needs to call updateBackendSigner()"