# vips-rs

[English](README.md) | [简体中文](README_CN.md)

[![Crates.io](https://img.shields.io/crates/v/vips.svg)](https://crates.io/crates/vips)
[![Docs](https://img.shields.io/badge/docs-online-blue)](https://elbaro.github.io/vips-rs/vips/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

`libvips` 的 Rust 绑定，为图片处理提供高性能与低内存占用的能力，并以安全、易用的 API 封装常用功能。

- 针对常用 `libvips` API 的安全封装
- 基于 RAII 的初始化/关闭管理
- 提供读取、变换、写入的常用便捷方法

文档：https://elbaro.github.io/vips-rs/vips/

## 环境要求

- Rust >= 1.80.0
- 系统已安装 `libvips`
    - macOS：`brew install vips`
    - Linux：`apt-get install -y libvips libvips-dev`（或使用对应发行版包名）

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
vips = "*"
```

## 快速开始

```rust
use vips::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 libvips（进程内仅需一次，通常布尔值用于控制自动关闭）
    let _instance = VipsInstance::new("app_example", true)?;

    // 从文件读取图片
    let img = VipsImage::from_file("kodim01.png")?;

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
let img = VipsImage::from_memory(pixels, 256, 256, 3, VipsBandFormat::VIPS_FORMAT_UCHAR) ?;
let thumb = img.thumbnail(200, 200, VipsSize::VIPS_SIZE_FORCE) ?;
thumb.write_to_file("black_200x200.png") ?;
```

- 借用像素内存（需保证被借用的数据活得更久）：

```rust
let pixels = vec![0u8; 256 * 256 * 3];
let img = VipsImage::from_memory_reference(
& pixels, 256, 256, 3, VipsBandFormat::VIPS_FORMAT_UCHAR
) ?; // 返回图像与 `pixels` 共享生命周期
let thumb = img.thumbnail(200, 200, VipsSize::VIPS_SIZE_FORCE) ?;
thumb.write_to_file("black_ref_200x200.png") ?;
```

## 生命周期与常见问题

- 优先使用拥有型构造（`from_file`、`from_memory`）。
- 借用型构造（`from_memory_reference`）会让返回图像绑定到输入切片生命周期；不要让底层切片先被释放。
- 避免在短作用域内创建图像却将其派生结果带到外层使用；请确保创建者与派生结果处于同一作用域或更长作用域。

## 功能概览

- 读写：文件、内存（像素/缓冲区）
- 几何：缩略图、比例缩放、降采样、快速缩小
- 绘制：线、圆、洪泛填充（原地修改）
- 拼接：`merge`、`mosaic`、`match_`、`globalbalance`
- 插值：最近邻、双线性、自定义

更多示例与细节请参考在线文档与 `examples`。

## 说明

- 初始化：通过 `VipsInstance` 与标准库原语（`OnceLock`）管理 `vips_init`/`vips_shutdown`。
- 副作用：大多数操作返回新图像；`draw*` 系列会原地修改 `self`。
- 若缺少更高层封装，可使用底层 `vips-sys` 或 `vips::call(...)` 直接调用 libvips 操作。

## 许可证

[MIT](LICENSE)

## 变更日志

参见 [`CHANGELOG.md`](CHANGELOG.md)。