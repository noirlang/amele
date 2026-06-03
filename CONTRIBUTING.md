<div align="center">

<img src="ui/assets/logo/logo.png" alt="Worm Logo" width="96" />

# Contributing

*How to support Worm Forensic Tool.*

[Repository](https://github.com/noirlang/worm) | [Website](https://worm.noirlang.tr) | [Issues](https://github.com/noirlang/worm/issues) | [Security](SECURITY.md)

</div>

## How to Contribute

Worm is a forensic acquisition tool, so every contribution should be clear, reproducible, and safe to test.

- Report bugs with the operating system, architecture, command or screen used, expected result, actual result, and logs.
- Use small test images or reproduction steps instead of sharing sensitive forensic data.
- Keep changes focused on one behavior, screen, platform flow, or documentation topic.
- Explain user-visible behavior changes in the pull request description.
- Add screenshots for UI changes and command output for CLI/build changes.

## Pull Requests

Before opening a pull request, run the relevant checks:

```bash
cargo fmt --all -- --check
cargo test --locked
node --check ui/app.js
```

For platform-specific work, also test the affected flow:

- Linux disk or RAM acquisition
- Windows disk or RAM acquisition
- Remote agent connection and job control
- Android ADB, logical, filesystem, volatile, or analysis flow
- Image viewing, hashing, reporting, or update checks

## Security Reports

Do not publish real case data, memory dumps, disk images, tokens, IP addresses, or private logs in issues or pull requests.

For suspected vulnerabilities, follow [SECURITY.md](SECURITY.md).
