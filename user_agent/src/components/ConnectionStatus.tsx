import React from 'react';

interface ConnectionState {
  stateType: string;
  isConnected: boolean;
  isConnecting: boolean;
  isReconnecting: boolean;
  lastHeartbeat: number;
  reconnectAttempts: number;
  currentRetryInterval: number;
  nextRetryTime: number | null;
}

interface ConnectionStatusProps {
  state: ConnectionState;
  error: string | null;
}

export function ConnectionStatus({ state, error }: ConnectionStatusProps) {
  const getStatusColor = () => {
    if (error) return 'bg-red-500';
    if (state.isConnected) return 'bg-green-500';
    if (state.isConnecting) return 'bg-yellow-500';
    if (state.isReconnecting) return 'bg-orange-500';
    return 'bg-gray-500';
  };

  const getStatusText = () => {
    if (error) return 'Error';
    if (state.isConnected) return 'Connected';
    if (state.isConnecting) return 'Connecting...';
    if (state.isReconnecting) return `Reconnecting... (${state.reconnectAttempts})`;
    return 'Disconnected';
  };

  const getStateTypeText = () => {
    switch (state.stateType) {
      case 'disconnected_not_to_connect':
        return 'Initial State';
      case 'trying_to_connect':
        return 'Trying to Connect';
      case 'connected':
        return 'Connected';
      case 'was_connected_trying_to_reconnect':
        return 'Reconnecting';
      case 'disconnecting_disconnect_requested':
        return 'Disconnecting';
      default:
        return 'Unknown State';
    }
  };

  const getRetryInfo = () => {
    if (state.stateType === 'was_connected_trying_to_reconnect' && state.nextRetryTime) {
      const now = Date.now();
      const timeUntilRetry = Math.max(0, state.nextRetryTime - now);
      const secondsUntilRetry = Math.ceil(timeUntilRetry / 1000);
      return `Next retry in ${secondsUntilRetry}s (${state.currentRetryInterval / 1000}s interval)`;
    }
    return null;
  };

  const getLastHeartbeatText = () => {
    if (!state.isConnected || state.lastHeartbeat === 0) return null;
    const now = Date.now();
    const diff = now - state.lastHeartbeat;
    const seconds = Math.floor(diff / 1000);
    return `Last heartbeat: ${seconds}s ago`;
  };

  return (
    <div className="space-y-2 p-3 bg-gray-100 rounded-lg">
      <div className="flex items-center space-x-2">
        <div className={`w-3 h-3 rounded-full ${getStatusColor()} animate-pulse`} />
        <span className="text-sm font-medium">{getStatusText()}</span>
        {getLastHeartbeatText() && (
          <span className="text-xs text-gray-500">{getLastHeartbeatText()}</span>
        )}
        {error && (
          <span className="text-xs text-red-500 ml-2">{error}</span>
        )}
      </div>
      
      {/* State Type Display */}
      <div className="text-xs text-blue-600 font-mono">
        State: {getStateTypeText()}
      </div>
      
      {/* Retry Information */}
      {getRetryInfo() && (
        <div className="text-xs text-orange-600">
          {getRetryInfo()}
        </div>
      )}
    </div>
  );
} 