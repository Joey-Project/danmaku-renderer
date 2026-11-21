---
id: 20261121-dmr01
title: 独立弹幕渲染 crate 与下游迁移
status: active
created: 2026-11-21
updated: 2026-11-21
branch:
pr:
supersedes: []
superseded_by:
---

# 独立弹幕渲染 crate 与下游迁移

## Summary

- 一个 crate 提供 Rust API 和可选 CLI。
- `naive` 默认使用 `original` 样式，兼容 BBDown 首版输出。两种模式均可选择 `tv`，即 3 倍字号、70% 不透明度和半屏，并可单独覆盖样式。
- 默认组合为 `compact` 和 `tv`。`compact` 迁移了 Telegram 第三版的同文同向滚动弹幕计数合并和双向时间检查。空间不足时跳过当前弹幕。

## Current State

- Rust API、可选 CLI、两种渲染模式、两个样式预设和独立的 Crates.io 发布 workflow 已实现。
- 《芯片能效横评》和《终将成为你》舞台剧的黄金样例来自真实 XML。两份快照共覆盖 19,796 条弹幕，并与 BBDown 首版逐字节比对。
- Rust 集成测试覆盖时间窗口去重、碰撞筛选及 50,000 条密集输入。libass 像素测试覆盖四档显示区域。

## Next Steps

- 发布 `0.1.0`，从注册表验证纯库依赖和 CLI 安装。
- 用独立 worktree 提交下游 PR。BBDown 保留 `naive` 默认行为，Telegram 传入现有样式设置并选择 `compact`。刷新保留旧版 ASS 识别和手工编辑保护。
- 使用统一 Cargo 构建目录，各里程碑结束后检查并清理本任务已用完的构建产物。

## Evidence

- [BBDown 首版 ASS 渲染](https://github.com/Joey-Project/BBDown-rust/commit/7c3e54d)
- [Telegram 第三版轨道布局](https://github.com/Joey-Project/Telegram-Video-Downloader/commit/9d0a5c3c82ffa957cbf33cbdf07dfba059e2cbdd)
