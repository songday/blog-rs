![BlogListPage](screenshot1.jpg)

## 自带服务端的博客系统

当前版本：`0.5.0`

## 亮点
1. 单文件（小于 6 Mb）跨平台可执行文件
2. 两种工作模式：1、带博客后台的创作模式，2、纯文本文件服务器模式（暂未实现）
3. 自带 HTTP 服务（暂时不支持 HTTPS）
4. 所有嵌入静态资源均通过`gzip`压缩，优化网络传输
5. 嵌入`Markdown`编辑器
6. 导出`Hugo`数据（暂未实现）

## A singleton self-serve Blog written in Rust (Warp + Yew)

Current version: `0.5.0`

## Features
1. Single executable file (less than 6Mb), support `Windows`, `Linux`, `macOS`
2. Two serve mode. One with `Blog backend`, another one is pure static file service ( Not implemented yet )
3. Self-hosting (port can be changed via command-line argument)
4. All static resources were gzipped for bandwidth optimization
5. Embed `Markdown` editor
6. Export posts for `Hugo` ( Not implemented yet `:)` )
