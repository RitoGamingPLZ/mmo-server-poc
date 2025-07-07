// Main game client class that orchestrates all modules

import { NetworkManager } from './network';
import { PlayerManager } from './player';
import { Renderer } from './renderer';
import { InputHandler } from './input';
import type { EntityUpdate } from './types';

export class GameClient {
  private networkManager: NetworkManager;
  private playerManager: PlayerManager;
  private renderer: Renderer;
  private inputHandler: InputHandler;
  private lastFrameTime = 0;

  constructor() {
    // Get DOM elements
    const canvas = document.getElementById('game-canvas') as HTMLCanvasElement;
    const connectionStatus = document.getElementById('connection-status')!;

    // Initialize modules
    this.networkManager = new NetworkManager(connectionStatus);
    this.playerManager = new PlayerManager();
    this.renderer = new Renderer(canvas);
    this.inputHandler = new InputHandler();

    // Set up communication between modules
    this.setupModuleCommunication();
    this.setupConnectionControls();
    this.startRenderLoop();
  }

  private setupModuleCommunication(): void {
    // Network message handling
    this.networkManager.setMessageHandler((data: any) => {
      this.handleNetworkMessage(data);
    });

    // Input command handling
    this.inputHandler.setSendCommandCallback((command) => {
      if (this.networkManager.connected) {
        this.networkManager.sendCommand(command);
      }
    });
  }

  private setupConnectionControls(): void {
    document.getElementById('btn-connect')?.addEventListener('click', () => {
      this.networkManager.connect();
    });
    
    document.getElementById('btn-disconnect')?.addEventListener('click', () => {
      this.networkManager.disconnect();
    });
  }

  private handleNetworkMessage(data: any): void {
    // Handle compact network message format from server
    if (data && typeof data === 'object') {
      // Extract player ID from new format
      if (data.p !== undefined) {
        this.playerManager.setMyPlayerId(data.p);
      }
      
      // Process component-based format (current server implementation)
      if (data.t && data.es && Array.isArray(data.es)) {
        const messageType = data.t;
        const isFullSync = messageType === 'full_sync';
        
        if (isFullSync) {
          // Clear existing players for full sync
          this.playerManager.clearPlayers();
          console.log(`🔄 Full sync received: ${data.es.length} entities`);
        } else if (messageType === 'delta_update') {
          console.log(`📦 Delta update received: ${data.es.length} entities changed`);
        }
        
        // Process each entity update
        data.es.forEach((entityUpdate: EntityUpdate) => {
          this.playerManager.updatePlayerFromEntity(entityUpdate);
        });
        
        return;
      }
      
      // Legacy format support (for backward compatibility)
      if (data.my_player_id !== undefined) {
        this.playerManager.setMyPlayerId(data.my_player_id);
      }
      
      if (data.type === 'full_sync' && data.players) {
        console.log('📦 Legacy full sync received');
        this.playerManager.handleLegacyPlayerUpdate(data.players, true);
      } else if (data.type === 'delta_update' && data.updates) {
        console.log('📦 Legacy delta update received');
        this.playerManager.handleLegacyPlayerUpdate(data.updates, false);
      } else if (data.players) {
        console.log('📦 Legacy player update received');
        this.playerManager.handleLegacyPlayerUpdate(data.players, true);
      }
    }
  }

  private startRenderLoop(): void {
    const render = (currentTime: number) => {
      const deltaTime = currentTime - this.lastFrameTime;
      this.lastFrameTime = currentTime;
      
      // Update client-side movement simulation
      const canvasSize = this.renderer.getCanvasSize();
      this.playerManager.updateClientSideMovement(deltaTime, canvasSize.width, canvasSize.height);
      
      // Render the game
      this.renderer.clear();
      this.renderer.renderPlayers(this.playerManager.getAllPlayers(), this.playerManager.getMyPlayerId());
      
      requestAnimationFrame(render);
    };
    requestAnimationFrame(render);
  }
}