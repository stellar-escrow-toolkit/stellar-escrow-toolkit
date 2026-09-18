import React from 'react';
import { MilestoneItem, MilestoneStatus } from '@stellar-escrow/sdk';

export interface MilestoneTimelineProps {
  milestones: MilestoneItem[];
  onSelectMilestone?: (milestone: MilestoneItem) => void;
}

const statusLabels: Record<MilestoneStatus, string> = {
  [MilestoneStatus.Pending]: 'Pending',
  [MilestoneStatus.Submitted]: 'Review Submitted',
  [MilestoneStatus.Approved]: 'Approved',
  [MilestoneStatus.Released]: 'Paid Out',
  [MilestoneStatus.Disputed]: 'In Dispute',
  [MilestoneStatus.Cancelled]: 'Cancelled',
};

const statusColors: Record<MilestoneStatus, string> = {
  [MilestoneStatus.Pending]: '#94a3b8',
  [MilestoneStatus.Submitted]: '#3b82f6',
  [MilestoneStatus.Approved]: '#10b981',
  [MilestoneStatus.Released]: '#059669',
  [MilestoneStatus.Disputed]: '#ef4444',
  [MilestoneStatus.Cancelled]: '#64748b',
};

export const MilestoneTimeline: React.FC<MilestoneTimelineProps> = ({
  milestones,
  onSelectMilestone,
}) => {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      {milestones.map((m, idx) => {
        const color = statusColors[m.status] || '#94a3b8';
        return (
          <div
            key={m.id}
            onClick={() => onSelectMilestone?.(m)}
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              padding: '1rem',
              border: `1px solid ${color}40`,
              borderRadius: '0.5rem',
              backgroundColor: '#1e293b',
              cursor: onSelectMilestone ? 'pointer' : 'default',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
              <div
                style={{
                  width: '28px',
                  height: '28px',
                  borderRadius: '50%',
                  backgroundColor: color,
                  color: '#ffffff',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  fontWeight: 600,
                  fontSize: '0.85rem',
                }}
              >
                {idx + 1}
              </div>
              <div>
                <div style={{ color: '#f8fafc', fontWeight: 600 }}>
                  Milestone #{m.id}
                </div>
                <div style={{ color: '#94a3b8', fontSize: '0.8rem' }}>
                  Amount: {m.amount.toString()} stroops
                </div>
              </div>
            </div>
            <div
              style={{
                padding: '0.25rem 0.6rem',
                borderRadius: '9999px',
                fontSize: '0.75rem',
                fontWeight: 600,
                backgroundColor: `${color}20`,
                color: color,
                border: `1px solid ${color}`,
              }}
            >
              {statusLabels[m.status]}
            </div>
          </div>
        );
      })}
    </div>
  );
};
