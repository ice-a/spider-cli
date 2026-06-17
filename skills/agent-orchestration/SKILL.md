# 智能体编排配置 (Agent Orchestration)

## 概述

定义多智能体协作模式，实现复杂逆向工程任务的自动化分解、并行执行、结果汇总。支持 Claude Code、Codex、Gemini 等多模型协作。

## 智能体角色定义

### 1. 协调者 (Coordinator)
- **职责**：任务分解、角色分配、结果汇总
- **能力**：全局视角、资源调度、质量把关
- **工具**：所有 MCP 工具的只读权限

### 2. 流量分析师 (Traffic Analyst)
- **职责**：HAR 解析、接口梳理、凭证提取
- **能力**：流量对比、协议识别、数据导出
- **工具**：`har_*`, `proxy_*`, `export_*`

### 3. JS 逆向专家 (JavaScript Reverse Engineer)
- **职责**：代码反混淆、算法定位、Hook 生成
- **能力**：静态分析、动态调试、代码生成
- **工具**：`js_*`, `export_*`

### 4. 加密专家 (Crypto Specialist)
- **职责**：算法识别、签名验证、密钥破解
- **能力**：分步验证、差异对比、复现编码
- **工具**：`crypto_*`, `calc_*`

### 5. 移动端专家 (Mobile Specialist)
- **职责**：小程序/APK 分析、Frida Hook、SSL Pinning 绕过
- **能力**：解包、资源提取、动态注入
- **工具**：`mini_*`, `proto_*`, Frida CLI

### 6. 执行者 (Executor)
- **职责**：请求重放、批量验证、自动化测试
- **能力**：HTTP 客户端、并发控制、结果收集
- **工具**：`crawl_*`, `export_*`

## 协作模式

### 模式 1：流水线模式 (Pipeline)

适用于顺序依赖的任务链。

```
流量分析师 → JS 逆向专家 → 加密专家 → 执行者
    ↓            ↓              ↓           ↓
  HAR 解析    代码反混淆     签名验证     请求重放
```

**示例任务**：破解某 API 的签名算法

```yaml
pipeline:
  - stage: traffic_analysis
    agent: Traffic Analyst
    tasks:
      - har_parse (path="capture.har")
      - har_extract_creds (path="capture.har")
      - har_filter (path="capture.har", regex="/api/sign")
    output: filtered_requests.json

  - stage: js_reverse
    agent: JavaScript Reverse Engineer
    input: filtered_requests.json
    tasks:
      - 下载 JS 文件
      - js_deobfuscate (technique="auto")
      - js_extract_keys
      - js_analyze_sign
    output: sign_algorithm.json

  - stage: crypto_analysis
    agent: Crypto Specialist
    input: sign_algorithm.json
    tasks:
      - crypto_hash (验证哈希算法)
      - calc_sign_sort (验证参数排序)
      - calc_step (分步验证流程)
    output: sign_logic.py

  - stage: execution
    agent: Executor
    input: sign_logic.py
    tasks:
      - crawl_http (重放请求)
      - 对比签名值
      - 生成最终报告
    output: final_report.md
```

### 模式 2：并行模式 (Parallel)

适用于独立无依赖的任务。

```
协调者分配任务
    ├─ 流量分析师 (分析 HAR 1)
    ├─ 流量分析师 (分析 HAR 2)
    ├─ JS 逆向专家 (反混淆 JS 1)
    └─ JS 逆向专家 (反混淆 JS 2)
协调者汇总结果
```

**示例任务**：分析多个接口的签名算法

```yaml
parallel:
  - task: analyze_login_api
    agent: Traffic Analyst + Crypto Specialist
    input: login.har
    output: login_sign.json

  - task: analyze_payment_api
    agent: Traffic Analyst + Crypto Specialist
    input: payment.har
    output: payment_sign.json

  - task: analyze_order_api
    agent: Traffic Analyst + Crypto Specialist
    input: order.har
    output: order_sign.json

merge:
  agent: Coordinator
  inputs: [login_sign.json, payment_sign.json, order_sign.json]
  output: unified_sign_logic.md
```

### 模式 3：迭代模式 (Iterative)

适用于需要反复验证的任务。

```
JS 逆向专家 → 加密专家 → 执行者
      ↑            ↓          ↓
      └───────── 验证失败 ←─────┘
```

**示例任务**：逐步破解混淆的签名算法

```yaml
iteration:
  max_rounds: 5
  steps:
    - stage: deobfuscate
      agent: JavaScript Reverse Engineer
      tasks:
        - js_deobfuscate (technique="decrypt")
        - js_extract_keys

    - stage: verify
      agent: Crypto Specialist
      tasks:
        - calc_sign_sort (尝试常见签名格式)
        - calc_diff_sign (对比差异)

    - stage: test
      agent: Executor
      tasks:
        - crawl_http (重放请求)

    - decision:
        if: 签名匹配
        then: 成功退出
        else: 下一轮迭代（使用不同的反混淆技术）
```

## 配置文件格式

### 项目配置：`.reptool/agents.yaml`

```yaml
# Reptool 智能体编排配置
version: "1.0"

# 智能体定义
agents:
  coordinator:
    role: Coordinator
    model: claude-sonnet-4-6
    tools: [all]
    readonly: true

  traffic_analyst:
    role: Traffic Analyst
    model: gemini-2.0-flash-exp
    tools: [har_*, proxy_*, export_*, crawl_*]

  js_expert:
    role: JavaScript Reverse Engineer
    model: gpt-5.5
    tools: [js_*, crypto_hash, calc_step]

  crypto_expert:
    role: Crypto Specialist
    model: claude-sonnet-4-6
    tools: [crypto_*, calc_*]

  mobile_expert:
    role: Mobile Specialist
    model: gpt-5.5
    tools: [mini_*, proto_*, js_extract_*]

  executor:
    role: Executor
    model: gemini-2.0-flash-exp
    tools: [crawl_*, export_*]

# 预设工作流
workflows:
  api_reverse:
    name: "API 签名逆向"
    mode: pipeline
    steps:
      - agent: traffic_analyst
        task: "解析 HAR 文件，提取 API 请求和凭证"
      - agent: js_expert
        task: "反混淆 JS 代码，定位签名函数"
      - agent: crypto_expert
        task: "验证签名算法，生成复现代码"
      - agent: executor
        task: "重放请求，验证签名正确性"

  mini_program_reverse:
    name: "小程序逆向"
    mode: parallel
    tasks:
      - agent: mobile_expert
        task: "解包 wxapkg，提取源码"
      - agent: js_expert
        task: "分析加密逻辑，提取密钥"
      - agent: traffic_analyst
        task: "梳理 API 接口"
    merge:
      agent: coordinator
      task: "汇总结果，生成完整报告"

  batch_api_analysis:
    name: "批量 API 分析"
    mode: parallel
    tasks:
      - agent: traffic_analyst
        input: "*.har"
        task: "并行分析所有 HAR 文件"
    merge:
      agent: coordinator
      task: "生成 API 清单和签名规则"
```

## 智能体通信协议

### 消息格式

```json
{
  "from": "traffic_analyst",
  "to": "crypto_expert",
  "task_id": "task_001",
  "type": "request",
  "payload": {
    "action": "verify_signature",
    "data": {
      "params": "{\"user\":\"test\"}",
      "sign": "abc123",
      "algorithm_hint": "md5"
    }
  },
  "context": {
    "previous_results": "...",
    "dependencies": ["task_000"]
  }
}
```

### 响应格式

```json
{
  "from": "crypto_expert",
  "to": "traffic_analyst",
  "task_id": "task_001",
  "type": "response",
  "status": "success",
  "result": {
    "algorithm": "md5",
    "salt": "secretKey",
    "verification": "passed",
    "code": "def sign(params): ..."
  },
  "next_action": "proceed_to_executor"
}
```

## 使用示例

### 示例 1：启动预设工作流

```bash
# 使用配置文件启动 API 逆向工作流
reptool agent run --workflow api_reverse --input capture.har
```

### 示例 2：动态分配任务

```python
# 在 Claude Code 中通过 MCP 调用
from reptool_mcp import Orchestrator

orchestrator = Orchestrator()

# 分配任务给流量分析师
result1 = orchestrator.delegate(
    agent="traffic_analyst",
    task="解析 capture.har 并提取登录接口"
)

# 根据结果分配给 JS 专家
if result1.found_js_urls:
    result2 = orchestrator.delegate(
        agent="js_expert",
        task=f"分析 {result1.js_urls[0]} 的签名逻辑",
        context=result1
    )

# 汇总结果
report = orchestrator.merge([result1, result2])
```

### 示例 3：人工介入模式

```yaml
# 半自动模式：在关键节点暂停等待人工确认
workflow:
  - agent: traffic_analyst
    task: "分析 HAR"
    auto: true

  - checkpoint: "人工确认"
    message: "发现 10 个 API 接口，是否继续分析签名？"
    options: ["全部分析", "仅分析登录接口", "跳过"]

  - agent: crypto_expert
    task: "根据用户选择分析签名"
    auto: true
```

## 最佳实践

### 1. 合理分配模型
- **快速任务**（HAR 解析、代码格式化）→ Gemini Flash
- **复杂分析**（反混淆、算法识别）→ GPT-5 / Claude Opus
- **协调汇总** → Claude Sonnet

### 2. 任务粒度
- 单个智能体的任务时长控制在 30 秒 - 2 分钟
- 过长任务拆分为多个子任务
- 过短任务合并避免通信开销

### 3. 错误处理
- 每个智能体设置超时（默认 5 分钟）
- 失败重试 3 次，超过后由协调者决策
- 保存中间结果，支持断点续传

### 4. 成本控制
- 优先使用 Flash 模型处理简单任务
- 仅在必要时调用 GPT-5 / Opus
- 启用结果缓存，避免重复计算

## 集成指南

### Claude Code 集成

在 `~/.claude/config.json` 中配置：

```json
{
  "mcpServers": {
    "reptool": {
      "command": "reptool",
      "args": ["mcp"]
    }
  },
  "agents": {
    "orchestration": {
      "enabled": true,
      "config_path": ".reptool/agents.yaml"
    }
  }
}
```

### Codex 集成

在项目根目录创建 `.codex/extensions.json`：

```json
{
  "extensions": [
    {
      "name": "reptool-orchestration",
      "type": "agent-system",
      "config": ".reptool/agents.yaml"
    }
  ]
}
```

## 故障排查

### 问题 1：智能体无响应
- 检查 MCP Server 是否启动：`reptool mcp`
- 查看日志：`~/.reptool/logs/agent.log`
- 验证工具权限：确认智能体有权限调用目标工具

### 问题 2：任务卡在某个阶段
- 查看任务状态：`reptool agent status --task-id <id>`
- 手动推进：`reptool agent retry --task-id <id>`
- 终止任务：`reptool agent cancel --task-id <id>`

### 问题 3：结果不符合预期
- 检查智能体角色分配是否合理
- 查看中间结果：`reptool agent logs --task-id <id>`
- 调整工作流配置，增加人工确认节点

## 参考资料

- [MCP 协议规范](https://spec.modelcontextprotocol.io/)
- [Multi-Agent Systems 设计模式](https://arxiv.org/abs/2308.10848)
- [LangGraph 多智能体编排](https://langchain-ai.github.io/langgraph/)
