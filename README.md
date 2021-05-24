# blog-rs

## 自带服务端的博客系统

当前版本：`0.2.1`

## 亮点
1. 单文件（小于 6 Mb）跨平台可执行文件
1. 自带 HTTP 服务（暂时不支持 HTTPS）
1. 所有嵌入资源都通过`gzip`压缩
1. 嵌入`Markdown`编辑器
1. 所有嵌入静态资源均通过`gzip`压缩，优化网络传输

## A singleton self-serve Blog written in Rust (Warp + Yew)

Current version: `0.2.1`

## Features
1. Single executable file (less than 6Mb), support `Windows`, `Linux`, `macOS`
1. Self-hosting (port can be changed via command-line argument)
1. All static resources were gzipped for bandwidth optimization
1. Embed `Markdown` editor
1. Export posts for `Hugo` ( Not implemented Yet `:)` )