# 🎨 Color War - Chain Reaction Game 🎮

## 🌟 Overview
Color War is a strategic board game where players compete to control the board by placing and exploding tiles in a chain reaction. The game features a colorful interface with animated explosions and captures, creating an engaging and dynamic gameplay experience.

## 🚀 Game Features
- **Multiplayer Support**: Play with up to 4 players on the same device
- **Strategic Gameplay**: Balance offense and defense to dominate the board
- **Chain Reactions**: Watch as your moves trigger cascading explosions across the board
- **Animated Interface**: Enjoy smooth animations for tile placement and explosions
- **Elimination Mechanics**: Players are eliminated when they lose all their tiles

## 📜 Game Rules

### 🛠️ Setup
- The game is played on an 8x8 grid
- Players take turns in a randomized order
- Each player is assigned a unique color (Red, Green, Blue, or Yellow)

### 🎲 First Move
- On a player's first turn, they can place a tile on any empty cell
- First move tiles start with a power level of 3

### 🔄 Subsequent Moves
- After the first move, players can only add tiles to cells they already own
- Each placement increases the cell's power level by 1

### 💥 Explosions
- When a cell's power level exceeds its capacity (4 for all cells), it explodes
- An explosion resets the cell's power to 0 and distributes power to adjacent cells
- Adjacent cells are captured by the exploding player regardless of previous ownership
- This can trigger chain reactions if adjacent cells also exceed their capacity

### 🏆 Winning
- A player is eliminated when they have no tiles left on the board
- The last player with tiles on the board wins the game

## 🎛️ Controls
- **Mouse**: Click on a cell to place a tile

## 📥 Installation

### 📋 Prerequisites
- Rust programming language (https://www.rust-lang.org/tools/install)
- Cargo package manager (included with Rust)

### 🛠️ Building and Running
1. Clone the repository
2. Navigate to the project directory
3. Run the following command:
   ```
   cargo run --release
   ```

## 🔧 Technical Details

### 📦 Dependencies
- **ggez**: A lightweight game framework for making 2D games with minimum friction
- **rand**: Random number generation for player order and game mechanics

### 💻 Implementation
- Written in Rust for performance and safety
- Uses the GGEZ game framework for graphics and input handling
- Features smooth animations for explosions and tile captures
- Implements a turn-based system with player elimination logic

## 🧠 Strategy Tips
- Corner cells are strategic as they only have two adjacent cells
- Edge cells are also valuable with only three adjacent cells
- Try to create chain reactions to capture multiple cells at once
- Be careful not to place tiles that could be easily captured by opponents
- Sometimes it's better to reinforce your existing positions than to expand

## 🙏 Credits
Developed as a Rust implementation of the classic Chain Reaction game concept.
