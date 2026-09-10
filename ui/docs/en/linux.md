# Linux Forensic Module

### Overview

The Linux module supports two operating modes:

1. **Local Imaging** — Direct disk imaging from the machine running Amele
2. **Remote Imaging** — Network-based disk and RAM acquisition via `amele-linux` agent deployed on the target

### Local Imaging

Requires root/sudo. Amele re-invokes itself via `pkexec` for privilege escalation.

#### Disk Image

| Feature | Description |
|---------|-------------|
| **Source** | `/dev/sdX`, `/dev/nvmeXn1`, `/dev/vdX` block devices |
| **Output** | RAW (`.img`) bit-by-bit full copy |
| **Hash** | SHA-256 (automatic, `.sha256` sidecar file) |
| **Size Detection** | `ioctl(BLKGETSIZE64)` for block device size |
| **Chunk Size** | 4 MB (default) |
| **Control** | Pause / Resume / Cancel support |
| **Partial Recovery** | Saved as `.partial` file on error |

**Workflow:**
1. Disk list obtained (`/dev/sd[a-p]`, `/dev/nvme[0-7]n1`, `/dev/vd[a-h]`)
2. User selects disk and case
3. Bit-by-bit copy begins (`read()` → `write()`)
4. SHA-256 hash calculated simultaneously
5. `.img` + `.sha256` files written to case folder

#### RAM Image (AVML)

| Feature | Description |
|---------|-------------|
| **Tool** | [AVML](https://github.com/microsoft/avml) (Microsoft) |
| **Privilege** | Root required |
| **Output** | Memory dump in Lime format |
| **Search Paths** | `PATH`, `/usr/bin/avml`, `/usr/local/bin/avml` |
| **Progress** | Tracked by monitoring output file size |
| **Timeout** | 2 hours |
| **Control** | Pause/resume via `SIGSTOP`/`SIGCONT`, cancel via `kill` |

### Remote Imaging (amele-linux Agent)

A `amele-linux` agent is deployed on the target Linux machine, communicating over TCP with a JSON protocol.

#### Protocol

| Field | Value | Description |
|-------|-------|-------------|
| Transport | TCP | JSON-over-TCP |
| Handshake | `{"komut":"merhaba"}` | Initial hello |
| Authentication | `guvenlik_anahtar_b64` | Base64 token (optional) |
| Data Transfer | Binary stream | Raw data between JSON control messages |

#### Remote Disk Image

| Command | Description |
|---------|-------------|
| `disk_listele` | Lists disks on the target (`id`, `ad`, `boyut`) |
| `imaj_baslat` | Starts disk image transfer (RAW binary stream) |
| `edinim_kontrol` | Continue / pause / cancel |

**Collected Data:**
- RAW disk image (bit-by-bit)
- SHA-256 + MD5 hash (computed by the agent)
- Real-time progress reporting

#### Remote RAM Image (AVML)

| Command | Description |
|---------|-------------|
| `avml_kontrol` | Is AVML present on agent? Root privilege? RAM size? |
| `ram_edinim_baslat` | Starts AVML remotely, captures RAM dump |
| `ram_dosya_indir` | Downloads captured RAM dump over the network |

### Agentless Remote Acquisition (SSH)

Directly acquire remote disks and memory without installing any agent on the target Linux system using native SSH authentication (password, private key, or ssh-agent).

#### Supported Operations:
- **Disk Listing:** Enumerates block devices via `lsblk -J` or `/proc/partitions` formatted as JSON.
- **Disk Acquisition:** Pipes remote `dd` binary stream over SSH directly into local working files.
- **RAM Acquisition:** Streams memory dumps via AVML or `/proc/kcore`.
- **Output Formats:** Bit-by-bit RAW (`.raw`) or forensic AFF4 (`.aff4`).
- **Live Hashing:** Generates SHA-256 and MD5 hashes simultaneously on the fly.

#### CLI Commands:
```bash
amele ssh-disks <ip> <user> [port] [password] [key_path]
amele ssh-tool-check <ip> <user> [port] [password] [key_path]
amele ssh-image <ip> <user> <disk_path> <out_dir> [case_name] [port] [password] [key_path] [raw|aff4]
amele ssh-ram <ip> <user> <out_dir> [case_name] [port] [password] [key_path] [raw|aff4]
```

### Output Folder Structure

```
~/Amele/Vakalar/{case_name}/
├── {disk_name}_{date}.img          # Disk image
├── {disk_name}_{date}.img.sha256   # SHA-256 hash
├── ram_{date}.lime                  # RAM dump
└── ram_{date}.lime.sha256           # RAM hash
```

### Supported Disk Types

| Type | Path | Description |
|------|------|-------------|
| SATA/SAS/USB | `/dev/sd[a-p]` | Traditional disks + USB |
| NVMe | `/dev/nvme[0-7]n1` | M.2 SSDs |
| VirtIO | `/dev/vd[a-h]` | Virtual machine disks |
