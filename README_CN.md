<h3 align="center">
一个基于 relm4 和 pdfium-render 库的新 PDF 阅读器。
其目标是提供一个功能丰富的 PDF 阅读器，重点关注性能、易用性，并支持 Windows、MacOS 和 Linux 跨平台使用。
现在它可以在 Linux 上运行了！
</h3>

<br>

## 快速开始
### Linux
- 从源代码安装：
```
克隆仓库：`git clone https://gitee.com/gxpdf/gxpdf-reader.git`
构建项目：`cargo build --release`
复制库文件：`cp -rv pdfium-lib/libpdfium.so ./target/release/`
运行应用：`./target/release/gxpdf-reader test/annotations-test.pdf`
```
- 使用 cargo run 运行
```
克隆仓库：`git clone https://gitee.com/gxpdf/gxpdf-reader.git`
构建调试版本：`cargo build`
复制库文件：`cp -rv pdfium-lib/libpdfium.so ./target/debug/`
运行应用：`cargo run -- test/annotations-test.pdf`
```

## 许可协议
许可协议将稍后补充。

## 版权
版权所有 (C) 2025, gxpdf.com