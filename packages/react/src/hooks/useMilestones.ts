import { useCallback, useEffect, useState } from 'react';
import {
  MilestoneClient,
  MilestoneEscrowConfig,
  MilestoneItem,
  NetworkConfig,
  TESTNET_CONFIG,
} from '@stellar-escrow/sdk';

export interface UseMilestonesReturn {
  config: MilestoneEscrowConfig | null;
  milestones: MilestoneItem[];
  isLoading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

export function useMilestones(
  contractId: string | null | undefined,
  network: NetworkConfig = TESTNET_CONFIG,
  pollIntervalMs: number = 5000
): UseMilestonesReturn {
  const [config, setConfig] = useState<MilestoneEscrowConfig | null>(null);
  const [milestones, setMilestones] = useState<MilestoneItem[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchData = useCallback(async () => {
    if (!contractId) {
      setConfig(null);
      setMilestones([]);
      setIsLoading(false);
      return;
    }

    try {
      const client = new MilestoneClient(contractId, network);
      const conf = await client.getConfig();
      setConfig(conf);

      const items: MilestoneItem[] = [];
      for (let i = 1; i <= conf.milestoneCount; i++) {
        const item = await client.getMilestone(i);
        items.push(item);
      }
      setMilestones(items);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err : new Error(String(err)));
    } finally {
      setIsLoading(false);
    }
  }, [contractId, network]);

  useEffect(() => {
    fetchData();
    if (!contractId || pollIntervalMs <= 0) return;

    const interval = setInterval(fetchData, pollIntervalMs);
    return () => clearInterval(interval);
  }, [fetchData, contractId, pollIntervalMs]);

  return { config, milestones, isLoading, error, refetch: fetchData };
}
