<p align="center">English | <a href="README.zh-CN.md">中文</a></p>

<div align="center">
  <img src="crates/yo/resources/app-icon.png" alt="Yo" width="128">
  <h1>Yo</h1>
</div>

---

Yo is a fast, native code editor written in Rust and built on the open-source [Zed](https://github.com/zed-industries/zed) editor. It adds an agent workspace, lightweight Vim editing, and a customizable activity bar.

## Features

- ⚡ **Native and fast.** Written in Rust with no Electron or webviews, and designed to stay responsive on large projects.
- 👾 **Agent workspace.** Chat with Claude Code, Codex, and OpenCode through the command-line tools you already use.
- ⌨️ **Lightweight Vim mode.** Optional support for common Vim motions, operators, text objects, and Visual mode.
- 🧭 **Flexible workspace.** Move the activity bar between the sides and top, adjust its icon size, and use Zed extensions.
- 🔒 **No separate account, usage telemetry, or crash reporting.** Agent chats use the provider accounts configured in your local CLI tools. Extensions are available from the Zed extension registry.

## Download

On macOS and Linux, run:

```sh
curl -fsSL https://raw.githubusercontent.com/magalab/yo/develop/script/install.sh | sh
```

For Windows, download the [installer](https://github.com/magalab/yo/releases/latest/download/Yo-x86_64.exe).

<details>
<summary>Or install manually</summary>

- [macOS (Apple Silicon)](https://github.com/magalab/yo/releases/latest/download/Yo-aarch64.dmg)
- [macOS (Intel)](https://github.com/magalab/yo/releases/latest/download/Yo-x86_64.dmg)
- [Windows x86-64](https://github.com/magalab/yo/releases/latest/download/Yo-x86_64.exe)
- [Linux x86-64](https://github.com/magalab/yo/releases/latest/download/yo-linux-x86_64.tar.gz)
- [Linux AArch64](https://github.com/magalab/yo/releases/latest/download/yo-linux-aarch64.tar.gz)

See [INSTALL.md](https://github.com/magalab/yo/blob/develop/docs/INSTALL.md) for more.
</details>

---

## Docs

See [docs](https://yo.lurenyang.com/docs/).

## Building

Yo follows Zed's Rust build system. See the [Zed development docs](https://zed.dev/docs/development) for build prerequisites and workflows.

## Licensing

Yo is licensed under GPL-3.0-or-later with Apache-2.0 components where marked. See [LICENSE-GPL](./LICENSE-GPL) and [LICENSE-APACHE](./LICENSE-APACHE).

## Acknowledgements

<a href="https://github.com/zed-industries/zed"><img src="https://zed.dev/logo_icon.webp" alt="Zed" width="48"></a>
