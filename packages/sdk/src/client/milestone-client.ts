import {
  rpc,
  scValToNative,
  xdr,
  Address,
  Contract,
  Keypair,
  nativeToScVal,
  TransactionBuilder,
} from 'stellar-sdk';
import {
  MilestoneEscrowConfig,
  MilestoneItem,
  MilestoneStatus,
  NetworkConfig,
  TESTNET_CONFIG,
} from '../types/index.js';

export class MilestoneClient {
  public readonly contractId: string;
  public readonly server: rpc.Server;
  public readonly networkPassphrase: string;
  private readonly contract: Contract;

  constructor(contractId: string, network: NetworkConfig = TESTNET_CONFIG) {
    this.contractId = contractId;
    this.server = new rpc.Server(network.rpcUrl);
    this.networkPassphrase = network.networkPassphrase;
    this.contract = new Contract(contractId);
  }

  /**
   * Fetches the overall milestone contract configuration.
   */
  async getConfig(): Promise<MilestoneEscrowConfig> {
    const response = await this.simulateMethod('get_config', []);
    if (!response || !('retval' in response)) {
      throw new Error('Failed to query milestone escrow config');
    }

    const val = scValToNative(response.retval as xdr.ScVal);
    return {
      client: val.client,
      contractor: val.contractor,
      arbiter: val.arbiter ?? null,
      token: val.token,
      totalAmount: BigInt(val.total_amount),
      releasedAmount: BigInt(val.released_amount),
      isFunded: val.is_funded,
      milestoneCount: Number(val.milestone_count),
    };
  }

  /**
   * Fetches details of a specific milestone by its ID.
   */
  async getMilestone(milestoneId: number): Promise<MilestoneItem> {
    const response = await this.simulateMethod('get_milestone', [
      nativeToScVal(milestoneId, { type: 'u32' }),
    ]);
    if (!response || !('retval' in response)) {
      throw new Error(`Failed to query milestone ${milestoneId}`);
    }

    const val = scValToNative(response.retval as xdr.ScVal);
    return {
      id: Number(val.id),
      descriptionHash: Buffer.from(val.description_hash).toString('hex'),
      amount: BigInt(val.amount),
      status: val.status as MilestoneStatus,
      proofHash: val.proof_hash ? Buffer.from(val.proof_hash).toString('hex') : null,
      submittedAt: Number(val.submitted_at),
      approvedAt: Number(val.approved_at),
    };
  }

  /**
   * Generates operation for depositing full funding.
   */
  createDepositOp(fromAddress: string): xdr.Operation {
    return this.contract.call('deposit', Address.fromString(fromAddress).toScVal());
  }

  /**
   * Generates operation for contractor proof submission.
   */
  createSubmitMilestoneOp(
    contractorAddress: string,
    milestoneId: number,
    proofHashHex: string
  ): xdr.Operation {
    const proofBytes = Buffer.from(proofHashHex.replace('0x', ''), 'hex');
    return this.contract.call(
      'submit_milestone',
      Address.fromString(contractorAddress).toScVal(),
      nativeToScVal(milestoneId, { type: 'u32' }),
      xdr.ScVal.scvBytes(proofBytes)
    );
  }

  /**
   * Generates operation for client or arbiter milestone approval and release.
   */
  createApproveAndReleaseOp(callerAddress: string, milestoneId: number): xdr.Operation {
    return this.contract.call(
      'approve_and_release',
      Address.fromString(callerAddress).toScVal(),
      nativeToScVal(milestoneId, { type: 'u32' })
    );
  }

  /**
   * Simulates a read-only method on the contract.
   */
  private async simulateMethod(method: string, args: xdr.ScVal[] = []) {
    const dummyAccount = Keypair.random();
    const sourceAccount = new (await import('stellar-sdk')).Account(dummyAccount.publicKey(), '0');

    const tx = new TransactionBuilder(sourceAccount, {
      fee: '100',
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(this.contract.call(method, ...args))
      .setTimeout(30)
      .build();

    const sim = await this.server.simulateTransaction(tx);
    if (rpc.Api.isSimulationError(sim)) {
      throw new Error(`Simulation error for ${method}: ${sim.error}`);
    }
    return sim.result;
  }
}
