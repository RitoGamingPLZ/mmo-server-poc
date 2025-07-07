// Input handling module

import type { InputCommand } from './types';

export class InputHandler {
  private sendCommandCallback?: (command: InputCommand) => void;

  constructor() {
    this.setupEventListeners();
  }

  setSendCommandCallback(callback: (command: InputCommand) => void): void {
    this.sendCommandCallback = callback;
  }

  private sendCommand(command: InputCommand): void {
    if (this.sendCommandCallback) {
      this.sendCommandCallback(command);
    }
  }

  private setupEventListeners(): void {
    // Button controls
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

    // Keyboard controls
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
      const command = keyMap[e.key];
      if (command) {
        this.sendCommand(command);
        if (e.key === ' ') e.preventDefault();
      }
    });
  }
}