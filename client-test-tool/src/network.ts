// Network communication module

import { decode } from './utils';
import type { 
  InputCommand, 
  CompactNetworkMessage, 
  EntityUpdate, 
  ComponentUpdate, 
  FieldUpdate 
} from './types';

export class NetworkManager {
  private ws: WebSocket | null = null;
  private isConnected = false;
  private connectionStatus: HTMLElement;
  private onMessageCallback?: (data: any) => void;

  constructor(connectionStatusElement: HTMLElement) {
    this.connectionStatus = connectionStatusElement;
  }

  connect(): void {
    if (this.ws?.readyState === WebSocket.OPEN) return;

    try {
      const wsUrl = import.meta.env.VITE_WEBSOCKET_URL || 'ws://localhost:5000';
      this.ws = new WebSocket(wsUrl);
      
      this.ws.onopen = () => {
        this.isConnected = true;
        this.connectionStatus.textContent = 'Connected';
        this.connectionStatus.className = 'connected';
      };

      this.ws.onclose = () => {
        this.isConnected = false;
        this.connectionStatus.textContent = 'Disconnected';
        this.connectionStatus.className = '';
      };

      this.ws.onmessage = (event) => {
        try {
          let data: any;
          
          // Handle both JSON and MessagePack
          if (typeof event.data === 'string') {
            data = JSON.parse(event.data);
          } else {
            data = decode(new Uint8Array(event.data));
          }
          
          if (this.onMessageCallback) {
            this.onMessageCallback(data);
          }
        } catch (error) {
          console.error('❌ Failed to decode message:', error, event.data);
        }
      };

      this.ws.onerror = (error) => {
        console.error('❌ WebSocket error:', error);
        this.connectionStatus.textContent = 'Connection Error';
        this.connectionStatus.className = 'error';
      };
    } catch (error) {
      console.error('Failed to connect:', error);
    }
  }

  disconnect(): void {
    if (this.ws) {
      this.ws.close();
    }
  }

  sendCommand(command: InputCommand): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(command));
    }
  }

  setMessageHandler(callback: (data: any) => void): void {
    this.onMessageCallback = callback;
  }

  get connected(): boolean {
    return this.isConnected;
  }
}