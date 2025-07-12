# Keahi Ambient Agent

A **Rust-powered desktop application** using Tauri, with a React UI frontend for user interaction.

## Architecture

This is **primarily a Rust application** that provides:
- System information gathering
- Command execution capabilities
- Environment variable access
- User agent functionality
- Cross-platform desktop app capabilities

The React frontend serves as a modern UI layer for interacting with the Rust backend.

## Tech Stack

- **Primary**: Rust (Tauri backend)
- **UI Layer**: React + TypeScript
- **Styling**: Tailwind CSS + shadcn/ui
- **Build Tool**: Vite
- **Package Manager**: npm (for frontend), Cargo (for Rust)

## Rust Backend Features

The Rust application provides several commands:

- `get_system_info()` - Retrieves OS, architecture, hostname, and username
- `get_user_agent_info()` - Returns agent name, version, and capabilities
- `execute_command()` - Executes system commands safely
- `get_environment_vars()` - Retrieves environment variables
- `greet()` - Simple greeting function

## Development

### Prerequisites

- Rust (latest stable)
- Node.js (v18 or higher)
- System dependencies for Tauri (see [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites))

### Running the Application

1. **Install dependencies**:
   ```bash
   # Frontend dependencies
   npm install
   
   # Rust dependencies (handled by Cargo)
   cd src-tauri
   cargo check
   cd ..
   ```

2. **Development mode**:
   ```bash
   # Full Tauri app (Rust + React)
   npm run tauri dev
   
   # Frontend only (for UI development)
   npm run dev
   ```

3. **Building**:
   ```bash
   # Build frontend
   npm run build
   
   # Build complete desktop app
   npm run tauri build
   ```

## Project Structure

```
user_agent/
├── src-tauri/           # 🦀 Rust backend (PRIMARY)
│   ├── src/
│   │   ├── main.rs      # Application entry point
│   │   └── lib.rs       # Core Rust logic & commands
│   ├── Cargo.toml       # Rust dependencies
│   └── tauri.conf.json  # Tauri configuration
├── src/                 # ⚛️ React frontend (UI layer)
│   ├── components/      # React components
│   ├── App.tsx         # Main React component
│   └── main.tsx        # React entry point
├── index.html           # HTML entry point
├── vite.config.ts       # Vite configuration
├── tailwind.config.js   # Tailwind configuration
└── package.json         # Frontend dependencies
```

## Rust Commands

The Rust backend exposes these commands to the frontend:

```rust
// System information
get_system_info() -> SystemInfo

// Agent information  
get_user_agent_info() -> UserAgentInfo

// Command execution
execute_command(command: &str, args: Vec<String>) -> Result<String, String>

// Environment variables
get_environment_vars() -> Result<HashMap<String, String>, String>

// Simple greeting
greet(name: &str) -> String
```

## Available Scripts

- `npm run dev` - Start frontend development server
- `npm run build` - Build frontend for production
- `npm run preview` - Preview frontend build
- `npm run tauri dev` - Start full Tauri app (Rust + React)
- `npm run tauri build` - Build complete desktop application

## Building for Distribution

The Rust application can be built into native executables:

```bash
# Build for current platform
npm run tauri build

# This creates native executables in src-tauri/target/release/
```

## Key Features

- **Rust Backend**: High-performance system operations
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Modern UI**: React with Tailwind CSS and shadcn/ui
- **Type Safety**: Full TypeScript support
- **Hot Reload**: Fast development experience
- **Native Performance**: Rust provides near-native performance

## Customization

- **Rust Logic**: Modify `src-tauri/src/lib.rs` for backend functionality
- **UI Components**: Add React components in `src/components/`
- **Styling**: Customize Tailwind in `tailwind.config.js`
- **Tauri Config**: Adjust app settings in `src-tauri/tauri.conf.json`
