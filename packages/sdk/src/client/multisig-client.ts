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
  MultisigConfig,
  MultisigProposal,
  NetworkConfig,
  TESTNET_CONFIG,
} from '../types/index.js';

export class MultisigClient {
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
   * Fetches multisig configuration (signers list, threshold, token).
   */
  async getConfig(): Promise<MultisigConfig> {
    const response = await this.simulateMethod('get_config', []);
    if (!response || !('retval' in response)) {
      throw new Error('Failed to query multisig config');
    }

    const val = scValToNative(response.retval as xdr.ScVal);
    return {
      signers: val.signers,
      threshold: Number(val.threshold),
      token: val.token,
    };
  }

  /**
   * Fetches a specific proposal by ID.
   */
  async getProposal(proposalId: number): Promise<MultisigProposal> {
    const response = await this.simulateMethod('get_proposal', [
      nativeToScVal(proposalId, { type: 'u32' }),
    ]);
    if (!response || !('retval' in response)) {
      throw new Error(`Failed to query proposal ${proposalId}`);
    }

    const val = scValToNative(response.retval as xdr.ScVal);
    return {
      id: Number(val.id),
      action: val.action,
      approvalsCount: Number(val.approvals_count),
      executed: val.executed,
      createdAt: Number(val.created_at),
    };
  }

  /**
   * Generates operation for approving a proposal.
   */
  createApproveOp(signerAddress: string, proposalId: number): xdr.Operation {
    return this.contract.call(
      'approve_proposal',
      Address.fromString(signerAddress).toScVal(),
      nativeToScVal(proposalId, { type: 'u32' })
    );
  }

  /**
   * Generates operation for executing an approved proposal.
   */
  createExecuteOp(callerAddress: string, proposalId: number): xdr.Operation {
    return this.contract.call(
      'execute_proposal',
      Address.fromString(callerAddress).toScVal(),
      nativeToScVal(proposalId, { type: 'u32' })
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
