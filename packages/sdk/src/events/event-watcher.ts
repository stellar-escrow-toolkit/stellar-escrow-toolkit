import { rpc, scValToNative, xdr } from 'stellar-sdk';
import { NetworkConfig, TESTNET_CONFIG } from '../types/index.js';

export interface EscrowEvent<T = unknown> {
  id: string;
  topic: string;
  contractId: string;
  ledger: number;
  ledgerClosedAt: string;
  data: T;
}

export type EventHandler<T = unknown> = (event: EscrowEvent<T>) => void;

export class EscrowEventWatcher {
  private readonly server: rpc.Server;
  private isRunning: boolean = false;
  private timer: NodeJS.Timeout | null = null;
  private lastLedger: number = 0;

  constructor(network: NetworkConfig = TESTNET_CONFIG) {
    this.server = new rpc.Server(network.rpcUrl);
  }

  /**
   * Polls for new contract events.
   */
  async pollEvents(
    contractId: string,
    startLedger?: number
  ): Promise<EscrowEvent[]> {
    const fromLedger = startLedger ?? this.lastLedger;
    if (!fromLedger) {
      const latest = await this.server.getLatestLedger();
      this.lastLedger = latest.sequence;
      return [];
    }

    const response = await this.server.getEvents({
      startLedger: fromLedger,
      filters: [
        {
          type: 'contract',
          contractIds: [contractId],
        },
      ],
    });

    const parsed: EscrowEvent[] = [];
    for (const evt of response.events) {
      const topicSymbol = evt.topic.length > 0 ? scValToNative(evt.topic[0]) : '';
      const data = scValToNative(evt.value);

      parsed.push({
        id: evt.id,
        topic: String(topicSymbol),
        contractId: evt.contractId
          ? typeof evt.contractId === 'string'
            ? evt.contractId
            : evt.contractId.contractId()
          : contractId,
        ledger: evt.ledger,
        ledgerClosedAt: evt.ledgerClosedAt,
        data,
      });

      if (evt.ledger > this.lastLedger) {
        this.lastLedger = evt.ledger;
      }
    }

    return parsed;
  }

  /**
   * Starts a continuous watcher interval for contract events.
   */
  watch(
    contractId: string,
    handler: EventHandler,
    intervalMs: number = 4000
  ): () => void {
    this.isRunning = true;

    const run = async () => {
      if (!this.isRunning) return;
      try {
        const events = await this.pollEvents(contractId);
        for (const evt of events) {
          handler(evt);
        }
      } catch (err) {
        console.error('Error polling escrow events:', err);
      }
      if (this.isRunning) {
        this.timer = setTimeout(run, intervalMs);
      }
    };

    run();

    return () => {
      this.isRunning = false;
      if (this.timer) {
        clearTimeout(this.timer);
      }
    };
  }
}
