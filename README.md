# YMeng Minecraft (Rust)

[![CI](https://github.com/Youmeng1226/ymeng-minecraft-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/Youmeng1226/ymeng-minecraft-rust/actions/workflows/ci.yml)
![Rust](https://img.shields.io/badge/rust-2021-orange)
![Status](https://img.shields.io/badge/status-pre--alpha-red)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

用 Rust 从零开始实现一个类 Minecraft 的体素沙盒游戏。

> **状态：Pre-alpha。** 现在已经有核心数据结构、地形生成和网格构建，还没有图形窗口。进度见 [路线图](docs/ROADMAP.md)。

*English: a Minecraft-like voxel sandbox written from scratch in Rust. Core voxel types, procedural terrain and chunk meshing are in place; the `wgpu` renderer is the next milestone.*

## 目标

- **性能优先**：多线程区块生成与网格构建，远视距下保持高帧率
- **纯 Rust**：渲染基于 `wgpu`，窗口基于 `winit`，跨平台（Windows / Linux / macOS）
- **模块化**：引擎各部分拆成独立 crate，方便测试和替换
- **可扩展**：长期目标是支持数据驱动的方块 / 物品定义和模组

## 已完成

- [x] 方块类型、面、纹理索引（`ymc-core`）
- [x] 16×128×16 区块存储、世界坐标 ↔ 区块坐标转换（支持负坐标）
- [x] 区块 RLE 压缩编码（存档格式雏形）
- [x] 自带 Perlin 噪声 + fBm 分形噪声（`ymc-worldgen`）
- [x] 地形生成：基岩、石头、泥土、草地、沙滩、水、橡树
- [x] 网格构建：隐藏面剔除，不透明 / 透明两个渲染通道（`ymc-mesh`）
- [x] 跨区块邻居查询，区块边界不产生多余的面
- [x] 无窗口预览程序：生成出生点附近的世界并输出俯视地图

## 快速开始

需要 Rust 1.78+（[安装 rustup](https://rustup.rs)）。

```bash
git clone https://github.com/Youmeng1226/ymeng-minecraft-rust.git
cd ymeng-minecraft-rust

cargo test                    # 运行全部单元测试
cargo run --release           # 默认种子
cargo run --release -- 12345  # 指定种子
```

输出示例：

```
YMeng Minecraft (Rust) v0.1.0
seed: 20261119

generated 49 chunks in 2.4ms
meshed 25 chunks in 6.7ms
  opaque quads:      15659
  transparent quads: 2571

top-down map (~ water, . sand, " grass, T tree, ^ mountain)
..........""""""""""""""""""""""""""""""""""""""""""TTTTT""""""""""
..~~~~~~~~~.....""""""""TTT"""""""""""""""""""""""TTTTT""""""""""""
~~~~~~~~~~~....."""""""TTTTT"""""""""""""""""""""""""""""""""""""""
~~~~~~~~~~...................""""........""""""""""""""""""""""""""
```

## 项目结构

```
crates/
├── ymc-core       方块、区块、坐标等核心类型
├── ymc-worldgen   噪声与地形生成
├── ymc-mesh       区块 → 顶点 / 索引缓冲
└── ymc-client     可执行程序（目前是无窗口预览）
docs/
├── ROADMAP.md     开发路线图
└── ARCHITECTURE.md 架构设计
```

详细设计见 [ARCHITECTURE.md](docs/ARCHITECTURE.md)。

## 参与贡献

欢迎提 Issue 和 PR，请先阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 声明

本项目是独立的学习 / 爱好项目，与 Mojang Studios 或 Microsoft 无关，不使用任何 Minecraft 官方资源。

## 许可证

[MIT](LICENSE)
