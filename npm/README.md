# reptool

**Rust 全栈逆向/爬虫/代理 CLI 工具链**

> 集成 mitmproxy + Burp + Frida + JsHook + AI 逆向助手 + 加密工具箱 + 爬虫框架 + MCP Server，单二进制开箱即用。

## 安装

```bash
npm install -g reptool
```

## 快速开始

```bash
# 查看所有命令
reptool --help

# 加密工具
reptool crypto hash --algo md5 --input "hello"
reptool crypto encrypt --algo aes-cbc --key "1234567890123256" --iv "0000000000000000" --data "hello"

# 签名计算
reptool calc sign-sort '{"a":"1","b":"2","c":"3"}' --salt mykey

# 启动 MITM 代理
reptool proxy start -p 8080

# JS 逆向
reptool js format app.js
reptool js deobfuscate app.js

# HAR 分析
reptool har parse capture.har

# Frida Hook
reptool hook generate com.example.App --hook-type ssl

# MCP Server (给 Claude/LLM)
reptool mcp
```

## 特性

- **MITM 代理** — HTTP/HTTPS 中间人拦截，JS Hook 注入
- **Frida Hook** — SSL Pinning 绕过 / Crypto / HTTP / 存储 Hook 脚本生成
- **JS 逆向** — 格式化 / 反混淆 / 接口提取 / 签名分析 / 密钥提取
- **AI 逆向助手** — 支持 Claude / GPT / Gemini / DeepSeek + 自定义 OpenAI 兼容
- **加密工具箱** — md5 / sha / aes / des / rsa / hmac / base64 / urlencode
- **HAR 分析** — 解析 / 对比 / 重放 / 过滤 / 导出 curl / python / fetch
- **MCP Server** — 41 个工具暴露给 Claude / LLM 直接调用
- **跨平台** — Windows / Linux / macOS

## 命令参考

### 代理

```bash
reptool proxy start -p 8080                 # 启动代理
reptool proxy cert --output ca.cer           # 导出 CA 证书
reptool proxy sessions                       # 查看抓包会话
```

### JS 逆向

```bash
reptool js format app.js                     # 格式化
reptool js deobfuscate app.js                # 反混淆
reptool js analyze-sign app.js               # 分析加密函数
reptool js extract-keys app.js               # 提取密钥
reptool js scan-api ./js_folder/             # 扫描接口
```

### 加密工具

```bash
reptool crypto hash --algo md5 --input "hello"
reptool crypto hmac --algo sha256 --key k --data d
reptool crypto encrypt --algo aes-cbc --key k --data d
reptool crypto decrypt --algo aes-cbc --key k --data d
reptool crypto random-ua --browser chrome
reptool crypto timestamp
```

### HAR 分析

```bash
reptool har parse capture.har
reptool har diff old.har new.har
reptool har export capture.har python
```

### Frida Hook

```bash
reptool hook generate com.example.App --hook-type ssl
reptool hook generate com.example.App --hook-type crypto
reptool hook attach com.example.App ./hook.js
```

### MCP Server

```bash
reptool mcp   # 启动 stdio MCP Server
```

支持 Claude Code / Claude Desktop / Codex CLI / VS Code / Cursor / Windsurf 等工具。

## 文档

完整文档请访问 [GitHub](https://github.com/ice-a/spider-cli)

## License

MIT
