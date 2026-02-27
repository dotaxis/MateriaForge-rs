# MateriaForge

> All-in-one installer for Final Fantasy mod loaders on Linux.

MateriaForge is the successor to [7thDeck](https://github.com/dotaxis/7thDeck), automating the installation and setup of [7th Heaven](https://github.com/tsunamods-codes/7th-Heaven) on Linux — no manual Wine wrangling required.

---

## Features

- **Automatic 7th Heaven installation**: downloads, configures, and launches 7th Heaven with no manual setup
- **Multi-platform game detection**: supports multiple storefronts out of the box
- **Junction VIII support**: *(planned)* FF8 mod loader support in active development
- **Written in Rust**: fast, reliable, and expandable

---

## Supported Mod Loaders

### 7th Heaven

| Game | Steam | GOG (Heroic) | GOG (Lutris) |
|------|-------|------------------|----------|
| Final Fantasy VII (2013) | ✅ | ➖ | ➖   |
| Final Fantasy VII (2026) | ✅ | ✅ | 🔜   |

---

## Installation

Pre-built binaries are available on the [Releases](https://github.com/dotaxis/MateriaForge-rs/releases) page.

Download the latest release, make it executable, and run it:

```bash
chmod +x MateriaForge
./MateriaForge
```

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

## Roadmap

- [x] FF7 2013 (Steam)
- [x] FF7 2026 (Steam)
- [x] FF7 2026 (GOG via Heroic Games Launcher)
- [x] FF7 2026 (GOG via Lutris)
- [ ] Junction VIII support

---

## Related Projects

- [7thDeck](https://github.com/dotaxis/7thdeck) — the predecessor this project succeeds
- [7th Heaven](https://github.com/tsunamods-codes/7th-Heaven) — the FF7 mod loader MateriaForge installs
- [Junction VIII](https://github.com/tsunamods-codes/Junction-VIII) — the FF8 mod loader MateriaForge installs (soon)
- [Heroic Games Launcher](https://heroicgameslauncher.com/) — GOG & Epic launcher for Linux

---

## Contributing

Issues and pull requests are welcome. If you run into problems with a specific game version or platform setup, please open an issue with your distro and `MateriaForge.log`.
