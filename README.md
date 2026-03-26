# MateriaForge

> All-in-one installer for 7th Heaven and Junction VIII on Linux. 

MateriaForge is the successor to [7thDeck](https://github.com/dotaxis/7thDeck) and [8thDeck](https://github.com/dotaxis/8thDeck), automating the installation and setup of [7th Heaven](https://github.com/tsunamods-codes/7th-Heaven) and [Junction VIII](https://github.com/tsunamods-codes/Junction-VIII) on Linux — now wrapped up nicely in a single application!

---

## Features

- **Automatic 7th Heaven installation**: downloads, configures, and launches 7th Heaven with no manual setup
- **Multi-platform game detection**: supports multiple storefronts out of the box
- **Junction VIII support**: *(planned)* FF8 mod loader support in active development
- **Written in Rust**: fast, reliable, and expandable

---

## Supported Mod Loaders

| Mod Loader | Game | Steam | GOG (Heroic) | GOG (Lutris) |
|------------|------|-------|--------------|--------------|
| **7th Heaven** | Final Fantasy VII (2026) | ✅ | ✅ | 🔜 |
| |Final Fantasy VII (2013) | ✅ | ➖ | ➖ |
| **Junction VIII** | Final Fantasy VIII | 🔜 | ➖ | ➖ |
---

## Installation

Pre-built binaries are available on the [Releases](https://github.com/dotaxis/MateriaForge-rs/releases) page.

1. Ensure you have installed FF7 and opened it to the launcher at least once.

2. Download the latest release, unzip, and run it:

```bash
./MateriaForge
```

3. On first launch of 7th Heaven, click **Save**. Do **NOT** click Reset Defaults.

---

## Options

| Flag | Description |
|------|-------------|
| `-c`, `--canary` | Install pre-release (canary) versions of 7th Heaven and FFNx |
| `-d`, `--deck` | Force detection of Steam Deck for controller config option |

---

## Configuration (TOML)

MateriaForge generates a `MateriaForge.toml` file in the 7th Heaven installation folder. This file is created automatically during setup, but you can edit it manually to customize behavior.

### Example

```toml
app_id = "3837340"
type = "steam"
steam_dir = "/home/user/.steam/root"
runner = "proton_9"
launch_args = "/launch /quit"

[env]
WINEDEBUG = "+err,+warn,+debugstr"
PROTON_LOG = "1"
MANGOHUD = "1"
```

### Keys

| Key | Description | Default | Required |
|-----|-------------|---------|----------|
| `type` | Game install type: `steam` or `gog` | `steam` | Yes |
| `app_id` | The game's app ID (Steam or GOG) | *(set during install)* | Yes |
| `steam_dir` | Path to Steam installation directory | *(set during install)* | Only for `type = "steam"` |
| `runner` | Proton version override | *(set during install)* | No |
| `launch_args` | Extra arguments passed to 7th Heaven on launch | *(none)* | No |

### Environment Variables

The `[env]` table lets you set environment variables that are passed to the game runner:

```toml
[env]
WINEDEBUG = "+err,+warn,+debugstr"
```

Any key/value pair under `[env]` will be set as an environment variable when launching the game. `WINEDEBUG` is included by default.

> **Note:** CLI arguments passed directly to the launcher take priority over `launch_args` in the TOML.

---

## Building from Source

You'll need a recent stable [Rust toolchain](https://rustup.rs/).

```bash
git clone https://github.com/dotaxis/MateriaForge-rs
cd MateriaForge-rs
make release
```

The compiled binary will be at `target/release/MateriaForge`.

---

## Related Projects

- [7thDeck](https://github.com/dotaxis/7thDeck) — the 7th Heaven Linux installer this project succeeds
- [8thDeck](https://github.com/dotaxis/8thDeck) — the Junction VIII Linux installer this project succeeds (coming soon)
- [7th Heaven](https://github.com/tsunamods-codes/7th-Heaven) — the FF7 mod loader MateriaForge installs
- [Junction VIII](https://github.com/tsunamods-codes/Junction-VIII) — the FF8 mod loader MateriaForge installs (coming soon)
- [Heroic Games Launcher](https://heroicgameslauncher.com/) — GOG & Epic launcher for Linux

---

## Contributing & Support

- Issues and pull requests are welcome. If you run into problems with a specific game version or platform setup, please open an issue with your distro and `MateriaForge.log`.
- You can find me at the [Tsunamods Discord](https://discord.gg/tsunamods-community-277610501721030656) in the #ff7-linux and #ff8-linux channels for quick questions.

---

## Donate

☕ You can [buy me a coffee on Ko-fi](https://ko-fi.com/dotaxis) if you appreciate my work!
