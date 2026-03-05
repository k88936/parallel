import { useEffect, useRef, useState, useCallback } from 'react';
import type { HumanNotification, HumanMessage } from '@/types/task';

const WS_BASE = process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:3000';

export interface UseWebSocketOptions {
  taskId: string;
  onMessage?: (notification: HumanNotification) => void;
  onConnect?: () => void;
  onDisconnect?: () => void;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
}

export interface WebSocketState {
  isConnected: boolean;
  isReconnecting: boolean;
  reconnectAttempts: number;
  error: string | null;
}

export function useWebSocket({
  taskId,
  onMessage,
  onConnect,
  onDisconnect,
  reconnectInterval = 3000,
  maxReconnectAttempts = 10,
}: UseWebSocketOptions) {
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttemptsRef = useRef(0);

  const [state, setState] = useState<WebSocketState>({
    isConnected: false,
    isReconnecting: false,
    reconnectAttempts: 0,
    error: null,
  });

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    const wsUrl = `${WS_BASE}/ws/human?task_id=${taskId}`;
    console.log('Connecting to WebSocket:', wsUrl);

    const ws = new WebSocket(wsUrl);
    wsRef.current = ws;

    ws.onopen = () => {
      console.log('WebSocket connected');
      reconnectAttemptsRef.current = 0;
      setState({
        isConnected: true,
        isReconnecting: false,
        reconnectAttempts: 0,
        error: null,
      });
      onConnect?.();
    };

    ws.onmessage = (event) => {
      try {
        const notification: HumanNotification = JSON.parse(event.data);
        console.log('Received notification:', notification);
        onMessage?.(notification);
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      setState((prev) => ({
        ...prev,
        error: 'WebSocket connection error',
      }));
    };

    ws.onclose = () => {
      console.log('WebSocket disconnected');
      setState((prev) => ({
        ...prev,
        isConnected: false,
      }));
      onDisconnect?.();

      if (reconnectAttemptsRef.current < maxReconnectAttempts) {
        setState((prev) => ({
          ...prev,
          isReconnecting: true,
          reconnectAttempts: reconnectAttemptsRef.current,
        }));
        reconnectTimeoutRef.current = setTimeout(() => {
          reconnectAttemptsRef.current++;
          connect();
        }, reconnectInterval);
      } else {
        setState((prev) => ({
          ...prev,
          isReconnecting: false,
          error: 'Max reconnection attempts reached',
        }));
      }
    };
  }, [taskId, onMessage, onConnect, onDisconnect, reconnectInterval, maxReconnectAttempts]);

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }
    wsRef.current?.close();
    wsRef.current = null;
  }, []);

  const sendMessage = useCallback((message: HumanMessage) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
      console.log('Sent message:', message);
    } else {
      console.error('WebSocket is not connected');
    }
  }, []);

  useEffect(() => {
    connect();
    return () => {
      disconnect();
    };
  }, []);

  return {
    ...state,
    sendMessage,
    disconnect,
    reconnect: connect,
  };
}
