# iOS Forensic Module

### Overview

The Amele iOS module normalizes iTunes/Finder-style iOS backup folders into a
browsable file-system tree using the `Manifest.db` index. The semantics match
Backup2FS: flat hashed backup files are rebuilt by domain and relative path.

The integration runs in Amele's Rust/native layer, using the same code path on
Windows and Linux. It does not require an external `.exe`, WPF UI, Wine, or the
dotnet runtime.

### Supported Backup Types

| Type | Status | Description |
|------|--------|-------------|
| Unencrypted iTunes/Finder backup | Active | `Manifest.db` is read directly and files are normalized. |
| Already decrypted backup | Active | A folder decrypted by Backup2FS or an equivalent tool can be processed. |
| Encrypted iOS backup | Guarded warning | Amele does not modify or decrypt the source backup. Disable backup encryption in iTunes/Finder and create a new unencrypted backup, or select a folder where Manifest.db and the files are already decrypted. |

### Output Layout

```
~/Amele/Vakalar/{case}/ios/{backup_name}_{date}/
├── ios_manifest.json
├── ios_manifest.json.sha256
├── extraction_log_{date}.csv
└── private/
    └── var/
        ├── mobile/
        ├── Keychains/
        ├── db/
        └── ...
```

### CSV Log

`extraction_log_*.csv` writes one row per `Manifest.db` entry:

```text
Timestamp,Status,Domain,RelativePath,FileID,OutputPath,SizeBytes,MD5,SHA1,SHA256
```

`Status` values:

- `Copied`: File copied and selected hashes calculated.
- `Directory`: Directory entry created.
- `Symlink`: Symlink entry processed; target metadata is written to a `*.symlink.txt` file. On Linux, Amele also creates a native symlink when the target can be safely mapped inside the case output tree.
- `Missing`: Hashed source file was missing from the backup.
- `Error`: Entry processing failed.

### Usage

1. Open **iOS Tools** from the sidebar.
2. Select the iTunes/Finder backup folder containing `Manifest.db`.
3. Use **Read Backup Profile** to inspect device and backup status.
4. Select or create a case.
5. Choose MD5/SHA1/SHA256 hash options.
6. Start **Normalize Backup**.

Pause/resume/stop controls and live logs are available during processing.

### CLI

```bash
amele ios-backup-profile /path/to/ios-backup
amele ios-backup-normalize /path/to/ios-backup Case_001
```
