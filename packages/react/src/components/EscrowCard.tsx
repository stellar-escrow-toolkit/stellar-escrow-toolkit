import React from 'react';
import { EscrowDetails, EscrowStatus } from '@stellar-escrow/sdk';

export interface EscrowCardProps {
  escrow: EscrowDetails;
  onDeposit?: () => void;
  onRelease?: () => void;
  onRefund?: () => void;
  onDispute?: () => void;
}

const statusText: Record<EscrowStatus, string> = {
  [EscrowStatus.Created]: 'Awaiting Deposit',
  [EscrowStatus.Funded]: 'Active & Locked',
  [EscrowStatus.Completed]: 'Completed & Released',
  [EscrowStatus.Refunded]: 'Refunded',
  [EscrowStatus.Disputed]: 'Disputed',
};

export const EscrowCard: React.FC<EscrowCardProps> = ({
  escrow,
  onDeposit,
  onRelease,
  onRefund,
  onDispute,
}) => {
  const isPastDeadline = Date.now() / 1000 > escrow.deadline;

  return (
    <div
      style={{
        padding: '1.5rem',
        borderRadius: '0.75rem',
        backgroundColor: '#0f172a',
        border: '1px solid #334155',
        color: '#f8fafc',
        fontFamily: 'sans-serif',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '1rem' }}>
        <h3 style={{ margin: 0, fontSize: '1.25rem' }}>Escrow #{escrow.id.toString()}</h3>
        <span
          style={{
            padding: '0.25rem 0.75rem',
            borderRadius: '9999px',
            fontSize: '0.8rem',
            fontWeight: 600,
            backgroundColor: escrow.status === EscrowStatus.Funded ? '#065f46' : '#1e293b',
            color: escrow.status === EscrowStatus.Funded ? '#34d399' : '#94a3b8',
          }}
        >
          {statusText[escrow.status]}
        </span>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '0.75rem', marginBottom: '1.5rem', fontSize: '0.875rem' }}>
        <div>
          <span style={{ color: '#94a3b8' }}>Amount:</span>
          <div>{escrow.amount.toString()} stroops</div>
        </div>
        <div>
          <span style={{ color: '#94a3b8' }}>Funded:</span>
          <div>{escrow.fundedAmount.toString()} stroops</div>
        </div>
        <div style={{ gridColumn: 'span 2' }}>
          <span style={{ color: '#94a3b8' }}>Initiator:</span>
          <div style={{ wordBreak: 'break-all', fontFamily: 'monospace' }}>{escrow.initiator}</div>
        </div>
        <div style={{ gridColumn: 'span 2' }}>
          <span style={{ color: '#94a3b8' }}>Beneficiary:</span>
          <div style={{ wordBreak: 'break-all', fontFamily: 'monospace' }}>{escrow.beneficiary}</div>
        </div>
        {escrow.arbiter && (
          <div style={{ gridColumn: 'span 2' }}>
            <span style={{ color: '#94a3b8' }}>Arbiter:</span>
            <div style={{ wordBreak: 'break-all', fontFamily: 'monospace' }}>{escrow.arbiter}</div>
          </div>
        )}
      </div>

      <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
        {escrow.status === EscrowStatus.Created && onDeposit && (
          <button
            onClick={onDeposit}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#2563eb',
              color: '#ffffff',
              border: 'none',
              borderRadius: '0.375rem',
              cursor: 'pointer',
              fontWeight: 600,
            }}
          >
            Deposit Funds
          </button>
        )}
        {escrow.status === EscrowStatus.Funded && onRelease && (
          <button
            onClick={onRelease}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#059669',
              color: '#ffffff',
              border: 'none',
              borderRadius: '0.375rem',
              cursor: 'pointer',
              fontWeight: 600,
            }}
          >
            Release to Beneficiary
          </button>
        )}
        {escrow.status === EscrowStatus.Funded && isPastDeadline && onRefund && (
          <button
            onClick={onRefund}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#d97706',
              color: '#ffffff',
              border: 'none',
              borderRadius: '0.375rem',
              cursor: 'pointer',
              fontWeight: 600,
            }}
          >
            Claim Refund (Timelock)
          </button>
        )}
        {escrow.status === EscrowStatus.Funded && onDispute && (
          <button
            onClick={onDispute}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#dc2626',
              color: '#ffffff',
              border: 'none',
              borderRadius: '0.375rem',
              cursor: 'pointer',
              fontWeight: 600,
            }}
          >
            Raise Dispute
          </button>
        )}
      </div>
    </div>
  );
};
