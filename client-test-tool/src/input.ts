// Input handling module

import type { InputCommand } from './types';

export class InputHandler {
  private sendCommandCallback?: (command: InputCommand) => void;
  private keysPressed = new Set<string>();
  private lastSentCommand: InputCommand | null = null;
  private inputInterval: number | null = null;

  constructor() {
    this.setupEventListeners();
    this.startInputLoop();
  }

  setSendCommandCallback(callback: (command: InputCommand) => void): void {
    this.sendCommandCallback = callback;
  }

  private sendCommand(command: InputCommand): void {
    if (this.sendCommandCallback) {
      this.sendCommandCallback(command);
      this.lastSentCommand = command;
    }
  }

  private startInputLoop(): void {
    // Send input updates at 30Hz to ensure responsive controls
    this.inputInterval = window.setInterval(() => {
      this.processInput();
    }, 1000 / 30); // 30 FPS
  }

  private processInput(): void {
    const currentCommand = this.getCurrentInputCommand();
    
    // Only send if command changed or we're actively moving
    if (!this.commandsEqual(currentCommand, this.lastSentCommand)) {
      if (currentCommand) {
        this.sendCommand(currentCommand);
      }
    } else if (currentCommand && currentCommand.Move) {
      // Continuously send move commands to maintain momentum
      this.sendCommand(currentCommand);
    }
  }

  private getCurrentInputCommand(): InputCommand | null {
    const moveKeys = {
      'w': [0, -1], 'ArrowUp': [0, -1],
      's': [0, 1], 'ArrowDown': [0, 1],
      'a': [-1, 0], 'ArrowLeft': [-1, 0],
      'd': [1, 0], 'ArrowRight': [1, 0]
    } as Record<string, [number, number]>;

    let x = 0, y = 0;

    // Combine all pressed movement keys
    for (const key of this.keysPressed) {
      if (moveKeys[key]) {
        x += moveKeys[key][0];
        y += moveKeys[key][1];
      }
    }

    // Handle stop key
    if (this.keysPressed.has(' ')) {
      return { Stop: null };
    }

    // Return movement command if any movement keys are pressed
    if (x !== 0 || y !== 0) {
      // Normalize diagonal movement
      const magnitude = Math.sqrt(x * x + y * y);
      if (magnitude > 1) {
        x /= magnitude;
        y /= magnitude;
      }
      return { Move: { direction: [x, y] } };
    }

    return null;
  }

  private commandsEqual(a: InputCommand | null, b: InputCommand | null): boolean {
    if (a === null && b === null) return true;
    if (a === null || b === null) return false;
    
    if ('Move' in a && 'Move' in b) {
      return a.Move.direction[0] === b.Move.direction[0] && 
             a.Move.direction[1] === b.Move.direction[1];
    }
    
    if ('Stop' in a && 'Stop' in b) {
      return true;
    }
    
    return false;
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

    // Keyboard controls - track key state for continuous input
    const inputKeys = ['w', 'ArrowUp', 's', 'ArrowDown', 'a', 'ArrowLeft', 'd', 'ArrowRight', ' '];

    document.addEventListener('keydown', (e) => {
      if (inputKeys.includes(e.key) && !e.repeat) {
        this.keysPressed.add(e.key);
        if (e.key === ' ') e.preventDefault();
      }
    });

    document.addEventListener('keyup', (e) => {
      if (inputKeys.includes(e.key)) {
        this.keysPressed.delete(e.key);
        if (e.key === ' ') e.preventDefault();
      }
    });

    // Handle window blur to prevent stuck keys
    window.addEventListener('blur', () => {
      this.keysPressed.clear();
    });
  }

  destroy(): void {
    if (this.inputInterval) {
      clearInterval(this.inputInterval);
      this.inputInterval = null;
    }
  }
}