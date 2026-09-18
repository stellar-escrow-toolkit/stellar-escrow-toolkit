import {
  rpc,
  scValToNative,
  xdr,
  Address,
  Contract,
  Keypair,
  Operation,
  Transaction,
  TransactionBuilder,
} from 'stellar-sdk';
import { EscrowDetails, EscrowStatus, NetworkConfig, TESTNET_CONFIG } from '../types/index.js';

export class EscrowClient {
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
   * Fetches the current state of the escrow.
   */
  async getEscrow(): Promise<EscrowDetails> {
    const response = await this.simulateMethod('get_escrow', []);
    if (!response || !('retval' in response)) {
      throw new Error('Failed to query escrow state from Soroban RPC');
    }

    const val = scValToNative(response.retval as xdr.ScVal);
    return {
      id: BigInt(val.id),
      initiator: val.initiator,
      beneficiary: val.beneficiary,
      arbiter: val.arbiter ?? null,
      token: val.token,
      amount: BigInt(val.amount),
      fundedAmount: BigInt(val.funded_amount),
      deadline: Number(val.deadline),
      status: val.status as EscrowStatus,
      createdAt: Number(val.created_at),
      engagementId: Buffer.from(val.engagement_id).toString('hex'),
    };
  }

  /**
   * Creates an invocation operation for deposit.
   */
  createDepositOp(fromAddress: string): xdr.Operation {
    return this.contract.call(
      'deposit',
      Address.fromString(fromAddress).toScVal()
    );
  }

  /**
   * Creates an invocation operation for release.
   */
  createReleaseOp(callerAddress: string): xdr.Operation {
    return this.contract.call(
      'release',
      Address.fromString(callerAddress).toScVal()
    );
  }

  /**
   * Creates an invocation operation for refund.
   */
  createRefundOp(callerAddress: string): xdr.Operation {
    return this.contract.call(
      'refund',
      Address.fromString(callerAddress).toScVal()
    );
  }

  /**
   * Creates an invocation operation for dispute.
   */
  createDisputeOp(callerAddress: string): xdr.Operation {
    return this.contract.call(
      'dispute',
      Address.fromString(callerAddress).toScVal()
    );
  }

  /**
   * Simulates a read-only method on the contract.
   */
  private async simulateMethod(method: string, args: xdr.ScVal[] = []) {
    // Generate a temporary source account keypair for simulation
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
