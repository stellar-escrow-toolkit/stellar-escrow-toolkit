import { useCallback, useEffect, useState } from 'react';
import { EscrowClient, EscrowDetails, NetworkConfig, TESTNET_CONFIG } from '@stellar-escrow/sdk';

export interface UseEscrowReturn {
  escrow: EscrowDetails | null;
  isLoading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

export function useEscrow(
  contractId: string | null | undefined,
  network: NetworkConfig = TESTNET_CONFIG,
  pollIntervalMs: number = 5000
): UseEscrowReturn {
  const [escrow, setEscrow] = useState<EscrowDetails | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchEscrow = useCallback(async () => {
    if (!contractId) {
      setEscrow(null);
      setIsLoading(false);
      return;
    }

    try {
      const client = new EscrowClient(contractId, network);
      const data = await client.getEscrow();
      setEscrow(data);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err : new Error(String(err)));
    } finally {
      setIsLoading(false);
    }
  }, [contractId, network]);

  useEffect(() => {
    fetchEscrow();
    if (!contractId || pollIntervalMs <= 0) return;

    const interval = setInterval(fetchEscrow, pollIntervalMs);
    return () => clearInterval(interval);
  }, [fetchEscrow, contractId, pollIntervalMs]);

  return { escrow, isLoading, error, refetch: fetchEscrow };
}
