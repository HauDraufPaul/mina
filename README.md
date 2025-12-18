# MINA - Comprehensive System Assistant & Monitoring Platform

## 🚀 Project Overview

MINA is a sophisticated desktop application built with Tauri, React, and Rust that provides comprehensive system monitoring, automation, AI integration, and DevOps capabilities. It features a beautiful glassmorphism UI with terminal aesthetics and supports real-time data visualization, process management, network analysis, and much more.

## 🏗️ Architecture

### Technology Stack
- **Frontend**: React 18 + TypeScript + Vite + Tailwind CSS
- **Backend**: Rust + Tauri 2.0 + SQLite
- **UI Framework**: Custom glassmorphism design system with terminal aesthetics
- **State Management**: Zustand + React Query
- **Database**: SQLite (primary) + Neo4j (optional for graph features)
- **Vector Store**: Qdrant (LanceDB integration)
- **Real-time**: WebSocket-based streaming architecture

## 🛠️ Development Setup

### Prerequisites
- **Node.js**: 18.0+ (for frontend)
- **Rust**: Latest stable (for backend)
- **System Dependencies**:
  - macOS: Xcode Command Line Tools
  - Linux: build-essential, libwebkit2gtk
  - Windows: Visual Studio Build Tools

### Quick Start

1. **Install Dependencies**:
```bash
npm install
```

2. **Run in Development Mode**:
```bash
npm run tauri:dev
```

3. **Build for Production**:
```bash
npm run tauri:build
```

### Available Scripts
- `npm run dev` - Frontend dev server
- `npm run build` - Production build
- `npm run tauri:dev` - Full app dev mode
- `npm run tauri:build` - Production build
- `npm test` - Run tests
- `npm run test:e2e` - Run E2E tests

## 🎯 Core Features & Modules

### 1. **System Monitor Hub** ⚡
- Real-time CPU, memory, disk, GPU, network usage
- Process management and resource monitoring
- System health diagnostics and alerts

### 2. **Network Constellation** 🌐
- Active network connections with bandwidth tracking
- Network interface statistics and configuration
- Firewall management and DNS resolution

### 3. **AI Consciousness** 🤖
- Multi-model support (OpenAI, Anthropic, local models)
- Conversational AI with context management
- Prompt templates and usage analytics

### 4. **DevOps Control** 🔧
- Prometheus integration
- Service health checks and alerts
- Synthetic testing and telemetry collection

### 5. **Automation Circuit** 🔄
- JavaScript/TypeScript script execution
- Workflow automation with scheduling
- Plugin system and script gallery

### 6. **Packages Repository** 📦
- Homebrew integration for macOS
- Dependency analysis and visualization
- Service management and cache optimization

### 7. **Reality & Timeline Studio** 🕐
- OSINT integration and RSS feed processing
- Entity extraction and Neo4j graph integration
- Time-based simulation and scenario planning

### 8. **Vector Store Manager** 🗃️
- Multi-collection vector storage
- Semantic search with filtering
- TTL management and batch operations

### 9. **Security Center** 🔐
- PIN-based secure access control
- Permission management and audit logging
- Rate limiting and access control

### 10. **System Utilities** 🛠️
- Disk management and storage analysis
- Service control and power management
- Hardware diagnostics and system profiling

### 11. **Create Hub** 🎨
- Playground (p5.js, three.js, vanilla JS)
- GLSL fragment shader editor
- Script Lab and interactive experiments

### 12. **Testing Center** 🧪
- Unit testing with Vitest
- Integration and E2E testing
- Test coverage and performance metrics

### 13-19. **Additional Modules**
- Configuration Manager
- Migration Manager
- WebSocket Monitor
- Error Dashboard
- Rate Limit Monitor
- Vector Search
- Advanced Analytics

## 🎨 Design System

### Glassmorphism Theme
- Frosted glass effects with 28px blur radius
- Terminal aesthetics with monospace fonts
- Neon color palette (Cyan, Green, Amber, Red)
- Terminal effects (phosphor glow, blinking cursors, ASCII separators)

## 📊 Project Structure

```
mina/
├── src/                    # Frontend React code
│   ├── components/         # UI components
│   │   ├── ui/            # Base components
│   │   ├── modules/        # Feature modules
│   │   └── layout/        # Layout components
│   ├── styles/            # CSS and design tokens
│   ├── stores/            # Zustand state management
│   ├── api/               # API client and types
│   └── utils/             # Utility functions
├── src-tauri/             # Rust backend
│   ├── src/               # Rust source code
│   │   ├── commands/      # Tauri command handlers
│   │   ├── providers/     # System service providers
│   │   └── storage/       # Database and persistence
│   └── Cargo.toml         # Rust dependencies
└── public/                # Static assets
```

## 🔧 Configuration

### Environment Variables
```bash
# WebSocket server
MINA_WS_ADDR=127.0.0.1:17602

# Logging
MINA_LOG_LEVEL=info

# Neo4j (optional)
NEO4J_URI=bolt://localhost:7687
NEO4J_USER=neo4j
NEO4J_PASSWORD=password

# AI Services
OPENAI_API_KEY=your_key_here
ANTHROPIC_API_KEY=your_key_here
```

## 🚀 Deployment

### Desktop App Build
```bash
# Build for current platform
npm run tauri:build

# Cross-platform builds
npm run tauri:build -- --target x86_64-apple-darwin    # macOS Intel
npm run tauri:build -- --target aarch64-apple-darwin   # macOS Apple Silicon
npm run tauri:build -- --target x86_64-pc-windows-msvc # Windows
```

## 📄 License

This project is licensed under the MIT License.

---

**MINA** - *Monitoring, Intelligence, Networking, Automation*

A comprehensive system assistant that combines the power of modern desktop applications with AI-driven insights and automation capabilities.

