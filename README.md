# Reptool

**Rust 全栈逆向/爬虫/代理 CLI 工具链 + Claude/Codex 智能体技能系统**

> 集成 mitmproxy + Burp + Frida + JsHook + AI 逆向助手 + 加密工具箱 + 爬虫框架 + MCP Server，通过智能体编排实现自动化逆向工程。

<div align="center">

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-41_tools-green.svg)](MCP_SETUP.md)
[![Skills](https://img.shields.io/badge/Skills-5_agents-purple.svg)](skills/README.md)

</div>

## 🎯 核心特性

### 智能体技能系统

- **5 大智能体角色** - 流量分析师 / JS 逆向专家 / 加密专家 / 移动端专家 / 协调者
- **41 个 MCP 工具** - 与 Claude Code / Codex / VS Code Copilot 深度集成
- **3 种编排模式** - 流水线 / 并行 / 迭代
- **自动化工作流** - 端到端 API 签名逆向，无需手动干预

### 传统 CLI 工具

- **MITM 代理** — HTTP/HTTPS 中间人拦截，自动 CA 证书生成，JS Hook 注入
- **Frida Hook** — 生成 SSL Pinning 绕过 / Crypto Hook / HTTP Hook / 存储 Hook 脚本
- **JS 逆向** — 格式化/反混淆/接口提取/签名分析/密钥提取/Hook 脚本生成
- **AI 逆向助手** — 支持 Claude/GPT/Gemini/小米/DeepSeek/通义/豆包 + 自定义 OpenAI 兼容
- **加密工具箱** — md5/sha/aes/des/rsa/hmac/base64/urlencode + 分步计算
- **HAR 分析** — 解析/对比/重放/过滤/凭证识别，导出 curl/python/fetch/postman 等 8 种格式
- **双内核渲染** — Chrome/Firefox 无头浏览器，指纹篡改，脚本注入
- **跨平台** — Windows / Linux / macOS 单二进制

## 🚀 快速开始

### 安装

#### 方式 1：npm 安装（推荐）

```bash
npm install -g reptool-cli
```

安装会自动下载对应平台的预编译二进制文件。

#### 方式 2：从源码编译

```bash
# 克隆仓库
git clone https://github.com/ice-a/spider-cli.git
cd spider-cli

# 编译（需要 Rust 1.70+）
cargo build --release

# 安装到系统 PATH
# Linux/macOS
sudo cp target/release/reptool /usr/local/bin/

# Windows
copy target\release\reptool.exe C:\Windows\System32\
```

#### 方式 3：下载预编译二进制

从 [Releases](https://github.com/ice-a/spider-cli/releases) 下载对应平台的二进制文件。

### 配置 MCP 服务器（Claude Code / Codex）

编辑 `~/.claude/config.json`（Claude Code）或 `~/.codex/config.json`（Codex）：

```json
{
  "mcpServers": {
    "reptool": {
      "command": "reptool",
      "args": ["mcp"],
      "description": "逆向工程工具链 - 41个工具"
    }
  }
}
```

### 验证安装

```bash
# 测试 MCP Server
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | reptool mcp

# 应返回包含 41 个工具的 JSON 列表
```

## 📚 智能体技能

完整的智能体技能系统，支持多智能体协作的逆向工程工作流。

### 技能列表

| 技能 | 描述 | 典型任务 |
|------|------|---------|
| [逆向工程智能体](skills/reverse-engineering/SKILL.md) | 端到端逆向流程 | Web/App 签名破解 |
| [JS 分析智能体](skills/js-analysis/SKILL.md) | 深度 JS 代码分析 | 反混淆 / 算法定位 |
| [加密工具智能体](skills/crypto-toolkit/SKILL.md) | 密码学验证破解 | 哈希识别 / 签名验证 |
| [流量分析智能体](skills/traffic-analysis/SKILL.md) | 流量捕获分析 | HAR 解析 / 代理抓包 |
| [智能体编排](skills/agent-orchestration/SKILL.md) | 多智能体协作 | 任务分解 / 并行执行 |

### 使用示例

在 **Claude Code** 中：

```
用户：分析这个 HAR 文件的签名算法

Claude 自动执行：
  1. har_parse - 解析文件提取请求
  2. js_extract_keys - 查找加密密钥
  3. calc_sign_sort - 验证参数排序规则
  4. 输出完整签名逻辑 + Python 复现代码
```

### 智能体编排示例

**流水线模式**（端到端 API 签名逆向）：

```
流量分析师 → JS 逆向专家 → 加密专家 → 执行者
    ↓            ↓              ↓           ↓
  HAR 解析    代码反混淆     签名验证     请求重放
```

**并行模式**（批量接口分析）：

```
协调者分配任务
  ├─ 流量分析师1（分析 login.har）
  ├─ 流量分析师2（分析 payment.har）
  └─ 流量分析师3（分析 order.har）
协调者汇总结果
```

详细配置参见：[智能体编排文档](skills/agent-orchestration/SKILL.md)

## 🛠️ MCP 工具清单

41 个工具分为 10 大类：

| 类别 | 工具数 | 典型工具 |
|------|-------|---------|
| **HAR 分析** | 5 | `har_parse`, `har_diff`, `har_extract_creds` |
| **JS 逆向** | 6 | `js_deobfuscate`, `js_extract_keys`, `js_hook_generate` |
| **加密工具** | 8 | `crypto_hash`, `crypto_encrypt`, `crypto_decrypt` |
| **签名计算** | 3 | `calc_sign_sort`, `calc_step`, `calc_diff_sign` |
| **代理抓包** | 3 | `proxy_start`, `proxy_stop`, `proxy_get_sessions` |
| **HTTP 客户端** | 2 | `crawl_http`, `crawl_ws_connect` |
| **代码导出** | 4 | `export_curl`, `export_python`, `export_fetch` |
| **辅助工具** | 4 | `tools_dns`, `tools_port_check`, `tools_json_format` |
| **移动端逆向** | 3 | `mini_wxapkg_parse`, `mini_apk_extract`, `proto_decode` |
| **配置管理** | 2 | `config_get`, `config_set` |

完整工具清单：[MCP 配置文档](MCP_SETUP.md)

## 💡 使用场景

### 场景 1：Web API 签名逆向

**目标**：破解某电商平台的 API 签名算法

**传统方式**（手动）：
1. 浏览器抓包导出 HAR
2. 手动查找签名参数
3. 下载 JS 文件反混淆
4. 阅读代码定位签名函数
5. 手写 Python 复现代码
6. 测试验证

**智能体方式**（自动）：
```
用户输入：分析 capture.har 的签名算法

流水线自动执行：
  流量分析师：har_parse + har_filter → 找到签名接口
  JS 逆向专家：js_deobfuscate + js_analyze_sign → 定位签名函数
  加密专家：calc_sign_sort → 验证签名逻辑
  执行者：生成 Python 代码并测试

输出：完整签名算法 + 可执行代码
```

### 场景 2：小程序逆向

```bash
# 传统 CLI 方式
reptool mini wxapkg-parse --file mini.wxapkg --output ./output
reptool js extract-apis --dir ./output
reptool js extract-keys --dir ./output

# 智能体方式
用户：分析这个小程序的 API 和加密
智能体：自动解包 + 提取接口 + 分析加密 → 生成完整报告
```

### 场景 3：实时代理抓包

```bash
# 启动 MITM 代理
reptool proxy --port 8080 --filter "api.target.com"

# 配置手机代理后，使用 MCP 工具获取会话
# 在 Claude Code 中：
用户：获取最近的登录请求
Claude：proxy_get_sessions(filter="login") → 自动解析并分析签名
```

## 📦 CLI 命令速查

### 代理抓包

```bash
reptool proxy --port 8080                          # 启动代理
reptool proxy --port 8080 --filter "api.example.com"  # 过滤域名
reptool proxy --hook-fetch                        # 注入 Hook
```

### JS 逆向

```bash
reptool js format --file app.js                   # 格式化
reptool js deobfuscate --file app.obf.js --technique auto  # 反混淆
reptool js extract-apis --file app.js             # 提取 API
reptool js extract-keys --file app.js             # 提取密钥
reptool js hook-generate --function sign --type sign  # 生成 Hook
```

### 加密工具

```bash
reptool crypto hash --algo md5 --input "hello"    # 哈希计算
reptool crypto encrypt --algo aes-cbc --key "1234567890123256" --data "hello"
reptool calc sign-sort --params '{"user":"test"}' --salt "abc"
```

### HAR 分析

```bash
reptool har parse --file capture.har              # 解析 HAR
reptool har diff --old old.har --new new.har      # 对比差异
reptool har extract-creds --file capture.har      # 提取凭证
reptool har export --file capture.har --format python  # 导出代码
```

### 移动端逆向

```bash
reptool mini wxapkg-parse --file mini.wxapkg      # 小程序解包
reptool mini apk-extract --file app.apk           # APK 提取
reptool hook ssl-pinning --platform android       # Frida Hook
```

完整命令列表：`reptool --help`

## 🔧 集成支持

### 支持的工具

| 工具 | 配置文件 | 状态 |
|------|---------|------|
| **Claude Code** | `~/.claude/config.json` | ✅ 完整支持 |
| **OpenAI Codex** | `~/.codex/config.json` | ✅ 完整支持 |
| **VS Code Copilot** | `.vscode/settings.json` | ✅ MCP 支持 |
| **Cursor** | `.vscode/settings.json` | ✅ MCP 支持 |
| **Windsurf** | `~/.codeium/config.yaml` | ✅ MCP 支持 |
| **Zed Editor** | `~/.config/zed/settings.json` | ✅ MCP 支持 |
| **Cline** | 插件设置 | ✅ MCP 支持 |

详细配置：[MCP 配置文档](MCP_SETUP.md)

## 📖 文档

- [技能系统完整文档](skills/README.md)
- [MCP 服务器配置](MCP_SETUP.md)
- [逆向工程智能体](skills/reverse-engineering/SKILL.md)
- [智能体编排指南](skills/agent-orchestration/SKILL.md)

## 🛡️ 安全提示

⚠️ **仅用于合法授权的安全测试和学习研究**

- MITM 代理仅在测试环境使用
- 不要拦截生产环境流量
- HAR 文件包含敏感凭证，不要提交到版本控制
- 爬虫/逆向需遵守目标网站的 robots.txt 和服务条款

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

- 新增 MCP 工具
- 新增智能体技能
- 改进工作流模板
- 修复 Bug

## 📝 许可证

MIT License - 详见 [LICENSE](LICENSE)

## 🔗 相关资源

- [MCP 协议规范](https://spec.modelcontextprotocol.io/)
- [Claude Code 文档](https://docs.anthropic.com/claude-code)
- [Frida 官方文档](https://frida.re/docs/)
- [OWASP 移动安全测试指南](https://owasp.org/www-project-mobile-security-testing-guide/)

## ⭐ Star History

如果这个项目对你有帮助，欢迎 Star！

---

**注意**：原 npm 包 `reptool` 已停止维护，请使用本地编译方式安装。MCP 服务器 + Skills 智能体系统是推荐的使用方式。
