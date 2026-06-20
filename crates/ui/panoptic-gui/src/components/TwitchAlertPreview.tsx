import React from 'react';

export type TwitchAlert = 
  | { type: 'Follow', data: { user_name: string } }
  | { type: 'Subscription', data: { user_name: string, tier: string, is_gift: boolean, cumulative_months: number } }
  | { type: 'GiftSubscription', data: { user_name: string, total: number, tier: string } }
  | { type: 'Raid', data: { from_broadcaster_name: string, viewers: number } }
  | { type: 'Cheer', data: { user_name: string, bits: number, message: string } };

export interface QueuedAlert {
  id: string;
  alert: TwitchAlert;
  timestamp: number;
}

export interface AlertState {
  active_alerts: QueuedAlert[];
}

interface TwitchAlertPreviewProps {
  state: AlertState;
  settings: Record<string, any>;
}

export const TwitchAlertPreview: React.FC<TwitchAlertPreviewProps> = ({ state, settings }) => {
  const alerts = state.active_alerts || [];

  const getAlertText = (alert: TwitchAlert) => {
    const { type, data } = alert;
    switch (type) {
      case 'Follow':
        return settings.follow_text?.replace('{user}', (data as any).user_name) || `${(data as any).user_name} just followed!`;
      case 'Subscription':
        return settings.sub_text?.replace('{user}', (data as any).user_name).replace('{tier}', (data as any).tier).replace('{months}', String((data as any).cumulative_months)) || `${(data as any).user_name} subscribed at Tier ${(data as any).tier} for ${(data as any).cumulative_months} months!`;
      case 'GiftSubscription':
        return settings.giftsub_text?.replace('{user}', (data as any).user_name).replace('{total}', String((data as any).total)).replace('{tier}', (data as any).tier) || `${(data as any).user_name} gifted ${(data as any).total} Tier ${(data as any).tier} subscriptions!`;
      case 'Raid':
        return settings.raid_text?.replace('{user}', (data as any).from_broadcaster_name).replace('{viewers}', String((data as any).viewers)) || `${(data as any).from_broadcaster_name} raided with ${(data as any).viewers} viewers!`;
      case 'Cheer':
        return settings.cheer_text?.replace('{user}', (data as any).user_name).replace('{bits}', String((data as any).bits)).replace('{message}', (data as any).message) || `${(data as any).user_name} cheered ${(data as any).bits} bits: ${(data as any).message}`;
      default:
        return "New Event!";
    }
  };

  const getIcon = (type: string) => {
    if (type === 'Follow') return settings.follow_icon || '✨';
    if (type === 'Subscription') return settings.sub_icon || '💖';
    if (type === 'GiftSubscription') return settings.giftsub_icon || '🎁';
    if (type === 'Raid') return settings.raid_icon || '⚔️';
    if (type === 'Cheer') return settings.cheer_icon || '💎';
    return '🔔';
  };

  return (
    <div className="panoptic-overlay-wrapper twitch-alerts-preview" style={{ alignItems: 'flex-end', justifyContent: 'flex-end', padding: '40px' }}>
      <div className="alert-stack" style={{ display: 'flex', flexDirection: 'column-reverse', gap: '12px' }}>
        {alerts.length === 0 ? (
          <div className="hype-train-card alert-card idle" style={{ width: '300px', opacity: 0.5 }}>
            <div className="hype-idle-state">
              <div className="status-icon">💤</div>
              <div className="hype-text">
                <div className="hype-title">No Active Alerts</div>
              </div>
            </div>
          </div>
        ) : (
          alerts.slice(-3).map((queued) => (
            <div 
              key={queued.id} 
              className={`hype-train-card alert-card active alert-${queued.alert.type.toLowerCase()}`}
              style={{ 
                width: '320px',
                animation: 'HTitemAppear 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275) both'
              }}
            >
              <div className="hype-active-state" style={{ gap: '8px' }}>
                <div className="hype-header" style={{ marginBottom: '4px' }}>
                  <div className="hype-title-row">
                    <span className="hype-icon">{getIcon(queued.alert.type)}</span>
                    <span className="hype-active-title" style={{ fontSize: '14px' }}>{queued.alert.type}</span>
                  </div>
                </div>
                <div className="alert-text" style={{ fontSize: '13px', fontWeight: 600 }}>
                  {getAlertText(queued.alert)}
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
