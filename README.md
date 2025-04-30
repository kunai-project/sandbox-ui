<div align="center"><img src="assets/logo.svg" width="150"/></div>

# Kunai Sandbox UI

Welcome to the Kunai Sandbox UI project! This UI complements the [Kunai Sandbox](https://github.com/kunai-project/sandbox) platform, offering a user-friendly interface for detection engineers to generate malware Kunai traces.

## About

The Kunai Sandbox UI aims to streamline the process of analyzing malware samples by providing a dedicated interface for detection engineers. Instead of competing with existing sandboxing solutions, it focuses on delivering actionable data directly usable by engineers working with Kunai.

## Features

- **User-Friendly Interface**: Simplifies interaction with Kunai Sandbox.
- **Detailed Kunai Traces**: Access to detailed Kunai traces and network traffic dumps.
- **Community Sharing**: Enables the community to share Kunai traces and collaborate on threat analysis and detection.

## Non-Goals

- **Defeat Anti-Sandboxing Techniques**: This UI does not focus on bypassing anti-sandboxing measures.
- **Use Hypervisor-Based Sandboxing**: The UI is designed to work with the existing Kunai Sandbox platform.
- **Unpack Malware Samples**: The primary goal is to provide analysis data, not to unpack malware.

# How to Build

## Requirements

These steps are required only the first time you build the application.

1. This project uses `npm` to build the frontend, so you will need to install `npm`.
2. Ensure you have `rustup` installed.
3. Install the **Node.js** modules:
   ```bash
   cd frontend
   npm install
   ```

## Building the Application

```bash
# Build in debug mode
cargo build-app

# Build in release mode
cargo build-app --release
```
    
## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more information.
