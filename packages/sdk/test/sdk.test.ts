import { describe, it, expect } from 'vitest';
import { Keypair } from 'stellar-sdk';
import { EscrowClient } from '../src/client/escrow-client.js';
import { MilestoneClient } from '../src/client/milestone-client.js';
import { MultisigClient } from '../src/client/multisig-client.js';
import { EscrowStatus, MilestoneStatus } from '../src/types/index.js';

describe('@stellar-escrow/sdk', () => {
  const dummyContractId = 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM';
  const dummyCaller = Keypair.random().publicKey();

  it('instantiates EscrowClient properly', () => {
    const client = new EscrowClient(dummyContractId);
    expect(client.contractId).toBe(dummyContractId);

    const depositOp = client.createDepositOp(dummyCaller);
    expect(depositOp).toBeDefined();

    const releaseOp = client.createReleaseOp(dummyCaller);
    expect(releaseOp).toBeDefined();

    const refundOp = client.createRefundOp(dummyCaller);
    expect(refundOp).toBeDefined();
  });

  it('instantiates MilestoneClient properly', () => {
    const client = new MilestoneClient(dummyContractId);
    expect(client.contractId).toBe(dummyContractId);

    const depositOp = client.createDepositOp(dummyCaller);
    expect(depositOp).toBeDefined();

    const submitOp = client.createSubmitMilestoneOp(
      dummyCaller,
      1,
      '00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff'
    );
    expect(submitOp).toBeDefined();

    const approveOp = client.createApproveAndReleaseOp(dummyCaller, 1);
    expect(approveOp).toBeDefined();
  });

  it('instantiates MultisigClient properly', () => {
    const client = new MultisigClient(dummyContractId);
    expect(client.contractId).toBe(dummyContractId);

    const approveOp = client.createApproveOp(dummyCaller, 1);
    expect(approveOp).toBeDefined();

    const executeOp = client.createExecuteOp(dummyCaller, 1);
    expect(executeOp).toBeDefined();
  });

  it('validates enums', () => {
    expect(EscrowStatus.Funded).toBe(1);
    expect(MilestoneStatus.Released).toBe(3);
  });
});
