# Docker Forensic Module

### Overview

The Amele Docker Forensic Module is an integrated digital forensics and evidence acquisition engine designed to investigate security breaches, container drift, malware persistence, and container escape attempts in Linux and Windows container environments.

The module operates in **local live system** (Linux `/var/lib/docker`, Windows `C:\ProgramData\Docker`), **mounted disk image**, and **remote agent** modes.

---

### Core Capabilities

| Feature | Description |
|---------|-------------|
| **Overlay2 UpperDir Drift Forensics** | Packages the container's runtime modification layer (dropped webshells, attacker binaries, altered configs) into a minimal `.tar.gz` archive. |
| **Container Escape Risk Engine** | Audits `--privileged` mode, `/var/run/docker.sock` mounts, `hostPID`/`hostNetwork`/`hostIPC` namespaces, dangerous Linux capabilities (`SYS_ADMIN`, `SYS_PTRACE`, `NET_ADMIN`), and `AppArmor/Seccomp: unconfined` profiles to assign `CRITICAL`, `HIGH`, `MEDIUM`, or `LOW` breakout risk ratings. |
| **Secret & Token Scanner** | Automatically inspects container environment variables (ENV) using regex patterns to uncover exposed database credentials, API keys, AWS secrets, and JWT tokens. |
| **Raw Configs & Logs Extraction** | Extracts `config.v2.json`, `hostconfig.json`, and `<id>-json.log` into the case repository. Supports 12-char short ID or full ID resolution. |
| **Cross-Platform & Remote Acquisition** | Supports Linux domain sockets (`/var/run/docker.sock`), Windows named pipes (`\\.\pipe\docker_engine`), and TCP agent connections with live SHA-256 verification. |
| **Profile & Case Vault Integration** | Acquired evidence is archived in the active case vault and automatically indexed in Profile > Acquisition History. |

---

### Case Output Layout

```text
~/Amele/Vakalar/{case}/docker/{container_name}_{short_id}/
├── docker_metadata.json          # Container metadata, risk assessment, and secret inventory
├── manifest.csv                  # SHA-256 integrity manifest
├── config.v2.json                # Raw Docker runtime configuration
├── hostconfig.json               # Host isolation and mount parameters
├── container.log                 # Console output and execution logs
└── overlay2_diff.tar.gz          # UpperDir drift / attacker modification archive
```

---

### Integrity Manifest (`manifest.csv`)

```text
Dosya_Adi,Boyut_Byte,SHA256
config.v2.json,12450,e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
hostconfig.json,4210,a1b2c3d4e5f6...
container.log,851200,9f8e7d6c5b4a...
overlay2_diff.tar.gz,14520300,7c6b5a4...
```

---

### CLI Reference

```bash
# Check Docker daemon / root path status
amele docker-status [/var/lib/docker]

# List containers with escape risk assessment and exposed secrets
amele docker-list [/var/lib/docker]

# View container console logs
amele docker-logs <container_id> [tail_lines] [/var/lib/docker]

# Acquire local container evidence into case vault
amele docker-acquire <container_id> [case_name] [/var/lib/docker]

# Query remote Docker status via Linux Agent
amele docker-remote-status <ip> <port> [token]

# List containers on remote server via Linux Agent
amele docker-remote-list <ip> <port> [token]

# Fetch remote container logs via Linux Agent
amele docker-remote-logs <ip> <port> <container_id> [tail] [token]

# Acquire remote container evidence into case vault via Linux Agent
amele docker-remote-acquire <ip> <port> <container_id> [case_name] [token]
```
