export enum EscrowStatus {
  Created = 0,
  Funded = 1,
  Completed = 2,
  Refunded = 3,
  Disputed = 4,
}

export interface EscrowDetails {
  id: bigint;
  initiator: string;
  beneficiary: string;
  arbiter?: string | null;
  token: string;
  amount: bigint;
  fundedAmount: bigint;
  deadline: number; // Unix timestamp in seconds
  status: EscrowStatus;
  createdAt: number;
  engagementId: string; // 32-byte hex string
}

export enum MilestoneStatus {
  Pending = 0,
  Submitted = 1,
  Approved = 2,
  Released = 3,
  Disputed = 4,
  Cancelled = 5,
}

export interface MilestoneItem {
  id: number;
  descriptionHash: string; // 32-byte hex
  amount: bigint;
  status: MilestoneStatus;
  proofHash?: string | null;
  submittedAt: number;
  approvedAt: number;
}

export interface MilestoneEscrowConfig {
  client: string;
  contractor: string;
  arbiter?: string | null;
  token: string;
  totalAmount: bigint;
  releasedAmount: bigint;
  isFunded: boolean;
  milestoneCount: number;
}

export type MultisigAction =
  | { type: 'Release'; to: string; amount: bigint }
  | { type: 'Refund'; to: string; amount: bigint }
  | { type: 'UpdateThreshold'; newThreshold: number };

export interface MultisigProposal {
  id: number;
  action: MultisigAction;
  approvalsCount: number;
  executed: boolean;
  createdAt: number;
}

export interface MultisigConfig {
  signers: string[];
  threshold: number;
  token: string;
}

export interface NetworkConfig {
  rpcUrl: string;
  networkPassphrase: string;
}

export const TESTNET_CONFIG: NetworkConfig = {
  rpcUrl: 'https://soroban-testnet.stellar.org',
  networkPassphrase: 'Test SDF Network ; September 2015',
};

export const MAINNET_CONFIG: NetworkConfig = {
  rpcUrl: 'https://mainnet.stellar.org/soroban/rpc',
  networkPassphrase: 'Public Global Stellar Network ; September 2015',
};
