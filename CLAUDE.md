# Reptool - 逆向工程智能体工具链

## 项目概述

Reptool 是一个 Rust 编写的全栈逆向/爬虫/代理 CLI 工具链，同时提供：
- **MCP Server**: 41 个工具供 Claude/Codex/LLM 直接调用
- **Skills 智能体**: 可在 Claude Code 中作为技能使用的逆向工程工作流

## 构建

```bash
cargo build --release
```

二进制输出: `target/release/reptool` (Linux/macOS) 或 `target/release/reptool.exe` (Windows)

## MCP 使用

```bash
reptool mcp
```

通过 stdio 提供 JSON-RPC 2.0 协议的 MCP 服务。

## Skills 目录

- `skills/reverse-engineering/` — 主逆向工程智能体
- `skills/js-analysis/` — JS 逆向分析智能体
- `skills/crypto-toolkit/` — 加密工具智能体
- `skills/traffic-analysis/` — 流量分析智能体
- `skills/agent-orchestration/` — 多智能体编排

## 代码结构

- `src/mcp/` — MCP Server 实现
- `src/js/` — JS 逆向引擎
- `src/crypto/` — 加密工具箱
- `src/har/` — HAR 流量分析
- `src/proxy/` — MITM 代理
- `src/ai/` — AI 助手集成
- `src/frida.rs` — Frida Hook 生成

## 提交规范

- 使用中文提交信息
- 格式: `类型: 简短描述`
