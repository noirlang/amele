<div align="center">

<img src="../ui/assets/logo/logo.png" alt="Amele Logo" width="120" />

# Amele Forensic Agents

*Remote forensic acquisition agents for Linux and Windows targets.*

[Back to Main Repo](../README.md) | [Releases](https://github.com/noirlang/amele/releases)

<table width="100%">
  <tr>
    <td width="50%" align="center">
      <b>Linux Agent</b><br/><br/>
      <img src="assets/linux.gif" alt="Linux Agent Demo" width="100%" />
    </td>
    <td width="50%" align="center">
      <b>Windows Agent</b><br/><br/>
      <img src="assets/windows.gif" alt="Windows Agent Demo" width="100%" />
    </td>
  </tr>
</table>

</div>

## Overview

Amele Forensic Agents are lightweight Python-based services that enable remote disk imaging, memory acquisition, and container forensics on Linux and Windows targets. They communicate with the main Amele application over TCP with token-based authentication.

## Features

- **Remote disk acquisition:** collect disk images from remote targets with live throughput and hashing
- **Remote memory acquisition:** capture RAM dumps via AVML (Linux) or WinPMEM (Windows)
- **Container forensics:** audit Docker containers, extract logs, configs, and drift layers
- **Token-based authentication:** secure communication with the main Amele application
- **Cross-platform:** Linux and Windows support

## Agents

- `linux.py` - Linux remote forensic agent (AVML, disk imaging, container forensics)
- `windows.py` - Windows remote forensic agent (WinPMEM, disk imaging)

## Dependencies

- `requirements-linux.txt` - Linux agent dependencies
- `requirements-windows.txt` - Windows agent dependencies

## Usage

### Linux Agent

```bash
pip install -r requirements-linux.txt
python linux.py --port 9000 --token <your-token>
```

### Windows Agent

```bash
pip install -r requirements-windows.txt
python windows.py --port 9000 --token <your-token>
```

## Build Standalone Binary

Debug build:

```bash
# Linux
pip install pyinstaller
pyinstaller --onefile --name amele-linux linux.py

# Windows
pip install pyinstaller
pyinstaller --onefile --name amele-win windows.py
```

## CI/CD

Agent builds are handled by `.github/workflows/agent-release.yml`. This workflow triggers only on changes to `agent/**` and publishes binaries to GitHub Releases independently from the main application release cycle.

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for details.

## Security

See [SECURITY.md](../SECURITY.md) for security policies.

## License

See [LICENSE](../LICENSE) for details.
