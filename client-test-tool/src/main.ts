import './style.css'
import { encode, decode } from '@msgpack/msgpack';

type InputCommand =
  | {"Move": { direction: [number, number] }}
  | {"Stop": null};

class GameClient {
  private ws: WebSocket | null = null;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private connectionStatus: HTMLElement;
  private isConnected = false;

  constructor() {
    this.canvas = document.getElementById('game-canvas') as HTMLCanvasElement;
    this.ctx = this.canvas.getContext('2d')!;
    this.connectionStatus = document.getElementById('connection-status')!;
    this.setupEventListeners();
    this.startRenderLoop();
  }

  connect() {
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
          const data = decode(new Uint8Array(event.data));
          console.log('Received:', data);
        } catch (error) {
          console.error('Failed to decode message:', error);
        }
      };

      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };
    } catch (error) {
      console.error('Failed to connect:', error);
    }
  }

  disconnect() {
    if (this.ws) {
      this.ws.close();
    }
  }

  private sendCommand(command: InputCommand) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(command));
    }
  }

  private setupEventListeners() {
    document.getElementById('btn-connect')?.addEventListener('click', () => this.connect());
    document.getElementById('btn-disconnect')?.addEventListener('click', () => this.disconnect());

    const directions = {
      'btn-up': [0, -1],
      'btn-down': [0, 1],
      'btn-left': [-1, 0],
      'btn-right': [1, 0]
    } as Record<string, [number, number]>;

    Object.entries(directions).forEach(([id, direction]) => {
      document.getElementById(id)?.addEventListener('click', () => {
        this.sendCommand({ Move: { direction } });
      });
    });

    document.getElementById('btn-stop')?.addEventListener('click', () => {
      this.sendCommand({ Stop: null });
    });

    const keyMap: Record<string, InputCommand> = {
      'w': { Move: { direction: [0, -1] } },
      'ArrowUp': { Move: { direction: [0, -1] } },
      's': { Move: { direction: [0, 1] } },
      'ArrowDown': { Move: { direction: [0, 1] } },
      'a': { Move: { direction: [-1, 0] } },
      'ArrowLeft': { Move: { direction: [-1, 0] } },
      'd': { Move: { direction: [1, 0] } },
      'ArrowRight': { Move: { direction: [1, 0] } },
      ' ': { Stop: null }
    };

    document.addEventListener('keydown', (e) => {
      if (!this.isConnected) return;
      
      const command = keyMap[e.key];
      if (command) {
        this.sendCommand(command);
        if (e.key === ' ') e.preventDefault();
      }
    });
  }

  private startRenderLoop() {
    const render = () => {
      this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
      
      this.ctx.fillStyle = '#333';
      this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
      
      this.ctx.fillStyle = '#646cff';
      this.ctx.font = '16px Arial';
      this.ctx.textAlign = 'center';
      this.ctx.fillText('Game World', this.canvas.width / 2, 30);
      
      requestAnimationFrame(render);
    };
    render();
  }
}

new GameClient();
