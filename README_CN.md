# vips-rs

[English](README.md) | [简体中文](README_CN.md)

[![Rust](https://github.com/houseme/vips-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/houseme/vips-rs/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/vips.svg)](https://crates.io/crates/vips)
[![docs.rs](https://docs.rs/vips/badge.svg)](https://docs.rs/vips/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Downloads](https://img.shields.io/crates/d/vips)](https://crates.io/crates/vips)

`libvips` 的 Rust 绑定，为图片处理提供高性能与低内存占用的能力，并以安全、易用的 API 封装常用功能。

- 针对常用 `libvips` API 的安全封装
- 基于 RAII 的初始化/关闭管理
- 提供读取、变换、写入的常用便捷方法
- 惰性流水线求值（libvips 按需计算）

文档：https://docs.rs/vips/

## 环境要求

- Rust >= 1.85.0（edition 2024）
- 系统已安装 `libvips`
    - macOS：`brew install vips`
    - Linux：`apt-get install -y pkg-config libvips libvips-dev`（或使用对应发行版包名）

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
vips = "0.1"
```

## 快速开始

```rust
use vips::*;

fn main() -> Result<()> {
    // 初始化 libvips（进程内仅需一次，通常布尔值用于控制自动关闭）
    let _instance = VipsInstance::new("app_example", true)?;

    // 从文件读取图片
    let img = VipsImage::from_file("./examples/images/kodim01.png")?;

    // 生成缩略图（强制宽高）
    let thumb = img.thumbnail(320, 240, VipsSize::VIPS_SIZE_FORCE)?;

    // 写入文件
    thumb.write_to_file("kodim01_320x240.jpg")?;
    Ok(())
}
```

## 从内存构造

- 拥有像素内存（推荐，简单无需额外生命周期约束）：

```rust
let pixels = vec![0u8; 256 * 256 * 3]; // RGB
let img = VipsImage::from_memory(pixels, 256, 256, 3, VipsBandFormat::VIPS_FORMAT_UCHAR)?;
let thumb = img.thumbnail(200, 200, VipsSize::VIPS_SIZE_FORCE)?;
thumb.write_to_file("black_200x200.png")?;
```

- 借用像素内存（需保证被借用的数据活得更久）：

```rust
let pixels = vec![0u8; 256 * 256 * 3];
let img = VipsImage::from_memory_reference(&pixels, 256, 256, 3, VipsBandFormat::VIPS_FORMAT_UCHAR)?;
let thumb = img.thumbnail(200, 200, VipsSize::VIPS_SIZE_FORCE)?;
thumb.write_to_file("black_ref_200x200.png")?;
```

## 生命周期与常见问题

- 优先使用拥有型构造（`from_file`、`from_memory`）。
- 借用型构造（`from_memory_reference`）会让返回图像绑定到输入切片生命周期；不要让底层切片先被释放。
- 避免在短作用域内创建图像却将其派生结果带到外层使用；请确保创建者与派生结果处于同一作用域或更长作用域。

## 功能概览

| 类别 | API |
|------|-----|
| 读写 | `from_file`、`from_memory`、`from_memory_reference`、`from_buffer`、`write_to_file`、`write_to_memory`、`write_to_buffer`、`write_jpeg`、`find_load`/`find_save` |
| 几何 | `thumbnail`、`resize`、`resize_reduce`、`reduce`、`shrink_box`、`crop`、`embed`、`flip`/`rot`/`rotate`/`autorot`、`zoom`、`insert`、`gravity`、`extract_band`、`bandjoin2`/`bandjoin_const`、`copy`/`copy_memory` |
| 算术 | `add`/`subtract`/`multiply`/`divide`、`linear`/`linear1`、`invert`、`cast`、`colourspace` |
| 滤波 | `gaussblur`、`sharpen` |
| 统计 | `avg`、`min_value`、`max_value`、`getpoint` |
| 属性 | `width`、`height`、`size`、`bands` |
| 绘制 | `draw_rect`、`draw_line`、`draw_circle`、`draw_flood` 等（原地修改） |
| 拼接 | `merge`、`mosaic`、`match_`、`globalbalance`、`remosaic` |
| 插值 | `VipsInterpolate` 最近邻 / 双线性 / 自定义 |

## 性能建议

- 目标是文件路径时优先 `write_to_file`，避免 `write_to_memory` 的整图拷贝。
- 大幅缩小（百万像素级、缩小倍数 ≳ 3）时优先 `resize_reduce`：先 box 预缩小再最终重采样。
- 按业务负载调整 `vips::set_concurrency`、`set_max_operations`、`set_max_mem_bytes`。
- libvips 流水线是惰性的：操作会入队，直到写出或物化时才真正计算。

## 说明

- 初始化：通过 `VipsInstance` 与标准库原语（`OnceLock`）管理 `vips_init`/`vips_shutdown`。
- 副作用：大多数操作返回新图像；`draw*` 系列会原地修改 `self`。
- 若缺少更高层封装，可使用底层 `vips-sys` 或 `vips::call(...)` 直接调用 libvips 操作。

## 本地开发

正式构建从 crates.io 解析 `vips-sys`。若要改用本地 sibling 源码，在**工作区/消费方根目录**（或临时在本 crate）增加：

```toml
[patch.crates-io]
vips-sys = { path = "../vips-sys" }
```

无需本机安装 libvips 的 Docker 验证：

```bash
docker run --rm -v "$PWD/..":/workspace -w /workspace/vips-rs rust:1.88-bookworm \
  bash -c 'apt-get update -qq && apt-get install -y -qq pkg-config libvips-dev && cargo test'
```

## 许可证

[MIT](LICENSE)

## 变更日志

参见 [`CHANGELOG.md`](CHANGELOG.md)。
