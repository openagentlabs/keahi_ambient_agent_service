import * as React from "react";
import { Button } from "@/components/ui/button";
import { ConnectionStatus } from "@/components/ConnectionStatus";
import { useSignalManager } from "@/hooks/useSignalManager";
import { ConnectionStateType, WebRTCRoomCreatePayload } from "@/lib/signal-manager-client";

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
    createRoom,
    messages,
    stateType,
    currentRetryInterval,
    nextRetryTime,
    roomInfo,
  } = useSignalManager();

  const [clientId, setClientId] = React.useState("");
  const [authToken, setAuthToken] = React.useState("");
  const [role, setRole] = React.useState<'sender' | 'receiver'>('sender');
  const [offerSdp, setOfferSdp] = React.useState("");
  const [creatingRoom, setCreatingRoom] = React.useState(false);

  const handleConnect = async () => {
    await connect();
  };

  const handleDisconnect = () => {
    disconnect();
    // Clear form fields when disconnecting
    setClientId("");
    setAuthToken("");
    setOfferSdp("");
  };

  const handleCreateRoom = (e: React.FormEvent) => {
    e.preventDefault();
    if (!clientId || !authToken) return;
    setCreatingRoom(true);
    const payload: WebRTCRoomCreatePayload = {
      version: "1.0.0",
      client_id: clientId,
      auth_token: authToken,
      role,
      offer_sdp: offerSdp || undefined,
      metadata: {
        userAgent: navigator.userAgent,
        timestamp: Date.now(),
      },
    };
    createRoom(payload);
    setCreatingRoom(false);
  };

  // Determine button states
  const canConnect = !isConnected && !isConnecting && !isReconnecting;
  const canDisconnect = isConnected && !isConnecting && !isReconnecting;
  const canCreateRoom = isConnected && clientId && authToken;

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

          {/* Debug info */}
          <div className="text-xs text-gray-500 bg-gray-50 p-2 rounded">
            <div>Connection: {isConnected ? '✅' : '❌'}</div>
            <div>Client ID: {clientId ? '✅' : '❌'}</div>
            <div>Auth Token: {authToken ? '✅' : '❌'}</div>
            <div>Button enabled: {canCreateRoom ? '✅' : '❌'}</div>
          </div>

          {/* Room creation form */}
          <form onSubmit={handleCreateRoom} className="space-y-2 mt-4">
            <div className="flex flex-col space-y-1">
              <label className="text-xs font-medium text-gray-700">Client ID</label>
              <input
                className="border rounded px-2 py-1 text-sm"
                value={clientId}
                onChange={e => setClientId(e.target.value)}
                placeholder="Enter client ID (e.g., test_client_1)"
                required
              />
            </div>
            <div className="flex flex-col space-y-1">
              <label className="text-xs font-medium text-gray-700">Auth Token</label>
              <input
                className="border rounded px-2 py-1 text-sm"
                value={authToken}
                onChange={e => setAuthToken(e.target.value)}
                placeholder="Enter auth token (e.g., test_token_1)"
                required
              />
            </div>
            <div className="flex flex-col space-y-1">
              <label className="text-xs font-medium text-gray-700">Role</label>
              <select
                className="border rounded px-2 py-1 text-sm"
                value={role}
                onChange={e => setRole(e.target.value as 'sender' | 'receiver')}
              >
                <option value="sender">Sender</option>
                <option value="receiver">Receiver</option>
              </select>
            </div>
            <div className="flex flex-col space-y-1">
              <label className="text-xs font-medium text-gray-700">Offer SDP (optional)</label>
              <textarea
                className="border rounded px-2 py-1 text-sm"
                value={offerSdp}
                onChange={e => setOfferSdp(e.target.value)}
                placeholder="Enter SDP offer (optional)"
                rows={2}
              />
            </div>
            <Button
              type="submit"
              disabled={!canCreateRoom || creatingRoom}
              className="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400"
            >
              {creatingRoom ? 'Creating Room...' : 'Create WebRTC Room'}
            </Button>
          </form>

          {/* Display room info after creation */}
          {roomInfo && roomInfo.status === 200 && (
            <div className="mt-4 p-3 bg-green-50 border border-green-200 rounded">
              <div className="font-semibold text-green-700 mb-1">Room Created!</div>
              <div className="text-xs text-gray-700">Room ID: <span className="font-mono">{roomInfo.room_id}</span></div>
              <div className="text-xs text-gray-700">Session ID: <span className="font-mono">{roomInfo.session_id}</span></div>
              <div className="text-xs text-gray-700">App ID: <span className="font-mono">{roomInfo.app_id}</span></div>
              <div className="text-xs text-gray-700">STUN URL: <span className="font-mono">{roomInfo.stun_url}</span></div>
              {roomInfo.connection_info && (
                <pre className="text-xs bg-gray-100 rounded p-2 mt-2 overflow-x-auto">{JSON.stringify(roomInfo.connection_info, null, 2)}</pre>
              )}
            </div>
          )}

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
