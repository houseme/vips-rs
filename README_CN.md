# vips-rs

[English](README.md) | [简体中文](README_CN.md)

[![Rust](https://github.com/houseme/vips-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/houseme/vips-rs/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/vips.svg)](https://crates.io/crates/vips)
[![docs.rs](https://docs.rs/vips/badge.svg)](https://docs.rs/vips/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Downloads](https://img.shields.io/crates/d/vips)](https://crates.io/crates/vips)

`libvips` 的 Rust 绑定，为图片处理提供高性能与低内存占用的能力，并以安全、易用的 API 封装常用功能。

- 针对常用 `libvips` API 的安全封装
- 进程级初始化与基于 RAII 的图像资源管理
- 提供读取、变换、写入的常用便捷方法
- 惰性流水线求值（libvips 按需计算）

文档：https://docs.rs/vips/

## 环境要求

- 当前 stable Rust（edition 2024）
- 系统已安装 `libvips`
    - macOS：`brew install vips`
    - Linux：`apt-get install -y pkg-config libvips-dev`（或使用对应发行版包名）

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
vips = "0.2"
```

## 快速开始

```rust
use vips::*;

fn main() -> Result<()> {
    // 初始化 libvips（进程内仅需一次，布尔值控制泄漏诊断）
    let _instance = VipsInstance::new("app_example", true)?;

    // 生成缩略图（文件来源优先走 vips_thumbnail，可 shrink-on-load）
    let thumb = VipsImage::thumbnail_file("./examples/images/kodim01.png", 320, 240, VipsSize::VIPS_SIZE_FORCE)?;

    // 写入文件
    thumb.write_to_file("kodim01_320x240.jpg")?;
    Ok(())
}
```

## 0.3.0 开发版本

`v8_19` feature 开放基于 **libvips 8.19.0-rc1** 验证的 API，包括 QOI 文件与
缓冲区读写、CICP/scRGB 转换、亮度百分位、动画循环次数、借用 blob 元数据，
以及指定关注点的裁剪和缩略图。该 feature 包含 `v8_18`，要求原生 libvips 8.19+。
默认功能和 `v8_18` 同时在原生 8.18.7 上验证。

本次 Git 版本锁定尚未发包的 `vips-sys` 0.4.0 commit，Cargo 自动拉取该修订。
后续发包需先发布 `vips-sys`，再切回 registry 依赖。下方 0.2 示例对应已发布版本。
验证记录、性能测量及 RC 的 QOI 缩略图限制见 [review](docs/review-2026-11-07.md)。

## 0.2.0 新增能力与迁移

0.2.0 版本依赖已正式发布的 `vips-sys` 0.3.0，无需检出同级仓库。

- `write_to_memory_owned` / `write_to_buffer_owned` 返回 `VipsOwnedBuffer`，
  持有 GLib 内存并提供不可变字节切片，省去额外复制到 Rust `Vec` 的成本；原 Vec API 保留。
- `from_file_with_access` / `from_buffer_with_access` 支持 `VipsAccess` 提示。
  单遍流水线可选顺序读取；文件或编码缓冲区缩略图仍优先使用 shrink-on-load 构造函数。
- `v8_18` feature 提供 `tile_size`、`gainmap`、相机 RAW 文件/缓冲区加载、
  UltraHDR 文件/缓冲区读写及 scRGB 转换、Oklab/Oklch 转换和 `interpretation_bands`。
  需要系统 libvips 8.18+，RAW/HDR 还取决于系统库是否编入相应可选 codec。
- `VipsRegion` 新增缓冲区生命周期；多图惰性操作保留所有输入缓冲区的生命周期。
  图像句柄允许先销毁，借用的像素数据仍需保持存活。
- 绘图首次修改借用或共享数据时先复制到自有可写内存；输入须满足样本格式的对齐要求。
- `VipsInterpolateMethod::call` 改为 unsafe，调用者须保证输出大小、对齐和 region 插值窗口有效。
- `VipsInstance` 与 `init` 共享初始化状态，句柄销毁不再关闭进程级 libvips。

```toml
vips = { version = "0.2", features = ["v8_18"] }
```

使用 `cargo run --release --example buffer_bench` 运行交替 ABBA 内存导出对比。
完整核查与后续优化见 [review](docs/review-2026-10-22.md)。

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
- 派生流水线保留原生图像引用，创建者句柄可以先销毁；所有借用输入缓冲区仍须覆盖派生结果的使用期。

## 功能概览

| 类别 | API |
|------|-----|
| 读写 | `from_file`、`from_memory`、`from_memory_reference`、`from_buffer`、`write_to_file`、`write_to_memory`、`write_to_buffer`、`write_jpeg`、`find_load`/`find_save`、`black`/`xyz`/`grey`/`sines`/`zone`/`perlin`/`gaussnoise` |
| 几何 | `thumbnail_file`、`thumbnail_buffer`、`thumbnail`（已解码像素）、`resize`、`resize_reduce`、`reduce`、`shrink_box`、`crop`、`embed`、`flip`/`rot`/`rotate`/`autorot`、`zoom`、`insert`、`gravity`、`extract_band`、`bandjoin2`/`bandjoin_const`、`copy`/`copy_memory`、`smartcrop`、`ifthenelse` |
| 算术 | `add`/`subtract`/`multiply`/`divide`、`linear`/`linear1`、`invert`、`pow_const`、`abs`/`sign`/`clamp`、`floor`/`ceil`/`rint` |
| 数学 | `sin`/`cos`/`tan`、`ln`/`log10`/`exp`/`exp10` |
| 关系/位运算 | `less`/`lesseq`/`more`/`moreeq`/`equal_const`/`notequal_const`、布尔/移位常量、`bandand`/`bandor`/`bando` |
| 色彩 | `cast`、`colourspace`、`falsecolour`、`premultiply`/`unpremultiply` |
| 滤波 | `gaussblur`、`sharpen`、`conv`、`erode`/`dilate`、`median`、`sobel`、`canny`、`maplut` |
| 直方图 | `hist_find`/`hist_local`/`hist_equal`/`hist_entropy`、`labelregions` |
| 统计 | `avg`、`min_value`、`max_value`、`deviate`、`percent`、`getpoint`、`find_trim` |
| 属性 | `width`、`height`、`size`、`bands` |
| 绘制 | `draw_rect`、`draw_line`、`draw_circle`、`draw_flood` 等（原地修改） |
| 拼接 | `merge`、`mosaic`、`match_`、`globalbalance`、`remosaic` |
| 插值 | `VipsInterpolate` 最近邻 / 双线性 / 自定义 |

## 性能建议

- 缩略图优先使用 `VipsImage::thumbnail_file` / `VipsImage::thumbnail_buffer`，不要先 `from_file`/`from_buffer` 再 `thumbnail`。前者调用 `vips_thumbnail` / `vips_thumbnail_buffer`，会把加载与缩放合并，libvips 可做 shrink-on-load（通常比 `thumbnail_image` 快数倍、峰值内存更低）。实例方法 `thumbnail` 仅用于已解码的原始像素。
- 目标是文件路径时优先 `write_to_file`，避免 `write_to_memory` 的整图拷贝。
- 对已解码大图，先对比 `resize_reduce` 与 `resize` 再决定是否加 box 预缩小；小图上额外预处理可能更慢。
- 按业务负载调整 `vips::set_concurrency`、`set_max_operations`、`set_max_mem_bytes`。
- libvips 流水线是惰性的：操作会入队，直到写出或物化时才真正计算。

## 说明

- 初始化：`init` 与 `VipsInstance` 共享进程级状态；图像资源采用 RAII，全局状态保持到进程退出。
- 副作用：大多数操作返回新图像；`draw*` 系列会原地修改 `self`。
- 若缺少更高层封装，可使用底层 `vips-sys` 或 unsafe `vips::vips_call(...)` 直接调用 libvips 操作。

## 本地开发

当前开发版本解析已固定的 `vips-sys` Git 修订。如需同时开发两个 crate，可在工作区或消费方根目录添加以下覆盖。

```toml
[patch."https://github.com/houseme/vips-sys"]
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
