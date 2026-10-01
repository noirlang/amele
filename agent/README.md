# Amele Forensic Agents

Remote forensic acquisition agents for Linux and Windows targets.

## Agents

- `linux.py` - Linux remote forensic agent (AVML, disk imaging, container forensics)
- `windows.py` - Windows remote forensic agent (WinPMEM, disk imaging)

## Demos

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

## Dependencies

- `requirements-linux.txt` - Linux agent dependencies
- `requirements-windows.txt` - Windows agent dependencies

## CI/CD

Agent builds are handled by `.github/workflows/agent-release.yml`. This workflow triggers only on changes to `agent/**` and publishes binaries to GitHub Releases independently from the main application release cycle.

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

```bash
# Linux
pip install pyinstaller
pyinstaller --onefile --name amele-linux linux.py

# Windows
pip install pyinstaller
pyinstaller --onefile --name amele-win windows.py
```
