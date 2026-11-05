<p align="center">English | <a href="README.zh-CN.md">中文</a></p>

<div align="center">
  <img src="crates/yo/resources/app-icon.png" alt="Yo" width="128">
  <h1>Yo</h1>
  <p>The fast, native code editor and agent workspace.</p>
  <p>Written in Rust. Batteries included.</p>
  <p><a href="#download"><strong>Download</strong></a></p>
</div>

---

Yo is a fast, native code editor written in Rust, with a clean, modern design and coding agents built in. The name comes from [Wu Wei](https://en.wikipedia.org/wiki/Wu_wei): effortless action. Yo is a fork of [Zed](https://github.com/zed-industries/zed) but diverges heavily from it.

## Features

- ⚡ **Native and fast.** No Electron or webviews. Opens instantly and stays responsive on large views.
- 🍃 **Lightweight.** Yo's memory footprint is lower than VS Code and Zed. See [benchmarks](https://github.com/magalab/yo/blob/main/docs/memory-benchmark.md).
- 👾 **Coding agents included.** Built-in support for Claude Code, Codex, and OpenCode.
- ✨ **Beautiful by default.** Clean, modern look with a delightful UX.
- 🤫 **No account or telemetry.** Nothing to sign in to. Yo never sends your usage data anywhere.

## Download

On macOS and Linux, run:

```sh
curl -fsSL https://yo.lurenyang.com/install.sh | sh
```

For Windows, download the [installer](https://github.com/magalab/yo/releases/latest/download/Yo-x86_64.exe).

The terminal command is `yo <path>`. The previous `yo` command remains available as an alias.

<details>
<summary>Or install manually</summary>

- [macOS (Apple Silicon)](https://github.com/magalab/yo/releases/latest/download/Yo-aarch64.dmg)
- [macOS (Intel)](https://github.com/magalab/yo/releases/latest/download/Yo-x86_64.dmg)
- [Windows x86-64](https://github.com/magalab/yo/releases/latest/download/Yo-x86_64.exe)
- [Linux x86-64](https://github.com/magalab/yo/releases/latest/download/yo-linux-x86_64.tar.gz)
- [Linux AArch64](https://github.com/magalab/yo/releases/latest/download/yo-linux-aarch64.tar.gz)

See [INSTALL.md](https://github.com/magalab/yo/blob/main/docs/INSTALL.md) for more.
</details>

---

![Yo screenshot](https://yo.lurenyang.com/app-preview.png)

## Docs

See [docs](https://yo.lurenyang.com/docs/).

## Building

Yo builds the same way Zed does. See the [Zed development docs](https://zed.dev/docs/development).

## Licensing

Yo is licensed under GPL-3.0-or-later with Apache-2.0 components where marked. See [LICENSE-GPL](./LICENSE-GPL) and [LICENSE-APACHE](./LICENSE-APACHE).

Yo is a derivative work of [Zed](https://github.com/zed-industries/zed) and shares the same licenses.

## Acknowledgements

Thanks to the Zed team for building an excellent editor and releasing it as open source. Yo would not exist without their work.
