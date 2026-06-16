# Reptool

**Rust 全栈逆向/爬虫/代理 CLI 工具链**

> 集成 mitmproxy + Burp + Frida + JsHook + AI 逆向助手 + 加密工具箱 + 爬虫框架 + MCP Server，单二进制开箱即用。

## 特性

- **MITM 代理** — HTTP/HTTPS 中间人拦截，自动 CA 证书生成，JS Hook 注入
- **Frida Hook** — 生成 SSL Pinning 绕过 / Crypto Hook / HTTP Hook / 存储 Hook 脚本
- **JS 逆向** — 格式化/反混淆/接口提取/签名分析/密钥提取/Hook 脚本生成
- **AI 逆向助手** — 支持 Claude/GPT/Gemini/小米/DeepSeek/通义/豆包 + 自定义 OpenAI 兼容
- **加密工具箱** — md5/sha/aes/des/rsa/hmac/base64/urlencode + 分步计算
- **HAR 分析** — 解析/对比/重放/过滤/凭证识别，导出 curl/python/fetch/postman 等 8 种格式
- **双内核渲染** — Chrome/Firefox 无头浏览器，指纹篡改，脚本注入
- **MCP Server** — 41 个工具暴露给 Claude/LLM 直接调用
- **跨平台** — Windows / Linux / macOS 单二进制，npm 一键安装

## 安装

### npm (推荐)

```bash
npm install -g reptool
```

### 从源码编译

```bash
git clone https://github.com/your/reptool.git
cd reptool
cargo build --release
# 二进制位于 target/release/reptool
```

### 下载预编译二进制

从 [Releases](https://github.com/your/reptool/releases) 下载对应平台的二进制文件。

## 快速开始

```bash
# 查看所有命令
reptool --help

# 加密工具
reptool crypto hash --algo md5 --input "hello"
reptool crypto hash --algo sha256 --input "reptool"
reptool crypto encrypt --algo aes-cbc --key "1234567890123256" --iv "0000000000000000" --data "hello"
reptool crypto random-ua --browser chrome --count 3

# 签名计算
reptool calc sign-sort '{"a":"1","b":"2","c":"3"}' --salt mykey
# Windows CMD: reptool calc sign-sort "{\"a\":\"1\",\"b\":\"2\",\"c\":\"3\"}" --salt mykey
# 或使用 key=value 格式: reptool calc sign-sort "a=1,b=2,c=3" --salt mykey
reptool calc step --step "sort-params:{\"a\":\"1\",\"b\":\"2\"}" --step "concat-salt:secret" --step "md5"

# 启动 MITM 代理
reptool proxy start -p 8080 --hook-fetch
reptool proxy cert --output ca.cer

# JS 逆向
reptool js format app.js
reptool js deobfuscate app.js --technique auto
reptool js analyze-sign app.js --functions sign,encrypt
reptool js extract-keys app.js
reptool js hook-generate getSign --hook-type sign
reptool js scan-api ./js_folder/

# HAR 分析
reptool har parse capture.har --extract cookies,params
reptool har diff old.har new.har
reptool har export capture.har python

# Frida Hook
reptool hook generate com.example.App --hook-type ssl
reptool hook generate com.example.App --hook-type crypto
reptool hook attach com.example.App ./hook.js

# AI 逆向助手
reptool ai-config setup                          # 交互式配置
reptool ai-config set --provider "GPT" --api-key "sk-xxx" --model "gpt-4"
reptool ai-config reverse app.js                  # AI 分析 JS
reptool ai-config chat "分析这段加密逻辑..."

# AI 分析
reptool ai analyze-js app.js                      # 逆向分析报告
reptool ai analyze-traffic capture.har            # 流量分析

# MCP Server (给 Claude/LLM)
reptool mcp
```

## 命令参考

### 代理

```bash
reptool proxy start -p 8080                          # 启动代理
reptool proxy start -p 8080 --filter "*.api.com"     # URL 过滤
reptool proxy start -p 8080 --hook-fetch             # 自动注入 fetch/XHR hook
reptool proxy cert --output ca.cer                    # 导出 CA 证书
reptool proxy rule add "Header:User-Agent=xxx"        # 添加篡改规则
reptool proxy sessions                                # 查看抓包会话
```

### JS 逆向

```bash
reptool js format app.js                              # 格式化
reptool js deobfuscate app.js --technique auto        # 反混淆
reptool js scan-api ./js_folder/                      # 扫描接口
reptool js analyze-sign app.js                        # 分析加密函数
reptool js extract-keys app.js                        # 提取密钥
reptool js hook-generate sign --hook-type sign        # 生成 Hook 脚本
reptool js batch-scan ./folder/                       # 批量扫描
```

### 加密工具

```bash
reptool crypto hash --algo md5 --input "hello"        # 哈希
reptool crypto hmac --algo sha256 --key k --data d    # HMAC
reptool crypto encrypt --algo aes-cbc --key k --data d # 对称加密
reptool crypto decrypt --algo aes-cbc --key k --data d # 对称解密
reptool crypto random-ua --browser chrome             # 随机 UA
reptool crypto timestamp                              # 时间戳
```

### 手工计算

```bash
reptool calc sign-sort "a=1,b=2" --salt key           # 参数排序签名 (key=value格式)
reptool calc step --step "md5" --step "base64-encode" # 分步计算
reptool calc diff-sign --src-str "..." --src-sign "..." # 签名对比
reptool calc url-encode "hello world"                 # URL 编码
reptool calc hex-view file.bin                        # 十六进制查看
```

### HAR 分析

```bash
reptool har parse capture.har                         # 解析
reptool har diff old.har new.har                      # 对比差异
reptool har extract-creds capture.har                 # 提取凭证
reptool har replay capture.har --concurrency 5       # 批量重放
reptool har export capture.har python                 # 导出
```

### Frida Hook

```bash
reptool hook generate com.example.App --hook-type ssl      # SSL Pinning 绕过
reptool hook generate com.example.App --hook-type crypto   # 加密函数 Hook
reptool hook generate com.example.App --hook-type http     # HTTP 请求 Hook
reptool hook generate com.example.App --hook-type storage  # 存储 Hook
reptool hook generate com.example.App --hook-type native   # Native 函数 Hook
reptool hook generate com.example.App --hook-type class    # 全类方法 Hook
reptool hook attach com.example.App ./hook.js              # 附加到进程
reptool hook ps                                             # 列出进程
```

### AI 逆向助手

```bash
# 配置 AI (交互式)
reptool ai-config setup

# 快速配置
reptool ai-config set --provider "GPT" --base-url "https://api.openai.com" --api-key "sk-xxx" --model "gpt-4"
reptool ai-config set --provider "Claude" --api-key "sk-ant-xxx" --model "claude-3-opus"
reptool ai-config set --provider "Gemini" --api-key "xxx" --model "gemini-pro"
reptool ai-config set --provider "DeepSeek" --api-key "sk-xxx" --model "deepseek-coder"
reptool ai-config set --provider "自定义" --base-url "http://localhost:11434" --api-key "xxx" --model "qwen2"

# AI 逆向分析
reptool ai-config reverse app.js               # 分析 JS 加密逻辑
reptool ai-config chat "分析这段签名算法..."    # AI 对话

# 内置分析报告
reptool ai analyze-js app.js                    # JS 逆向分析报告
reptool ai analyze-traffic capture.har          # 流量分析报告
```

### 爬虫

```bash
reptool crawl http https://api.example.com            # HTTP 请求
reptool crawl ws wss://echo.websocket.org             # WebSocket
reptool req single -X POST -u https://api.test -b '{}' # 手工单发
```

### 双内核渲染

```bash
reptool render --engine chrome --url https://target.com
reptool render --engine firefox --script hook.js --spoof-canvas
reptool render --engine chrome --url https://target.com --debug
```

### 辅助工具

```bash
reptool tools dns example.com                         # DNS 解析
reptool tools port-check 8080                         # 端口检测
reptool tools json-format '{"a":1}'                   # JSON 格式化
reptool tools json-extract data.json "$.data.list"    # JSON 提取
```

### 配置

```bash
reptool config set global-ua "Mozilla/5.0..."         # 设置配置
reptool config get global-ua                          # 读取配置
reptool config list                                   # 列出配置
```

## MCP Server

Reptool 内置 MCP Server，可被所有支持 MCP 协议的 AI 编程工具调用。

```bash
reptool mcp   # 启动 stdio MCP Server
```

### 可用工具 (41个)

| 分类 | 工具 |
|------|------|
| HAR | `har_parse`, `har_diff`, `har_extract_creds`, `har_filter`, `har_export` |
| JS 逆向 | `js_format`, `js_deobfuscate`, `js_extract_apis`, `js_analyze_sign`, `js_extract_keys`, `js_hook_generate` |
| 加密 | `crypto_hash`, `crypto_hmac`, `crypto_encrypt`, `crypto_decrypt`, `crypto_base64`, `crypto_urlencode`, `crypto_timestamp`, `crypto_random_ua` |
| 计算 | `calc_sign_sort`, `calc_step`, `calc_diff_sign` |
| 代理 | `proxy_start`, `proxy_stop`, `proxy_get_sessions` |
| 爬虫 | `crawl_http`, `crawl_ws_connect` |
| 导出 | `export_curl`, `export_python`, `export_fetch`, `export_go` |
| 工具 | `tools_dns`, `tools_port_check`, `tools_json_format`, `tools_json_extract` |
| APP | `mini_wxapkg_parse`, `mini_apk_extract`, `proto_decode` |
| 配置 | `config_get`, `config_set` |

### 各工具配置教程

> **通用提示：** 如果 `reptool` 不在 PATH 中，将命令替换为完整路径：
> - Windows: `C:/Users/<用户名>/AppData/Roaming/npm/node_modules/reptool/bin/reptool.exe`
> - macOS/Linux: `$(which reptool)` 或 `/usr/local/bin/reptool`

---

#### 1. Claude Code (终端 AI 编程助手)

Claude Code 是 Anthropic 官方的终端 AI 编程工具，支持 MCP。

**方法 A: 全局配置 (推荐)**

编辑 `~/.claude/settings.json`：

```json
{
  "mcpServers": {
    "reptool": {
      "command": "reptool",
      "args": ["mcp"]
    }
  }
}
```

**方法 B: 项目级配置**

在项目根目录创建 `.mcp.json`：

```json
{
  "mcpServers": {
    "reptool": {
      "command": "reptool",
      "args": ["mcp"]
    }
  }
}
```

**使用示例：**

```bash
# 启动 Claude Code 后直接对话
> 帮我分析这段 JS 的加密逻辑
> 把这个 HAR 请求导出为 curl
> 计算 md5: hello
```

---

#### 2. Claude Desktop (桌面客户端)

**Windows 配置：**

编辑 `%APPDATA%\Claude\claude_desktop_config.json`：

```json
{
  "mcpServers": {
    "reptool": {
      "command": "C:/Users/<用户名>/AppData/Roaming/npm/node_modules/reptool/bin/reptool.exe",
      "args": ["mcp"]
    }
  }
}
```

**macOS 配置：**

编辑 `~/Library/Application Support/Claude/claude_desktop_config.json`：

```json
{
  "mcpServers": {
    "reptool": {
      "command": "reptool",
      "args": ["mcp"]
    }
  }
}
```

重启 Claude Desktop 后生效。

---

#### 3. OpenAI Codex CLI

Codex CLI 是 OpenAI 的终端编程工具，支持 MCP Server。

编辑 `~/.codex/config.json` (或 `~/.codex/config.yaml`)：

```json
{
  "mcpServers": {
    "reptool": {
      "command": "reptool",
      "args": ["mcp"]
    }
  }
}
```

或使用 YAML 格式：

```yaml
mcpServers:
  reptool:
    command: reptool
    args: ["mcp"]
```

---

#### 4. VS Code / Cursor (GitHub Copilot)

**方法 A: 使用 Copilot Chat 的 MCP 配置**

在 `.vscode/mcp.json` (或 Cursor 的 `.cursor/mcp.json`)：

```json
{
  "servers": {
    "reptool": {
      "command": "reptool",
      "args": ["mcp"]
    }
  }
}
```

**方法 B: VS Code 原生 MCP 支持 (1.99+)**

在 `.vscode/settings.json` 中添加：

```json
{
  "mcp": {
    "servers": {
      "reptool": {
        "command": "reptool",
        "args": ["mcp"]
      }
    }
  }
}
```

---

#### 5. Windsurf (Codeium)

Windsurf 是 Codeium 推出的 AI 编辑器，原生支持 MCP。

编辑 `~/.codeium/windsurf/mcp_config.json`：

```json
{
  "mcpServers": {
    "reptool": {
      "command": "reptool",
      "args": ["mcp"]
    }
  }
}
```

---

#### 6. Cline (VS Code 插件)

安装 [Cline](https://marketplace.visualstudio.com/items?itemName=saoudrizwan.claude-dev) 插件后：

1. 打开 Cline 侧边栏
2. 点击右上角齿轮图标 → MCP Servers
3. 点击 "Add new MCP server"
4. 填写：
   - Name: `reptool`
   - Type: `command`
   - Command: `reptool mcp`

或直接编辑 `~/.cline/mcp_settings.json`：

```json
{
  "mcpServers": {
    "reptool": {
      "command": "reptool",
      "args": ["mcp"],
      "type": "stdio"
    }
  }
}
```

---

#### 7. Zed Editor

编辑 `~/.config/zed/settings.json`，在 `context_servers` 中添加：

```json
{
  "context_servers": {
    "reptool": {
      "command": "reptool",
      "args": ["mcp"]
    }
  }
}
```

---

#### 8. Continue.dev

编辑 `~/.continue/config.json`：

```json
{
  "mcpServers": [
    {
      "name": "reptool",
      "command": "reptool",
      "args": ["mcp"]
    }
  ]
}
```

---

### 验证 MCP 连接

配置完成后，启动对应工具并输入：

```
请帮我用 reptool 计算 md5: hello
```

如果返回正确结果，说明 MCP 连接成功。

### 常用对话示例

| 你说的话 | 调用的工具 |
|----------|-----------|
| "帮我分析 app.js 的加密函数" | `js_analyze_sign` |
| "把这个请求导出为 curl" | `export_curl` |
| "计算 md5: hello" | `crypto_hash` |
| "对比这两份 HAR 的差异" | `har_diff` |
| "生成 Frida SSL bypass 脚本" | `js_hook_generate` |
| "分析这段流量的可疑接口" | `har_filter` |
| "AES-CBC 加密 hello，key 是 1234567890123456" | `crypto_encrypt` |
| "扫描这个目录下所有 JS 的 API 接口" | `js_extract_apis` |
| "解析这个 HAR 文件的 cookie 和参数" | `har_parse` |
| "导出为 Python requests 代码" | `export_python` |

## 架构

```
reptool (Rust 单二进制)
├── proxy/     MITM 代理引擎 (tokio + rustls + rcgen)
├── js/        JS 逆向 (regex AST + 反混淆)
├── crypto/    加密工具箱 (aes/des/rc4/hmac/sha)
├── har/       HAR 解析/对比/重放
├── crawl/     多协议爬虫 (HTTP/WS)
├── calc/      手工逆向计算
├── export/    多格式代码导出
├── render/    双内核无头渲染 (Chrome/Firefox)
├── mcp/       MCP Server (41工具)
├── app_reverse/ 小程序/APK/protobuf
├── tools/     辅助工具
└── config/    配置持久化
```

## 三模式切换

```bash
--mode manual    # 纯手工: 禁用自动提取/解密
--mode semi      # 半自动: 基础解析, 执行前确认
--mode auto      # 全自动: 一键批量 (默认)
```

## 本地测试

### 编译

```bash
cargo build --release
# 零 warnings 编译通过
# 二进制位于 target/release/reptool (Linux/macOS) 或 target/release/reptool.exe (Windows)
```

### 功能测试

```bash
# 加密工具
reptool crypto hash --algo md5 --input "test"
reptool crypto hmac --algo sha256 --key "secret" --data "hello"
reptool crypto encrypt --algo aes-cbc --key "1234567890123456" --iv "0000000000000000" --data "hello"

# 签名计算
reptool calc sign-sort '{"a":"1","b":"2"}' --salt mykey
reptool calc timestamp

# JS 逆向
reptool js hook-generate getSign --hook-type sign
reptool js analyze-js app.js

# Frida Hook
reptool hook generate com.example.App --hook-type ssl
reptool hook generate com.example.App --hook-type crypto

# AI 分析
reptool ai analyze-js app.js

# 代理
reptool proxy start -p 8080
reptool proxy cert --output ca.cer

# MCP Server
reptool mcp
```

### npm 本地安装测试

```bash
# 1. 把二进制复制到 npm 目录
cp target/release/reptool npm/bin/        # Linux/macOS
cp target/release/reptool.exe npm/bin/    # Windows

# 2. 打包
cd npm && npm pack

# 3. 全局安装测试
npm install -g ./reptool-0.1.0.tgz

# 4. 验证
reptool --version
reptool crypto hash --algo md5 --input "hello"

# 5. 卸载
npm uninstall -g reptool
```

## 发布到 npm

### 前置准备

```bash
# 注册 npm 账号
npm adduser

# 或设置 registry
npm config set registry https://registry.npmjs.org/
```

### 一键发布

```bash
# Linux/macOS
bash publish.sh 1.0.0

# Windows
publish.bat 1.0.0
```

### 手动发布

```bash
# 1. 更新版本号 (修改 Cargo.toml 和 npm/package.json)

# 2. 编译
cargo build --release

# 3. 复制二进制
cp target/release/reptool npm/bin/        # Linux/macOS
cp target/release/reptool.exe npm/bin/    # Windows

# 4. 打包 Windows 发布包
zip reptool-win-x64.zip target/release/reptool.exe

# 5. 发布 npm
cd npm && npm publish
```

### 用户安装

```bash
npm install -g reptool
reptool --help
reptool ai-config setup    # 配置 AI
```

### 全平台构建

```bash
# 安装交叉编译目标
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# 全平台构建
bash build-all.sh

# 产物在 release/ 目录
```

## License

MIT
