# Model Checker

**Model Checker** is a high-performance, multi-modal AI orchestrator and playground built with **Rust**, **Dioxus 0.7**, and **Tailwind CSS**. It enables developers and enthusiasts to interact with multiple AI models simultaneously, compare responses, and manage both local and cloud-based inference from a unified interface.

## 🚀 Features

- **Multi-Model Interaction**: Send a single prompt to multiple models (e.g., Phi-4, ChatGPT, Gemini) and see their responses side-by-side.
- **Hybrid Compute**: Support for both local inference (via the `ai-core` local provider) and cloud-based providers.
- **Hardware Optimized**: Built-in support for model optimizations targeting **CPU**, **GPU**, and **NPU** for maximum local performance.
- **Real-time Streaming**: Seamlessly stream responses from all connected models with a responsive, modern UI.
- **Extensible Architecture**: A modular `ai-core` library that makes it easy to add new model providers and compute modes.
- **Dioxus Native Experience**: Leveraging Dioxus 0.7 for a truly cross-platform desktop experience.

## 🛠️ Technology Stack

- **Core Logic**: [Rust](https://rust-lang.org)
- **Frontend Framework**: [Dioxus 0.7](https://dioxuslabs.com)
- **Styling**: [Tailwind CSS](https://tailwindcss.com)
- **Async Runtime**: [Tokio](https://tokio.rs)
- **AI Core**: Custom internal workspace for model management and inference abstraction.

## 📂 Project Structure

- `src/`: Frontend application code, including component layouts and the orchestrator UI.
- `ai-core/`: The heart of the application, containing model provider traits, local inference logic, and cloud API integrations.
- `models-optimization/`: specialized configurations and tools for optimizing local model execution across different hardware backends (CPU/GPU/NPU).
- `assets/`: UI assets, styles, and static resources.

## 🚦 Getting Started

> [!IMPORTANT]
> **Windows Only**: This project currently only supports Windows. Support for other platforms is not yet implemented.

### Prerequisites

- [Rust Toolchain](https://rust-lang.org/learn/get-started) (Edition 2024 recommended)
- [Dioxus CLI](https://dioxuslabs.com/learn/0.7/getting_started) (`cargo install dioxus-cli`)

### Quick Start

1. Clone the repository:
   ```bash
   git clone https://github.com/neeleshpoli/model-checker.git
   cd model-checker
   ```

2. Set up your environment variables (optional, for cloud providers) in a `.env` file:
   ```env
   # Example .env content
   OPENAI_API_KEY=your_key_here
   GEMINI_API_KEY=your_key_here
   ```

3. Run the application in development mode:
   ```bash
   dx serve
   ```

4. For desktop execution:
   ```bash
   dx serve --platform desktop
   ```

## 📜 License

This project is licensed under the [MIT License](LICENSE).
