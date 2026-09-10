# Windows Forensic Module

### Overview

The Windows module supports two operating modes:

1. **Local Imaging** — Direct disk imaging from the machine running Amele
2. **Remote Imaging** — Network-based disk and RAM acquisition via `amele-win` agent deployed on the target

### Local Imaging

Requires Administrator privileges.

#### Disk Image

| Feature | Description |
|---------|-------------|
| **Source** | `\\.\PhysicalDrive0`, `\\.\PhysicalDrive1`, ... (0–31) |
| **Output** | RAW (`.img`) bit-by-bit full copy |
| **Hash** | SHA-256 (automatic, `.sha256` sidecar file) |
| **Size Detection** | `DeviceIoControl(IOCTL_DISK_GET_LENGTH_INFO)` for disk size |
| **File Access** | `CreateFileW` + `GENERIC_READ` + `FILE_SHARE_READ/WRITE` |
| **Chunk Size** | 4 MB (default) |
| **Control** | Pause / Resume / Cancel support |
| **Partial Recovery** | Saved as `.partial` file on error |

**Workflow:**
1. `\\.\PhysicalDrive[0-31]` scanned to build disk list
2. User selects disk and case
3. Bit-by-bit copy begins
4. SHA-256 hash calculated simultaneously
5. `.img` + `.sha256` files written to case folder

#### RAM Image (WinPMEM)

| Feature | Description |
|---------|-------------|
| **Tool** | [go-winpmem](https://github.com/Velocidex/go-winpmem) (Velocidex) |
| **File** | `go-winpmem_amd64_1.0-rc2_signed.exe` |
| **Privilege** | Administrator required |
| **Output** | AFF4 or RAW memory dump |
| **Search Paths** | `PATH`, working directory, `C:\Forensics\`, `C:\Tools\` |
| **Command Variants** | 3 different CLI syntaxes tried automatically |
| **Progress** | Tracked by monitoring output file size |
| **Timeout** | 1 hour |
| **Control** | Pause/resume via `Suspend-Process`/`Resume-Process` |

**WinPMEM command variants (tried in order):**
```
1. winpmem.exe acquire <output_file>
2. winpmem.exe acquire --format raw <output_file>
```

### Remote Imaging (amele-win Agent)

A `amele-win` agent is deployed on the target Windows machine, communicating over TCP with a JSON protocol.

#### Protocol

| Field | Value | Description |
|-------|-------|-------------|
| Transport | TCP | JSON-over-TCP |
| Handshake | `{"komut":"merhaba"}` | Initial hello |
| Authentication | `guvenlik_anahtar_b64` | Base64 token (optional) |
| Data Transfer | Binary stream | Raw data between JSON control messages |

> **Note:** Protocol field names are in Turkish (`komut`, `durum`, `diskler`, `boyut`, etc.). This is mandatory for backward compatibility with `amele-linux` and `amele-win` agents.

#### Remote Disk Image

| Command | Description |
|---------|-------------|
| `disk_listele` | Lists disks on the target (`id`, `ad`, `boyut`) |
| `imaj_baslat` | Starts disk image transfer (RAW binary stream) |
| `edinim_kontrol` | Continue / pause / cancel |

**Collected Data:**
- RAW disk image (bit-by-bit, `\\.\PhysicalDriveN`)
- SHA-256 + MD5 hash (computed by the agent)
- Real-time progress reporting

#### Remote RAM Image (WinPMEM)

| Command | Description |
|---------|-------------|
| `winpmem_kontrol` | Is WinPMEM present on agent? Administrator privilege? RAM size? |
| `ram_edinim_baslat` | Starts WinPMEM remotely, captures RAM dump |
| `ram_dosya_indir` | Downloads captured RAM dump over the network |

### Agentless Remote Acquisition (OpenSSH / WinRM)

Directly acquire remote disks and memory without installing any agent on the target Windows system using native OpenSSH or WinRM.

#### Supported Operations:
- **Disk Listing:** Enumerates physical disks via `Get-CimInstance Win32_DiskDrive` or `wmic` formatted as JSON.
- **Disk Acquisition:** Pipes raw binary streams from `\\.\PhysicalDrive[0-31]` directly over SSH into local working files.
- **RAM Acquisition:** Streams memory dumps remotely via `winpmem.exe` pipe.
- **Output Formats:** Bit-by-bit RAW (`.raw` / `.img`) or forensic AFF4 (`.aff4`).
- **Live Hashing:** Generates SHA-256 and MD5 hashes simultaneously on the fly.

#### CLI Commands:
```bash
amele ssh-disks <ip> <user> [port] [password] [key_path]
amele ssh-tool-check <ip> <user> [port] [password] [key_path]
amele ssh-image <ip> <user> <PhysicalDriveN> <out_dir> [case_name] [port] [password] [key_path] [raw|aff4]
amele ssh-ram <ip> <user> <out_dir> [case_name] [port] [password] [key_path] [raw|aff4]
```

### Output Folder Structure

```
~/Amele/Vakalar/{case_name}/
├── {ip}_{disk}_{date}.img          # Disk image
├── {ip}_{disk}_{date}.img.sha256   # SHA-256 hash
├── {ip}_{disk}_{date}.img.md5      # MD5 hash (remote acquisition)
├── ram_{date}.raw                   # RAM dump
└── ram_{date}.raw.sha256            # RAM hash
```

### Supported Disk Types

| Type | Path | Description |
|------|------|-------------|
| All physical drives | `\\.\PhysicalDrive[0-31]` | SATA, NVMe, USB, SAS — all physical disks visible to Windows |
