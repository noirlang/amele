<div align="center">

<img src="../ui/assets/logo/logo.png" alt="Amele Logo" width="120" />

# Amele Forensic Agents

*Remote forensic acquisition agents for Linux and Windows targets.*

[Back to Main Repo](../README.md) | [Website](https://amele.noirlang.tr) | [License (EULA)](../LICENSE)

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

Amele Forensic Agents are lightweight standalone services that enable remote disk imaging, memory acquisition, and container forensics on Linux and Windows targets. They communicate securely with the main Amele desktop application over TCP with token-based authentication.

## Features

- **Remote disk acquisition:** collect disk images from remote targets with live throughput and hashing
- **Remote memory acquisition:** capture RAM dumps via AVML (Linux) or WinPMEM (Windows)
- **Container forensics:** audit Docker containers, extract logs, configs, and drift layers
- **Token-based authentication:** secure communication with the main Amele application
- **Cross-platform:** Linux and Windows standalone binary support
- **Zero runtime dependencies:** pre-compiled self-contained binaries

## License

Copyright (c) 2026 noirLang. All rights reserved.

Distributed under the noirLang Proprietary EULA. See [LICENSE](../LICENSE) or visit [https://amele.noirlang.tr/license](https://amele.noirlang.tr/license) for full terms.
