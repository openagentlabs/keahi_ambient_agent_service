import * as React from "react";
import { Button } from "@/components/ui/button";
import { ConnectionStatus } from "@/components/ConnectionStatus";
import { useSignalManager } from "@/hooks/useSignalManager";
import { ConnectionStateType } from "@/lib/signal-manager-client";

export default function App() {
  const {
    isConnected,
    isConnecting,
    isReconnecting,
    lastHeartbeat,
    reconnectAttempts,
    error,
    connect,
    disconnect,
    messages,
    stateType,
    currentRetryInterval,
    nextRetryTime,
  } = useSignalManager();

  const handleConnect = async () => {
    await connect();
  };

  const handleDisconnect = () => {
    disconnect();
  };

  // Determine button states
  const canConnect = !isConnected && !isConnecting && !isReconnecting;
  const canDisconnect = isConnected && !isConnecting && !isReconnecting;

  return (
    <div className="min-h-screen flex items-center justify-center bg-blue-900/60">
      <div className="bg-white rounded-lg shadow-lg p-6 max-w-md w-full mx-4">

        

        
        {/* Status message */}
        <div className="text-sm text-gray-600 mb-4 text-center">
          {isConnected && '✅ Connected to server'}
          {isConnecting && '🔄 Connecting to server...'}
          {isReconnecting && `🔄 Reconnecting to server... (Attempt ${reconnectAttempts})`}
          {!isConnected && !isConnecting && !isReconnecting && '❌ Disconnected from server'}
        </div>
        

        
        <div className="space-y-4">
          <ConnectionStatus 
            state={{
              stateType,
              isConnected,
              isConnecting,
              isReconnecting,
              lastHeartbeat,
              reconnectAttempts,
              currentRetryInterval,
              nextRetryTime,
            }}
            error={error}
          />
          
          <div className="flex space-x-2">
            <Button
              onClick={handleConnect}
              disabled={!canConnect}
              className="flex-1 bg-green-600 hover:bg-green-700 disabled:bg-gray-400"
            >
              {isConnecting ? 'Connecting...' : isReconnecting ? 'Reconnecting...' : 'Connect'}
            </Button>
            
            <Button
              onClick={handleDisconnect}
              disabled={!canDisconnect}
              className="flex-1 bg-red-600 hover:bg-red-700 disabled:bg-gray-400"
            >
              Disconnect
            </Button>
          </div>
          
          {messages.length > 0 && (
            <div className="mt-4">
              <h3 className="text-sm font-medium text-gray-700 mb-2">
                Recent Messages ({messages.length})
              </h3>
              <div className="max-h-32 overflow-y-auto space-y-1">
                {messages.slice(-5).map((message: any, index: number) => (
                  <div key={index} className="text-xs bg-gray-50 p-2 rounded">
                    <div className="font-mono">
                      Type: 0x{message.message_type.toString(16).padStart(2, '0')}
                    </div>
                    <div className="text-gray-500">
                      {new Date().toLocaleTimeString()}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
