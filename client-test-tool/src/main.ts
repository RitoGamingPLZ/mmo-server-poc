import './style.css'

type InputCommand =
  | {"Move": { direction: [number, number] }}
  | {"Stop": null};

// New network message format from server
type FieldUpdate = {
  f: string;  // field name (shortened)
  v: any;     // value
};

type ComponentUpdate = {
  c: string;  // component name (shortened)
  u: FieldUpdate[];  // updates (shortened)
};

type EntityUpdate = {
  id: number;  // entity_id (network_id)
  cs: ComponentUpdate[];  // components (shortened)
};

type CompactNetworkMessage = {
  t: string;  // message_type (shortened)
  es: EntityUpdate[];  // entity_updates (shortened)  
  p: number;  // my_player_id (shortened)
};

// Client-side player state with interpolation
type Player = {
  id: number;
  
  // Current authoritative state from server
  serverPosition: { x: number; y: number };
  serverVelocity: { x: number; y: number };
  lastServerUpdate: number;
  
  // Client-side simulated state for smooth movement
  position: { x: number; y: number };
  velocity: { x: number; y: number };
  
  // Visual state
  isMyPlayer: boolean;
};

class GameClient {
  private ws: WebSocket | null = null;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private connectionStatus: HTMLElement;
  private isConnected = false;
  private players: Map<number, Player> = new Map();
  private myPlayerId: number | null = null;
  
  // Client-side simulation
  private lastFrameTime = 0;
  private heartbeatInterval: number | null = null;

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
        this.startHeartbeat();
      };

      this.ws.onclose = () => {
        this.isConnected = false;
        this.connectionStatus.textContent = 'Disconnected';
        this.connectionStatus.className = '';
        this.stopHeartbeat();
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
          
          console.log('Received:', data);
          this.handleNetworkMessage(data);
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
    this.stopHeartbeat();
    if (this.ws) {
      this.ws.close();
    }
  }

  private startHeartbeat() {
    this.stopHeartbeat(); // Clear any existing heartbeat
    this.heartbeatInterval = window.setInterval(() => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        this.ws.send('heartbeat');
      }
    }, 15000); // Send heartbeat every 15 seconds
  }

  private stopHeartbeat() {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }
  }

  private sendCommand(command: InputCommand) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(command));
    }
  }

  private handleNetworkMessage(data: any) {
    // Handle new compact network message format
    if (data && typeof data === 'object') {
      // Extract player ID from new format
      if (data.p !== undefined) {
        this.myPlayerId = data.p;
      }
      
      // Process new component-based format
      if (data.t && data.es && Array.isArray(data.es)) {
        const isFullSync = data.t === 'full_sync';
        
        if (isFullSync) {
          // Clear existing players for full sync
          this.players.clear();
          console.log(`Full sync: ${data.es.length} entities`);
        } else {
          console.log(`Delta update: ${data.es.length} entities changed`);
        }
        
        // Process each entity update
        data.es.forEach((entityUpdate: EntityUpdate) => {
          this.updatePlayerFromEntity(entityUpdate);
        });
        
        return;
      }
      
      // Legacy format support (old player-based messages)
      if (data.my_player_id !== undefined) {
        this.myPlayerId = data.my_player_id;
      }
      
      if (data.type === 'full_sync' && data.players) {
        this.handleLegacyPlayerUpdate(data.players, true);
      } else if (data.type === 'delta_update' && data.updates) {
        this.handleLegacyPlayerUpdate(data.updates, false);
      } else if (data.players) {
        this.handleLegacyPlayerUpdate(data.players, true);
      }
    }
  }

  private updatePlayerFromEntity(entityUpdate: EntityUpdate) {
    const playerId = entityUpdate.id;
    const now = performance.now();
    
    // Get or create player
    let player = this.players.get(playerId);
    if (!player) {
      player = {
        id: playerId,
        serverPosition: { x: 0, y: 0 },
        serverVelocity: { x: 0, y: 0 },
        position: { x: 0, y: 0 },
        velocity: { x: 0, y: 0 },
        lastServerUpdate: now,
        isMyPlayer: playerId === this.myPlayerId
      };
      this.players.set(playerId, player);
    }
    
    // Process component updates
    entityUpdate.cs.forEach((componentUpdate: ComponentUpdate) => {
      if (componentUpdate.c === 'NetworkedPosition') {
        this.updatePosition(player!, componentUpdate.u, now);
      } else if (componentUpdate.c === 'NetworkedVelocity') {
        this.updateVelocity(player!, componentUpdate.u, now);
      }
    });
  }

  private updatePosition(player: Player, fieldUpdates: FieldUpdate[], timestamp: number) {
    fieldUpdates.forEach((update: FieldUpdate) => {
      if (update.f === 'x') {
        player.serverPosition.x = update.v;
      } else if (update.f === 'y') {
        player.serverPosition.y = update.v;
      }
    });
    
    // Update client position to server position
    player.position.x = player.serverPosition.x;
    player.position.y = player.serverPosition.y;
    player.lastServerUpdate = timestamp;
  }

  private updateVelocity(player: Player, fieldUpdates: FieldUpdate[], timestamp: number) {
    fieldUpdates.forEach((update: FieldUpdate) => {
      if (update.f === 'x') {
        player.serverVelocity.x = update.v;
      } else if (update.f === 'y') {
        player.serverVelocity.y = update.v;
      }
    });
    
    // Update client velocity to server velocity
    player.velocity.x = player.serverVelocity.x;
    player.velocity.y = player.serverVelocity.y;
    player.lastServerUpdate = timestamp;
  }

  private handleLegacyPlayerUpdate(players: any[], isFullSync: boolean) {
    if (isFullSync) {
      this.players.clear();
    }
    
    const now = performance.now();
    players.forEach((legacyPlayer: any) => {
      const player: Player = {
        id: legacyPlayer.id,
        serverPosition: { x: legacyPlayer.x || 0, y: legacyPlayer.y || 0 },
        serverVelocity: { x: legacyPlayer.vel_x || 0, y: legacyPlayer.vel_y || 0 },
        position: { x: legacyPlayer.x || 0, y: legacyPlayer.y || 0 },
        velocity: { x: legacyPlayer.vel_x || 0, y: legacyPlayer.vel_y || 0 },
        lastServerUpdate: now,
        isMyPlayer: legacyPlayer.id === this.myPlayerId
      };
      this.players.set(player.id, player);
    });
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
      
      this.renderPlayers();
      
      requestAnimationFrame(render);
    };
    render();
  }

  private renderPlayers() {
    this.players.forEach((player) => {
      const isMyPlayer = player.id === this.myPlayerId;
      
      // Draw player dot
      this.ctx.fillStyle = isMyPlayer ? '#00ff00' : '#ff0000';
      this.ctx.beginPath();
      this.ctx.arc(player.x, player.y, 8, 0, 2 * Math.PI);
      this.ctx.fill();
      
      // Draw player ID
      this.ctx.fillStyle = '#ffffff';
      this.ctx.font = '12px Arial';
      this.ctx.textAlign = 'center';
      this.ctx.fillText(player.id.toString(), player.x, player.y - 12);
      
      // Draw velocity vector if available
      if (player.vel_x !== undefined && player.vel_y !== undefined) {
        const speed = Math.sqrt(player.vel_x * player.vel_x + player.vel_y * player.vel_y);
        if (speed > 1) { // Only draw if moving
          this.ctx.strokeStyle = isMyPlayer ? '#00ff00' : '#ff0000';
          this.ctx.lineWidth = 2;
          this.ctx.beginPath();
          this.ctx.moveTo(player.x, player.y);
          this.ctx.lineTo(player.x + player.vel_x * 0.3, player.y + player.vel_y * 0.3);
          this.ctx.stroke();
        }
      }
    });
  }
}

new GameClient();
