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
