<div align="center">

<img src="ui/assets/logo/logo.png" alt="Worm Logo" width="96" />

# Security

*Responsible security reporting for Worm Forensic Tool.*

[Repository](https://github.com/noirlang/worm) | [Website](https://worm.noirlang.tr) | [Contributing](CONTRIBUTING.md)

</div>

## Supported Versions

Security fixes are handled for the latest public release and the active development branch.

Older builds should be upgraded before reporting a defect unless the issue is needed for compatibility testing.

## Reporting a Vulnerability

Do not open a public issue for suspected security vulnerabilities.

Send the report privately to the maintainer or use GitHub private vulnerability reporting if it is enabled for the repository.

Include only the information needed to reproduce the issue:

- affected version or commit
- operating system and architecture
- affected flow, such as local disk, remote agent, RAM, Android, update, or report generation
- reproduction steps
- expected result and actual result
- relevant logs with sensitive data removed

## Sensitive Data

Never attach real case data, disk images, memory dumps, Android exports, access tokens, IP addresses, passwords, or private logs to a public issue or pull request.

Use synthetic test images, redacted logs, or a minimal reproduction whenever possible.

## Scope

Security reports should focus on issues that could affect evidence integrity, privilege handling, update safety, agent communication, file output paths, or exposure of sensitive data.
