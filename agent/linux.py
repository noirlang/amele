#!/usr/bin/env python3
"""
Amele Linux Agent v0.0.7
- TUI startup wizard
- Remote disk imaging protocol compatible with controller
- AVML check/install guidance + RAM acquisition over protocol
"""

import base64
import binascii
import hashlib
import json
import os
import shutil
import socket
import stat
import subprocess
import sys
import tarfile
import tempfile
import threading
import time
from datetime import datetime

VERSION = "0.0.7"
HOST = "0.0.0.0"
DEFAULT_PORT = 4444
BUFFER_SIZE = 1024 * 1024
AVML_BIN_NAME = "avml"
AVML_RELEASE_URL = "https://github.com/microsoft/avml/releases"
AVML_DIRECT_URL = "https://download.amele.noirlang.tr/avml"
SUPPORTED_OUTPUT_FORMATS = {"raw", "aff4"}

ASCII_LOGO = r"""          ⣠⣧⡀
         ⣰⠃⡏⢳⡀
        ⡴⠁ ⡇ ⠳⡀
       ⡼⠁ ⡼⠹⡄ ⠹⡄
     ⢀⡜⠁⢀⡜⠁ ⠘⣆ ⠙⣆
    ⢀⡞ ⢀⣀⣙⡦⠦⣞⣁⣀ ⠘⣆
   ⢠⠎⠑⣤⣏⣉⣉⠑⡖⢉⣉⣉⣳⡔⠉⢆
  ⢠⠿⣄⡰⠋⠳⣤⣤⣤⢧⣤⣤⡴⠋⠳⣀⡼⢧
 ⣰⠋ ⡼⠛⢦⡞      ⠘⣦⠞⠻⡄⠈⢳⡀
⣰⣇⣀⣼⣁⣠⠞        ⠘⢦⣀⣹⣄⣀⣷⡀"""


TR = {
    "banner": f"Amele Linux Agent v{VERSION} (TUI)",
    "ask_lang": "Dil secin [tr/en] (varsayilan: tr): ",
    "ask_sec": "Guvenlik parolasi kullanilsin mi? [E/h]: ",
    "ask_pw": "Guvenlik parolasi: ",
    "ask_port": f"Port [{DEFAULT_PORT}]: ",
    "invalid_port": "Gecersiz port. Varsayilan port kullaniliyor.",
    "avml_found": "AVML bulundu:",
    "avml_missing": "AVML bulunamadi.",
    "ask_repo_install": "Distro deposundan AVML kurulumu denensin mi? [E/h]: ",
    "repo_try": "Repo kurulumu deneniyor:",
    "repo_ok": "AVML repo uzerinden kuruldu.",
    "repo_fail": "Repo kurulum basarisiz.",
    "wget_try": "wget ile AVML indiriliyor:",
    "wget_ok": "AVML wget ile indirildi.",
    "wget_fail": "wget ile AVML indirilemedi.",
    "manual_needed": "AVML otomatik kurulamadı. Lutfen AVML'i indirip bu dosya ile ayni klasore koyun:",
    "distro": "Distro:",
    "server_start": "Sunucu basladi:",
    "server_stop": "Sunucu durduruldu.",
    "ctrlc": "Durdurmak icin Ctrl+C",
    "conn": "Baglanti:",
    "auth_reject": "Yetkisiz baglanti reddedildi:",
    "auth_accept": "Yetkili baglanti kabul edildi:",
    "unknown_cmd": "Bilinmeyen komut",
    "ram_need_root": "RAM edinimi icin root yetkisi gerekli.",
    "linux_only": "Bu ajan Linux uzerinde calisir.",
    "progress_disk": "Disk aktarim",
    "progress_ram": "RAM edinim",
    "progress_file": "Dosya aktarim",
}

EN = {
    "banner": f"Amele Linux Agent v{VERSION} (TUI)",
    "ask_lang": "Select language [tr/en] (default: tr): ",
    "ask_sec": "Enable security password? [Y/n]: ",
    "ask_pw": "Security password: ",
    "ask_port": f"Port [{DEFAULT_PORT}]: ",
    "invalid_port": "Invalid port. Using default.",
    "avml_found": "AVML found:",
    "avml_missing": "AVML not found.",
    "ask_repo_install": "Try installing AVML from distro repositories? [Y/n]: ",
    "repo_try": "Trying repository install:",
    "repo_ok": "AVML installed from repository.",
    "repo_fail": "Repository install failed.",
    "wget_try": "Downloading AVML with wget:",
    "wget_ok": "AVML downloaded with wget.",
    "wget_fail": "Failed to download AVML with wget.",
    "manual_needed": "Could not install AVML automatically. Please download AVML and place it in the same directory as this file:",
    "distro": "Distro:",
    "server_start": "Server started:",
    "server_stop": "Server stopped.",
    "ctrlc": "Press Ctrl+C to stop",
    "conn": "Connection:",
    "auth_reject": "Unauthorized connection rejected:",
    "auth_accept": "Authorized connection accepted:",
    "unknown_cmd": "Unknown command",
    "ram_need_root": "Root privileges required for RAM acquisition.",
    "linux_only": "This agent runs on Linux.",
    "progress_disk": "Disk transfer",
    "progress_ram": "RAM acquisition",
    "progress_file": "File transfer",
}


def now_str():
    return datetime.now().strftime("%Y-%m-%d %H:%M:%S")


def app_base_dir():
    if getattr(sys, "frozen", False):
        return os.path.dirname(os.path.abspath(sys.executable))
    return os.path.dirname(os.path.abspath(__file__))


def load_os_release():
    data = {}
    try:
        with open("/etc/os-release", "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line or "=" not in line:
                    continue
                k, v = line.split("=", 1)
                data[k] = v.strip().strip('"')
    except Exception:
        pass
    return data


def is_yes(answer, lang):
    a = (answer or "").strip().lower()
    if not a:
        return True
    if lang == "en":
        return a in {"y", "yes"}
    return a in {"e", "evet", "y", "yes"}


def json_send(conn, payload):
    conn.sendall(json.dumps(payload, ensure_ascii=False).encode("utf-8") + b"\n")


def normalize_output_format(value):
    fmt = str(value or "raw").strip().lower()
    if fmt in {"dd", "img", "raw_sparse", "raw-sparse", "raw_full", "raw-full"}:
        fmt = "raw"
    elif fmt in {"aff4_sparse", "aff4-sparse", "aff4_full", "aff4-full"}:
        fmt = "aff4"
    if fmt not in SUPPORTED_OUTPUT_FORMATS:
        return "", f"Unsupported output format: {fmt}. Supported formats: raw, aff4"
    return fmt, ""


def parse_sparse_preference(value, default=False):
    val = str(value or "").strip().lower()
    if any(x in val for x in ["sparse", "smart"]):
        return True
    if any(x in val for x in ["full"]):
        return False
    return default


def calc_mem_total_bytes():
    try:
        with open("/proc/meminfo", "r", encoding="utf-8") as f:
            for line in f:
                if line.startswith("MemTotal:"):
                    parts = line.split()
                    return int(parts[1]) * 1024
    except Exception:
        pass
    return 0


def list_disks_linux():
    try:
        cmd = ["lsblk", "-J", "-b", "-n", "-o", "NAME,SIZE,TYPE"]
        out = subprocess.check_output(cmd, stderr=subprocess.STDOUT)
        obj = json.loads(out.decode("utf-8", errors="ignore"))
    except Exception:
        cmd = ["lsblk", "-J", "-b", "-dn", "-o", "NAME,SIZE,TYPE"]
        out = subprocess.check_output(cmd, stderr=subprocess.STDOUT)
        obj = json.loads(out.decode("utf-8", errors="ignore"))

    result = []

    def add_device(entry):
        dev_type = entry.get("type", "")
        if dev_type in {"disk", "part", "lvm", "crypt"}:
            name = entry.get("name", "")
            size = int(entry.get("size", 0) or 0)
            if name and size > 0:
                result.append({
                    # Keep id slash-free because controller uses disk_id in output filename.
                    "id": f"{name}",
                    "ad": f"{name}",
                    "boyut": size,
                })
        for child in entry.get("children", []):
            add_device(child)

    for entry in obj.get("blockdevices", []):
        add_device(entry)
    return result


def resolve_disk_path(disk_id):
    if not disk_id:
        return ""
    if disk_id.startswith("/dev/"):
        return disk_id
    return f"/dev/{disk_id}"


def disk_size_bytes_linux(path):
    try:
        st = os.stat(path)
    except Exception:
        return 0

    # Regular files can use getsize directly.
    if stat.S_ISREG(st.st_mode):
        try:
            return os.path.getsize(path)
        except Exception:
            return 0

    # Block devices often report 0 via getsize; use blockdev/lsblk.
    if stat.S_ISBLK(st.st_mode):
        try:
            out = subprocess.check_output(["blockdev", "--getsize64", path], stderr=subprocess.STDOUT)
            size = int(out.decode("utf-8", errors="ignore").strip() or "0")
            if size > 0:
                return size
        except Exception:
            pass

        try:
            out = subprocess.check_output(["lsblk", "-b", "-dn", "-o", "SIZE", path], stderr=subprocess.STDOUT)
            size = int(out.decode("utf-8", errors="ignore").strip() or "0")
            if size > 0:
                return size
        except Exception:
            pass

    return 0


def find_avml(script_dir):
    local_path = os.path.join(script_dir, AVML_BIN_NAME)
    if os.path.isfile(local_path) and os.access(local_path, os.X_OK):
        return local_path

    found = shutil.which(AVML_BIN_NAME)
    if found:
        return found

    if os.path.isfile(local_path):
        # Allow non-executable local file but we will chmod before running.
        return local_path

    return ""


def detect_repo_install_cmd():
    if shutil.which("apt-get"):
        return ["apt-get", "install", "-y", "avml"]
    if shutil.which("dnf"):
        return ["dnf", "install", "-y", "avml"]
    if shutil.which("pacman"):
        return ["pacman", "-Sy", "--noconfirm", "avml"]
    if shutil.which("zypper"):
        return ["zypper", "--non-interactive", "install", "avml"]
    if shutil.which("apk"):
        return ["apk", "add", "avml"]
    return []


def try_install_avml(lang, t):
    cmd = detect_repo_install_cmd()
    if not cmd:
        return False

    full_cmd = cmd
    if os.geteuid() != 0 and shutil.which("sudo"):
        full_cmd = ["sudo"] + cmd

    print(f"{t['repo_try']} {' '.join(full_cmd)}")
    try:
        subprocess.run(full_cmd, check=True)
        return True
    except Exception:
        return False


def try_install_avml_via_wget(script_dir, t):
    hedef = os.path.join(script_dir, AVML_BIN_NAME)
    if shutil.which("wget"):
        print(f"{t['wget_try']} wget {AVML_DIRECT_URL}")
        try:
            subprocess.run(["wget", AVML_DIRECT_URL, "-O", hedef], check=True)
            subprocess.run(["chmod", "+x", hedef], check=True)
            return os.path.isfile(hedef) and os.access(hedef, os.X_OK)
        except Exception:
            pass
    if shutil.which("curl"):
        print(f"{t['wget_try']} curl {AVML_DIRECT_URL}")
        try:
            subprocess.run(["curl", "-sSL", AVML_DIRECT_URL, "-o", hedef], check=True)
            subprocess.run(["chmod", "+x", hedef], check=True)
            return os.path.isfile(hedef) and os.access(hedef, os.X_OK)
        except Exception:
            pass
    return False


def docker_get_status():
    root = "/var/lib/docker"
    containers_dir = os.path.join(root, "containers")
    overlay2_dir = os.path.join(root, "overlay2")
    sock = "/var/run/docker.sock"

    is_avail = os.path.exists(containers_dir) or os.path.exists(overlay2_dir)
    is_running = os.path.exists(sock)

    count = 0
    running = 0
    paused = 0
    stopped = 0

    if os.path.exists(containers_dir):
        try:
            for item in os.listdir(containers_dir):
                full = os.path.join(containers_dir, item)
                if os.path.isdir(full):
                    count += 1
                    cfg_file = os.path.join(full, "config.v2.json")
                    if os.path.exists(cfg_file):
                        try:
                            with open(cfg_file, "r", encoding="utf-8") as f:
                                val = json.load(f)
                                st = val.get("State", {})
                                if st.get("Running"):
                                    running += 1
                                elif st.get("Paused"):
                                    paused += 1
                                else:
                                    stopped += 1
                        except Exception:
                            stopped += 1
        except Exception:
            pass

    if count == 0 and is_running:
        try:
            out = subprocess.run(["docker", "ps", "-a", "--format", "{{.Status}}"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=5)
            if out.returncode == 0:
                lines = [l.strip() for l in out.stdout.strip().splitlines() if l.strip()]
                count = len(lines)
                for l in lines:
                    if l.lower().startswith("up"):
                        running += 1
                    elif "paused" in l.lower():
                        paused += 1
                    else:
                        stopped += 1
                is_avail = True
        except Exception:
            pass

    return {
        "durum": "ok",
        "docker_available": is_avail,
        "docker_running": is_running,
        "docker_mevcut": is_avail,
        "docker_calisiyor": is_running,
        "storage_driver": "overlay2" if os.path.exists(overlay2_dir) else "unknown",
        "depolama_surucusu": "overlay2" if os.path.exists(overlay2_dir) else "unknown",
        "root_dir": root,
        "kok_dizin": root,
        "containers_count": count,
        "konteyner_sayisi": count,
        "running_count": running,
        "calisan_sayisi": running,
        "paused_count": paused,
        "duraklatilan_sayisi": paused,
        "stopped_count": stopped,
        "durdurulan_sayisi": stopped,
    }


def docker_evaluate_risk(config_v2, host_config, mounts):
    reasons = []
    score = 0

    priv = bool(host_config.get("Privileged"))
    if priv:
        reasons.append("Konteyner tam yetkili (Privileged) modda çalışıyor (Kritik Host Ele Geçirme Riski).")
        score += 50

    for m in mounts:
        src = m.get("source", "")
        dst = m.get("destination", "")
        if "docker.sock" in src or "docker.sock" in dst:
            reasons.append("Host Docker soketi (/var/run/docker.sock) konteyner içine mount edilmiş (DoD Escape riski).")
            score += 40
        if src in {"/", "/etc", "/root", "/var/run", "/proc", "/sys"} and m.get("rw"):
            reasons.append(f"Kritik host dizini ({src}) yazılabilir (RW) olarak konteynere mount edilmiş.")
            score += 35

    caps = host_config.get("CapAdd") or []
    danger_caps = {"CAP_SYS_ADMIN", "CAP_SYS_PTRACE", "CAP_SYS_RAWIO", "CAP_DAC_OVERRIDE", "SYS_ADMIN", "SYS_PTRACE"}
    for c in caps:
        if c.upper() in danger_caps:
            reasons.append(f"Tehlikeli Linux Capability tespit edildi: {c}")
            score += 25

    pid_mode = str(host_config.get("PidMode", ""))
    if pid_mode.lower() == "host":
        reasons.append("Host PID alanı paylaşılıyor (Host Process Injection riski).")
        score += 30

    net_mode = str(host_config.get("NetworkMode", ""))
    if net_mode.lower() == "host":
        reasons.append("Host Network alanı paylaşılıyor (Host Port Sniffing riski).")
        score += 15

    user_val = (config_v2.get("Config") or {}).get("User") or config_v2.get("User") or ""
    if not user_val or user_val == "0" or user_val == "root":
        reasons.append("Konteyner doğrudan ROOT kullanıcısı ile çalışıyor.")
        score += 10

    if score >= 60:
        level = "CRITICAL"
    elif score >= 30:
        level = "HIGH"
    elif score >= 15:
        level = "MEDIUM"
    else:
        level = "LOW"

    return level, reasons


def docker_scan_secrets(env_list):
    secrets = []
    if not env_list:
        return secrets

    patterns = [
        ("API_KEY", "API / Token Anahtari"),
        ("SECRET", "Secret / Gizli Anahtar"),
        ("PASSWORD", "Parola / Sifre"),
        ("PASS", "Parola / Sifre"),
        ("TOKEN", "Guvenlik Belirteci (Token)"),
        ("PRIVATE_KEY", "Ozel Anahtar (Private Key)"),
        ("AWS_ACCESS_KEY", "Bulut Erisim Anahtari (AWS)"),
        ("DB_PASS", "Veritabani Parolasi"),
    ]

    for env in env_list:
        if "=" not in env:
            continue
        k, v = env.split("=", 1)
        k_upper = k.upper()
        if len(v.strip()) < 3:
            continue
        for p, kind in patterns:
            if p in k_upper:
                masked = v[:2] + "..." + v[-2:] if len(v) > 4 else "***"
                secrets.append({
                    "key": k,
                    "value_preview": masked,
                    "secret_type": kind
                })
                break
    return secrets


def docker_list_containers():
    root = "/var/lib/docker/containers"
    res = []

    if os.path.exists(root):
        try:
            for item in os.listdir(root):
                c_dir = os.path.join(root, item)
                if not os.path.isdir(c_dir) or len(item) < 12:
                    continue

                cfg_file = os.path.join(c_dir, "config.v2.json")
                host_file = os.path.join(c_dir, "hostconfig.json")

                config_v2 = {}
                host_config = {}

                if os.path.exists(cfg_file):
                    try:
                        with open(cfg_file, "r", encoding="utf-8") as f:
                            config_v2 = json.load(f)
                    except Exception:
                        pass

                if os.path.exists(host_file):
                    try:
                        with open(host_file, "r", encoding="utf-8") as f:
                            host_config = json.load(f)
                    except Exception:
                        pass

                c_name = config_v2.get("Name", "").lstrip("/") or "unnamed"
                img = (config_v2.get("Config") or {}).get("Image") or config_v2.get("Image") or "unknown"
                created = config_v2.get("Created", "")
                st = config_v2.get("State", {})
                running = bool(st.get("Running"))
                pid = int(st.get("Pid") or 0)
                exit_code = int(st.get("ExitCode") or 0)

                if running:
                    state_str = "running"
                elif st.get("Paused"):
                    state_str = "paused"
                elif st.get("Restarting"):
                    state_str = "restarting"
                else:
                    state_str = "exited"

                gdata = (config_v2.get("GraphDriver") or {}).get("Data") or {}
                upper_dir = gdata.get("UpperDir")
                merged_dir = gdata.get("MergedDir")
                work_dir = gdata.get("WorkDir")

                log_path = config_v2.get("LogPath")
                if not log_path and os.path.exists(os.path.join(c_dir, f"{item}-json.log")):
                    log_path = os.path.join(c_dir, f"{item}-json.log")

                net = config_v2.get("NetworkSettings") or {}
                ip_addr = net.get("IPAddress") or None

                port_bindings = (host_config.get("PortBindings") or {})
                ports = []
                for c_p, b_list in port_bindings.items():
                    if b_list:
                        for b in b_list:
                            h_p = b.get("HostPort", "")
                            h_ip = b.get("HostIp", "0.0.0.0") or "0.0.0.0"
                            ports.append(f"{h_ip}:{h_p} -> {c_p}")

                mounts_raw = config_v2.get("MountPoints") or {}
                mounts = []
                if isinstance(mounts_raw, dict):
                    for _, m in mounts_raw.items():
                        mounts.append({
                            "source": m.get("Source", ""),
                            "destination": m.get("Destination", ""),
                            "mode": m.get("Mode", ""),
                            "rw": bool(m.get("RW", True)),
                            "propagation": m.get("Propagation", ""),
                        })
                elif isinstance(mounts_raw, list):
                    for m in mounts_raw:
                        mounts.append({
                            "source": m.get("Source", ""),
                            "destination": m.get("Destination", ""),
                            "mode": m.get("Mode", ""),
                            "rw": bool(m.get("RW", True)),
                            "propagation": m.get("Propagation", ""),
                        })

                env_list = (config_v2.get("Config") or {}).get("Env") or []
                secrets = docker_scan_secrets(env_list)
                risk_level, risk_reasons = docker_evaluate_risk(config_v2, host_config, mounts)

                privileged = bool(host_config.get("Privileged"))
                driver = config_v2.get("Driver", "overlay2")

                res.append({
                    "id": item,
                    "short_id": item[:12],
                    "name": c_name,
                    "image": img,
                    "created": created,
                    "state": state_str,
                    "durum": state_str,
                    "running": running,
                    "calisiyor": running,
                    "pid": pid,
                    "exit_code": exit_code,
                    "upper_dir": upper_dir,
                    "merged_dir": merged_dir,
                    "work_dir": work_dir,
                    "log_path": log_path,
                    "ip_address": ip_addr,
                    "ip_adresi": ip_addr,
                    "ports": ports,
                    "portlar": ports,
                    "privileged": privileged,
                    "risk_level": risk_level,
                    "risk_seviyesi": risk_level,
                    "risk_reasons": risk_reasons,
                    "risk_nedenleri": risk_reasons,
                    "mounts": mounts,
                    "mountlar": mounts,
                    "secrets_found": secrets,
                    "bulunan_gizli_bilgiler": secrets,
                    "driver": driver,
                    "depolama_surucusu": driver,
                })
        except Exception:
            pass

    if not res:
        try:
            out = subprocess.run(["docker", "ps", "-aq"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=5)
            if out.returncode == 0:
                cids = out.stdout.strip().split()
                if cids:
                    insp_out = subprocess.run(["docker", "inspect"] + cids, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=10)
                    if insp_out.returncode == 0:
                        data = json.loads(insp_out.stdout)
                        for d in data:
                            c_id = d.get("Id", "")
                            c_name = d.get("Name", "").lstrip("/") or "unnamed"
                            img = (d.get("Config") or {}).get("Image") or "unknown"
                            created = d.get("Created", "")
                            st = d.get("State", {})
                            running = bool(st.get("Running"))
                            pid = int(st.get("Pid") or 0)
                            exit_code = int(st.get("ExitCode") or 0)
                            state_str = "running" if running else ("paused" if st.get("Paused") else "exited")
                            host_config = d.get("HostConfig") or {}
                            gdata = (d.get("GraphDriver") or {}).get("Data") or {}
                            upper_dir = gdata.get("UpperDir")
                            merged_dir = gdata.get("MergedDir")
                            work_dir = gdata.get("WorkDir")
                            log_path = d.get("LogPath")
                            net = d.get("NetworkSettings") or {}
                            ip_addr = net.get("IPAddress") or None
                            port_bindings = host_config.get("PortBindings") or {}
                            ports = []
                            for c_p, b_list in port_bindings.items():
                                if b_list:
                                    for b in b_list:
                                        h_p = b.get("HostPort", "")
                                        h_ip = b.get("HostIp", "0.0.0.0") or "0.0.0.0"
                                        ports.append(f"{h_ip}:{h_p} -> {c_p}")
                            mounts_raw = d.get("Mounts") or []
                            mounts = []
                            for m in mounts_raw:
                                mounts.append({
                                    "source": m.get("Source", ""),
                                    "destination": m.get("Destination", ""),
                                    "mode": m.get("Mode", ""),
                                    "rw": bool(m.get("RW", True)),
                                    "propagation": m.get("Propagation", ""),
                                })
                            env_list = (d.get("Config") or {}).get("Env") or []
                            secrets = docker_scan_secrets(env_list)
                            risk_level, risk_reasons = docker_evaluate_risk(d, host_config, mounts)
                            privileged = bool(host_config.get("Privileged"))
                            driver = d.get("Driver", "overlay2")
                            res.append({
                                "id": c_id,
                                "short_id": c_id[:12],
                                "name": c_name,
                                "image": img,
                                "created": created,
                                "state": state_str,
                                "durum": state_str,
                                "running": running,
                                "calisiyor": running,
                                "pid": pid,
                                "exit_code": exit_code,
                                "upper_dir": upper_dir,
                                "merged_dir": merged_dir,
                                "work_dir": work_dir,
                                "log_path": log_path,
                                "ip_address": ip_addr,
                                "ip_adresi": ip_addr,
                                "ports": ports,
                                "portlar": ports,
                                "privileged": privileged,
                                "risk_level": risk_level,
                                "risk_seviyesi": risk_level,
                                "risk_reasons": risk_reasons,
                                "risk_nedenleri": risk_reasons,
                                "mounts": mounts,
                                "mountlar": mounts,
                                "secrets_found": secrets,
                                "bulunan_gizli_bilgiler": secrets,
                                "driver": driver,
                                "depolama_surucusu": driver,
                            })
        except Exception:
            pass

    res.sort(key=lambda x: x["name"].lower())
    return res


def docker_get_logs(container_id, tail=200):
    log_file = os.path.join("/var/lib/docker/containers", container_id, f"{container_id}-json.log")
    logs = []
    if os.path.exists(log_file):
        try:
            with open(log_file, "r", encoding="utf-8", errors="ignore") as f:
                for line in f:
                    line = line.strip()
                    if line:
                        try:
                            logs.append(json.loads(line))
                        except Exception:
                            pass
        except Exception:
            pass

    if not logs:
        try:
            tail_arg = str(tail) if tail > 0 else "200"
            out = subprocess.run(["docker", "logs", "--tail", tail_arg, container_id], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=5)
            if out.returncode == 0:
                combined = out.stdout + out.stderr
                for l in combined.splitlines():
                    if l.strip():
                        logs.append({"log": l + "\n", "stream": "cli", "time": datetime.now().isoformat()})
        except Exception:
            pass

    if tail > 0 and len(logs) > tail:
        logs = logs[-tail:]
    return logs


class LinuxAgentController:
    def __init__(self, language="tr", security_key="", port=DEFAULT_PORT, avml_path=""):
        self.language = language if language in {"tr", "en"} else "tr"
        self.t = EN if self.language == "en" else TR
        self.security_key = security_key
        self.port = port
        self.sock = None
        self.running = False
        self.script_dir = app_base_dir()
        self.avml_path = avml_path
        self._progress_lock = threading.Lock()
        self.ram_output_index = {}
        self._job_lock = threading.Lock()
        self._job_state = {}

    def _set_job_state(self, job_id, state):
        if not job_id:
            return
        with self._job_lock:
            self._job_state[job_id] = state

    def _get_job_state(self, job_id):
        if not job_id:
            return "running"
        with self._job_lock:
            return self._job_state.get(job_id, "running")

    def _clear_job_state(self, job_id):
        if not job_id:
            return
        with self._job_lock:
            self._job_state.pop(job_id, None)

    def _control_job(self, job_id, action):
        if not job_id or not action:
            return False, "is_id ve eylem gerekli"
        action = str(action).strip().lower()
        action_aliases = {
            "duraklat": "pause",
            "pause": "pause",
            "beklet": "pause",
            "devam": "resume",
            "resume": "resume",
            "surdur": "resume",
            "sürdür": "resume",
            "durdur": "stop",
            "stop": "stop",
            "iptal": "stop",
            "cancel": "stop",
        }
        action = action_aliases.get(action, action)
        if action not in {"pause", "resume", "stop"}:
            return False, "Desteklenmeyen eylem"

        with self._job_lock:
            if job_id not in self._job_state:
                return False, "Is bulunamadi"
            if action == "pause":
                self._job_state[job_id] = "paused"
            elif action == "resume":
                self._job_state[job_id] = "running"
            else:
                self._job_state[job_id] = "stopped"

        return True, "Kontrol komutu uygulandi"

    def _fs_type(self, path):
        try:
            out = subprocess.check_output(["stat", "-f", "-c", "%T", path], stderr=subprocess.STDOUT)
            return out.decode("utf-8", errors="ignore").strip().lower()
        except Exception:
            return ""

    def _format_bytes(self, value):
        try:
            value = float(value or 0)
        except Exception:
            value = 0.0
        units = ["B", "KB", "MB", "GB", "TB"]
        idx = 0
        while value >= 1024 and idx < len(units) - 1:
            value /= 1024
            idx += 1
        if idx == 0:
            return f"{int(value)} {units[idx]}"
        return f"{value:.1f} {units[idx]}"

    def _dir_free_bytes(self, path):
        try:
            return shutil.disk_usage(path).free
        except Exception:
            return 0

    def _dir_is_writable(self, path):
        probe = os.path.join(path, ".amele-write-test")
        try:
            with open(probe, "ab"):
                pass
            os.remove(probe)
            return True
        except Exception:
            try:
                if os.path.exists(probe):
                    os.remove(probe)
            except Exception:
                pass
            return False

    def _preallocate_probe(self, directory, required_bytes):
        if not required_bytes:
            return ""
        probe = os.path.join(directory, ".amele-ram-space-test")
        try:
            with open(probe, "wb") as f:
                if hasattr(os, "posix_fallocate"):
                    os.posix_fallocate(f.fileno(), 0, required_bytes)
                else:
                    f.truncate(required_bytes)
            return ""
        except OSError as exc:
            return f"{directory}: ayrilmis alan testi basarisiz ({exc})"
        except Exception as exc:
            return f"{directory}: ayrilmis alan testi basarisiz ({exc})"
        finally:
            try:
                if os.path.exists(probe):
                    os.remove(probe)
            except Exception:
                pass

    def _select_ram_output_path(self, requested_name, required_bytes=0):
        base = os.path.basename(requested_name or "memory_dump_linux.raw")
        if not base:
            base = "memory_dump_linux.raw"

        risky_fs = {"vfat", "msdos", "fat", "fuseblk"}
        large_dump = required_bytes >= 4 * 1024 * 1024 * 1024
        free_margin = max(512 * 1024 * 1024, int(required_bytes * 0.05)) if required_bytes else 0
        needed = required_bytes + free_margin if required_bytes else 0
        candidates = [
            self.script_dir,
            "/var/tmp/Amele/ram",
            "/tmp/Amele/ram",
        ]
        skipped = []

        for directory in candidates:
            try:
                os.makedirs(directory, exist_ok=True)
            except Exception as exc:
                skipped.append(f"{directory}: olusturulamadi ({exc})")
                continue

            fs_type = self._fs_type(directory)
            free_bytes = self._dir_free_bytes(directory)
            if large_dump and fs_type in risky_fs:
                skipped.append(f"{directory}: {fs_type or 'unknown'} 4GB ustu RAM dump icin riskli")
                continue
            if needed and free_bytes and free_bytes < needed:
                skipped.append(
                    f"{directory}: bos alan yetersiz ({self._format_bytes(free_bytes)} / gerekli {self._format_bytes(needed)})"
                )
                continue
            if not self._dir_is_writable(directory):
                skipped.append(f"{directory}: yazilabilir degil")
                continue
            prealloc_error = self._preallocate_probe(directory, required_bytes)
            if prealloc_error:
                skipped.append(prealloc_error)
                continue

            return os.path.join(directory, base), ""

        details = "; ".join(skipped) if skipped else "uygun klasor bulunamadi"
        message = (
            "RAM dump icin uygun cikti klasoru bulunamadi. "
            f"Gerekli alan: {self._format_bytes(needed or required_bytes)}. "
            "Btrfs quota/subvolume limiti, yetersiz bos alan veya 4GB limitli dosya sistemi buna sebep olabilir. "
            f"Detay: {details}"
        )
        return "", message

    def _ram_output_diagnostics(self, output_file, final_size=0, total=0):
        directory = os.path.dirname(output_file) or "."
        fs_type = self._fs_type(directory) or "unknown"
        free_bytes = self._dir_free_bytes(directory)
        parts = [
            f"output_fs={fs_type}",
            f"output_dir={directory}",
            f"free={self._format_bytes(free_bytes)}",
        ]
        if final_size:
            parts.append(f"partial={self._format_bytes(final_size)}")
        if total:
            parts.append(f"ram={self._format_bytes(total)}")
        if fs_type == "btrfs":
            parts.append("btrfs_notu=quota/subvolume metadata alani df ciktisindan once dolabilir")
        return ", ".join(parts)

    def _cleanup_transferred_file(self, file_path, index_key=""):
        target = os.path.abspath(file_path)
        try:
            if os.path.exists(target):
                os.remove(target)
                self.log(f"Transferred file deleted from agent: {target}")
        except Exception as exc:
            self.log(f"Transferred file could not be deleted from agent: {target} ({exc})")
            return

        for key, value in list(self.ram_output_index.items()):
            if key == index_key or os.path.abspath(value) == target:
                self.ram_output_index.pop(key, None)

        tmp_dir = os.path.join(os.path.dirname(target), ".tmp")
        try:
            if os.path.isdir(tmp_dir) and not os.listdir(tmp_dir):
                os.rmdir(tmp_dir)
        except Exception:
            pass

    def _show_progress(self, label, done, total):
        if total <= 0:
            return

        pct = int((done * 100) / total)
        if pct < 0:
            pct = 0
        if pct > 100:
            pct = 100

        width = 32
        filled = int((width * pct) / 100)
        bar = ("#" * filled) + ("-" * (width - filled))
        done_mb = done / (1024.0 * 1024.0)
        total_mb = total / (1024.0 * 1024.0)
        line = f"\r[{label}] [{bar}] {pct:3d}% {done_mb:8.1f}/{total_mb:8.1f} MB"

        with self._progress_lock:
            sys.stdout.write(line)
            sys.stdout.flush()

    def _finish_progress(self):
        with self._progress_lock:
            sys.stdout.write("\n")
            sys.stdout.flush()

    def log(self, msg):
        print(f"[{now_str()}] {msg}")

    def start_server(self):
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.sock.bind((HOST, self.port))
        self.sock.listen(5)
        self.running = True
        threading.Thread(target=self._accept_loop, daemon=True).start()

    def stop_server(self):
        self.running = False
        if self.sock:
            try:
                self.sock.close()
            except Exception:
                pass
            self.sock = None
        self.log(self.t["server_stop"])

    def _accept_loop(self):
        while self.running:
            try:
                conn, addr = self.sock.accept()
            except Exception:
                break
            threading.Thread(target=self._handle_client, args=(conn, addr), daemon=True).start()

    def _auth_check(self, message):
        key_b64 = message.get("guvenlik_anahtar_b64")

        if not self.security_key:
            if key_b64:
                return False, "Agent security key is not configured"
            return True, ""

        if not key_b64:
            return False, "Security key was not provided"

        try:
            decoded = base64.b64decode(key_b64, validate=True).decode("utf-8")
        except (binascii.Error, UnicodeDecodeError):
            return False, "Security key base64 is invalid"

        if decoded != self.security_key:
            return False, "Security key mismatch"

        return True, ""

    def _stream_disk(self, conn, disk_id, chunk_size, job_id, output_format="raw", sparse=False):
        disk_path = resolve_disk_path(disk_id)

        if not os.path.exists(disk_path):
            json_send(conn, {"tur": "hata", "format": output_format, "sparse": sparse, "mesaj": f"Disk not found: {disk_path}"})
            return

        total_size = disk_size_bytes_linux(disk_path)

        if total_size <= 0:
            json_send(conn, {"tur": "hata", "format": output_format, "sparse": sparse, "mesaj": "Disk size could not be read"})
            return

        self._set_job_state(job_id, "running")

        json_send(conn, {"durum": "ok", "is_id": job_id, "format": output_format, "sparse": sparse, "tahmini_boyut": total_size})
        json_send(conn, {"tur": "veri_basliyor", "is_id": job_id, "format": output_format, "sparse": sparse, "toplam": total_size})

        sha256 = hashlib.sha256()
        md5 = hashlib.md5()
        sent = 0
        last_report = 0.0
        label = self.t["progress_disk"]

        with open(disk_path, "rb", buffering=0) as f:
            while sent < total_size:
                state = self._get_job_state(job_id)
                if state == "paused":
                    time.sleep(0.2)
                    continue
                if state == "stopped":
                    self._finish_progress()
                    self.log(f"Disk transfer stopped by user: {disk_path} ({sent}/{total_size} bytes)")
                    try:
                        conn.shutdown(socket.SHUT_RDWR)
                    except Exception:
                        pass
                    try:
                        conn.close()
                    except Exception:
                        pass
                    self._clear_job_state(job_id)
                    return

                to_read = min(chunk_size, total_size - sent)
                data = f.read(to_read)
                if not data:
                    break
                conn.sendall(data)
                sent += len(data)
                sha256.update(data)
                md5.update(data)

                now = time.time()
                if now - last_report >= 0.25 or sent == total_size:
                    self._show_progress(label, sent, total_size)
                    last_report = now

        if sent == total_size:
            self._show_progress(label, total_size, total_size)
            self._finish_progress()
            mode_str = "sparse" if sparse else "full"
            self.log(f"Disk transfer completed: {disk_path} ({sent} bytes, mode: {mode_str})")
            json_send(conn, {
                "tur": "bitti",
                "is_id": job_id,
                "format": output_format,
                "sparse": sparse,
                "sha256": sha256.hexdigest(),
                "md5": md5.hexdigest(),
            })
        else:
            self._finish_progress()
            self.log(f"Disk transfer interrupted: {disk_path} ({sent}/{total_size} bytes)")
            json_send(conn, {
                "tur": "hata",
                "is_id": job_id,
                "format": output_format,
                "sparse": sparse,
                "mesaj": "Image transfer stopped by user" if self._get_job_state(job_id) == "stopped" else "Image transfer interrupted",
                "okunan": sent,
                "toplam": total_size,
            })

        self._clear_job_state(job_id)

    def _stream_file(self, conn, file_path, job_id, delete_after_success=False, index_key=""):
        if not os.path.exists(file_path):
            json_send(conn, {"durum": "hata", "is_id": job_id, "mesaj": f"File not found: {file_path}"})
            return

        total = os.path.getsize(file_path)
        self._set_job_state(job_id, "running")
        sha256 = hashlib.sha256()

        json_send(conn, {"durum": "ok", "is_id": job_id, "tahmini_boyut": total})
        json_send(conn, {"tur": "veri_basliyor", "is_id": job_id, "toplam": total})

        sent = 0
        last_report = 0.0
        label = self.t["progress_file"]
        with open(file_path, "rb") as f:
            while True:
                state = self._get_job_state(job_id)
                if state == "paused":
                    time.sleep(0.2)
                    continue
                if state == "stopped":
                    self._finish_progress()
                    self.log(f"File transfer stopped by user: {file_path} ({sent}/{total} bytes)")
                    try:
                        conn.shutdown(socket.SHUT_RDWR)
                    except Exception:
                        pass
                    try:
                        conn.close()
                    except Exception:
                        pass
                    self._clear_job_state(job_id)
                    return

                buf = f.read(BUFFER_SIZE)
                if not buf:
                    break
                conn.sendall(buf)
                sha256.update(buf)
                sent += len(buf)

                now = time.time()
                if now - last_report >= 0.25 or sent == total:
                    self._show_progress(label, sent, total)
                    last_report = now

        if sent == total:
            self._show_progress(label, total, total)
            self._finish_progress()
            self.log(f"File transfer completed: {file_path} ({sent} bytes)")
            json_send(conn, {
                "tur": "bitti",
                "is_id": job_id,
                "sha256": sha256.hexdigest(),
                "mesaj": "File transfer completed",
            })
            if delete_after_success:
                self._cleanup_transferred_file(file_path, index_key)
        else:
            self._finish_progress()
            self.log(f"File transfer interrupted: {file_path} ({sent}/{total} bytes)")
            json_send(conn, {"tur": "hata", "is_id": job_id, "mesaj": "File transfer interrupted"})

        self._clear_job_state(job_id)

    def _ram_acquire_avml(self, conn, output_file, job_id, output_format="raw"):
        self._set_job_state(job_id, "running")

        if os.geteuid() != 0:
            json_send(conn, {"tur": "hata", "is_id": job_id, "format": output_format, "mesaj": self.t["ram_need_root"], "kod": "ROOT_REQUIRED"})
            self._clear_job_state(job_id)
            return

        avml = self.avml_path or find_avml(self.script_dir)
        if not avml:
            json_send(conn, {"tur": "hata", "is_id": job_id, "format": output_format, "mesaj": "AVML not found", "kod": "AVML_NOT_FOUND"})
            self._clear_job_state(job_id)
            return

        if os.path.isfile(avml) and not os.access(avml, os.X_OK):
            try:
                os.chmod(avml, 0o755)
            except Exception:
                pass

        if self._get_job_state(job_id) == "stopped":
            json_send(conn, {
                "tur": "hata",
                "is_id": job_id,
                "format": output_format,
                "mesaj": "RAM acquisition stopped by user",
                "kod": "STOPPED_BY_USER",
            })
            self._clear_job_state(job_id)
            return

        total = calc_mem_total_bytes()
        json_send(conn, {"durum": "ok", "is_id": job_id, "format": output_format, "toplam_boyut": total, "avml_yol": avml})
        json_send(conn, {"tur": "veri_basliyor", "is_id": job_id, "format": output_format, "toplam": total})

        self.log(f"RAM output path: {output_file}")
        ram_work_dir = os.path.dirname(output_file) or self.script_dir
        avml_tmp_dir = os.path.join(ram_work_dir, ".tmp")
        try:
            os.makedirs(avml_tmp_dir, exist_ok=True)
        except Exception:
            avml_tmp_dir = ram_work_dir
        avml_env = os.environ.copy()
        avml_env["TMPDIR"] = avml_tmp_dir
        avml_env["TMP"] = avml_tmp_dir
        avml_env["TEMP"] = avml_tmp_dir
        self.log(f"AVML temp path: {avml_tmp_dir}")

        cmd_candidates = [
            [avml, output_file],
            [avml, "--source", "/proc/kcore", output_file],
        ]

        label = self.t["progress_ram"]

        try:
            last_err = ""
            last_rc = -1
            for idx, cmd in enumerate(cmd_candidates):
                if idx > 0:
                    self.log(f"AVML retry with fallback source: {' '.join(cmd)}")

                try:
                    if os.path.exists(output_file):
                        os.remove(output_file)
                except Exception:
                    pass

                proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=avml_env)
                last_report = 0.0
                was_paused = False

                while proc.poll() is None:
                    state = self._get_job_state(job_id)
                    if state == "paused":
                        if not was_paused:
                            try:
                                proc.send_signal(signal.SIGSTOP)
                            except Exception:
                                pass
                            was_paused = True
                        time.sleep(0.2)
                        continue
                    if was_paused:
                        try:
                            proc.send_signal(signal.SIGCONT)
                        except Exception:
                            pass
                        was_paused = False
                    if state == "stopped":
                        try:
                            proc.terminate()
                        except Exception:
                            pass
                        break

                    if os.path.exists(output_file) and total > 0:
                        current = os.path.getsize(output_file)
                        pct = int((current * 100) / total)
                        if pct >= 100:
                            pct = 99
                        now = time.time()
                        if now - last_report >= 0.25:
                            self._show_progress(label, current, total)
                            last_report = now
                        json_send(conn, {
                            "tur": "ilerleme",
                            "is_id": job_id,
                            "format": output_format,
                            "okunan": current,
                            "toplam": total,
                            "yuzde": pct,
                        })
                    time.sleep(1)

                final_size = os.path.getsize(output_file) if os.path.exists(output_file) else 0
                last_rc = proc.returncode
                try:
                    last_err = (proc.stderr.read() or b"").decode(errors="ignore").strip()
                except Exception:
                    last_err = ""

                if self._get_job_state(job_id) == "stopped":
                    self._finish_progress()
                    json_send(conn, {
                        "tur": "hata",
                        "is_id": job_id,
                        "format": output_format,
                        "mesaj": f"RAM acquisition stopped by user | partial_size={final_size}",
                        "kod": "STOPPED_BY_USER",
                    })
                    self.log(f"RAM acquisition stopped: {output_file} ({final_size} bytes)")
                    return

                if proc.returncode == 0 and final_size > 0:
                    if total > 0:
                        self._show_progress(label, total, total)
                    self._finish_progress()
                    sha256 = hashlib.sha256()
                    with open(output_file, "rb") as f:
                        while True:
                            chunk = f.read(BUFFER_SIZE)
                            if not chunk:
                                break
                            sha256.update(chunk)

                    json_send(conn, {
                        "tur": "bitti",
                        "is_id": job_id,
                        "format": output_format,
                        "boyut": final_size,
                        "sha256": sha256.hexdigest(),
                        "mesaj": "RAM acquisition completed",
                    })
                    self.log(f"RAM acquisition completed: {output_file} ({final_size} bytes)")
                    return

                retry_hint = "unable to create memory snapshot" in last_err.lower() or "/dev/crash" in last_err.lower()
                if idx < len(cmd_candidates) - 1 and retry_hint:
                    self._finish_progress()
                    continue

                break

            self._finish_progress()
            self.log(f"RAM acquisition failed: {output_file} ({final_size} bytes, rc={last_rc})")
            diagnostics = self._ram_output_diagnostics(output_file, final_size, total)
            json_send(conn, {
                "tur": "hata",
                "is_id": job_id,
                "format": output_format,
                "mesaj": f"AVML error: {last_err or last_rc} | {diagnostics}",
                "kod": "AVML_ERROR",
            })
        except Exception as e:
            self._finish_progress()
            json_send(conn, {"tur": "hata", "is_id": job_id, "format": output_format, "mesaj": str(e), "kod": "EXCEPTION"})
        finally:
            self._clear_job_state(job_id)

    def _docker_stream_acquisition(self, conn, container_id, acquire_diff, acquire_logs, acquire_config, job_id):
        self._set_job_state(job_id, "running")
        c_dir = os.path.join("/var/lib/docker/containers", container_id)
        cfg_file = os.path.join(c_dir, "config.v2.json")
        host_file = os.path.join(c_dir, "hostconfig.json")
        log_file = os.path.join(c_dir, f"{container_id}-json.log")

        config_v2 = {}
        if os.path.exists(cfg_file):
            try:
                with open(cfg_file, "r", encoding="utf-8") as f:
                    config_v2 = json.load(f)
            except Exception:
                pass

        if not config_v2:
            try:
                insp = subprocess.run(["docker", "inspect", container_id], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=10)
                if insp.returncode == 0:
                    arr = json.loads(insp.stdout)
                    if arr:
                        config_v2 = arr[0]
            except Exception:
                pass

        if not config_v2:
            json_send(conn, {"durum": "hata", "tur": "hata", "is_id": job_id, "mesaj": f"Container directory / inspect not found for: {container_id}"})
            self._clear_job_state(job_id)
            return

        upper_dir = ((config_v2.get("GraphDriver") or {}).get("Data") or {}).get("UpperDir")
        c_name = config_v2.get("Name", "").lstrip("/") or "container"

        temp_tar = tempfile.NamedTemporaryFile(suffix=".tar.gz", delete=False)
        temp_tar_path = temp_tar.name
        temp_tar.close()

        try:
            import io
            self.log(f"Building Docker evidence bundle for {container_id} ({c_name})...")
            with tarfile.open(temp_tar_path, "w:gz") as tar:
                if acquire_config:
                    if os.path.exists(cfg_file):
                        tar.add(cfg_file, arcname="config.v2.json")
                    else:
                        cfg_bytes = json.dumps(config_v2, indent=2).encode("utf-8")
                        ti = tarfile.TarInfo(name="config.v2.json")
                        ti.size = len(cfg_bytes)
                        ti.mtime = int(time.time())
                        tar.addfile(ti, io.BytesIO(cfg_bytes))

                    if os.path.exists(host_file):
                        tar.add(host_file, arcname="hostconfig.json")
                    elif config_v2.get("HostConfig"):
                        hc_bytes = json.dumps(config_v2.get("HostConfig"), indent=2).encode("utf-8")
                        ti = tarfile.TarInfo(name="hostconfig.json")
                        ti.size = len(hc_bytes)
                        ti.mtime = int(time.time())
                        tar.addfile(ti, io.BytesIO(hc_bytes))

                if acquire_logs:
                    if os.path.exists(log_file):
                        tar.add(log_file, arcname="container.log")
                    else:
                        try:
                            l_out = subprocess.run(["docker", "logs", container_id], stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=10)
                            log_bytes = l_out.stdout + l_out.stderr
                            ti = tarfile.TarInfo(name="container.log")
                            ti.size = len(log_bytes)
                            ti.mtime = int(time.time())
                            tar.addfile(ti, io.BytesIO(log_bytes))
                        except Exception:
                            pass

                meta_bytes = json.dumps({
                    "edinim_zamani": datetime.now().isoformat(),
                    "konteyner_id": container_id,
                    "isim": c_name,
                    "config_v2": config_v2,
                }, indent=2).encode("utf-8")

                ti = tarfile.TarInfo(name="docker_metadata.json")
                ti.size = len(meta_bytes)
                ti.mtime = int(time.time())
                tar.addfile(ti, io.BytesIO(meta_bytes))

                if acquire_diff and upper_dir and os.path.exists(upper_dir) and os.path.isdir(upper_dir):
                    tar.add(upper_dir, arcname="upper_drift_files")

            self._stream_file(conn, temp_tar_path, job_id, delete_after_success=True)

        except Exception as e:
            self.log(f"Docker acquisition error: {e}")
            json_send(conn, {"durum": "hata", "tur": "hata", "is_id": job_id, "mesaj": str(e)})
            try:
                if os.path.exists(temp_tar_path):
                    os.remove(temp_tar_path)
            except Exception:
                pass
        finally:
            self._clear_job_state(job_id)

    def _handle_client(self, conn, addr):
        self.log(f"{self.t['conn']} {addr}")
        authorized = False

        try:
            reader = conn.makefile("rb")
            while True:
                line = reader.readline()
                if not line:
                    return

                try:
                    message = json.loads(line.decode("utf-8").strip())
                except json.JSONDecodeError:
                    json_send(conn, {"durum": "hata", "mesaj": "Invalid JSON"})
                    continue

                cmd = message.get("komut")

                if cmd == "merhaba":
                    ok, reason = self._auth_check(message)
                    if not ok:
                        json_send(conn, {"durum": "hata", "mesaj": reason, "kod": "AUTH_FAILED"})
                        self.log(f"{self.t['auth_reject']} {addr} | {reason}")
                        return

                    authorized = True
                    self.log(f"{self.t['auth_accept']} {addr}")
                    json_send(conn, {
                        "durum": "ok",
                        "sunucu": "linux-ajan",
                        "surum": "0.1",
                        "ozellikler": ["disk_imaj", "linux_avml_ram"],
                    })
                    continue

                if not authorized:
                    json_send(conn, {"durum": "hata", "mesaj": "Authorization required. Authenticate with hello first.", "kod": "AUTH_REQUIRED"})
                    continue

                if cmd == "disk_listele":
                    try:
                        disks = list_disks_linux()
                        if not disks:
                            json_send(conn, {"durum": "hata", "mesaj": "No disk found or access denied"})
                        else:
                            json_send(conn, {"durum": "ok", "diskler": disks, "tani": {"platform": "linux", "disk_sayisi": len(disks)}})
                    except Exception as e:
                        json_send(conn, {"durum": "hata", "mesaj": str(e)})

                elif cmd == "imaj_baslat":
                    disk_id = message.get("disk_id", "")
                    raw_fmt = message.get("format", "raw")
                    fmt, format_error = normalize_output_format(raw_fmt)
                    if format_error:
                        json_send(conn, {"durum": "hata", "mesaj": format_error, "kod": "UNSUPPORTED_FORMAT"})
                        continue
                    sparse = bool(message.get("sparse", False)) or parse_sparse_preference(raw_fmt)
                    chunk_size = int(message.get("parca_boyutu", 4 * 1024 * 1024))
                    job_id = message.get("is_id") or ("IMG_" + str(int(time.time())))
                    mode_str = "sparse" if sparse else "full"
                    self.log(f"Starting disk acquisition for {disk_id} in {fmt} format (mode: {mode_str})")
                    self._stream_disk(conn, disk_id, chunk_size, job_id, fmt, sparse)

                elif cmd == "winpmem_kontrol":
                    json_send(conn, {
                        "durum": "ok",
                        "winpmem_mevcut": False,
                        "winpmem_yol": "",
                        "yonetici_yetkisi": os.geteuid() == 0,
                        "ram_boyut": calc_mem_total_bytes(),
                        "mesaj": "WinPMEM is not used on Linux. Use AVML.",
                    })

                elif cmd == "avml_kontrol":
                    avml = self.avml_path or find_avml(self.script_dir)
                    json_send(conn, {
                        "durum": "ok",
                        "avml_mevcut": bool(avml),
                        "avml_yol": avml or "",
                        "yonetici_yetkisi": os.geteuid() == 0,
                        "ram_boyut": calc_mem_total_bytes(),
                        "mesaj": "AVML ready" if avml else "AVML not found",
                    })

                elif cmd == "ram_edinim_baslat":
                    job_id = message.get("is_id") or ("RAM_" + str(int(time.time())))
                    fmt, format_error = normalize_output_format(message.get("format", "raw"))
                    if format_error:
                        json_send(conn, {"durum": "hata", "is_id": job_id, "mesaj": format_error, "kod": "UNSUPPORTED_FORMAT"})
                        continue
                    output_file = os.path.basename(message.get("cikti_dosya", "memory_dump_linux.raw"))
                    self.log(f"Starting RAM acquisition for {output_file} in {fmt} format")
                    output_path, output_error = self._select_ram_output_path(output_file, calc_mem_total_bytes())
                    if output_error:
                        json_send(conn, {
                            "tur": "hata",
                            "is_id": job_id,
                            "format": fmt,
                            "mesaj": output_error,
                            "kod": "RAM_OUTPUT_UNAVAILABLE",
                        })
                        continue
                    self.ram_output_index[output_file] = output_path
                    self._ram_acquire_avml(conn, output_path, job_id, fmt)

                elif cmd == "ram_dosya_indir":
                    job_id = message.get("is_id") or ("RAMDL_" + str(int(time.time())))
                    file_name = os.path.basename(message.get("dosya", "memory_dump_linux.raw"))
                    full = self.ram_output_index.get(file_name, os.path.join(self.script_dir, file_name))
                    self._stream_file(conn, full, job_id, delete_after_success=True, index_key=file_name)

                elif cmd == "edinim_kontrol":
                    job_id = message.get("is_id", "")
                    action = message.get("eylem", "") or message.get("action", "")
                    ok, msg = self._control_job(job_id, action)
                    json_send(conn, {
                        "durum": "ok" if ok else "hata",
                        "is_id": job_id,
                        "eylem": action,
                        "mesaj": msg,
                    })

                elif cmd == "docker_durum":
                    try:
                        st = docker_get_status()
                        json_send(conn, {"durum": "ok", **st})
                    except Exception as e:
                        json_send(conn, {"durum": "hata", "mesaj": str(e)})

                elif cmd == "docker_listele":
                    try:
                        containers = docker_list_containers()
                        json_send(conn, {"durum": "ok", "konteynerler": containers, "toplam": len(containers)})
                    except Exception as e:
                        json_send(conn, {"durum": "hata", "mesaj": str(e)})

                elif cmd == "docker_loglar":
                    try:
                        cid = message.get("konteyner_id", "")
                        tail = int(message.get("tail", 200))
                        logs = docker_get_logs(cid, tail)
                        json_send(conn, {"durum": "ok", "konteyner_id": cid, "loglar": logs, "toplam": len(logs)})
                    except Exception as e:
                        json_send(conn, {"durum": "hata", "mesaj": str(e)})

                elif cmd == "docker_edinim_baslat":
                    cid = message.get("konteyner_id", "")
                    job_id = message.get("is_id") or ("DOCKER_" + str(int(time.time())))
                    acquire_diff = bool(message.get("acquire_diff", True))
                    acquire_logs = bool(message.get("acquire_logs", True))
                    acquire_config = bool(message.get("acquire_config", True))
                    self.log(f"Starting Docker acquisition for {cid}")
                    self._docker_stream_acquisition(conn, cid, acquire_diff, acquire_logs, acquire_config, job_id)

                else:
                    json_send(conn, {"durum": "hata", "mesaj": f"{self.t['unknown_cmd']}: {cmd}"})

        except Exception as e:
            self.log(f"Client error: {e}")
        finally:
            try:
                conn.close()
            except Exception:
                pass


def startup_wizard(show_logo=True):
    if os.name == "nt":
        print(TR["linux_only"])
        raise SystemExit(1)

    if show_logo:
        print(f"{ASCII_LOGO}\n")
        print(f"Amele Linux Agent v{VERSION} (TUI)\n")

    lang = input(TR["ask_lang"]).strip().lower() or "tr"
    if lang not in {"tr", "en"}:
        lang = "tr"
    t = EN if lang == "en" else TR

    sec_answer = input(t["ask_sec"]).strip()
    security_key = ""
    if is_yes(sec_answer, lang):
        security_key = input(t["ask_pw"]).strip()

    port_text = input(t["ask_port"]).strip()
    try:
        port = int(port_text) if port_text else DEFAULT_PORT
        if port <= 0 or port > 65535:
            raise ValueError("port")
    except Exception:
        print(t["invalid_port"])
        port = DEFAULT_PORT

    script_dir = app_base_dir()
    os_info = load_os_release()
    distro = os_info.get("PRETTY_NAME", os_info.get("ID", "unknown"))
    print(f"{t['distro']} {distro}")

    avml_path = find_avml(script_dir)
    if avml_path:
        print(f"{t['avml_found']} {avml_path}")
    else:
        print(t["avml_missing"])
        install_answer = input(t["ask_repo_install"]).strip()
        if is_yes(install_answer, lang):
            ok = try_install_avml(lang, t)
            if ok:
                avml_path = find_avml(script_dir)
                if avml_path:
                    print(f"{t['repo_ok']} {avml_path}")
                else:
                    print(t["repo_fail"])
            else:
                print(t["repo_fail"])

            if not avml_path:
                ok_wget = try_install_avml_via_wget(script_dir, t)
                if ok_wget:
                    avml_path = find_avml(script_dir)
                    if avml_path:
                        print(f"{t['wget_ok']} {avml_path}")
                    else:
                        print(t["wget_fail"])
                else:
                    print(t["wget_fail"])

        if not avml_path:
            print(t["manual_needed"])
            print(AVML_RELEASE_URL)
            print(script_dir)

    return lang, security_key, port, avml_path


def main():
    import argparse
    parser = argparse.ArgumentParser(
        description=f"{ASCII_LOGO}\n\nAmele Linux Agent v{VERSION}",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--port", type=int, default=DEFAULT_PORT, help="Port to listen on")
    parser.add_argument("--key", type=str, default="", help="Security key / password")
    parser.add_argument("--lang", type=str, default="tr", choices=["tr", "en"], help="Language (tr/en)")
    parser.add_argument("--non-interactive", action="store_true", help="Start agent without prompt wizard")
    parser.add_argument("--no-logo", "-q", "--quiet", action="store_true", help="Suppress ASCII logo")
    args, _ = parser.parse_known_args()

    show_logo = not args.no_logo
    if args.non_interactive or "--port" in sys.argv:
        if show_logo:
            print(f"{ASCII_LOGO}\n")
            print(f"Amele Linux Agent v{VERSION}\n")
        lang = args.lang
        security_key = args.key
        port = args.port
        script_dir = app_base_dir()
        avml_path = find_avml(script_dir)
    else:
        lang, security_key, port, avml_path = startup_wizard(show_logo=show_logo)

    controller = LinuxAgentController(language=lang, security_key=security_key, port=port, avml_path=avml_path)
    t = controller.t

    controller.start_server()
    controller.log(f"{t['server_start']} {HOST}:{port}")
    print(t["ctrlc"])

    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        controller.stop_server()


if __name__ == "__main__":
    main()

