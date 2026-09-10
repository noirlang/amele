# Android Forensic Module

### Overview

Amele's Android module is built on a modular architecture. Each collection job runs as an independent module, sharing the common `AndroidSession` / `AndroidCapabilityReport` infrastructure.

### Architecture

```
src/android/
├── adb.rs          — ADB wrapper, device listing, command execution
├── session.rs      — AndroidSession: serial, transport kind, API level, ADB path
├── capability.rs   — AndroidCapabilityReport: supported/partial/unsupported per feature
├── profile.rs      — AndroidDeviceProfile: model, build, root, encryption, SoC
├── manifest.rs     — AndroidAcquisitionManifest: output items, hash, size
├── errors.rs       — Standardized error format (code + reason + fix + technical detail)
├── extractors.rs   — Profile definitions (QuickLogical, FullLogical, RootLogical, Volatile)
├── logical.rs      — 50+ step logical acquisition engine
├── filesystem.rs   — Non-root / Root filesystem acquisition
├── ram.rs          — Volatile data, Root process memory, Lemon physical RAM
├── remote.rs       — TCP/IP ADB + MESH relay connection management, Lemon preflight
└── orchestrator.rs — Coordinates all flows, merges profile + device + manifest
```

### Output Folder Structure

Each acquisition is organized as follows:

```
~/Amele/Cases/{case}/android/{device}_{date}/
├── manifest.json          ← Summary of all artifacts (hash, size, module)
├── manifest.json.sha256
├── device_profile.json    ← Model, build, root, API, SoC, kernel
├── capabilities.json      ← Supported/partial/unsupported report per feature
├── logical/               ← Logical acquisition outputs
│   ├── device_info.txt
│   ├── packages.txt
│   ├── logcat.txt
│   ├── dumpsys_*.txt
│   ├── content_*.txt
│   └── ...
├── filesystem/            ← Filesystem acquisition
│   ├── shared_storage/    (non-root: /sdcard, media)
│   ├── userdata.img       (root: /data block image)
│   └── filesystem.tar     (root fallback archive)
├── memory/                ← Memory acquisition
│   ├── volatile_data.txt  (non-root: ps, meminfo, procstats, logcat)
│   ├── process_dumps/     (root: process memory)
│   └── physical.lime      (Lemon: eBPF physical RAM, LiME format)
└── hashes.json            ← SHA-256 list of all files
```

### Phases

| Phase | Privilege | Status | Description |
|-------|-----------|--------|-------------|
| **Logical Image** | No root required | ✅ Active | 50+ modules via ADB |
| **Non-Root Filesystem** | No root required | ✅ Active | `/sdcard`, media, file index |
| **Root Filesystem** | Root / su | ✅ Active | `/data`, `/system`, block image or tar |
| **Volatile Data** | ADB (optional root) | ✅ Active | ps, meminfo, procstats, logcat |
| **Root Process Memory** | Root / su | ✅ Active | Selected process memory dumps |
| **Lemon Physical RAM** | Root + eBPF | ✅ Experimental | eBPF-based physical RAM (LiME format) |
| **Remote / TCP ADB** | ADB (network) | ✅ Active | `adb connect ip:port` or MESH relay |
| **Physical Image (EDL)** | Bootloader / EDL | 🔜 Soon | Hardware-level bit-by-bit image |

---

### 1. Logical Image — 50+ Steps

Data collected via ADB with USB debugging enabled, **without root**.

#### Profiles

| Profile | Steps | Content |
|---------|-------|---------|
| `quick_logical` | ~16 steps | Device info, packages, dumpsys summary, content queries, bugreport |
| `full_logical` | 50+ steps | All modules |
| `root_logical` | 50+ steps + root extras | Root binaries, keystore, heap dump candidates, procfs summary |
| `volatile` | 8 steps | Memory-focused: procfs, meminfo, heap candidates, logcat |

#### System Information

| Step | File | ADB Command | Description |
|------|------|-------------|-------------|
| `device_info` | `device_info.txt` | `adb shell getprop` | Device model, manufacturer, Android version, serial, IMEI, build number |
| `packages` | `packages.txt` | `adb shell pm list packages -f` | All installed apps and APK paths |
| `packages_json` | `packages.json` | `adb shell pm list packages` | Structured package data (JSON) |
| `processes` | `processes.txt` | `adb shell ps -A` | Running processes list |
| `disk_usage` | `disk_usage.txt` | `adb shell df -h` | Disk partitions and usage |
| `logcat` | `logcat.txt` | `adb logcat -d` | System logs |
| `system_logs` | `system_logs.txt` | `adb shell logcat -b system -d` | System buffer logs |

#### Dumpsys Services

| Step | File | ADB Command | Description |
|------|------|-------------|-------------|
| `dumpsys_battery` | `dumpsys_battery.txt` | `dumpsys battery` | Battery status, charge level, temperature |
| `dumpsys_wifi` | `dumpsys_wifi.txt` | `dumpsys wifi` | Connected/saved Wi-Fi networks, SSIDs, MACs |
| `dumpsys_bluetooth` | `dumpsys_bluetooth.txt` | `dumpsys bluetooth_manager` | Paired Bluetooth devices |
| `dumpsys_usagestats` | `dumpsys_usagestats.txt` | `dumpsys usagestats` | App usage stats (which app, duration) |
| `dumpsys_account` | `dumpsys_account.txt` | `dumpsys account` | Signed-in accounts (Google, Samsung, etc.) |
| `dumpsys_connectivity` | `dumpsys_connectivity.txt` | `dumpsys connectivity` | Network status, VPN, mobile data |
| `dumpsys_notification` | `dumpsys_notification.txt` | `dumpsys notification --noredact` | Notification history — full SMS and message content |
| `dumpsys_telephony` | `dumpsys_telephony.txt` | `dumpsys telephony.registry` | SIM info, signal, carrier, phone number, IMSI |
| `dumpsys_location` | `dumpsys_location.txt` | `dumpsys location` | Location providers, last known locations |
| `dumpsys_netstats` | `dumpsys_netstats.txt` | `dumpsys netstats` | Per-app network traffic stats |
| `dumpsys_activity` | `dumpsys_activity.txt` | `dumpsys activity` | Running activities, recent tasks, intent history |
| `dumpsys_meminfo` | `dumpsys_meminfo.txt` | `dumpsys meminfo` | Per-app memory usage (PSS, heap, dalvik) |
| `dumpsys_appops` | `dumpsys_appops.txt` | `dumpsys appops` | Permission usage history (camera, mic, location) |
| `dumpsys_package` | `dumpsys_package.txt` | `dumpsys package` | Package details, permissions, signatures (critical for malware analysis) |
| `dumpsys_diskstats` | `dumpsys_diskstats.txt` | `dumpsys diskstats` | Per-package disk usage, cache sizes |
| `dumpsys_deviceidle` | `dumpsys_deviceidle.txt` | `dumpsys deviceidle` | Doze mode, whitelisted apps, sleep stats |
| `dumpsys_alarm` | `dumpsys_alarm.txt` | `dumpsys alarm` | Scheduled alarms, app wakeup frequency |
| `dumpsys_jobscheduler` | `dumpsys_jobscheduler.txt` | `dumpsys jobscheduler` | Background jobs (crucial for persistence detection) |
| `dumpsys_procstats` | `dumpsys_procstats.txt` | `dumpsys procstats` | Process run times, historical memory consumption |
| `dumpsys_sensorservice` | `dumpsys_sensorservice.txt` | `dumpsys sensorservice` | Active sensors and listening apps |
| `dumpsys_power` | `dumpsys_power.txt` | `dumpsys power` | Wake locks, battery saving config |
| `dumpsys_window` | `dumpsys_window.txt` | `dumpsys window` | Active windows, focus, screen layout |
| `dumpsys_clipboard` | `dumpsys_clipboard.txt` | `dumpsys clipboard` | Clipboard dump: texts, passwords, OTPs, links |
| `dumpsys_batterystats` | `dumpsys_batterystats.txt` | `dumpsys batterystats` | Detailed battery log, usage timeline |
| `dumpsys_keystore` | `dumpsys_keystore.txt` | `dumpsys keystore` | Keystore metadata, key aliases |

#### Device Settings

| Step | File | ADB Commands | Description |
|------|------|--------------|-------------|
| `device_settings` | `device_settings.txt` | `settings list system/secure/global` | All device settings, developer options, accessibility services |

#### Content Provider Queries

| Step | File | Description |
|------|------|-------------|
| `content_sms` | `content_sms.txt` | SMS messages (sender, recipient, date, body) |
| `content_calls` | `content_calls.txt` | Call log (number, duration, date, type) |
| `content_contacts` | `content_contacts.txt` | Contacts (name, phone number) |
| `content_user_dictionary` | `content_user_dictionary.txt` | User's custom dictionary |
| `content_calendar` | `content_calendar.txt` | Calendar events |
| `content_media_images` | `content_media_images.txt` | Image metadata index (EXIF, location) |
| `content_media_videos` | `content_media_videos.txt` | Video metadata index |
| `content_media_audio` | `content_media_audio.txt` | Audio file metadata index |
| `content_media_files` | `content_media_files.txt` | Full file hierarchy index (hidden files included) |
| `content_telephony_carriers` | `content_telephony_carriers.txt` | Carrier settings, APN configurations |

#### Network, Screenshot & Media

| Step | File | Description |
|------|------|-------------|
| `network_info` | `network_info.txt` | IP, routes, open ports, ARP table |
| `screenshot` | `screenshot.png` | Screenshot at the moment of acquisition |
| `whatsapp_media` | `whatsapp_media/` | WhatsApp media files |
| `telegram_media` | `telegram_media/` | Telegram media files |
| `app_media` | `app_media/` | 26 known app media directories |
| `all_app_media` | `all_app_media/` | All packages under `/sdcard/Android/media/` |

#### Security & Diagnostics

| Step | File | Description |
|------|------|-------------|
| `services` | `services.txt` | Running Android services |
| `environment` | `environment.txt` | Environment variables |
| `temp_files` | `temp_files.txt` | `/tmp`, `/data/local/tmp` contents |
| `intrusion_indicators` | `intrusion_indicators.txt` | Suspicious binaries, files, and processes |
| `file_index` | `file_index.txt` | Full index of files under `/sdcard` |
| `adb_backup` | `adb_backup.ab` | `adb backup -all -shared -nosystem` |
| `bugreport` | `bugreport.zip` | Full diagnostic (logcat + all dumpsys + kernel log) |
| `shared_storage` | `shared_storage/` | Internal storage: DCIM, Download, Documents, etc. |

#### Root-Only Extra Steps

| Step | File | Description |
|------|------|-------------|
| `root_status` | `root_status.txt` | Root type and activation status |
| `root_binaries` | `root_binaries.txt` | Detection of `su`, `magisk`, `busybox` etc. |
| `selinux_status` | `selinux_status.txt` | SELinux mode (Enforcing/Permissive) |
| `mounts` | `mounts.txt` | Mounted filesystems |
| `procfs_summary` | `procfs_summary.txt` | `/proc` summary: uptime, version, cpuinfo, meminfo |
| `proc_memory_maps` | `proc_memory_maps/` | `/proc/[pid]/maps` for running processes |
| `heapdump_candidates` | `heapdump_candidates.txt` | Processes eligible for debug heap dump |
| `debug_heap_dumps` | `debug_heap_dumps/` | Heap dumps from JDWP debug-mode apps |

---

### 2. Filesystem Acquisition

#### Non-Root Mode
Accessible without root:
- `/sdcard` — internal storage (DCIM, Download, Documents, Pictures, Music)
- `/sdcard/Android/media/` — app media files
- MediaStore index — metadata mapping of all files

#### Root Mode
With root or `adb root`:
- `/data` partition — block image (`userdata.img`) or tar archive (`filesystem.tar`)
- `/system`, `/vendor` — system partitions
- App private data (`/data/data/<package>/`)

#### Physical Image (EDL)
Requires bootloader/EDL access. Marked as "device mode required" in the UI. Coming soon.

---

### 3. Memory Acquisition

#### Volatile Data (Non-Root)
No root required, safe:
- `ps -A` — running processes
- `/proc/meminfo`, `/proc/vmstat` — memory statistics
- `dumpsys meminfo`, `dumpsys procstats`
- `logcat -d` — recent logs
- Activity process list

#### Root Process Memory
Requires root/su:
- Read `/proc/[pid]/mem` for selected processes
- `/proc/[pid]/maps` — memory maps
- JDWP heap dump (debug-mode apps)

#### Lemon Physical RAM (Experimental)
eBPF-based physical RAM dump tool. A **preflight check** is required first:

| Check | Description |
|-------|-------------|
| **Architecture (ABI)** | `arm64-v8a` or `x86_64` required |
| **Root** | `su` or `adb root` mandatory |
| **eBPF/BTF** | `/sys/kernel/btf/vmlinux` must be present |
| **/proc/kcore** | Read access (if available) |
| **Storage** | RAM size + 512 MB free space required |
| **SoC Warning** | Exynos/MediaTek → EL2 reboot risk |

Output: `.lime` file in LiME format — directly analyzable with Volatility 3.

---

### 4. Remote Android / MESH

Connect without a USB cable, over the network:

| Type | Description |
|------|-------------|
| **TCP/IP ADB** | `adb connect <ip>:<port>` — over Wi-Fi ADB or hotspot |
| **MESH Relay** | ADB endpoint provided over MESH infrastructure |

After connection, the device is automatically added to the device list and all acquisition modules can target it.

---

### 5. Preflight Check

Shown automatically on every acquisition page:

```
ADB:          ✅ Ready
Device:       ✅ Authorized
Root:         ❌ None
API:          34
Architecture: arm64-v8a
Encryption:   file-based

Supported:    Logical, Bugreport, Non-Root Filesystem, Volatile Data
Unsupported:  Root Filesystem, Root Process Memory, Lemon RAM
```

---

### 6. Error Format

All Android errors are reported in a standardized format:

```
Operation failed:  Bugreport could not be acquired
Code:              ANDROID_BUGREPORT_FAILED
Reason:            The device did not complete the bugreport command or the connection was lost.
Fix:               Unlock the device, confirm USB debugging authorization, and retry.
Technical detail:  adb bugreport exited with code 1
```
