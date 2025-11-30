# WAQL Tool

<div align="center">

A powerful graphical query tool for Wwise WAQL (Wwise Authoring Query Language)

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../LICENSE)

</div>

## 📖 Introduction

WAQL Tool is a graphical query tool designed for Audiokinetic Wwise, helping audio designers and game developers quickly query and manage objects in Wwise projects through an intuitive interface.

## 📷 Screenshot

<div align="center">
  <img src="../img/screen.png" alt="WAQL Tool Screenshot" width="800">
</div>

### ✨ Main Features

- 🎨 **Syntax Highlighting** - WAQL syntax highlighting for better code readability
- 💡 **Intelligent Completion** - Built-in code completion for WAAPI properties and accessors
- 📊 **Result Visualization** - Display query results in a clear table format
- 📁 **CSV Export** - One-click export of query results to CSV
- 💾 **Query Saving** - Save frequently used queries for quick reuse
- 🎯 **Custom Keywords** - Add project-specific custom keywords
- 🎨 **Multiple Themes** - Built-in editor themes
- ⚙️ **Persistent Configuration** - Automatically save user settings and preferences

## 🚀 Quick Start

### Prerequisites

- Wwise 2021 or later (WAAPI must be enabled)
- Windows OS

### Installation

#### Method 1: Download Pre-built Version (Recommended)

1. Visit the [Releases page](https://github.com/xmimu/waql-tool/releases)
2. Download the latest `waql-tool.exe`
3. Double-click to run

#### Method 2: Build from Source

If you want to modify the code or use the latest development version:

1. **Prerequisites**
   - Rust 1.70 or later

2. **Clone the repository**
```bash
git clone https://github.com/xmimu/waql-tool.git
cd waql-tool
```

3. **Build the project**
```bash
# Development build
cargo build

# Release build (recommended)
cargo build --release
```

4. **Run the application**
```bash
# Development mode
cargo run

# Release mode
cargo run --release
```

The executable will be in `target/release/waql-tool.exe`

### Configure Wwise

Make sure WAAPI is enabled in Wwise:

1. Open Wwise
2. Go to `Project` -> `User Preferences` -> `WAAPI`
3. Check `Enable WAAPI`
4. Restart Wwise

## 📚 User Guide

### Basic Usage

1. **Write Query**
   - Enter WAQL query in the code editor
   - Use `Ctrl+Space` or type to trigger code completion
   - Press `Enter` or click "Run" to execute the query

2. **View Results**
   - Results are displayed in a table below
   - Scroll to view all columns and rows
   - Shows the number of returned objects

3. **Export Data**
   - Click "Export CSV"
   - Choose save location and filename
   - Results will be saved as CSV

### WAQL Query Examples

```waql
# Find all sound objects
$ from type Sound

# Find objects by name
$ from object "\\Actor-Mixer Hierarchy\\Default Work Unit\\MySound"

# Filter with where clause
$ from type Sound where name:"Footstep"

# Find child objects
$ from type ActorMixer select children
```

### Support for options after | separator

```waql
# Select specific properties after |
$ from type ActorMixer | name @Volume
```

### Settings Panel

Click "Settings" to open the panel, where you can:

- 📝 **Saved Queries** - Manage frequently used queries
  - Click to quickly load into the editor
  - Delete unused queries
  
- 🔤 **Custom Keywords** - Add project-specific keywords
  - Enter keyword and click "Add"
  - Keywords appear in code completion
  
- 🎨 **Editor Theme** - Choose editor color scheme
  - GRUVBOX (default)
  - GITHUB DARK
  - AURA
  - And more
  
- 🔤 **Font Size** - Adjust editor font size (8-24)

## 🏗️ Project Structure

```
waql-tool/
├── src/
│   ├── main.rs              # Application entry
│   ├── lib.rs               # Library entry
│   ├── config.rs            # Config management
│   ├── query_executor.rs    # Query executor
│   ├── ui.rs                # UI rendering
│   ├── fonts/               # Custom fonts
│   │   └── SIMKAI.TTF
│   └── waql/
│       ├── mod.rs           # WAQL module
│       ├── properties.rs    # WAAPI property definitions
│       └── syntax.rs        # Syntax highlighting
├── py_helper/               # Python helper tools
│   ├── main.py
│   ├── pyproject.toml
│   └── README.md
├── Cargo.toml               # Rust project config
├── REFACTORING.md           # Refactoring docs
└── README.md                # This file
```

### Module Description

- **config** - Serialization, deserialization, and persistence of user config
- **query_executor** - WAQL query execution, result parsing, and data conversion
- **ui** - All UI rendering logic
- **waql** - WAQL syntax, WAAPI property and accessor list

## 🔧 Tech Stack

- **[egui](https://github.com/emilk/egui)** - Immediate mode GUI framework
- **[eframe](https://github.com/emilk/egui/tree/master/crates/eframe)** - Native window backend for egui
- **[egui_code_editor](https://github.com/rylev/egui-code-editor)** - Code editor component
- **[waapi-rs](https://github.com/xmimu/waapi-rs)** - Wwise WAAPI Rust client
- **[serde](https://serde.rs/)** - Serialization framework
- **[rfd](https://github.com/PolyMeilex/rfd)** - Native file dialog

## 📝 Config File

User config is saved in:
```
.\user_data.json
```

Config includes:
- Saved queries
- Custom keywords
- Editor theme
- Font size

## 🤝 Contributing

Issues and Pull Requests are welcome!

### Development Workflow

1. Fork this repo
2. Create a feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

### Code Style

- Follow Rust official style
- Use `cargo fmt` to format code
- Use `cargo clippy` for linting
- Add doc comments for public APIs

## 📄 License

This project is licensed under the MIT License. See [LICENSE](../LICENSE) for details.

## 🙏 Acknowledgements

- [Audiokinetic](https://www.audiokinetic.com/) - Powerful Wwise middleware
- [egui](https://github.com/emilk/egui) - Excellent Rust GUI framework
- All contributors and users

## 📧 Contact

- Project Home: https://github.com/xmimu/waql-tool
- Issues: https://github.com/xmimu/waql-tool/issues

---

<div align="center">
Made with ❤️ by <a href="https://github.com/xmimu">xmimu</a>
</div>
