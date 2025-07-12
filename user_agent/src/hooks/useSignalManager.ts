import { useState, useEffect, useCallback, useRef } from 'react';
import { SignalManagerClient, ConnectionState, Message, ConnectionStateType } from '@/lib/signal-manager-client';
import { loadConfig, Config } from '@/lib/config';

export interface UseSignalManagerReturn {
  isConnected: boolean;
  isConnecting: boolean;
  isReconnecting: boolean;
  lastHeartbeat: number;
  reconnectAttempts: number;
  error: string | null;
  connect: () => Promise<void>;
  disconnect: () => void;
  messages: Message[];
  stateType: ConnectionStateType;
  currentRetryInterval: number;
  nextRetryTime: number | null;
}

export function useSignalManager(): UseSignalManagerReturn {
  const [state, setState] = useState<ConnectionState>({
    stateType: ConnectionStateType.DISCONNECTED_NOT_TO_CONNECT,
    isConnected: false,
    isConnecting: false,
    isReconnecting: false,
    lastHeartbeat: 0,
    reconnectAttempts: 0,
    currentRetryInterval: 0,
    nextRetryTime: null,
  });
  const [error, setError] = useState<string | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [config, setConfig] = useState<Config | null>(null);
  const clientRef = useRef<SignalManagerClient | null>(null);

  // Load configuration
  useEffect(() => {
    loadConfig()
      .then(setConfig)
      .catch((err) => setError(`Failed to load config: ${err.message}`));
  }, []);

  // Initialize client when config is loaded
  useEffect(() => {
    if (!config) return;

    const client = new SignalManagerClient(config.signal_manager);
    
    client.setCallbacks(
      (newState) => {
        setState(newState);
        setError(null); // Clear error on successful state change
      },
      (message) => {
        setMessages(prev => [...prev, message]);
      },
      (error) => {
        setError(error.message);
      }
    );

    clientRef.current = client;

    return () => {
      if (clientRef.current) {
        clientRef.current.disconnect();
      }
    };
  }, [config]);

  const connect = useCallback(async () => {
    if (!clientRef.current) {
      setError('Client not initialized');
      return;
    }

    try {
      setError(null);
      await clientRef.current.connect();
    } catch (err) {
      setError(`Connection failed: ${err instanceof Error ? err.message : 'Unknown error'}`);
    }
  }, []);

  const disconnect = useCallback(() => {
    if (clientRef.current) {
      clientRef.current.disconnect();
    }
  }, []);

  return {
    ...state,
    error,
    connect,
    disconnect,
    messages,
    stateType: state.stateType,
    currentRetryInterval: state.currentRetryInterval,
    nextRetryTime: state.nextRetryTime,
  };
} 