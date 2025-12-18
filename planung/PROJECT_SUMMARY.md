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

### Core Components

#### Frontend Architecture (`frontend/`)
```
src/
├── components/          # Reusable UI components (158+ files)
│   ├── ui/             # Base components (Card, Button, Table, etc.)
│   ├── modules/        # Feature modules (19+ modules)
│   ├── layout/         # Layout components
│   ├── visualizations/ # Charts and data viz
│   └── RadialHub/      # Main dashboard
├── hooks/              # Custom React hooks
├── stores/             # Zustand state management
├── api/                # API client and types (25+ API modules)
├── utils/              # Utility functions
└── styles/             # CSS and design tokens
```

#### Backend Architecture (`src-tauri/`)
```
src/
├── commands/           # Tauri command handlers (25+ modules)
├── providers/          # System service providers (9 providers)
├── storage/            # Database and persistence (18 modules)
├── entity_extraction/  # NLP and entity processing
├── scenario_engine/    # Simulation and scenario management
├── world_graph/        # Graph database integration
├── vector_store/       # Vector embeddings and search
├── ws.rs              # WebSocket server implementation
└── main.rs            # Application bootstrap
```

## 🎯 Core Features & Modules

### 1. **System Monitor Hub** ⚡
- **Real-time Metrics**: CPU, memory, disk, GPU, network usage
- **Process Management**: Tree visualization, process killing, resource monitoring
- **System Health**: Comprehensive system diagnostics and alerts
- **Performance Profiling**: Command execution timing and optimization

### 2. **Network Constellation** 🌐
- **Connection Monitoring**: Active network connections with bandwidth tracking
- **Interface Analysis**: Network interface statistics and configuration
- **Firewall Management**: Rule inspection and basic firewall controls
- **DNS Resolution**: DNS lookup and caching
- **Speed Testing**: Ookla Speedtest integration with historical data

### 3. **AI Consciousness** 🤖
- **Multi-Model Support**: OpenAI, Anthropic, local models
- **Chat Interface**: Conversational AI with context management
- **Prompt Templates**: Reusable prompt engineering templates
- **Usage Analytics**: Token usage tracking and cost optimization
- **Embedding Generation**: Vector embeddings for semantic search

### 4. **DevOps Control** 🔧
- **Prometheus Integration**: Metrics scraping and visualization
- **Health Monitoring**: Service health checks and alerts
- **Synthetic Testing**: Automated API endpoint testing
- **Telemetry Collection**: Performance and error tracking
- **Alert Management**: Alertmanager integration and configuration

### 5. **Automation Circuit** 🔄
- **Script Management**: JavaScript/TypeScript script execution
- **Workflow Automation**: Trigger-based automation with scheduling
- **Plugin System**: Extensible automation architecture
- **Script Gallery**: Shared automation scripts and templates
- **Advanced Triggers**: Complex conditional automation logic

### 6. **Packages Repository** 📦
- **Homebrew Integration**: Complete package management for macOS
- **Dependency Analysis**: Package dependency visualization
- **Service Management**: Homebrew service control and monitoring
- **Cache Management**: Package cache cleanup and optimization
- **Version Pinning**: Package version locking and upgrades

### 7. **Reality & Timeline Studio** 🕐
- **OSINT Integration**: RSS feed processing and entity extraction
- **World Graph**: Neo4j-powered knowledge graph with temporal features
- **Entity Extraction**: spaCy and DeepKE integration for NLP
- **Scenario Engine**: Time-based simulation and scenario planning
- **Temporal Analytics**: Time-series analysis and forecasting

### 8. **Vector Store Manager** 🗃️
- **Multi-Collection Support**: Organized vector storage
- **Semantic Search**: Vector similarity search with filtering
- **TTL Management**: Automatic data expiration
- **Batch Operations**: Bulk vector operations and imports
- **Index Optimization**: Vector index performance monitoring

### 9. **Security Center** 🔐
- **Authentication**: PIN-based secure access control
- **Permission Management**: Granular permission system
- **Audit Logging**: Comprehensive security event tracking
- **Rate Limiting**: API rate limiting with analytics
- **Access Control**: Role-based access management

### 10. **System Utilities** 🛠️
- **Disk Management**: Storage analysis and cleanup
- **Service Control**: System service management (macOS/Windows)
- **Power Management**: Sleep prevention and display controls
- **Hardware Diagnostics**: System hardware health checks
- **System Information**: Comprehensive system profiling

### 11. **Create Hub** 🎨 (Development Environment)
- **Playground**: p5.js, three.js, vanilla JS creative coding
- **Shader Studio**: GLSL fragment shader editor with live preview
- **Script Lab**: JavaScript/TypeScript scripting environment
- **Games & Experiments**: Interactive experiments gallery
- **Godot Integration**: Game development project management

### 12. **Testing Center** 🧪
- **Unit Testing**: Comprehensive test suite with Vitest
- **Integration Testing**: API and component integration tests
- **E2E Testing**: Playwright-based end-to-end automation
- **Visual Regression**: UI consistency testing
- **Test Analytics**: Test coverage and performance metrics

### 13. **Configuration Manager** ⚙️
- **Dynamic Configuration**: Runtime configuration management
- **Schema Validation**: JSON Schema-based config validation
- **Environment Detection**: Automatic environment adaptation
- **Config Persistence**: Secure configuration storage

### 14. **Migration Manager** 📊
- **Database Migrations**: SQLite schema management
- **Data Transformation**: Automated data migration pipelines
- **Migration Tracking**: Migration status and rollback capabilities
- **Integrity Validation**: Data integrity checks and repairs

### 15. **WebSocket Monitor** 🔌
- **Connection Analytics**: WebSocket connection monitoring
- **Message Inspection**: Real-time message analysis
- **Topic Management**: Pub/sub topic monitoring
- **Performance Metrics**: Connection latency and throughput

### 16. **Error Dashboard** ⚠️
- **Error Aggregation**: Centralized error collection and analysis
- **Error Categorization**: Intelligent error classification
- **Trend Analysis**: Error pattern detection and alerting
- **Resolution Tracking**: Error resolution workflow management

### 17. **Rate Limit Monitor** ⏱️
- **Rate Limit Analytics**: API rate limiting visualization
- **Bucket Management**: Token bucket algorithm monitoring
- **Usage Patterns**: Rate limit usage pattern analysis
- **Alerting**: Rate limit threshold alerts

### 18. **Vector Search** 🔍
- **Advanced Search**: Multi-vector similarity search
- **Filter Support**: Metadata-based filtering and faceting
- **Search Analytics**: Search performance and usage metrics
- **Result Visualization**: Interactive search result exploration

### 19. **Advanced Analytics** 📈
- **Data Visualization**: Interactive charts and dashboards
- **Statistical Analysis**: Advanced statistical computations
- **Trend Detection**: Automated trend analysis and forecasting
- **Report Generation**: Automated analytics report creation

## 🎨 Design System

### Glassmorphism Theme
- **Frosted Glass Effects**: 28px blur radius with backdrop-filter
- **Terminal Aesthetics**: Monospace fonts, ASCII borders, scanlines
- **Neon Color Palette**: Cyan (#00d9ff), Green (#00ff88), Amber (#ffb000), Red (#ff2d55)
- **Terminal Effects**: Phosphor glow, blinking cursors, ASCII separators

### Component Library
- **Cards**: Glass cards with terminal titles and hover effects
- **Buttons**: Primary/secondary/ghost variants with phosphor glow
- **Tables**: Sortable data tables with ASCII borders
- **Charts**: Hybrid ASCII + modern chart visualizations
- **Forms**: Terminal-style input controls
- **Status Indicators**: Blinking dots and badges

### Responsive Design
- **Adaptive Layouts**: Grid-based responsive design
- **Density Controls**: Compact/Balanced/Spacious density options
- **Theme Switching**: Light/Dark theme support
- **Accessibility**: Full keyboard navigation and screen reader support

## 🛠️ Development Setup

### Prerequisites
- **Node.js**: 18.0+ (for frontend)
- **Rust**: Latest stable (for backend)
- **System Dependencies**:
  - macOS: Xcode Command Line Tools
  - Linux: build-essential, libwebkit2gtk
  - Windows: Visual Studio Build Tools

### Quick Start (2 minutes)

1. **Clone and Install**:
```bash
git clone <repository-url>
cd mina
npm install
```

2. **Try New Design System**:
```bash
# Edit frontend/src/main.tsx to import AppNew instead of App
cd frontend
npm run dev
```

3. **Explore Features**:
- Visit `http://localhost:5173`
- Toggle themes and density with top controls
- Navigate through all 19+ modules via top tabs

### Full Development Setup

```bash
# Install frontend dependencies
cd frontend
npm install

# Install Rust dependencies (automatic via Cargo.toml)
# Build and run in development mode
npm run tauri:dev

# Build for production
npm run tauri:build
```

### Available Scripts
```json
{
  "dev": "vite",           // Frontend dev server
  "build": "production build",
  "test": "vitest",        // Unit tests
  "test:e2e": "playwright", // E2E tests
  "tauri:dev": "tauri dev", // Full app dev
  "tauri:build": "tauri build" // Production build
}
```

## 🔧 Configuration

### Environment Variables
```bash
# WebSocket server
MINA_WS_ADDR=127.0.0.1:17602

# Logging
MINA_LOG_LEVEL=info

# Neo4j (optional, for graph features)
NEO4J_URI=bolt://localhost:7687
NEO4J_USER=neo4j
NEO4J_PASSWORD=password

# AI Services
OPENAI_API_KEY=your_key_here
ANTHROPIC_API_KEY=your_key_here

# External Services (optional)
SPACY_SERVICE_URL=http://localhost:5001
DEEPKE_SERVICE_URL=http://localhost:5002
```

### Database Setup

#### SQLite (Primary Database)
- **Auto-initialized** on first run
- **Location**: `~/Library/Application Support/mina/reality.db`
- **Migrations**: Automatic schema updates

#### Neo4j (Optional, for Graph Features)
```bash
# Using Docker
docker run -d \
  --name neo4j \
  -p 7474:7474 -p 7687:7687 \
  -e NEO4J_AUTH=neo4j/password \
  neo4j:latest
```

#### Qdrant (Vector Store)
- **Auto-initialized** on first run
- **Location**: Application data directory
- **Collections**: Auto-created for different data types

## 📊 Data Flow & Architecture

### Real-time Streaming
```
System Sensors → Providers → WebSocket Server → React Components → UI Updates
```

### API Architecture
- **Tauri Commands**: 150+ typed commands between frontend/backend
- **REST-like Interface**: Command-based API with request/response
- **Streaming Support**: WebSocket for real-time data
- **Type Safety**: Specta-generated TypeScript bindings

### State Management
- **Local State**: Zustand stores for UI state
- **Server State**: React Query for API data
- **Persistent State**: SQLite for long-term storage
- **Real-time State**: WebSocket subscriptions

### Storage Layers
1. **SQLite**: Primary data persistence
2. **Neo4j**: Graph relationships and complex queries
3. **Qdrant**: Vector embeddings and similarity search
4. **File System**: Logs, configurations, assets

## 🧪 Testing Strategy

### Test Types
- **Unit Tests**: Component and utility function testing (Vitest)
- **Integration Tests**: API and provider testing
- **E2E Tests**: Full user workflow testing (Playwright)
- **Visual Regression**: UI consistency testing
- **Performance Tests**: Load and stress testing

### Test Coverage
- **Frontend**: 80%+ coverage target
- **Backend**: Integration tests for all providers
- **API**: Full command handler testing

### CI/CD Pipeline
```yaml
# GitHub Actions workflow
- Lint & Type Check
- Unit Tests
- Integration Tests
- Build & Package
- Release Automation
```

## 🚀 Deployment & Distribution

### Desktop App Build
```bash
# Build for current platform
npm run tauri:build

# Cross-platform builds
npm run tauri:build -- --target x86_64-apple-darwin    # macOS Intel
npm run tauri:build -- --target aarch64-apple-darwin   # macOS Apple Silicon
npm run tauri:build -- --target x86_64-pc-windows-msvc # Windows
```

### Platform Support
- **macOS**: 11.0+ (Intel/Apple Silicon)
- **Windows**: 10+ (x64)
- **Linux**: Ubuntu 18.04+, CentOS 7+ (x64)

### Distribution Channels
- **GitHub Releases**: Automated releases with installers
- **Auto-updating**: Built-in update mechanism via Tauri
- **Homebrew**: macOS package distribution

## 🔒 Security & Privacy

### Authentication
- **PIN-based Access**: Secure local authentication
- **Session Management**: Automatic session timeout
- **Audit Logging**: All authentication events logged

### Data Protection
- **Local Storage**: All data stored locally (no cloud required)
- **Encryption**: Sensitive data encrypted at rest
- **Permission System**: Granular access controls

### Network Security
- **Local Communication**: WebSocket server bound to localhost
- **Rate Limiting**: API protection against abuse
- **Input Validation**: Comprehensive input sanitization

## 📈 Performance Optimization

### Frontend Optimizations
- **Code Splitting**: Route-based and component-based splitting
- **Lazy Loading**: Components loaded on demand
- **Bundle Analysis**: Webpack Bundle Analyzer integration
- **Image Optimization**: Lazy loading and compression

### Backend Optimizations
- **Async Processing**: Tokio async runtime
- **Connection Pooling**: Database connection reuse
- **Caching**: Multi-level caching strategy
- **Profiling**: Built-in performance monitoring

### Memory Management
- **Efficient Data Structures**: Optimized Rust data structures
- **Streaming Processing**: Large dataset streaming
- **Resource Cleanup**: Automatic resource management

## 🐛 Troubleshooting

### Common Issues

#### Glass Effects Not Showing
```css
/* Ensure CSS imports are in correct order in index.css */
@import url('./styles/tokens.css');
@import url('./styles/glass.css');
@import url('./styles/terminal-effects.css');
```

#### Performance Issues with Blur
```tsx
// Use optimized blur hook for heavy blur effects
import { useOptimizedBlur } from './utils/performance'

function MyComponent() {
  const { ref, style } = useOptimizedBlur()
  return <div ref={ref} style={style} className="glass-card">Content</div>
}
```

#### Database Connection Issues
```bash
# Check SQLite database integrity
sqlite3 ~/Library/Application\ Support/mina/reality.db ".integrity_check"

# Reset Neo4j connection
rm ~/Library/Application\ Support/mina/config.json
# Restart app to trigger auto-detection
```

## 🔄 Migration & Updates

### Database Migrations
- **Automatic**: Schema updates applied on startup
- **Versioned**: All migrations tracked and checksummed
- **Rollback**: Migration rollback capabilities

### Configuration Migration
- **Backward Compatible**: Old configs automatically migrated
- **Validation**: Configuration validation on load
- **Backup**: Automatic configuration backups

## 🌟 Advanced Features

### AI Integration
- **Multi-Provider Support**: OpenAI, Anthropic, Ollama
- **Context Management**: Conversation history and context
- **Prompt Engineering**: Template system for complex prompts
- **Embedding Search**: Semantic search across all content

### Graph Analytics
- **Temporal Graph**: Time-aware relationship modeling
- **Entity Extraction**: NLP-powered entity recognition
- **Graph Algorithms**: Neo4j Graph Data Science integration
- **Visualization**: Interactive graph exploration

### Automation Engine
- **Script Execution**: Sandboxed JavaScript/TypeScript runtime
- **Trigger System**: Event-driven automation
- **Workflow Management**: Complex automation pipelines
- **Plugin Architecture**: Extensible automation system

## 📚 Documentation

### Developer Documentation
- **API Reference**: Complete API documentation
- **Component Library**: UI component documentation
- **Architecture Guide**: System architecture documentation
- **Contributing Guide**: Development workflow and standards

### User Documentation
- **Quick Start**: 2-minute setup guide
- **Feature Guides**: Detailed feature documentation
- **Troubleshooting**: Common issues and solutions
- **Best Practices**: Usage recommendations

## 🤝 Contributing

### Development Workflow
1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Write** tests for new functionality
4. **Implement** the feature with comprehensive documentation
5. **Ensure** all tests pass and linting succeeds
6. **Submit** a pull request with detailed description

### Code Standards
- **TypeScript Strict Mode**: All type checking enabled
- **ESLint + Prettier**: Automated code formatting
- **Conventional Commits**: Standardized commit messages
- **Comprehensive Testing**: 80%+ test coverage required

### Architecture Principles
- **Modular Design**: Clear separation of concerns
- **Type Safety**: Full TypeScript/Rust type coverage
- **Performance First**: Optimized for desktop performance
- **Accessibility**: WCAG 2.1 AA compliance
- **Security**: Defense-in-depth security approach

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- **Tauri**: Desktop application framework
- **React**: UI framework and ecosystem
- **Rust**: Systems programming language
- **Neo4j**: Graph database
- **Qdrant**: Vector database
- **OpenAI/Anthropic**: AI service providers
- **Homebrew**: macOS package management

---

**MINA** - *Monitoring, Intelligence, Networking, Automation*

A comprehensive system assistant that combines the power of modern desktop applications with AI-driven insights and automation capabilities. Built for developers, system administrators, and power users who demand both beauty and functionality in their tools.
