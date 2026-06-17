# Reptool MCP 服务器配置

## 概述

Reptool 提供 41 个 MCP 工具，通过 stdio 协议与 Claude、Codex、VS Code 等工具集成，实现逆向工程任务的智能化自动化。

## 快速开始

### 1. 编译安装

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

### 2. 验证安装

```bash
# 测试 MCP Server
reptool mcp

# 输入以下 JSON 测试（手动输入或管道输入）
{"jsonrpc":"2.0","method":"tools/list","id":1}

# 应返回 41 个工具的列表
```

### 3. 配置 MCP 客户端

#### Claude Code (推荐)

编辑 `~/.claude/config.json`：

```json
{
  "mcpServers": {
    "reptool": {
      "command": "reptool",
      "args": ["mcp"],
      "description": "逆向工程工具链 - HAR分析/JS反混淆/加密工具/代理抓包"
    }
  }
}
```

启动 Claude Code：

```bash
# 加载 MCP 服务器
claude code

# 或直接启动并跳过权限确认（仅开发模式）
claude code --dangerously-skip-permissions
```

#### OpenAI Codex

编辑 `~/.codex/config.json`：

```json
{
  "mcp_servers": [
    {
      "name": "reptool",
      "command": ["reptool", "mcp"],
      "enabled": true
    }
  ]
}
```

#### VS Code / Cursor

安装 GitHub Copilot 扩展后，编辑 `.vscode/settings.json`：

```json
{
  "github.copilot.advanced": {
    "mcp": {
      "servers": {
        "reptool": {
          "command": "reptool",
          "args": ["mcp"]
        }
      }
    }
  }
}
```

#### Windsurf (Codeium)

编辑 `~/.codeium/config.yaml`：

```yaml
mcp:
  servers:
    - name: reptool
      command: reptool
      args: [mcp]
```

#### Cline (VS Code 插件)

在 Cline 设置中添加 MCP 服务器：

```
命令：reptool
参数：mcp
```

#### Zed Editor

编辑 `~/.config/zed/settings.json`：

```json
{
  "context_servers": {
    "reptool": {
      "command": {
        "path": "reptool",
        "args": ["mcp"]
      }
    }
  }
}
```

## 工具分类

### 流量分析 (5 个工具)

| 工具名 | 描述 | 主要参数 |
|-------|------|---------|
| `har_parse` | 解析 HAR 文件 | path, extract |
| `har_diff` | 对比两份 HAR 差异 | old, new |
| `har_extract_creds` | 提取登录凭证 | path |
| `har_filter` | 过滤请求 | path, method, regex |
| `har_export` | 导出为代码 | path, format |

### JS 逆向 (6 个工具)

| 工具名 | 描述 | 主要参数 |
|-------|------|---------|
| `js_format` | 格式化/美化 | code/file_path, minify |
| `js_deobfuscate` | 反混淆 | file_path, technique |
| `js_extract_apis` | 提取 API | file_path |
| `js_analyze_sign` | 分析签名 | file_path, functions |
| `js_extract_keys` | 提取密钥 | file_path |
| `js_hook_generate` | 生成 Hook | function_name, hook_type |

### 加密工具 (8 个工具)

| 工具名 | 描述 | 主要参数 |
|-------|------|---------|
| `crypto_hash` | 哈希计算 | algorithm, input |
| `crypto_hmac` | HMAC 签名 | algorithm, key, data |
| `crypto_encrypt` | 对称加密 | algorithm, key, data, iv |
| `crypto_decrypt` | 对称解密 | algorithm, key, data, iv |
| `crypto_base64` | Base64 编解码 | action, data |
| `crypto_urlencode` | URL 编解码 | action, data |
| `crypto_timestamp` | 时间戳工具 | bits, offset |
| `crypto_random_ua` | 随机 UA | browser, count |

### 签名计算 (3 个工具)

| 工具名 | 描述 | 主要参数 |
|-------|------|---------|
| `calc_sign_sort` | 参数排序签名 | params, salt, algorithm |
| `calc_step` | 分步计算器 | steps, data |
| `calc_diff_sign` | 签名差异对比 | src_str, src_sign, dst_str, dst_sign |

### 代理抓包 (3 个工具)

| 工具名 | 描述 | 主要参数 |
|-------|------|---------|
| `proxy_start` | 启动 MITM 代理 | port, filter, callback_url, hook_fetch |
| `proxy_stop` | 停止代理 | - |
| `proxy_get_sessions` | 获取会话 | filter, limit |

### HTTP 客户端 (2 个工具)

| 工具名 | 描述 | 主要参数 |
|-------|------|---------|
| `crawl_http` | 发送请求 | url, method, headers, body |
| `crawl_ws_connect` | WebSocket | url, message, listen |

### 代码导出 (4 个工具)

| 工具名 | 描述 | 主要参数 |
|-------|------|---------|
| `export_curl` | 生成 curl | url, method, headers, body |
| `export_python` | 生成 Python | url, method, headers, body |
| `export_fetch` | 生成 JS | url, method, headers, body |
| `export_go` | 生成 Go | url, method, headers, body |

### 辅助工具 (4 个工具)

| 工具名 | 描述 | 主要参数 |
|-------|------|---------|
| `tools_dns` | DNS 解析 | domain |
| `tools_port_check` | 端口检测 | port |
| `tools_json_format` | JSON 格式化 | data |
| `tools_json_extract` | JSON 提取 | data, path |

### 移动端逆向 (3 个工具)

| 工具名 | 描述 | 主要参数 |
|-------|------|---------|
| `mini_wxapkg_parse` | 小程序解包 | path |
| `mini_apk_extract` | APK 提取 | path, extract_urls, extract_keys |
| `proto_decode` | Protobuf 解码 | path, schema |

### 配置管理 (2 个工具)

| 工具名 | 描述 | 主要参数 |
|-------|------|---------|
| `config_get` | 读取配置 | key |
| `config_set` | 设置配置 | key, value |

## 使用示例

### 在 Claude Code 中使用

```
用户：分析这个 HAR 文件的签名算法

Claude：
  1. 使用 har_parse 解析文件
  2. 使用 har_extract_creds 提取凭证
  3. 使用 js_extract_keys 查找密钥
  4. 使用 calc_sign_sort 验证签名
  5. 输出完整签名逻辑
```

### 在 VS Code Copilot 中使用

```
// 注释中引导 Copilot 调用 MCP 工具
// @reptool har_parse capture.har

// Copilot 自动生成调用代码
```

### 在 Codex CLI 中使用

```bash
codex "使用 reptool 分析 capture.har 中的登录接口"
```

## 权限管理

### 自动允许模式

在 Claude Code 配置中添加：

```json
{
  "mcpServers": {
    "reptool": {
      "command": "reptool",
      "args": ["mcp"],
      "alwaysAllow": [
        "har_parse",
        "har_filter",
        "js_format",
        "js_extract_*",
        "crypto_hash",
        "calc_*"
      ]
    }
  }
}
```

### 危险操作需确认

以下工具需要用户确认：

- `proxy_start` - 启动 MITM 代理（可能拦截敏感流量）
- `crawl_http` - 发送 HTTP 请求（可能触发业务逻辑）
- `mini_apk_extract` - APK 分析（可能包含敏感信息）

## 故障排查

### 问题 1：工具列表为空

```bash
# 检查 MCP Server 是否正常
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | reptool mcp

# 应返回包含 41 个工具的 JSON
```

### 问题 2：工具调用无响应

```bash
# 检查日志
reptool --log-level debug mcp

# 或查看系统日志
tail -f ~/.reptool/logs/mcp.log
```

### 问题 3：Claude Code 无法加载 MCP

```bash
# 验证配置文件格式
cat ~/.claude/config.json | jq

# 重启 Claude Code
claude code --reload
```

## 性能优化

### 1. 工具调用缓存

重复调用相同参数的工具（如 `crypto_hash`）会自动缓存结果。

### 2. 并行调用

多个独立工具可以并行调用：

```python
# Claude Code 会自动并行化
results = await asyncio.gather(
    har_parse("file1.har"),
    har_parse("file2.har"),
    har_parse("file3.har")
)
```

### 3. 批量操作

使用通配符批量处理：

```bash
# 批量解析 HAR
for file in *.har; do
  reptool mcp <<< '{"jsonrpc":"2.0","method":"tools/call","id":1,"params":{"name":"har_parse","arguments":{"path":"'$file'"}}}'
done
```

## 安全建议

1. **MITM 代理**：仅在测试环境使用，不要拦截生产流量
2. **HAR 文件**：包含敏感凭证，不要提交到版本控制
3. **MCP 权限**：生产环境禁用 `alwaysAllow`，每次手动确认
4. **API 请求**：`crawl_http` 可能触发业务逻辑，谨慎使用

## 升级指南

### 从 npm 包迁移

如果之前通过 npm 安装：

```bash
# 卸载 npm 包
npm uninstall -g reptool

# 编译安装最新版本
cd spider-cli
git pull
cargo build --release
sudo cp target/release/reptool /usr/local/bin/
```

### 配置迁移

旧版配置（npm）：
```json
{
  "mcpServers": {
    "reptool": {
      "command": "npx",
      "args": ["reptool", "mcp"]
    }
  }
}
```

新版配置（本地编译）：
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

## 参考资料

- [MCP 协议文档](https://spec.modelcontextprotocol.io/)
- [Reptool GitHub 仓库](https://github.com/ice-a/spider-cli)
- [Claude Code 官方文档](https://docs.anthropic.com/claude-code)
- [技能文档目录](./skills/)
