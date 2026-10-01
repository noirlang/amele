#!/usr/bin/env python3
"""
Amele Windows Agent v0.0.8
- Uzak disk imaji alma
- WinPMEM kontrol / otomatik indirme
- Secilebilir port + hafif Tk arayuz
"""

import glob
import hashlib
import json
import os
import queue
import base64
import binascii
import socket
import ssl
import struct
import subprocess
import sys
import threading
import time
import urllib.request
import urllib.error
import tempfile
import tarfile
from datetime import datetime

VERSION = "0.0.8"

try:
    import tkinter as tk
    from tkinter import ttk, messagebox
    HAS_TK = True
except Exception:
    HAS_TK = False

try:
    import win32con
    import win32file
    WINDOWS = True
except ImportError:
    WINDOWS = False
    win32con = None
    win32file = None

PYWIN32_OK = WINDOWS

HOST = "0.0.0.0"
DEFAULT_PORT = 4444
WINPMEM_NAME = "winpmem.exe"
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

WINPMEM_URLS = [
    "https://download.amele.noirlang.tr/winpmem.exe",
]

DOWNLOAD_HEADERS = {
    "User-Agent": (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
        "AppleWebKit/537.36 (KHTML, like Gecko) "
        "Chrome/125.0.0.0 Safari/537.36"
    ),
    "Accept": "application/octet-stream,application/vnd.microsoft.portable-executable,*/*",
    "Accept-Language": "tr-TR,tr;q=0.9,en-US;q=0.8,en;q=0.7",
    "Referer": "https://download.amele.noirlang.tr/",
    "Connection": "close",
}


def now_str():
    return datetime.now().strftime("%Y-%m-%d %H:%M:%S")


def app_base_dir():
    if getattr(sys, "frozen", False):
        return os.path.dirname(os.path.abspath(sys.executable))
    return os.path.dirname(os.path.abspath(__file__))


def json_gonder(conn, veri):
    conn.sendall(json.dumps(veri, ensure_ascii=False).encode("utf-8") + b"\n")


def normalize_output_format(value):
    fmt = str(value or "raw").strip().lower()
    if fmt in {"dd", "img"}:
        fmt = "raw"
    if fmt not in SUPPORTED_OUTPUT_FORMATS:
        return "", f"Unsupported output format: {fmt}. Supported formats: raw, aff4"
    return fmt, ""


def find_winpmem_paths(script_dir):
    return [
        os.path.join(script_dir, WINPMEM_NAME),
        WINPMEM_NAME,
        os.path.join(script_dir, "go-winpmem_amd64_1.0-rc2_signed.exe"),
        "go-winpmem_amd64_1.0-rc2_signed.exe",
        r"C:\Forensics\winpmem.exe",
        r"C:\Tools\winpmem.exe",
        r"C:\Forensics\go-winpmem_amd64_1.0-rc2_signed.exe",
        r"C:\Tools\go-winpmem_amd64_1.0-rc2_signed.exe",
    ]


def winpmem_kontrol(script_dir):
    if not WINDOWS:
        return False, "", "Non-Windows environment"

    for yol in find_winpmem_paths(script_dir):
        if os.path.exists(yol):
            return True, yol, "WinPMEM bulundu"

    return False, "", "WinPMEM not found"


def winpmem_indir(script_dir, log_cb=None, progress_cb=None):
    if not WINDOWS:
        return False, "", "Non-Windows environment"

    hedef = os.path.join(script_dir, WINPMEM_NAME)
    gecici_hedef = hedef + ".download"

    def progress_guncelle(indirilen, toplam):
        if not progress_cb:
            return
        if toplam <= 0:
            progress_cb("Downloading: size unknown")
            return
        yuzde = int((indirilen * 100) / toplam)
        progress_cb(f"Downloading: %{min(yuzde, 100)}")

    son_hata = ""
    for url in WINPMEM_URLS:
        attempts = [("verified TLS", None)]
        if url.startswith("https://amele.noirlang.tr/") or url.startswith("https://download.amele.noirlang.tr/"):
            attempts.append(("certificate fallback", ssl._create_unverified_context()))

        for attempt_label, ssl_context in attempts:
            try:
                if os.path.exists(gecici_hedef):
                    try:
                        os.remove(gecici_hedef)
                    except Exception:
                        pass

                if log_cb:
                    log_cb(f"Downloading WinPMEM ({attempt_label}): {url}")

                req = urllib.request.Request(url, headers=DOWNLOAD_HEADERS)
                with urllib.request.urlopen(req, timeout=60, context=ssl_context) as response:
                    toplam = int(response.headers.get("Content-Length") or 0)
                    indirilen = 0

                    with open(gecici_hedef, "wb") as f:
                        while True:
                            blok = response.read(1024 * 1024)
                            if not blok:
                                break
                            f.write(blok)
                            indirilen += len(blok)
                            progress_guncelle(indirilen, toplam)

                if os.path.exists(gecici_hedef) and os.path.getsize(gecici_hedef) > 0:
                    os.replace(gecici_hedef, hedef)

                if os.path.exists(hedef) and os.path.getsize(hedef) > 0:
                    if log_cb:
                        log_cb(f"WinPMEM downloaded: {hedef}")
                    return True, hedef, "WinPMEM downloaded"
            except urllib.error.HTTPError as e:
                son_hata = f"HTTP {e.code}: {e.reason}"
                if e.code == 403:
                    son_hata += " (server rejected the app request)"
                if os.path.exists(gecici_hedef):
                    try:
                        os.remove(gecici_hedef)
                    except Exception:
                        pass
                if log_cb:
                    log_cb(f"Download attempt failed ({attempt_label}): {son_hata}")
                break
            except Exception as e:
                son_hata = str(e)
                if os.path.exists(gecici_hedef):
                    try:
                        os.remove(gecici_hedef)
                    except Exception:
                        pass
                if log_cb:
                    log_cb(f"Download attempt failed ({attempt_label}): {e}")
                if attempt_label == "verified TLS" and "CERTIFICATE_VERIFY_FAILED" in son_hata:
                    if log_cb:
                        log_cb("Certificate validation failed; retrying amele.noirlang.tr with certificate fallback")
                    continue
                break

    return False, "", f"WinPMEM download failed: {son_hata}"


def yonetici_yetkisi_kontrol():
    if not WINDOWS:
        return False
    try:
        import ctypes
        return bool(ctypes.windll.shell32.IsUserAnAdmin())
    except Exception:
        return False


def ram_boyut_al():
    if not WINDOWS:
        return 0
    try:
        import ctypes

        class MEMORYSTATUSEX(ctypes.Structure):
            _fields_ = [
                ("dwLength", ctypes.c_ulong),
                ("dwMemoryLoad", ctypes.c_ulong),
                ("ullTotalPhys", ctypes.c_ulonglong),
                ("ullAvailPhys", ctypes.c_ulonglong),
                ("ullTotalPageFile", ctypes.c_ulonglong),
                ("ullAvailPageFile", ctypes.c_ulonglong),
                ("ullTotalVirtual", ctypes.c_ulonglong),
                ("ullAvailVirtual", ctypes.c_ulonglong),
                ("ullAvailExtendedVirtual", ctypes.c_ulonglong),
            ]

        mem = MEMORYSTATUSEX()
        mem.dwLength = ctypes.sizeof(MEMORYSTATUSEX)
        ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(mem))
        return int(mem.ullTotalPhys)
    except Exception:
        return 0


def disk_boyut_al(disk_id):
    if not WINDOWS:
        return 0

    handle = None
    try:
        handle = win32file.CreateFile(
            f"\\\\.\\PhysicalDrive{disk_id}",
            win32con.GENERIC_READ,
            win32con.FILE_SHARE_READ | win32con.FILE_SHARE_WRITE,
            None,
            win32con.OPEN_EXISTING,
            0,
            None,
        )
        ioctl_disk_get_length_info = 0x0007405C
        buf = win32file.DeviceIoControl(handle, ioctl_disk_get_length_info, None, 8)
        return struct.unpack("<Q", buf)[0]
    except Exception:
        return 0
    finally:
        if handle:
            win32file.CloseHandle(handle)


def disk_listele_tani():
    tani = {
        "windows_mod": WINDOWS,
        "pywin32_ok": PYWIN32_OK,
        "yonetici": yonetici_yetkisi_kontrol(),
        "errors": [],
    }

    if not WINDOWS:
        tani["mesaj"] = "pywin32 modules could not be loaded"
        return [], tani

    diskler = []
    for i in range(32):
        try:
            handle = win32file.CreateFile(
                f"\\\\.\\PhysicalDrive{i}",
                win32con.GENERIC_READ,
                win32con.FILE_SHARE_READ | win32con.FILE_SHARE_WRITE,
                None,
                win32con.OPEN_EXISTING,
                0,
                None,
            )
            win32file.CloseHandle(handle)
            diskler.append({
                "id": str(i),
                "ad": f"PhysicalDrive{i}",
                "boyut": disk_boyut_al(i),
            })
        except Exception as e:
            if len(tani["errors"]) < 5:
                tani["errors"].append(f"PhysicalDrive{i}: {e}")

    tani["disk_sayisi"] = len(diskler)
    if len(diskler) == 0:
        tani["mesaj"] = "No disks found"
    else:
        tani["mesaj"] = "Disks listed"

    return diskler, tani


class AgentController:
    def __init__(self, ui=None):
        self.ui = ui
        self.sock = None
        self.running = False
        self.port = DEFAULT_PORT
        self.script_dir = app_base_dir()
        self.winpmem_path = ""
        self.security_key = ""
        self.language = "tr"
        self.log_file_path = self._init_log_file()
        self.ram_output_index = {}
        self.job_lock = threading.Lock()
        self.job_state = {}

    def _set_job_state(self, job_id, state):
        if not job_id:
            return
        with self.job_lock:
            self.job_state[job_id] = state

def docker_get_status():
    is_avail = False
    is_running = False
    count = 0
    running = 0
    paused = 0
    stopped = 0
    root = "C:\\ProgramData\\Docker"

    try:
        out = subprocess.run(["docker", "info"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=5)
        if out.returncode == 0:
            is_avail = True
            is_running = True
    except Exception:
        pass

    if is_running:
        try:
            out2 = subprocess.run(["docker", "ps", "-a", "--format", "{{.ID}}|{{.State}}"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=5)
            if out2.returncode == 0:
                lines = [l.strip() for l in out2.stdout.splitlines() if l.strip()]
                count = len(lines)
                for l in lines:
                    parts = l.split("|")
                    st = parts[1].lower() if len(parts) > 1 else ""
                    if "running" in st or "up" in st:
                        running += 1
                    elif "paused" in st:
                        paused += 1
                    else:
                        stopped += 1
        except Exception:
            pass

    return {
        "durum": "ok",
        "docker_available": is_avail,
        "docker_running": is_running,
        "docker_mevcut": is_avail,
        "docker_calisiyor": is_running,
        "storage_driver": "windowsfilter" if os.path.exists("C:\\ProgramData\\Docker\\windowsfilter") else "unknown",
        "depolama_surucusu": "windowsfilter" if os.path.exists("C:\\ProgramData\\Docker\\windowsfilter") else "unknown",
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
        if "docker.sock" in src or "docker.sock" in dst or "docker_engine" in src or "docker_engine" in dst:
            reasons.append("Host Docker soketi / borusu konteyner içine mount edilmiş (DoD Escape riski).")
            score += 40

    user_val = (config_v2.get("Config") or {}).get("User") or config_v2.get("User") or ""
    if not user_val or user_val == "0" or user_val.lower() in {"root", "containeradministrator"}:
        reasons.append("Konteyner yönetici (Root / ContainerAdministrator) yetkileriyle çalışıyor.")
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
    res = []
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
                        driver = d.get("Driver", "windowsfilter")
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
    logs = []
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


    def _get_job_state(self, job_id):
        if not job_id:
            return "running"
        with self.job_lock:
            return self.job_state.get(job_id, "running")

    def _clear_job_state(self, job_id):
        if not job_id:
            return
        with self.job_lock:
            self.job_state.pop(job_id, None)

    def _control_job(self, job_id, action):
        if not job_id or not action:
            return False, "is_id and action are required"

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
            return False, "Unsupported action"

        with self.job_lock:
            if job_id not in self.job_state:
                return False, "Job not found"
            if action == "pause":
                self.job_state[job_id] = "paused"
            elif action == "resume":
                self.job_state[job_id] = "running"
            else:
                self.job_state[job_id] = "stopped"

        return True, "Control command applied"

    def cevir(self, metin):
        if not isinstance(metin, str):
            return metin

        eslemeler = [
            ("Amele Windows Agent", "Amele Windows Ajan"),
            ("Server", "Sunucu"),
            ("Connection", "Baglanti"),
            ("Security Key:", "Guvenlik Anahtari:"),
            ("Approve", "Onayla"),
            ("Reset", "Sifirla"),
            ("Start", "Baslat"),
            ("Stop", "Durdur"),
            ("Check", "Kontrol Et"),
            ("Download", "Indir"),
            ("Transfer", "Aktarim"),
            ("Transfer info appears here", "Aktarim bilgisi burada gorunur"),
            ("Log", "Gunluk"),
            ("Ready", "Hazir"),
            ("Running", "Calisiyor"),
            ("Stopped", "Durduruldu"),
            ("Startup error", "Baslatma hatasi"),
            ("Port must be between 1 and 65535", "Port 1 ile 65535 arasinda olmali"),
            ("Invalid port", "Gecersiz port"),
            ("Enter a security key to approve", "Onaylamak icin guvenlik anahtari girin"),
            ("WinPMEM status: not checked", "WinPMEM durumu: kontrol edilmedi"),
            ("WinPMEM ready:", "WinPMEM hazir:"),
            ("WinPMEM unavailable:", "WinPMEM kullanilamiyor:"),
            ("WinPMEM downloaded:", "WinPMEM indirildi:"),
            ("Download failed:", "Indirme basarisiz:"),
            ("Server started", "Sunucu basladi"),
            ("Server stopped", "Sunucu durduruldu"),
            ("Authorized connection accepted", "Yetkili baglanti kabul edildi"),
            ("Unauthorized connection rejected", "Yetkisiz baglanti reddedildi"),
            ("Key verification enabled", "Anahtar dogrulama aktif"),
            ("Key verification disabled", "Anahtar dogrulama kapali"),
            ("Key approved", "Anahtar onaylandi"),
            ("Key reset", "Anahtar sifirlandi"),
            ("Key: Disabled", "Anahtar: Kapali"),
            ("Key: Active", "Anahtar: Aktif"),
            ("Client error", "Istemci hatasi"),
            ("Unknown command", "Bilinmeyen komut"),
            ("RAM acquisition", "RAM edinimi"),
            ("File transfer", "Dosya aktarimi"),
            ("Downloading", "Indiriliyor"),
            ("Error", "Hata"),
            ("Warning", "Uyari"),
            ("Info", "Bilgi"),
        ]

        sonuc = metin
        if self.language == "en":
            for en, tr in eslemeler:
                sonuc = sonuc.replace(tr, en)
        else:
            for en, tr in eslemeler:
                sonuc = sonuc.replace(en, tr)
        return sonuc

    def _init_log_file(self):
        try:
            home = os.path.expanduser("~")
            docs = os.path.join(home, "Documents")
            if not os.path.isdir(docs):
                docs = home
            log_dir = os.path.join(docs, "Amele", "logs")
            os.makedirs(log_dir, exist_ok=True)
            dosya = datetime.now().strftime("windows_agent_%Y%m%d_%H%M%S.log")
            return os.path.join(log_dir, dosya)
        except Exception:
            return ""

    def log(self, msg):
        satir = f"[{now_str()}] {self.cevir(msg)}"
        print(satir)
        if self.log_file_path:
            try:
                with open(self.log_file_path, "a", encoding="utf-8") as f:
                    f.write(satir + "\n")
            except Exception:
                pass
        if self.ui:
            self.ui.log(satir)

    def transfer_bilgi(self, msg):
        msg = self.cevir(msg)
        if self.ui:
            self.ui.set_transfer(msg)
        else:
            print(msg)

    def winpmem_hazirla(self, auto_download=True):
        var_mi, yol, mesaj = winpmem_kontrol(self.script_dir)
        if var_mi:
            self.winpmem_path = yol
            return True, yol, mesaj

        if auto_download:
            self.log("WinPMEM not found, starting automatic download...")
            ok, yol, mesaj = winpmem_indir(
                self.script_dir,
                log_cb=self.log,
                progress_cb=self.transfer_bilgi,
            )
            if ok:
                self.winpmem_path = yol
            return ok, yol, mesaj

        return False, "", mesaj

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

    def start_server(self, port):
        if self.running:
            return False, "Server is already running"

        self.port = port
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.sock.bind((HOST, self.port))
        self.sock.listen(5)
        self.running = True

        threading.Thread(target=self._accept_loop, daemon=True).start()
        self.log(f"Server started: {HOST}:{self.port}")
        return True, "Server started"

    def stop_server(self):
        self.running = False
        if self.sock:
            try:
                self.sock.close()
            except Exception:
                pass
            self.sock = None
        self.log("Server stopped")

    def _accept_loop(self):
        while self.running:
            try:
                conn, addr = self.sock.accept()
            except Exception:
                break
            threading.Thread(target=self._istemci_yonet, args=(conn, addr), daemon=True).start()

    def _imaj_gonder(self, conn, disk_id, parca_boyutu=4 * 1024 * 1024, is_id=None, output_format="raw"):
        handle = None
        try:
            handle = win32file.CreateFile(
                f"\\\\.\\PhysicalDrive{disk_id}",
                win32con.GENERIC_READ,
                win32con.FILE_SHARE_READ | win32con.FILE_SHARE_WRITE,
                None,
                win32con.OPEN_EXISTING,
                0,
                None,
            )

            is_id = is_id or ("IMG_" + str(int(time.time())))
            toplam_boyut = disk_boyut_al(disk_id)
            if toplam_boyut <= 0:
                json_gonder(conn, {"tur": "hata", "format": output_format, "mesaj": "Disk size could not be read"})
                return
            self._set_job_state(is_id, "running")

            json_gonder(conn, {
                "durum": "ok",
                "is_id": is_id,
                "format": output_format,
                "tahmini_boyut": toplam_boyut,
            })

            sha256 = hashlib.sha256()
            md5 = hashlib.md5()
            okunan = 0
            son_rapor = time.time()
            baslangic = time.time()

            json_gonder(conn, {
                "tur": "veri_basliyor",
                "is_id": is_id,
                "format": output_format,
                "toplam": toplam_boyut,
            })

            while okunan < toplam_boyut:
                state = self._get_job_state(is_id)
                if state == "paused":
                    time.sleep(0.2)
                    continue
                if state == "stopped":
                    self.transfer_bilgi(f"Disk transfer stopped by user ({is_id})")
                    try:
                        conn.shutdown(socket.SHUT_RDWR)
                    except Exception:
                        pass
                    try:
                        conn.close()
                    except Exception:
                        pass
                    return

                okunacak = min(parca_boyutu, toplam_boyut - okunan)
                hr, buf = win32file.ReadFile(handle, okunacak)
                if hr != 0 or not buf:
                    break

                conn.sendall(buf)
                sha256.update(buf)
                md5.update(buf)
                okunan += len(buf)

                simdi = time.time()
                if simdi - son_rapor >= 1:
                    yuzde = int((okunan * 100) / toplam_boyut)
                    gecen = max(simdi - baslangic, 0.001)
                    hiz_mb = (okunan / 1024 / 1024) / gecen
                    self.transfer_bilgi(
                        f"Disk transfer ({is_id}): %{yuzde} | {okunan // (1024*1024)} MB / {toplam_boyut // (1024*1024)} MB | {hiz_mb:.1f} MB/s"
                    )
                    son_rapor = simdi

            if okunan == toplam_boyut:
                json_gonder(conn, {
                    "tur": "bitti",
                    "is_id": is_id,
                    "format": output_format,
                    "sha256": sha256.hexdigest(),
                    "md5": md5.hexdigest(),
                })
                self.transfer_bilgi(f"Disk transfer completed ({is_id})")
            else:
                json_gonder(conn, {
                    "tur": "hata",
                    "is_id": is_id,
                    "format": output_format,
                    "mesaj": "Image transfer stopped by user" if self._get_job_state(is_id) == "stopped" else "Image transfer interrupted",
                    "okunan": okunan,
                    "toplam": toplam_boyut,
                })
                self.transfer_bilgi(f"Disk transfer interrupted ({is_id})")

        except Exception as e:
            json_gonder(conn, {"tur": "hata", "format": output_format, "mesaj": str(e)})
            self.transfer_bilgi(f"Disk transfer error: {e}")
        finally:
            self._clear_job_state(is_id)
            if handle:
                win32file.CloseHandle(handle)

    def _ram_edinim_baslat(self, conn, cikti_dosya, is_id, output_format="raw"):
        self._set_job_state(is_id, "running")

        if not WINDOWS:
            json_gonder(conn, {"tur": "hata", "is_id": is_id, "format": output_format, "mesaj": "Windows required"})
            self._clear_job_state(is_id)
            return

        ok, yol, mesaj = self.winpmem_hazirla(auto_download=True)
        if not ok:
            json_gonder(conn, {"tur": "hata", "is_id": is_id, "format": output_format, "mesaj": mesaj, "kod": "WINPMEM_NOT_FOUND"})
            self._clear_job_state(is_id)
            return

        if not yonetici_yetkisi_kontrol():
            json_gonder(conn, {"tur": "hata", "is_id": is_id, "format": output_format, "mesaj": "Administrator privileges required", "kod": "ADMIN_REQUIRED"})
            self._clear_job_state(is_id)
            return

        if self._get_job_state(is_id) == "stopped":
            json_gonder(conn, {
                "tur": "hata",
                "is_id": is_id,
                "format": output_format,
                "mesaj": "RAM acquisition stopped by user",
                "kod": "STOPPED_BY_USER",
            })
            self._clear_job_state(is_id)
            return

        toplam_ram = ram_boyut_al()
        json_gonder(conn, {"durum": "ok", "is_id": is_id, "format": output_format, "toplam_boyut": toplam_ram, "winpmem_yol": yol})

        komut_adaylari = [
            [yol, "acquire", cikti_dosya],
            [yol, "acquire", "--format", "raw", cikti_dosya],
        ]

        self.transfer_bilgi(f"RAM acquisition started: {cikti_dosya}")

        try:
            process = None
            secilen_komut = None
            son_hata = ""
            ilk_hata = ""

            for aday in komut_adaylari:
                try:
                    p = subprocess.Popen(aday, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
                    time.sleep(1)

                    if p.poll() is None:
                        process = p
                        secilen_komut = aday
                        break

                    stderr = (p.stderr.read() or b"").decode(errors="ignore")
                    hata = stderr.strip() or f"returncode={p.returncode}"
                    if not ilk_hata:
                        ilk_hata = hata
                    son_hata = hata
                except Exception as e:
                    hata = str(e)
                    if not ilk_hata:
                        ilk_hata = hata
                    son_hata = hata

            if process is None:
                json_gonder(conn, {
                    "tur": "hata",
                    "is_id": is_id,
                    "format": output_format,
                    "mesaj": f"WinPMEM command could not be started: {ilk_hata}",
                    "kod": "WINPMEM_CMD_ERROR",
                })
                self.transfer_bilgi("RAM acquisition failed: WinPMEM command could not be executed")
                return

            self.log(f"Selected WinPMEM command: {' '.join(secilen_komut)}")
            json_gonder(conn, {"tur": "veri_basliyor", "is_id": is_id, "format": output_format, "toplam": toplam_ram})

            was_paused = False
            while process.poll() is None:
                state = self._get_job_state(is_id)
                if state == "paused":
                    if not was_paused:
                        try:
                            subprocess.run(
                                [
                                    "powershell",
                                    "-NoProfile",
                                    "-Command",
                                    f"Suspend-Process -Id {process.pid} -ErrorAction SilentlyContinue",
                                ],
                                check=False,
                                stdout=subprocess.DEVNULL,
                                stderr=subprocess.DEVNULL,
                            )
                        except Exception:
                            pass
                        was_paused = True
                    time.sleep(0.2)
                    continue

                if was_paused:
                    try:
                        subprocess.run(
                            [
                                "powershell",
                                "-NoProfile",
                                "-Command",
                                f"Resume-Process -Id {process.pid} -ErrorAction SilentlyContinue",
                            ],
                            check=False,
                            stdout=subprocess.DEVNULL,
                            stderr=subprocess.DEVNULL,
                        )
                    except Exception:
                        pass
                    was_paused = False

                if state == "stopped":
                    try:
                        process.terminate()
                    except Exception:
                        pass
                    break

                if os.path.exists(cikti_dosya) and toplam_ram > 0:
                    mevcut = os.path.getsize(cikti_dosya)
                    yuzde = int((mevcut * 100) / toplam_ram)
                    # Bazi WinPMEM surumleri dosyayi erken pre-allocate edebilir.
                    # Gercek tamamlanma sadece process basariyla bittiginde 100 olmali.
                    if yuzde >= 100:
                        yuzde = 99
                    json_gonder(conn, {
                        "tur": "ilerleme",
                        "is_id": is_id,
                        "format": output_format,
                        "okunan": mevcut,
                        "toplam": toplam_ram,
                        "yuzde": min(yuzde, 100),
                    })
                    self.transfer_bilgi(f"RAM acquisition: %{min(yuzde, 100)}")
                time.sleep(1)

            stdout_txt = (process.stdout.read() or b"").decode(errors="ignore") if process.stdout else ""
            stderr_txt = (process.stderr.read() or b"").decode(errors="ignore") if process.stderr else ""

            if self._get_job_state(is_id) == "stopped":
                partial = os.path.getsize(cikti_dosya) if os.path.exists(cikti_dosya) else 0
                json_gonder(conn, {
                    "tur": "hata",
                    "is_id": is_id,
                    "format": output_format,
                    "mesaj": f"RAM acquisition stopped by user | partial_size={partial}",
                    "kod": "STOPPED_BY_USER",
                })
                self.transfer_bilgi("RAM acquisition stopped by user")
                return

            if process.returncode == 0 and os.path.exists(cikti_dosya):
                sha256_hash = hashlib.sha256()
                with open(cikti_dosya, "rb") as f:
                    while True:
                        chunk = f.read(1024 * 1024)
                        if not chunk:
                            break
                        sha256_hash.update(chunk)

                tamamlandi_mesaj = "RAM acquisition completed"
                if output_format == "aff4":
                    tamamlandi_mesaj = "RAM acquisition completed; ready for AFF4 packaging"

                json_gonder(conn, {
                    "tur": "bitti",
                    "is_id": is_id,
                    "format": output_format,
                    "boyut": os.path.getsize(cikti_dosya),
                    "sha256": sha256_hash.hexdigest(),
                    "mesaj": tamamlandi_mesaj,
                })
                self.transfer_bilgi(tamamlandi_mesaj)
            else:
                detay = f"returncode={process.returncode}"
                if stderr_txt.strip():
                    detay += f" | stderr: {stderr_txt.strip()}"
                if stdout_txt.strip():
                    detay += f" | stdout: {stdout_txt.strip()}"
                json_gonder(conn, {
                    "tur": "hata",
                    "is_id": is_id,
                    "format": output_format,
                    "mesaj": f"WinPMEM error: {detay}",
                    "kod": "WINPMEM_ERROR",
                })
                self.transfer_bilgi(f"RAM acquisition failed: {detay}")
        except Exception as e:
            json_gonder(conn, {"tur": "hata", "is_id": is_id, "format": output_format, "mesaj": str(e), "kod": "EXCEPTION"})
            self.transfer_bilgi(f"RAM acquisition error: {e}")
        finally:
            self._clear_job_state(is_id)

    def _dosya_stream_gonder(self, conn, dosya_yolu, is_id, delete_after_success=False, index_key=""):
        try:
            if not os.path.exists(dosya_yolu):
                json_gonder(conn, {
                    "durum": "hata",
                    "is_id": is_id,
                    "mesaj": f"File not found: {dosya_yolu}",
                })
                self.log(f"RAM file download error ({is_id}): File not found: {dosya_yolu}")
                return

            toplam = os.path.getsize(dosya_yolu)
            self._set_job_state(is_id, "running")
            self.log(f"RAM file stream started ({is_id}): {dosya_yolu} ({toplam} bytes)")
            json_gonder(conn, {
                "durum": "ok",
                "is_id": is_id,
                "tahmini_boyut": toplam,
            })

            sha256 = hashlib.sha256()
            json_gonder(conn, {
                "tur": "veri_basliyor",
                "is_id": is_id,
                "toplam": toplam,
            })

            gonderilen = 0
            son_rapor = time.time()
            with open(dosya_yolu, "rb") as f:
                while True:
                    state = self._get_job_state(is_id)
                    if state == "paused":
                        time.sleep(0.2)
                        continue
                    if state == "stopped":
                        self.transfer_bilgi(f"RAM file transfer stopped by user ({is_id})")
                        self.log(f"RAM file stream stopped by user ({is_id})")
                        try:
                            conn.shutdown(socket.SHUT_RDWR)
                        except Exception:
                            pass
                        try:
                            conn.close()
                        except Exception:
                            pass
                        return

                    buf = f.read(1024 * 1024)
                    if not buf:
                        break
                    conn.sendall(buf)
                    sha256.update(buf)
                    gonderilen += len(buf)

                    simdi = time.time()
                    if toplam > 0 and (simdi - son_rapor >= 1 or gonderilen == toplam):
                        yuzde = int((gonderilen * 100) / toplam)
                        self.transfer_bilgi(
                            f"RAM file transfer ({is_id}): %{yuzde} | {gonderilen // (1024*1024)} MB / {toplam // (1024*1024)} MB"
                        )
                        son_rapor = simdi

            if gonderilen == toplam:
                json_gonder(conn, {
                    "tur": "bitti",
                    "is_id": is_id,
                    "sha256": sha256.hexdigest(),
                    "mesaj": "File transfer completed",
                })
                self.transfer_bilgi(f"RAM file transfer completed ({is_id})")
                self.log(f"RAM file stream completed ({is_id})")
                if delete_after_success:
                    self._cleanup_transferred_file(dosya_yolu, index_key)
            else:
                json_gonder(conn, {
                    "tur": "hata",
                    "is_id": is_id,
                    "mesaj": f"File transfer interrupted | partial_size={gonderilen}",
                })
                self.transfer_bilgi(f"RAM file transfer interrupted ({is_id})")
                self.log(f"RAM file stream interrupted ({is_id})")
        except Exception as e:
            json_gonder(conn, {
                "tur": "hata",
                "is_id": is_id,
                "mesaj": f"File transfer error: {e}",
            })
            self.log(f"RAM file stream error ({is_id}): {e}")
        finally:
            self._clear_job_state(is_id)

    def _docker_stream_acquisition(self, conn, container_id, acquire_diff, acquire_logs, acquire_config, job_id):
        self._set_job_state(job_id, "running")
        c_dir = os.path.join("C:\\ProgramData\\Docker\\containers", container_id)
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
            json_gonder(conn, {"durum": "hata", "tur": "hata", "is_id": job_id, "mesaj": f"Container directory / inspect not found for: {container_id}"})
            self._clear_job_state(job_id)
            return

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

            self._dosya_stream_gonder(conn, temp_tar_path, job_id, delete_after_success=True)

        except Exception as e:
            self.log(f"Docker acquisition error: {e}")
            json_gonder(conn, {"durum": "hata", "tur": "hata", "is_id": job_id, "mesaj": str(e)})
            try:
                if os.path.exists(temp_tar_path):
                    os.remove(temp_tar_path)
            except Exception:
                pass
        finally:
            self._clear_job_state(job_id)

    def _istemci_yonet(self, conn, addr):
        self.log(f"Connection: {addr}")
        yetkili = False

        def anahtar_coz_ve_karsilastir(mesaj):
            anahtar_b64 = mesaj.get("guvenlik_anahtar_b64")

            # Fail-closed: istemci anahtar gonderdiyse ajan tarafinda da anahtar zorunlu.
            if not self.security_key:
                if anahtar_b64:
                    return False, "Agent security key is not configured"
                return True, ""

            if not anahtar_b64:
                return False, "Security key was not provided"

            try:
                cozulmus = base64.b64decode(anahtar_b64, validate=True).decode("utf-8")
            except (binascii.Error, UnicodeDecodeError):
                return False, "Security key base64 is invalid"

            if cozulmus != self.security_key:
                return False, "Security key mismatch"

            return True, ""

        try:
            dosya = conn.makefile("rb")
            while True:
                data = dosya.readline()
                if not data:
                    return

                try:
                    mesaj = json.loads(data.decode("utf-8").strip())
                except json.JSONDecodeError:
                    json_gonder(conn, {"durum": "hata", "mesaj": "Invalid JSON"})
                    continue

                komut = mesaj.get("komut")

                if komut == "merhaba":
                    ok, hata = anahtar_coz_ve_karsilastir(mesaj)
                    if not ok:
                        json_gonder(conn, {
                            "durum": "hata",
                            "mesaj": hata,
                            "kod": "AUTH_FAILED",
                        })
                        self.log(f"Unauthorized connection rejected: {addr} | {hata}")
                        return

                    yetkili = True
                    self.log(f"Authorized connection accepted: {addr}")
                    json_gonder(conn, {
                        "durum": "ok",
                        "sunucu": "windows-ajan",
                        "surum": "0.4",
                        "ozellikler": ["disk_imaj", "winpmem_ram", "winpmem_otomatik_indirme"],
                    })

                elif not yetkili:
                    json_gonder(conn, {
                        "durum": "hata",
                        "mesaj": "Authorization required. Authenticate with hello first.",
                        "kod": "AUTH_REQUIRED",
                    })
                    continue

                elif komut == "disk_listele":
                    diskler, tani = disk_listele_tani()
                    self.log(f"disk_listele: {tani}")

                    if not WINDOWS:
                        json_gonder(conn, {
                            "durum": "hata",
                            "mesaj": "Agent is running without pywin32. pywin32 is missing in exe package.",
                            "tani": tani,
                        })
                    elif len(diskler) == 0:
                        json_gonder(conn, {
                            "durum": "hata",
                            "mesaj": "No disk found. Check administrator privileges and security software blocking.",
                            "tani": tani,
                        })
                    else:
                        json_gonder(conn, {"durum": "ok", "diskler": diskler, "tani": tani})

                elif komut == "imaj_baslat":
                    if not WINDOWS:
                        json_gonder(conn, {
                            "durum": "hata",
                            "mesaj": "Agent is running without pywin32. Disk imaging cannot be started.",
                        })
                        continue

                    disk_id = mesaj.get("disk_id", "0")
                    fmt, format_error = normalize_output_format(mesaj.get("format", "raw"))
                    if format_error:
                        json_gonder(conn, {"durum": "hata", "mesaj": format_error, "kod": "UNSUPPORTED_FORMAT"})
                        continue
                    parca = int(mesaj.get("parca_boyutu", 4 * 1024 * 1024))
                    is_id = mesaj.get("is_id") or ("IMG_" + str(int(time.time())))
                    self.log(f"Starting disk acquisition for {disk_id} in {fmt} format")
                    self._imaj_gonder(conn, disk_id, parca, is_id, fmt)

                elif komut == "winpmem_kontrol":
                    mevcut, yol, durum = self.winpmem_hazirla(auto_download=True)
                    json_gonder(conn, {
                        "durum": "ok",
                        "winpmem_mevcut": mevcut,
                        "winpmem_yol": yol,
                        "yonetici_yetkisi": yonetici_yetkisi_kontrol(),
                        "ram_boyut": ram_boyut_al(),
                        "mesaj": durum,
                    })

                elif komut == "winpmem_indir":
                    ok, yol, durum = winpmem_indir(
                        self.script_dir,
                        log_cb=self.log,
                        progress_cb=self.transfer_bilgi,
                    )
                    if ok:
                        self.winpmem_path = yol
                    json_gonder(conn, {
                        "durum": "ok" if ok else "hata",
                        "winpmem_mevcut": ok,
                        "winpmem_yol": yol,
                        "mesaj": durum,
                    })

                elif komut == "ram_edinim_baslat":
                    is_id = mesaj.get("is_id") or ("RAM_" + str(int(time.time())))
                    fmt, format_error = normalize_output_format(mesaj.get("format", "raw"))
                    if format_error:
                        json_gonder(conn, {"durum": "hata", "is_id": is_id, "mesaj": format_error, "kod": "UNSUPPORTED_FORMAT"})
                        continue
                    cikti_dosya = os.path.basename(mesaj.get("cikti_dosya", "memory_dump.raw"))
                    hedef = os.path.join(self.script_dir, cikti_dosya)
                    self.log(f"Starting RAM acquisition for {cikti_dosya} in {fmt} format")
                    self.ram_output_index[cikti_dosya] = hedef
                    self._ram_edinim_baslat(conn, hedef, is_id, fmt)

                elif komut == "ram_dosya_indir":
                    is_id = mesaj.get("is_id") or ("RAMDL_" + str(int(time.time())))
                    dosya = mesaj.get("dosya", "memory_dump.raw")
                    # Guvenlik: sadece dosya adi kabul et, dizin gecisine izin verme.
                    dosya = os.path.basename(dosya)
                    hedef = self.ram_output_index.get(dosya, os.path.join(self.script_dir, dosya))
                    self._dosya_stream_gonder(conn, hedef, is_id, delete_after_success=True, index_key=dosya)

                elif komut == "edinim_kontrol":
                    is_id = mesaj.get("is_id", "")
                    eylem = mesaj.get("eylem", "") or mesaj.get("action", "")
                    ok, msg = self._control_job(is_id, eylem)
                    json_gonder(conn, {
                        "durum": "ok" if ok else "hata",
                        "is_id": is_id,
                        "eylem": eylem,
                        "mesaj": msg,
                    })

                elif komut in {
                    "hyperv_varlik_kontrol",
                    "hyperv_vm_listele",
                    "hyperv_bellek_listele",
                    "hyperv_dosya_indir",
                }:
                    json_gonder(conn, {
                        "durum": "hata",
                        "mesaj": "Hyper-V support has been removed. Use WinPMEM.",
                    })

                elif komut in {"docker_durum", "docker_status"}:
                    json_gonder(conn, docker_get_status())

                elif komut in {"docker_listele", "docker_list"}:
                    containers = docker_list_containers()
                    json_gonder(conn, {
                        "durum": "ok",
                        "konteynerler": containers,
                        "containers": containers,
                    })

                elif komut in {"docker_loglar", "docker_logs"}:
                    c_id = mesaj.get("konteyner_id") or mesaj.get("container_id") or ""
                    tail = int(mesaj.get("tail") or 200)
                    logs = docker_get_logs(c_id, tail)
                    json_gonder(conn, {
                        "durum": "ok",
                        "loglar": logs,
                        "logs": logs,
                    })

                elif komut in {"docker_edinim", "docker_acquire"}:
                    c_id = mesaj.get("konteyner_id") or mesaj.get("container_id") or ""
                    is_id = mesaj.get("is_id") or ("DOCKER_" + str(int(time.time())))
                    acq_diff = bool(mesaj.get("acquire_diff", True))
                    acq_logs = bool(mesaj.get("acquire_logs", True))
                    acq_cfg = bool(mesaj.get("acquire_config", True))
                    self._docker_stream_acquisition(conn, c_id, acq_diff, acq_logs, acq_cfg, is_id)

                else:
                    json_gonder(conn, {"durum": "hata", "mesaj": f"Unknown command: {komut}"})

        except Exception as e:
            self.log(f"Client error: {e}")
        finally:
            try:
                conn.close()
            except Exception:
                pass


class AgentUI:
    def __init__(self):
        self.controller = AgentController(ui=self)
        self.root = tk.Tk()
        self.root.title(f"Amele Windows Agent v{VERSION}")
        self.root.geometry("760x520")

        self.log_queue = queue.Queue()

        self.port_var = tk.StringVar(value=str(DEFAULT_PORT))
        self.key_var = tk.StringVar(value="")
        self.lang_var = tk.StringVar(value="tr")
        self.key_status_var = tk.StringVar(value="Key: Disabled")
        self.status_var = tk.StringVar(value="Ready")
        self.transfer_var = tk.StringVar(value="Transfer info appears here")
        self.winpmem_var = tk.StringVar(value="WinPMEM status: not checked")
        self.active_key = ""

        self._build()
        self.dil_degistir()
        self._poll_log_queue()

    def _build(self):
        frm = ttk.Frame(self.root, padding=10)
        frm.pack(fill="both", expand=True)

        ust = ttk.LabelFrame(frm, text="Server")
        ust.pack(fill="x", pady=4)

        ttk.Label(ust, text="Port:").grid(row=0, column=0, padx=6, pady=6, sticky="w")
        self.port_entry = ttk.Entry(ust, textvariable=self.port_var, width=12)
        self.port_entry.grid(row=0, column=1, padx=6, pady=6, sticky="w")

        ttk.Label(ust, text="Security Key:").grid(row=1, column=0, padx=6, pady=6, sticky="w")
        self.key_entry = ttk.Entry(ust, textvariable=self.key_var, width=24, show="*")
        self.key_entry.grid(row=1, column=1, padx=6, pady=6, sticky="w")
        self.key_onay_btn = ttk.Button(ust, text="Approve", command=self.anahtar_onayla)
        self.key_onay_btn.grid(row=1, column=2, padx=6, pady=6)
        self.key_sifirla_btn = ttk.Button(ust, text="Reset", command=self.anahtar_sifirla)
        self.key_sifirla_btn.grid(row=1, column=3, padx=6, pady=6)
        ttk.Label(ust, textvariable=self.key_status_var).grid(row=1, column=4, padx=8, pady=6, sticky="w")

        ttk.Label(ust, text="Dil / Language:").grid(row=2, column=0, padx=6, pady=6, sticky="w")
        self.lang_combo = ttk.Combobox(ust, textvariable=self.lang_var, width=10, state="readonly")
        self.lang_combo["values"] = ("tr", "en")
        self.lang_combo.grid(row=2, column=1, padx=6, pady=6, sticky="w")
        self.lang_combo.bind("<<ComboboxSelected>>", lambda _e: self.dil_degistir())

        self.start_btn = ttk.Button(ust, text="Start", command=self.server_baslat)
        self.start_btn.grid(row=0, column=2, padx=6, pady=6)
        self.stop_btn = ttk.Button(ust, text="Stop", command=self.server_durdur)
        self.stop_btn.grid(row=0, column=3, padx=6, pady=6)

        ttk.Label(ust, textvariable=self.status_var).grid(row=0, column=4, padx=8, pady=6, sticky="w")

        winpmem = ttk.LabelFrame(frm, text="WinPMEM")
        winpmem.pack(fill="x", pady=4)

        ttk.Button(winpmem, text="Check", command=self.winpmem_kontrol_et).grid(row=0, column=0, padx=6, pady=6)
        ttk.Button(winpmem, text="Download", command=self.winpmem_indir).grid(row=0, column=1, padx=6, pady=6)
        ttk.Label(winpmem, textvariable=self.winpmem_var).grid(row=0, column=2, padx=8, pady=6, sticky="w")

        transfer = ttk.LabelFrame(frm, text="Transfer")
        transfer.pack(fill="x", pady=4)
        ttk.Label(transfer, textvariable=self.transfer_var).pack(fill="x", padx=8, pady=8)

        log_box = ttk.LabelFrame(frm, text="Log")
        log_box.pack(fill="both", expand=True, pady=4)
        self.log_text = tk.Text(log_box, height=16, wrap="word")
        self.log_text.pack(fill="both", expand=True, padx=6, pady=6)

        self.root.protocol("WM_DELETE_WINDOW", self.kapat)

    def _poll_log_queue(self):
        try:
            while True:
                line = self.log_queue.get_nowait()
                self.log_text.insert("end", line + "\n")
                self.log_text.see("end")
        except queue.Empty:
            pass
        self.root.after(150, self._poll_log_queue)

    def log(self, line):
        self.log_queue.put(line)

    def _cevir_widget_metinleri(self, kok):
        for cocuk in kok.winfo_children():
            try:
                metin = cocuk.cget("text")
                if isinstance(metin, str) and metin:
                    cocuk.configure(text=self.controller.cevir(metin))
            except Exception:
                pass
            self._cevir_widget_metinleri(cocuk)

    def set_transfer(self, msg):
        self.transfer_var.set(msg)
        self.log(f"[TRANSFER] {msg}")

    def dil_degistir(self):
        self.controller.language = self.lang_var.get().strip() or "tr"
        self.root.title(self.controller.cevir("Amele Windows Agent"))
        self._cevir_widget_metinleri(self.root)
        self.key_status_var.set(self.controller.cevir(self.key_status_var.get()))
        self.status_var.set(self.controller.cevir(self.status_var.get()))
        self.transfer_var.set(self.controller.cevir(self.transfer_var.get()))
        self.winpmem_var.set(self.controller.cevir(self.winpmem_var.get()))

    def anahtar_onayla(self):
        key = self.key_var.get().strip()
        if not key:
            messagebox.showwarning(self.controller.cevir("Warning"), self.controller.cevir("Enter a security key to approve"))
            return

        self.active_key = key
        self.controller.security_key = key
        self.key_entry.configure(state="disabled")
        self.key_status_var.set(self.controller.cevir("Key: Active"))
        self.log(self.controller.cevir("[SECURITY] Key approved"))

    def anahtar_sifirla(self):
        self.active_key = ""
        self.controller.security_key = ""
        self.key_var.set("")
        self.key_entry.configure(state="normal")
        self.key_status_var.set(self.controller.cevir("Key: Disabled"))
        self.log(self.controller.cevir("[SECURITY] Key reset"))

    def server_baslat(self):
        try:
            port = int(self.port_var.get().strip())
            if port <= 0 or port > 65535:
                raise ValueError(self.controller.cevir("Invalid port"))
        except Exception:
            messagebox.showerror(self.controller.cevir("Error"), self.controller.cevir("Port must be between 1 and 65535"))
            return

        try:
            self.controller.language = self.lang_var.get().strip() or "tr"
            self.controller.security_key = self.active_key
            ok, msg = self.controller.start_server(port)
            if ok:
                self.status_var.set(self.controller.cevir(f"Running ({HOST}:{port})"))
                if self.controller.security_key:
                    self.log(self.controller.cevir("[SECURITY] Key verification enabled"))
                else:
                    self.log(self.controller.cevir("[SECURITY] Key verification disabled"))
            else:
                self.status_var.set(self.controller.cevir(msg))
                messagebox.showwarning(self.controller.cevir("Info"), self.controller.cevir(msg))
        except Exception as e:
            self.status_var.set(self.controller.cevir("Startup error"))
            messagebox.showerror(self.controller.cevir("Error"), self.controller.cevir(str(e)))

    def server_durdur(self):
        self.controller.stop_server()
        self.status_var.set(self.controller.cevir("Stopped"))

    def winpmem_kontrol_et(self):
        ok, yol, mesaj = self.controller.winpmem_hazirla(auto_download=True)
        if ok:
            self.winpmem_var.set(self.controller.cevir(f"WinPMEM ready: {yol}"))
        else:
            self.winpmem_var.set(self.controller.cevir(f"WinPMEM unavailable: {mesaj}"))

    def winpmem_indir(self):
        def worker():
            ok, yol, mesaj = winpmem_indir(
                self.controller.script_dir,
                log_cb=self.controller.log,
                progress_cb=self.set_transfer,
            )
            if ok:
                self.controller.winpmem_path = yol
                self.winpmem_var.set(self.controller.cevir(f"WinPMEM downloaded: {yol}"))
            else:
                self.winpmem_var.set(self.controller.cevir(f"Download failed: {mesaj}"))

        threading.Thread(target=worker, daemon=True).start()

    def kapat(self):
        self.controller.stop_server()
        self.root.destroy()

    def run(self):
        self.root.mainloop()


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(
        description=f"{ASCII_LOGO}\n\nAmele Windows Agent v{VERSION}",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--port", type=int, default=DEFAULT_PORT, help="Port to listen on")
    parser.add_argument("--key", type=str, default="", help="Security key / password")
    parser.add_argument("--cli", "--headless", "--non-interactive", action="store_true", help="Start in CLI / headless mode without GUI")
    parser.add_argument("--no-logo", "-q", "--quiet", action="store_true", help="Suppress ASCII logo")
    args, _ = parser.parse_known_args()

    show_logo = not args.no_logo
    if show_logo:
        print(f"{ASCII_LOGO}\n")
        print(f"Amele Windows Agent v{VERSION}\n")

    if getattr(args, "cli", False) or not HAS_TK:
        if not HAS_TK and not getattr(args, "cli", False):
            print("Tkinter not available. Starting in headless CLI mode...")
        controller = AgentController()
        controller.security_key = args.key
        ok, msg = controller.start_server(args.port)
        if not ok:
            print(f"Error starting server: {msg}")
            sys.exit(1)
        print(f"Server started on {HOST}:{args.port}. Press Ctrl+C to stop.")
        try:
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            controller.stop_server()
    else:
        ui = AgentUI()
        ui.run()
