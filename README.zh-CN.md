<p align="center"><a href="README.md">English</a> | 中文</p>

<div align="center">
  <img src="crates/yo/resources/app-icon.png" alt="Yo" width="128">
  <h1>Yo</h1>
</div>

---

Yo 是一款用 Rust 编写的快速原生代码编辑器，基于开源的 [Zed](https://github.com/zed-industries/zed) 编辑器，并加入了智能体工作区、轻量 Vim 编辑模式和可自定义的活动栏。

## 功能

- ⚡ **原生、快速。** 用 Rust 编写，不依赖 Electron 或网页视图，面向大型项目保持响应流畅。
- 👾 **智能体工作区。** 通过你日常使用的命令行工具，在编辑器中使用 Claude Code、Codex 和 OpenCode。
- ⌨️ **轻量 Vim 模式。** 可选启用，支持常用移动、操作符、文本对象和 Visual 模式。
- 🧭 **灵活的工作区。** 活动栏可放在左右两侧或顶部，也能调整图标大小，并支持使用 Zed 扩展。
- 🔒 **无需账号，不收集使用遥测或崩溃报告。** 智能体对话使用本机命令行工具中配置的服务商账号；扩展来自 Zed 扩展商店。

## 下载

在 macOS 上，可以从 Homebrew tap 安装：

```sh
brew install --cask magalab/tap/yo
```

在 macOS 和 Linux 上也可以运行：

```sh
curl -fsSL https://raw.githubusercontent.com/magalab/yo/develop/script/install.sh | sh
```

Windows 用户请下载[安装程序](https://github.com/magalab/yo/releases/latest/download/Yo-x86_64.exe)。

<details>
<summary>或者手动安装</summary>

- [macOS（Apple 芯片）](https://github.com/magalab/yo/releases/latest/download/Yo-aarch64.dmg)
- [macOS（Intel 芯片）](https://github.com/magalab/yo/releases/latest/download/Yo-x86_64.dmg)
- [Windows x86-64](https://github.com/magalab/yo/releases/latest/download/Yo-x86_64.exe)
- [Linux x86-64](https://github.com/magalab/yo/releases/latest/download/yo-linux-x86_64.tar.gz)
- [Linux AArch64](https://github.com/magalab/yo/releases/latest/download/yo-linux-aarch64.tar.gz)

更多内容请看 [INSTALL.md](https://github.com/magalab/yo/blob/develop/docs/INSTALL.md)。
</details>

---

## 文档

请看[文档](https://yo.lurenyang.com/zh/docs/)。

[轻量 Vim 模式](docs/lightweight-vim-mode.md)

## 构建

Yo 沿用 Zed 的 Rust 构建系统。构建依赖和流程请看 [Zed 开发文档](https://zed.dev/docs/development)。

## 许可证

Yo 采用 GPL-3.0-or-later 许可证，标注的部分采用 Apache-2.0。请看 [LICENSE-GPL](./LICENSE-GPL) 和 [LICENSE-APACHE](./LICENSE-APACHE)。

## 致谢

<a href="https://github.com/zed-industries/zed"><img src="https://zed.dev/logo_icon.webp" alt="Zed" width="48"></a>
