// Rendering module for the game canvas

import type { Player } from './types';

export class Renderer {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d')!;
  }

  clear(): void {
    this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    
    // Draw background
    this.ctx.fillStyle = '#333';
    this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
    
    // Draw title
    this.ctx.fillStyle = '#646cff';
    this.ctx.font = '16px Arial';
    this.ctx.textAlign = 'center';
    this.ctx.fillText('Game World', this.canvas.width / 2, 30);
  }

  renderPlayers(players: Player[], myPlayerId: number | null): void {
    players.forEach((player) => {
      const isMyPlayer = player.id === myPlayerId;
      
      // Draw player dot
      this.ctx.fillStyle = isMyPlayer ? '#00ff00' : '#ff0000';
      this.ctx.beginPath();
      this.ctx.arc(player.position.x, player.position.y, 8, 0, 2 * Math.PI);
      this.ctx.fill();
      
      // Draw player ID
      this.ctx.fillStyle = '#ffffff';
      this.ctx.font = '12px Arial';
      this.ctx.textAlign = 'center';
      this.ctx.fillText(player.id.toString(), player.position.x, player.position.y - 12);
      
      // Draw velocity vector if available
      const speed = Math.sqrt(player.velocity.x * player.velocity.x + player.velocity.y * player.velocity.y);
      if (speed > 1) { // Only draw if moving
        this.ctx.strokeStyle = isMyPlayer ? '#00ff00' : '#ff0000';
        this.ctx.lineWidth = 2;
        this.ctx.beginPath();
        this.ctx.moveTo(player.position.x, player.position.y);
        this.ctx.lineTo(player.position.x + player.velocity.x * 0.3, player.position.y + player.velocity.y * 0.3);
        this.ctx.stroke();
      }
    });
  }

  getCanvasSize(): { width: number; height: number } {
    return {
      width: this.canvas.width,
      height: this.canvas.height
    };
  }
}