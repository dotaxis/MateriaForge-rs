# MateriaForge

> All-in-one installer for Final Fantasy game series on Linux.

MateriaForge is the successor to [7thDeck](https://github.com/dotaxis/7thDeck) and [8thDeck](https://github.com/dotaxis/8thDeck), automating the installation and setup of various Final Fantasy mod managers on Linux — now wrapped up nicely in a single application!

---

## Features

- **7th Heaven installation**: downloads, configures, and launches 7th Heaven with no manual setup
- **Junction VIII installation**: downloads, configures, and launches Junction VIII for Final Fantasy VIII and Final Fantasy VIII Remastered with no manual setup
- **Multi-platform game detection**: supports multiple storefronts out of the box
- **Written in Rust**: fast, reliable, and expandable

---

## Supported Mod Loaders

| Mod Loader | Game | Steam | GOG (Heroic) | GOG (Lutris) | [Discord Support](https://discord.gg/tsunamods-community-277610501721030656) channel |
|------------|------|-------|--------------|--------------|-------------------------|
| **7th Heaven** | Final Fantasy VII (2026) | ✅ | ✅ | 🔜 | #ff7-linux |
| |Final Fantasy VII (2013) | ✅ | N/A | N/A | #ff7-linux |
| **Junction VIII** | Final Fantasy VIII Remastered | ✅ | ✅ | N/A | #ff8-linux |
| | Final Fantasy VIII (2013) | ✅ | N/A | N/A | #ff8-linux |
---

## Installation

Pre-built binaries are available on the [Releases](https://github.com/dotaxis/MateriaForge-rs/releases) page.

1. Ensure you have installed any of the supported games and clicked "Play" in the game's launcher at least once. You can close the game once you get to the main menu.

2. Download the latest release, unzip, and run it in a terminal:

```bash
./MateriaForge
```

> **IMPORTANT:** On first launch of 7th Heaven or Junction VIII, click **Save**. Do **NOT** click Reset Defaults.

---

## Options

| Flag | Description |
|------|-------------|
| `-c`, `--canary` | Install pre-release (canary) versions of the chosen mod loader |
| `-d`, `--deck` | Force detection of Steam Deck for controller config option |

---

## Configuration (TOML)

MateriaForge generates a `MateriaForge.toml` file in the chosen mod loader installation folder. This file is created automatically during setup, but you can edit it manually to customize behavior.

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
| `runner` | Proton version override | *(none)* | No |
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
- [8thDeck](https://github.com/dotaxis/8thDeck) — the Junction VIII Linux installer this project succeeds
- [7th Heaven](https://github.com/tsunamods-codes/7th-Heaven) — the FF7 mod loader MateriaForge installs
- [Junction VIII](https://github.com/tsunamods-codes/Junction-VIII) — the FF8 mod loader MateriaForge installs
- [Heroic Games Launcher](https://heroicgameslauncher.com/) — GOG & Epic launcher for Linux
- [lib_game_detector](https://github.com/Rolv-Apneseth/lib_game_detector) — Rust crate used for detecting installed GOG games
- [steamlocate](https://github.com/WilliamVenner/steamlocate-rs) — Rust crate used for parsing Steam libraries

---

## Contributing & Support

- Issues and pull requests are welcome. If you run into problems with a specific game version or platform setup, please open an issue with your distro and `MateriaForge.log`.
- You can find me at the [Tsunamods Discord](https://discord.gg/tsunamods-community-277610501721030656) for quick questions.

---

## Donate

☕ You can [buy me a coffee on Ko-fi](https://ko-fi.com/dotaxis) if you appreciate my work!
