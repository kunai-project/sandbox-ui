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

# Running the Application

1. Install [kunai-sandbox](https://github.com/kunai-project/sandbox)
2. Prepare some sandboxes (using the supported images) using `kunai-sandbox-init`
3. The initialization script generates a `config.yaml` file for each prepared sandbox
   yet the `analysis` section needs to be adjusted.
```yaml
...
analysis:
  timeout: 60
  kunai:
    # this must point to a valid kunai binary for that VM archictecture
    path: /app-data/bin/kunai-amd64
    # arguments to pass to kunai for each analysis
    args: ["run", "--max-eps-fs=8096", "--max-buffered-events=2048", "--send-data-min-len=1", "--harden", "--include=all"]
  tcpdump: 
    filter: '! (net 10.0.2.0/24 and port ssh)'
...
``` 
5. Create a configuration file for the application using `sandbox-ui config` command. The following settings need to be adjusted.
     * configure `kunai_sandbox_exe` setting that must point to a valid `kunai-sandbox` path
     * configure `sandboxes_config` with all sandboxes you want to use (each key is the sandbox name you will see in the UI)
     * configure `default_sandbox_name` with the name of the default sandbox you want to application to use
       
6.You should be able to run the application with `sandbox-ui run -c /path/to/app-config.yaml`
    
## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more information.
