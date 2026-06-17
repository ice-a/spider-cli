# Reptool Skills 索引

## 概述

Reptool Skills 是一套完整的逆向工程智能体技能系统，通过 MCP 协议与 Claude、Codex、VS Code 等工具深度集成，实现自动化的流量分析、JS 反混淆、加密破解、代理抓包。

## 核心特性

✅ **41 个 MCP 工具** - 覆盖逆向工程全流程  
✅ **5 大智能体角色** - 专业分工，高效协作  
✅ **3 种编排模式** - 流水线/并行/迭代  
✅ **跨平台支持** - Windows/Linux/macOS  
✅ **多模型兼容** - Claude/GPT/Gemini/Codex

## 快速开始

### 1. 编译安装

```bash
git clone https://github.com/ice-a/spider-cli.git
cd spider-cli
cargo build --release
sudo cp target/release/reptool /usr/local/bin/
```

### 2. 配置 MCP 服务器

编辑 `~/.claude/config.json`：

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

### 3. 加载技能

```bash
# Claude Code
claude code

# 使用技能
用户：分析这个 HAR 文件的签名算法
Claude：自动调用 har_parse, js_extract_keys, calc_sign_sort 等工具
```

## 技能列表

### 1. [逆向工程智能体](./reverse-engineering/SKILL.md)

**适用场景**：Web/App 完整逆向流程

**核心能力**：
- 流量拦截与分析（MITM 代理 + HAR 解析）
- JavaScript 逆向（反混淆 + 签名分析）
- 加密分析（算法识别 + 密钥提取）
- 移动端逆向（小程序/APK/Frida Hook）

**典型任务**：
- 破解 API 签名算法
- 分析小程序加密逻辑
- 绕过 SSL Pinning

### 2. [JS 分析智能体](./js-analysis/SKILL.md)

**适用场景**：JavaScript 代码深度分析

**核心能力**：
- 代码反混淆（控制流还原 + 字符串解密）
- 加密算法识别（CryptoJS/jsrsasign/自定义实现）
- 签名算法分析（参数排序 + 拼接规则）
- Hook 脚本生成（Proxy/defineProperty/XHR）

**典型任务**：
- 反混淆 obfuscator.io 代码
- 定位签名函数调用链
- 生成浏览器验证脚本

### 3. [加密工具智能体](./crypto-toolkit/SKILL.md)

**适用场景**：密码学算法验证与破解

**核心能力**：
- 哈希算法（MD5/SHA1/SHA256/SHA512）
- 对称加密（AES/DES/RC4）
- HMAC 签名
- 分步计算器（逐步验证加密流程）

**典型任务**：
- 识别哈希算法（根据输出长度）
- 破解参数签名（字典排序 + 盐值）
- 验证 AES 加密逻辑

### 4. [流量分析智能体](./traffic-analysis/SKILL.md)

**适用场景**：HTTP/HTTPS 流量捕获与分析

**核心能力**：
- HAR 文件解析（提取 API/凭证/参数）
- 流量对比（发现动态参数）
- 代码导出（curl/Python/Go/Java）
- MITM 代理（实时抓包 + JS Hook 注入）

**典型任务**：
- 提取 API 接口清单
- 对比两次请求差异
- 实时抓包 App 流量

### 5. [智能体编排](./agent-orchestration/SKILL.md)

**适用场景**：复杂任务的多智能体协作

**核心能力**：
- 角色定义（协调者/分析师/专家/执行者）
- 三种模式（流水线/并行/迭代）
- 任务分解与汇总
- 错误处理与重试

**典型任务**：
- 批量 API 分析（并行模式）
- 端到端签名逆向（流水线模式）
- 迭代破解混淆代码（迭代模式）

## 使用场景

### 场景 1：Web API 签名逆向（流水线模式）

```
流量分析师：解析 HAR，提取登录接口
    ↓
JS 逆向专家：反混淆代码，定位签名函数
    ↓
加密专家：验证签名算法，生成复现代码
    ↓
执行者：重放请求，验证签名正确性
```

### 场景 2：小程序批量分析（并行模式）

```
协调者：分配任务
  ├─ 移动端专家1：解包 wxapkg_1
  ├─ 移动端专家2：解包 wxapkg_2
  └─ 移动端专家3：解包 wxapkg_3
协调者：汇总结果，生成 API 清单
```

### 场景 3：混淆代码迭代破解（迭代模式）

```
Round 1: JS专家反混淆(decrypt) → 加密专家验证 → 失败
Round 2: JS专家反混淆(control_flow) → 加密专家验证 → 失败
Round 3: JS专家反混淆(eval) → 加密专家验证 → 成功
```

## 配置文件

### 项目配置：`.reptool/agents.yaml`

```yaml
version: "1.0"

agents:
  traffic_analyst:
    role: Traffic Analyst
    model: gemini-2.0-flash-exp
    tools: [har_*, proxy_*, export_*]

  js_expert:
    role: JavaScript Reverse Engineer
    model: gpt-5.5
    tools: [js_*, crypto_hash, calc_step]

  crypto_expert:
    role: Crypto Specialist
    model: claude-sonnet-4-6
    tools: [crypto_*, calc_*]

workflows:
  api_reverse:
    name: "API 签名逆向"
    mode: pipeline
    steps:
      - agent: traffic_analyst
      - agent: js_expert
      - agent: crypto_expert
```

## 最佳实践

### 1. 选择合适的模式
- **简单任务**（单个 API 分析）→ 单智能体模式
- **顺序依赖**（端到端逆向）→ 流水线模式
- **独立任务**（批量分析）→ 并行模式
- **复杂破解**（多次尝试）→ 迭代模式

### 2. 合理分配模型
- **快速任务** → Gemini Flash
- **复杂分析** → GPT-5 / Claude Opus
- **协调汇总** → Claude Sonnet

### 3. 任务粒度控制
- 单个任务时长：30 秒 - 2 分钟
- 过长任务拆分为子任务
- 保存中间结果支持断点续传

### 4. 安全与权限
- MITM 代理仅在测试环境使用
- HAR 文件不要提交到版本控制
- 危险操作（proxy_start, crawl_http）需手动确认

## 工具清单

### 按类别分类（41 个工具）

| 类别 | 工具数 | 典型工具 |
|------|-------|---------|
| HAR 分析 | 5 | har_parse, har_diff, har_extract_creds |
| JS 逆向 | 6 | js_deobfuscate, js_extract_keys, js_hook_generate |
| 加密工具 | 8 | crypto_hash, crypto_encrypt, crypto_decrypt |
| 签名计算 | 3 | calc_sign_sort, calc_step, calc_diff_sign |
| 代理抓包 | 3 | proxy_start, proxy_stop, proxy_get_sessions |
| HTTP 客户端 | 2 | crawl_http, crawl_ws_connect |
| 代码导出 | 4 | export_curl, export_python, export_fetch |
| 辅助工具 | 4 | tools_dns, tools_port_check, tools_json_format |
| 移动端 | 3 | mini_wxapkg_parse, mini_apk_extract, proto_decode |
| 配置管理 | 2 | config_get, config_set |

## 安装与配置

详细安装指南请参考：
- [MCP 服务器配置](../MCP_SETUP.md)
- [编译指南](../README.md#安装)

## 故障排查

### 问题 1：工具无法调用
```bash
# 验证 MCP Server
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | reptool mcp
```

### 问题 2：智能体无响应
```bash
# 查看日志
tail -f ~/.reptool/logs/agent.log
```

### 问题 3：权限被拒绝
```json
// 添加自动允许配置
{
  "mcpServers": {
    "reptool": {
      "alwaysAllow": ["har_parse", "js_format", "crypto_hash"]
    }
  }
}
```

## 贡献指南

欢迎贡献新的技能定义和工作流模板：

1. Fork 仓库
2. 在 `skills/` 下创建新目录
3. 编写 `SKILL.md` 文档
4. 提交 Pull Request

## 许可证

MIT License - 详见 [LICENSE](../LICENSE)

## 相关资源

- [GitHub 仓库](https://github.com/ice-a/spider-cli)
- [MCP 协议规范](https://spec.modelcontextprotocol.io/)
- [Claude Code 文档](https://docs.anthropic.com/claude-code)
- [问题反馈](https://github.com/ice-a/spider-cli/issues)
