#!/usr/bin/env node

import { Command } from 'commander';
import {
  EscrowClient,
  MilestoneClient,
  MultisigClient,
  MAINNET_CONFIG,
  TESTNET_CONFIG,
} from '@stellar-escrow/sdk';

const program = new Command();

program
  .name('stellar-escrow')
  .description('CLI tool to manage and inspect Stellar & Soroban escrows')
  .version('0.1.0');

program
  .command('inspect')
  .description('Inspect the current state of a two-party or arbiter escrow')
  .requiredOption('-c, --contract <contractId>', 'Soroban Contract ID')
  .option('-n, --network <network>', 'Stellar Network (testnet|mainnet)', 'testnet')
  .action(async (options) => {
    const net = options.network === 'mainnet' ? MAINNET_CONFIG : TESTNET_CONFIG;
    console.log(`Querying escrow on ${options.network}...`);
    try {
      const client = new EscrowClient(options.contract, net);
      const escrow = await client.getEscrow();
      console.log('Escrow Details:');
      console.log(JSON.stringify(escrow, (_, v) => (typeof v === 'bigint' ? v.toString() : v), 2));
    } catch (err) {
      console.error('Error fetching escrow:', err);
      process.exit(1);
    }
  });

program
  .command('inspect-milestones')
  .description('Inspect all milestones for a milestone escrow contract')
  .requiredOption('-c, --contract <contractId>', 'Soroban Contract ID')
  .option('-n, --network <network>', 'Stellar Network (testnet|mainnet)', 'testnet')
  .action(async (options) => {
    const net = options.network === 'mainnet' ? MAINNET_CONFIG : TESTNET_CONFIG;
    console.log(`Querying milestones on ${options.network}...`);
    try {
      const client = new MilestoneClient(options.contract, net);
      const config = await client.getConfig();
      console.log('Milestone Escrow Config:');
      console.log(JSON.stringify(config, (_, v) => (typeof v === 'bigint' ? v.toString() : v), 2));

      console.log(`\nMilestones (${config.milestoneCount}):`);
      for (let i = 1; i <= config.milestoneCount; i++) {
        const m = await client.getMilestone(i);
        console.log(`- Milestone #${m.id}: amount=${m.amount.toString()} status=${m.status} proof=${m.proofHash ?? 'none'}`);
      }
    } catch (err) {
      console.error('Error fetching milestones:', err);
      process.exit(1);
    }
  });

program
  .command('inspect-multisig')
  .description('Inspect multisig escrow vault signers and threshold')
  .requiredOption('-c, --contract <contractId>', 'Soroban Contract ID')
  .option('-n, --network <network>', 'Stellar Network (testnet|mainnet)', 'testnet')
  .action(async (options) => {
    const net = options.network === 'mainnet' ? MAINNET_CONFIG : TESTNET_CONFIG;
    console.log(`Querying multisig escrow on ${options.network}...`);
    try {
      const client = new MultisigClient(options.contract, net);
      const config = await client.getConfig();
      console.log('Multisig Config:');
      console.log(JSON.stringify(config, null, 2));
    } catch (err) {
      console.error('Error fetching multisig config:', err);
      process.exit(1);
    }
  });

program.parse(process.argv);
