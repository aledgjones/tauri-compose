# 🎼 Compose

> **Music composition simplified.** A powerful desktop application for creating, sequencing, and engraving musical scores with a modern, intuitive interface.

[![Version](https://img.shields.io/badge/version-1.19.0-blue.svg)](https://github.com/aledjones/compose)
[![Tauri](https://img.shields.io/badge/Tauri-2.9-purple.svg)](https://tauri.app)
[![React](https://img.shields.io/badge/React-19.2-61dafb.svg)](https://react.dev)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://rust-lang.org)

---

## ✨ What is Compose?

Compose is a native desktop application that brings together the power of modern web technologies and native performance to create a seamless music composition experience. Built with Tauri, React, and Rust, it combines an elegant UI with high-performance audio rendering and score engraving.

### Key Features

- **🎹 Multi-instrument Sampler** - High-quality instrument patches including orchestral instruments, guitars, keyboards, and percussion
- **✍️ Notation Editor** - Intuitive score writing with professional engraving capabilities
- **🎵 Audio Sequencing** - Powerful sequencing engine with expression control and tempo management
- **🎨 Multiple Layouts** - Organize your score into different layouts for various instruments and ensembles
- **🖥️ Native Performance** - Hardware-accelerated rendering using Skia (Metal on macOS, OpenGL elsewhere)
- **🎯 Modern Workflow** - Tab-based interface with Setup, Write, Engrave, Sequence, and Print modes

---

## 🚀 Getting Started

### Prerequisites

Before you begin, make sure you have the following installed:

- **Node.js** (v18 or later) - [Download](https://nodejs.org/)
- **npm** or **yarn** - Comes with Node.js
- **Rust** (latest stable) - [Install via rustup](https://rustup.rs/)
- **System dependencies** for Tauri development:
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Linux**: See [Tauri Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites#linux)
  - **Windows**: See [Tauri Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites#windows)

### Installation

1. **Clone the repository**

   ```bash
   git clone https://github.com/aledjones/compose.git
   cd compose
   ```

2. **Install dependencies**

   ```bash
   npm install
   ```

3. **Start the development server**

   ```bash
   npm run start:dev
   ```

   This will compile the Rust backend and launch the application in development mode with hot-reload enabled.

### Development Commands

```bash
# Start development mode with hot reload
npm run start:dev

# Build for production
npm run start:release

# Run Vite development server only (no Tauri)
npm run dev

# Type checking
npm run check-types

# Lint code
npm run lint
npm run lint:fix

# Preview production build
npm run preview
```

---

## 🎨 Design Principles

Compose is built with several core principles in mind:

### 1. **Performance First**

- **Native Rendering**: Uses Skia for hardware-accelerated graphics rendering
- **Rust Backend**: Leverages Rust's zero-cost abstractions for audio processing and score rendering
- **Efficient State Management**: Optimized React state with minimal re-renders
- **Web Audio API**: High-performance audio playback with precise scheduling

### 2. **Intuitive User Experience**

- **Tab-Based Workflow**: Clear separation between Setup, Write, Engrave, Sequence, and Print modes
- **Drag & Drop**: Natural interactions for arranging notes, instruments, and layouts
- **Keyboard Shortcuts**: Power user features for rapid composition
- **Real-time Feedback**: Instant visual and audio feedback during editing

### 3. **Professional Quality**

- **High-Quality Samples**: Extensive library of professionally recorded instrument patches
- **Engraving Standards**: Follows music notation best practices
- **Export Options**: Multiple formats for sharing and printing
- **Precision Control**: Fine-grained control over dynamics, articulation, and expression

### 4. **Modern Architecture**

- **Type Safety**: Full TypeScript coverage with strict mode enabled
- **Component-Driven**: Reusable UI components with clear interfaces
- **Separation of Concerns**: Clean boundaries between UI, business logic, and audio engine
- **Testable Code**: Designed for easy testing and maintenance

---

## 🏗️ Project Structure

```
compose/
├── src/                          # Frontend React/TypeScript code
│   ├── windows/                  # Application windows
│   │   ├── main/                 # Main composition window
│   │   │   ├── screens/          # Setup, Write, Sequence, etc.
│   │   │   ├── components/       # Transport, Tools, etc.
│   │   │   ├── hooks/            # Main window specific hooks
│   │   │   └── utils/            # Main window utilities
│   │   ├── debugger/             # Debug console window
│   │   ├── instrument-picker/    # Instrument selection
│   │   ├── layout-settings/      # Layout configuration
│   │   ├── player-settings/      # Playback settings
│   │   ├── preferences/          # Application preferences
│   │   └── project-settings/     # Project preferences
│   ├── sampler/                  # Audio playback engine
│   │   ├── audio-player.ts       # Sample player
│   │   ├── scheduler.ts          # Timing and scheduling
│   │   └── instrument-player.ts  # Per-instrument playback
│   ├── services/                 # Business logic and state management
│   │   ├── state/                # Global application state
│   │   │   ├── context.tsx       # React contexts and state hooks
│   │   │   ├── provider.tsx      # State provider component
│   │   │   └── index.ts          # State module exports
│   │   ├── playback/             # Audio playback state management
│   │   │   ├── context.tsx       # Playback contexts and hooks
│   │   │   ├── provider.tsx      # Playback provider component
│   │   │   └── index.ts          # Playback module exports
│   │   ├── actions.ts            # State mutation actions
│   │   ├── sequencer-utils.ts    # Music sequencing utilities
│   │   └── types.ts              # Shared TypeScript definitions
│   ├── components/               # Shared React components
│   ├── ui/                       # UI components and utilities
│   │   ├── hooks/                # Shared UI hooks
│   │   │   └── use-drag-handler/ # Drag interaction utilities
│   │   ├── text/                 # Text processing and tokenization
│   │   │   ├── context.ts        # Text tokenization context
│   │   │   ├── index.tsx         # Text display components
│   │   │   └── provider.tsx      # Text provider component
│   │   └── [other UI modules]/   # Buttons, inputs, icons, etc.
│   ├── utils/                    # Utility functions
│   └── assets/                   # Static assets
│       └── patches/              # Instrument sample libraries
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── lib.rs                # Library entry point
│   │   ├── main.rs               # Application entry point
│   │   ├── files.rs              # File system operations
│   │   ├── preferences.rs        # Application preferences
│   │   ├── utils.rs              # Utility functions
│   │   ├── migrations/           # soloc file migrations
│   │   ├── native_renderer/      # Skia rendering engine
│   │   ├── render/               # Score rendering logic
│   │   ├── sampler/              # Audio sampling (Rust side)
│   │   ├── score/                # Score data structures
│   │   └── ui/                   # UI bridge to frontend
│   └── tauri.conf.json           # Tauri configuration
├── package.json                  # Node dependencies
├── tsconfig.json                 # TypeScript configuration
├── vite.config.ts                # Vite bundler config
└── postcss.config.js             # PostCSS/Tailwind config
```

---

## 🛠️ Technology Stack

### Frontend

- **React 19** - Modern React with hooks and concurrent features
- **TypeScript 5** - Full type safety with strict mode
- **Vite 7** - Lightning-fast development and optimized builds
- **Tailwind CSS 4** - Utility-first styling
- **Material Design Icons** - Comprehensive icon library

### Backend

- **Tauri 2.9** - Secure, lightweight native app framework
- **Rust 2021** - Memory-safe systems programming
- **Skia (skia-safe)** - Hardware-accelerated 2D graphics
- **Serde** - Efficient serialization between Rust and JavaScript

### Audio

- **Web Audio API** - Native browser audio processing
- **Custom Scheduler** - Precise timing for musical events
- **AudioContext** - Low-latency audio playback
- **Expression Engine** - Dynamic articulation and expression control

---

## 🎯 Recommended IDE Setup

For the best development experience:

- **[VS Code](https://code.visualstudio.com/)** with extensions:
  - [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) - Tauri development tools
  - [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) - Rust language support
  - [ESLint](https://marketplace.visualstudio.com/items?itemName=dbaeumer.vscode-eslint) - JavaScript/TypeScript linting
  - [Prettier](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode) - Code formatting
  - [Tailwind CSS IntelliSense](https://marketplace.visualstudio.com/items?itemName=bradlc.vscode-tailwindcss) - Tailwind autocomplete

---

## 📝 Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes with clear, descriptive commits
4. Run linting and type checks (`npm run lint && npm run check-types`)
5. Test your changes thoroughly
6. Submit a pull request

---

## 📄 License

This project is the property of Aled Jones. See the repository for licensing information.

---

## 🙏 Acknowledgments

- Built with [Tauri](https://tauri.app/) - The modern app framework
- UI powered by [React](https://react.dev/) and [Tailwind CSS](https://tailwindcss.com/)
- Graphics rendering by [Skia](https://skia.org/)
- Icons from [Material Design Icons](https://materialdesignicons.com/)

---

**Made with ❤️ by Aled Jones**
