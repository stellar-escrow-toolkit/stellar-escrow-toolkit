import { EscrowClient, TESTNET_CONFIG, EscrowStatus } from '@stellar-escrow/sdk';

async function main() {
  console.log('=== Stellar Escrow Toolkit: Basic Escrow Example ===');

  // Replace with a deployed contract ID on testnet
  const contractId = process.env.CONTRACT_ID || 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM';
  const client = new EscrowClient(contractId, TESTNET_CONFIG);

  console.log(`Connecting to Testnet RPC: ${TESTNET_CONFIG.rpcUrl}`);
  console.log(`Contract ID: ${contractId}`);

  // 1. Check operations generation
  const user = 'GA7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVWAC';
  const depositOp = client.createDepositOp(user);
  console.log('Generated Deposit Operation successfully.');

  const releaseOp = client.createReleaseOp(user);
  console.log('Generated Release Operation successfully.');

  console.log('Example completed successfully!');
}

main().catch(console.error);
