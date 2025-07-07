// Player management and state handling

import type { Player, EntityUpdate, ComponentUpdate, FieldUpdate } from './types';

export class PlayerManager {
  private players: Map<number, Player> = new Map();
  private myPlayerId: number | null = null;

  setMyPlayerId(playerId: number): void {
    this.myPlayerId = playerId;
    
    // Update existing player if found
    const player = this.players.get(playerId);
    if (player) {
      player.isMyPlayer = true;
    }
  }

  getMyPlayerId(): number | null {
    return this.myPlayerId;
  }

  clearPlayers(): void {
    this.players.clear();
  }

  getAllPlayers(): Player[] {
    return Array.from(this.players.values());
  }

  getPlayer(playerId: number): Player | undefined {
    return this.players.get(playerId);
  }

  updatePlayerFromEntity(entityUpdate: EntityUpdate): void {
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
      switch (componentUpdate.c) {
        case 'NetworkedPosition':
          this.updatePosition(player!, componentUpdate.u, now);
          break;
        case 'NetworkedVelocity':
          this.updateVelocity(player!, componentUpdate.u, now);
          break;
        default:
          // Log unknown components for debugging new server features
          console.log(`🔍 Unknown component: ${componentUpdate.c}`, componentUpdate.u);
          break;
      }
    });
  }

  private updatePosition(player: Player, fieldUpdates: FieldUpdate[], timestamp: number): void {
    fieldUpdates.forEach((update: FieldUpdate) => {
      switch (update.f) {
        case 'x':
          if (typeof update.v === 'number') {
            player.serverPosition.x = update.v;
          }
          break;
        case 'y':
          if (typeof update.v === 'number') {
            player.serverPosition.y = update.v;
          }
          break;
        default:
          console.log(`🔍 Unknown position field: ${update.f} = ${update.v}`);
          break;
      }
    });
    
    // Server reconciliation - smoothly correct client position
    const timeSinceUpdate = timestamp - player.lastServerUpdate;
    const errorThreshold = 5.0; // pixels
    
    const positionError = Math.sqrt(
      Math.pow(player.position.x - player.serverPosition.x, 2) +
      Math.pow(player.position.y - player.serverPosition.y, 2)
    );
    
    if (positionError > errorThreshold || player.isMyPlayer) {
      // Significant error or local player - snap to server position for authority
      player.position.x = player.serverPosition.x;
      player.position.y = player.serverPosition.y;
    } else {
      // Small error - interpolate smoothly for remote players
      const lerpFactor = Math.min(1.0, (timeSinceUpdate / 100)); // 100ms lerp time
      player.position.x = player.position.x + (player.serverPosition.x - player.position.x) * lerpFactor;
      player.position.y = player.position.y + (player.serverPosition.y - player.position.y) * lerpFactor;
    }
    
    player.lastServerUpdate = timestamp;
  }

  private updateVelocity(player: Player, fieldUpdates: FieldUpdate[], timestamp: number): void {
    fieldUpdates.forEach((update: FieldUpdate) => {
      switch (update.f) {
        case 'x':
          if (typeof update.v === 'number') {
            player.serverVelocity.x = update.v;
          }
          break;
        case 'y':
          if (typeof update.v === 'number') {
            player.serverVelocity.y = update.v;
          }
          break;
        default:
          console.log(`🔍 Unknown velocity field: ${update.f} = ${update.v}`);
          break;
      }
    });
    
    // For non-local players, use server velocity directly
    // For local player, we could implement prediction here
    if (!player.isMyPlayer) {
      player.velocity.x = player.serverVelocity.x;
      player.velocity.y = player.serverVelocity.y;
    } else {
      // For local player, blend with server velocity for smoother feel
      const blend = 0.7; // 70% server, 30% client
      player.velocity.x = player.velocity.x * (1 - blend) + player.serverVelocity.x * blend;
      player.velocity.y = player.velocity.y * (1 - blend) + player.serverVelocity.y * blend;
    }
    
    player.lastServerUpdate = timestamp;
  }

  handleLegacyPlayerUpdate(players: any[], isFullSync: boolean): void {
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

  updateClientSideMovement(deltaTimeMs: number, canvasWidth: number, canvasHeight: number): void {
    const deltaTime = deltaTimeMs / 1000; // Convert to seconds
    
    this.players.forEach((player) => {
      // Simple client-side movement simulation
      // Update position based on current velocity
      player.position.x += player.velocity.x * deltaTime;
      player.position.y += player.velocity.y * deltaTime;
      
      // Apply friction (same as server-side)
      const friction = 0.95;
      player.velocity.x *= Math.pow(friction, deltaTime);
      player.velocity.y *= Math.pow(friction, deltaTime);
      
      // Stop very small velocities
      if (Math.abs(player.velocity.x) < 0.01) player.velocity.x = 0;
      if (Math.abs(player.velocity.y) < 0.01) player.velocity.y = 0;
      
      // Keep within canvas bounds
      player.position.x = Math.max(10, Math.min(canvasWidth - 10, player.position.x));
      player.position.y = Math.max(40, Math.min(canvasHeight - 10, player.position.y));
    });
  }
}