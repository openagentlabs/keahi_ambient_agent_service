import * as React from "react";
import { Button } from "@/components/ui/button";
import { ConnectionStatus } from "@/components/ConnectionStatus";
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface ConnectionState {
  state_type: 'disconnected_not_to_connect' | 'trying_to_connect' | 'connected' | 'was_connected_trying_to_reconnect' | 'disconnecting_disconnect_requested';
  is_connected: boolean;
  is_connecting: boolean;
  is_reconnecting: boolean;
  last_heartbeat: number;
  reconnect_attempts: number;
  current_retry_interval: number;
  next_retry_time: number | null;
}

interface WebRTCOffer {
  sdp: string;
  type_: string;
}

interface RoomCreatedEvent {
  roomId: string | null;
  sessionId: string | null;
}

export default function App() {
  const [state, setState] = React.useState<ConnectionState>({
    state_type: 'disconnected_not_to_connect',
    is_connected: false,
    is_connecting: false,
    is_reconnecting: false,
    last_heartbeat: 0,
    reconnect_attempts: 0,
    current_retry_interval: 0,
    next_retry_time: null,
  });
  const [error, setError] = React.useState<string | null>(null);
  const [webrtcError, setWebrtcError] = React.useState<string | null>(null);
  const [isGeneratingOffer, setIsGeneratingOffer] = React.useState(false);
  const [isConnecting, setIsConnecting] = React.useState(false);
  const [isDisconnecting, setIsDisconnecting] = React.useState(false);
  const [creatingRoom, setCreatingRoom] = React.useState(false);

  // UI state
  const [clientId, setClientId] = React.useState("");
  const [authToken, setAuthToken] = React.useState("");
  const [role, setRole] = React.useState<'sender' | 'receiver'>('sender');
  const [offerSdp, setOfferSdp] = React.useState("");
  const [createdRoomId, setCreatedRoomId] = React.useState<string | null>(null);
  const [createdSessionId, setCreatedSessionId] = React.useState<string | null>(null);
  const [roomId, setRoomId] = React.useState<string | null>(null);

  // Set up event listeners for real-time updates
  React.useEffect(() => {
    const unlistenFns: (() => void)[] = [];

    const setupEventListeners = async () => {
      try {
        // Listen for state changes
        const unlistenStateChanged = await listen<ConnectionState>('signal-manager:state-changed', (event) => {
          console.log('State changed event received:', event.payload);
          setState(event.payload);
          setError(null); // Clear any previous errors when state changes
        });
        unlistenFns.push(unlistenStateChanged);

        // Listen for connection events
        const unlistenConnecting = await listen('signal-manager:connecting', () => {
          console.log('Connecting event received');
          setIsConnecting(true);
          setError(null);
        });
        unlistenFns.push(unlistenConnecting);

        const unlistenConnected = await listen('signal-manager:connected', () => {
          console.log('Connected event received');
          setIsConnecting(false);
          setError(null);
        });
        unlistenFns.push(unlistenConnected);

        const unlistenDisconnecting = await listen('signal-manager:disconnecting', () => {
          console.log('Disconnecting event received');
          setIsDisconnecting(true);
        });
        unlistenFns.push(unlistenDisconnecting);

        const unlistenDisconnected = await listen('signal-manager:disconnected', () => {
          console.log('Disconnected event received');
          setIsDisconnecting(false);
        });
        unlistenFns.push(unlistenDisconnected);

        const unlistenReconnecting = await listen<number>('signal-manager:reconnecting', (event) => {
          console.log('Reconnecting event received, attempt:', event.payload);
        });
        unlistenFns.push(unlistenReconnecting);

        // Listen for error events
        const unlistenConnectionError = await listen<string>('signal-manager:connection-error', (event) => {
          console.log('Connection error event received:', event.payload);
          setError(`Connection failed: ${event.payload}`);
          setIsConnecting(false);
        });
        unlistenFns.push(unlistenConnectionError);

        const unlistenDisconnectError = await listen<string>('signal-manager:disconnect-error', (event) => {
          console.log('Disconnect error event received:', event.payload);
          setError(`Disconnect failed: ${event.payload}`);
          setIsDisconnecting(false);
        });
        unlistenFns.push(unlistenDisconnectError);

        const unlistenGeneralError = await listen<string>('signal-manager:error', (event) => {
          console.log('General error event received:', event.payload);
          setError(event.payload);
        });
        unlistenFns.push(unlistenGeneralError);

        // Listen for room events
        const unlistenRoomCreating = await listen('room:creating', () => {
          console.log('Room creating event received');
          setCreatingRoom(true);
          setError(null);
        });
        unlistenFns.push(unlistenRoomCreating);

        const unlistenRoomCreated = await listen<RoomCreatedEvent>('room:created', (event) => {
          console.log('Room created event received:', event.payload);
          setCreatingRoom(false);
          setCreatedRoomId(event.payload.roomId);
          setCreatedSessionId(event.payload.sessionId);
          setRoomId(event.payload.roomId);
          setError(null);
        });
        unlistenFns.push(unlistenRoomCreated);

        const unlistenRoomCreationError = await listen<string>('room:creation-error', (event) => {
          console.log('Room creation error event received:', event.payload);
          setCreatingRoom(false);
          setError(`Room creation failed: ${event.payload}`);
        });
        unlistenFns.push(unlistenRoomCreationError);

      } catch (err) {
        console.error('Failed to set up event listeners:', err);
        setError(`Failed to set up event listeners: ${err instanceof Error ? err.message : 'Unknown error'}`);
      }
    };

    setupEventListeners();

    // Cleanup function
    return () => {
      unlistenFns.forEach(unlisten => unlisten());
    };
  }, []);

  // UI Event Handlers - Pure UI logic only
  const handleConnect = async () => {
    setIsConnecting(true);
    setError(null);
    
    try {
      await invoke('init_signal_manager', {
        url: "127.0.0.1",
        port: 8080,
        clientId,
        authToken,
      });
      await invoke('connect_signal_manager');
    } catch (err) {
      setError(`Connection failed: ${err instanceof Error ? err.message : 'Unknown error'}`);
      setIsConnecting(false);
    }
  };

  const handleDisconnect = async () => {
    setIsDisconnecting(true);
    setError(null);
    
    try {
      await invoke('disconnect_signal_manager');
      await invoke('cleanup_webrtc_connection');
      // Clear form fields when disconnecting
      setClientId("");
      setAuthToken("");
      setOfferSdp("");
      setCreatedRoomId(null);
      setCreatedSessionId(null);
      setRoomId(null);
    } catch (err) {
      setError(`Disconnect failed: ${err instanceof Error ? err.message : 'Unknown error'}`);
      setIsDisconnecting(false);
    }
  };

  const generateWebRTCOffer = async () => {
    setIsGeneratingOffer(true);
    setWebrtcError(null);
    
    try {
      const offer = await invoke<WebRTCOffer>('generate_webrtc_offer');
      setOfferSdp(offer.sdp);
      return offer;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to generate WebRTC offer';
      setWebrtcError(errorMessage);
      return null;
    } finally {
      setIsGeneratingOffer(false);
    }
  };

  const clearWebRTCOffer = () => {
    setOfferSdp("");
  };

  const handleCreateRoom = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!clientId || !authToken) return;
    setCreatingRoom(true);
    setCreatedRoomId(null);
    setCreatedSessionId(null);
    setRoomId(null);
    try {
      // If role is sender and no offer SDP is provided, generate one
      let finalOfferSdp = offerSdp;
      if (role === 'sender' && !offerSdp) {
        const webrtcOffer = await generateWebRTCOffer();
        if (webrtcOffer) {
          finalOfferSdp = webrtcOffer.sdp;
          setOfferSdp(webrtcOffer.sdp);
        } else {
          console.error('Failed to generate WebRTC offer');
          return;
        }
      }
      const [roomIdResult, sessionId] = await invoke<[string | null, string | null]>('send_room_create', {
        version: "1.0.0",
        clientId: clientId,
        authToken: authToken,
        role: role,
        offerSdp: finalOfferSdp || undefined,
        metadata: {
          userAgent: navigator.userAgent,
          timestamp: Date.now(),
        },
      });
      setCreatedRoomId(roomIdResult || null);
      setCreatedSessionId(sessionId || null);
      setRoomId(roomIdResult || null);
    } catch (error) {
      console.error('Failed to create room:', error);
      setCreatingRoom(false);
    }
  };

  // UI state computations
  const canConnect = !state.is_connected && !state.is_connecting && !state.is_reconnecting && !isConnecting;
  const canDisconnect = state.is_connected && !state.is_connecting && !state.is_reconnecting && !isDisconnecting;
  const canCreateRoom = state.is_connected && clientId && authToken;

  return (
    <div className="min-h-screen flex items-center justify-center bg-blue-900/60">
      <div className="bg-white rounded-lg shadow-lg p-6 max-w-md w-full mx-4">
        {/* Status message */}
        <div className="text-sm text-gray-600 mb-4 text-center">
          {state.is_connected && '✅ Connected to server'}
          {state.is_connecting && '🔄 Connecting to server...'}
          {state.is_reconnecting && `🔄 Reconnecting to server... (Attempt ${state.reconnect_attempts})`}
          {!state.is_connected && !state.is_connecting && !state.is_reconnecting && '❌ Disconnected from server'}
        </div>
        <div className="space-y-4">
          <ConnectionStatus 
            state={{
              stateType: state.state_type,
              isConnected: state.is_connected,
              isConnecting: state.is_connecting,
              isReconnecting: state.is_reconnecting,
              lastHeartbeat: state.last_heartbeat,
              reconnectAttempts: state.reconnect_attempts,
              currentRetryInterval: state.current_retry_interval,
              nextRetryTime: state.next_retry_time,
            }}
            error={error}
          />
          <div className="flex space-x-2">
            <Button
              onClick={handleConnect}
              disabled={!canConnect}
              className="flex-1 bg-green-600 hover:bg-green-700 disabled:bg-gray-400"
            >
              {isConnecting ? 'Connecting...' : state.is_connecting ? 'Connecting...' : state.is_reconnecting ? 'Reconnecting...' : 'Connect'}
            </Button>
            <Button
              onClick={handleDisconnect}
              disabled={!canDisconnect}
              className="flex-1 bg-red-600 hover:bg-red-700 disabled:bg-gray-400"
            >
              {isDisconnecting ? 'Disconnecting...' : 'Disconnect'}
            </Button>
          </div>

          {/* Debug info */}
          <div className="text-xs text-gray-500 bg-gray-50 p-2 rounded">
            <div>Connection: {state.is_connected ? '✅' : '❌'}</div>
            <div>Client ID: {clientId ? '✅' : '❌'}</div>
            <div>Auth Token: {authToken ? '✅' : '❌'}</div>
            <div>WebRTC Offer: {offerSdp ? '✅' : '❌'}</div>
            <div>Button enabled: {canCreateRoom ? '✅' : '❌'}</div>
          </div>

          {/* Room ID display */}
          {roomId && (
            <div className="text-sm text-blue-800 bg-blue-100 rounded p-2 mb-2">
              <b>Current Room ID:</b> {roomId}
            </div>
          )}

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
              <div className="flex space-x-2 mb-2">
                <Button
                  type="button"
                  onClick={generateWebRTCOffer}
                  disabled={isGeneratingOffer}
                  className="text-xs bg-purple-600 hover:bg-purple-700 disabled:bg-gray-400"
                >
                  {isGeneratingOffer ? 'Generating...' : 'Generate WebRTC Offer'}
                </Button>
                {offerSdp && (
                  <Button
                    type="button"
                    onClick={clearWebRTCOffer}
                    className="text-xs bg-gray-600 hover:bg-gray-700"
                  >
                    Clear
                  </Button>
                )}
              </div>
              <textarea
                className="border rounded px-2 py-1 text-sm"
                value={offerSdp}
                onChange={e => setOfferSdp(e.target.value)}
                placeholder="Enter SDP offer or click 'Generate WebRTC Offer' to auto-generate"
                rows={4}
              />
              {webrtcError && (
                <div className="text-xs text-red-600 mt-1">
                  WebRTC Error: {webrtcError}
                </div>
              )}
              {offerSdp && (
                <div className="text-xs text-green-600 mt-1">
                  ✅ WebRTC offer generated successfully
                </div>
              )}
            </div>
            <Button
              type="submit"
              disabled={!canCreateRoom || creatingRoom}
              className="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400"
            >
              {creatingRoom ? 'Creating Room...' : 'Create WebRTC Room'}
            </Button>
            {createdRoomId && (
              <div className="text-xs text-green-700 bg-green-100 rounded p-2 mt-2">
                <div>✅ Room created successfully!</div>
                <div><b>Room ID:</b> {createdRoomId}</div>
                {createdSessionId && <div><b>Session ID:</b> {createdSessionId}</div>}
              </div>
            )}
          </form>
        </div>
      </div>
    </div>
  );
}
