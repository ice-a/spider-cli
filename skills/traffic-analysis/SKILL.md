# 流量分析智能体 (Traffic Analysis Agent)

## 技能概述

专注于 HTTP/HTTPS 流量的捕获、分析和重放。解析 HAR 文件，提取接口信息、凭证、参数模式，对比请求差异，导出为多种开发语言的可执行代码。

## 核心能力

### 1. HAR 文件解析
- **完整解析** - 提取所有请求/响应详情
- **选择性提取** - cookies/headers/params/urls
- **凭证识别** - 自动发现登录信息、token、session
- **请求过滤** - 按 method/URL 正则筛选

### 2. 流量对比
- **HAR 差异对比** - 两份抓包文件逐请求对比
- **参数差异** - 发现动态变化的字段
- **签名差异** - 定位签名算法的输入变量

### 3. 代码导出
- **curl** - 命令行直接复现
- **Python** - requests 库代码
- **JavaScript** - fetch API 代码
- **Go** - net/http 代码
- **Java** - OkHttp 代码
- **Postman** - Collection JSON 导入

### 4. MITM 代理
- **实时抓包** - HTTP/HTTPS 中间人拦截
- **自动 CA** - 证书自动生成和管理
- **JS Hook 注入** - 在响应中注入自定义脚本
- **过滤规则** - 白名单/黑名单模式
- **回调转发** - 将请求实时转发到指定 URL

### 5. HTTP 客户端
- **单次请求** - 支持所有 HTTP 方法
- **自定义 Headers** - 完整请求头控制
- **WebSocket** - 连接、发送、监听

## MCP 工具清单

### HAR 分析
- `har_parse` - 解析 HAR 文件
  - 参数：`path`, `extract`（cookies/headers/params/urls）
- `har_diff` - 对比两份 HAR 文件差异
  - 参数：`old`, `new`
- `har_extract_creds` - 识别 HAR 中的登录凭证
  - 参数：`path`
- `har_filter` - 过滤 HAR 请求
  - 参数：`path`, `method`, `regex`
- `har_export` - 导出为指定格式代码
  - 参数：`path`, `format`（curl/python/fetch/go/java/postman）

### 代理控制
- `proxy_start` - 启动 MITM 代理
  - 参数：`port`, `filter`, `callback_url`, `hook_fetch`
- `proxy_stop` - 停止代理
- `proxy_get_sessions` - 获取抓包会话
  - 参数：`filter`, `limit`

### HTTP 客户端
- `crawl_http` - 发送 HTTP 请求
  - 参数：`url`, `method`, `headers`, `body`
- `crawl_ws_connect` - WebSocket 连接
  - 参数：`url`, `message`, `listen`

### 代码导出
- `export_curl` - 生成 curl 命令
- `export_python` - 生成 Python requests 代码
- `export_fetch` - 生成 JS fetch 代码
- `export_go` - 生成 Go HTTP 代码
  - 通用参数：`url`, `method`, `headers`, `body`

## 使用场景

### 场景 1：分析 API 接口

**目标**：从浏览器抓包的 HAR 中提取所有 API 接口

**步骤**：

1. **解析 HAR 文件**
   ```
   使用 har_parse (path="network.har", extract="urls")
   → 输出所有请求 URL 列表
   ```

2. **过滤 API 请求**
   ```
   使用 har_filter (path="network.har", regex="/api/")
   → 筛选出 API 路径
   ```

3. **提取认证信息**
   ```
   使用 har_extract_creds (path="network.har")
   → 输出：
     Token: Bearer eyJ...
     Cookie: session=abc123
   ```

4. **导出为可执行代码**
   ```
   使用 har_export (path="network.har", format="python")
   → 生成完整的 Python requests 脚本
   ```

### 场景 2：对比两次请求差异

**目标**：找出什么参数是动态生成的

**步骤**：

1. **抓包两次相同操作**
   ```
   第一次操作 → first.har
   第二次操作 → second.har
   ```

2. **差异对比**
   ```
   使用 har_diff (old="first.har", new="second.har")
   → 输出：
     变化的字段：
       - timestamp: 1234567890 → 1234567900
       - nonce: "abc123" → "def456"
       - sign: "xxx" → "yyy"
   ```

3. **定位动态参数**
   ```
   已识别动态参数：timestamp, nonce, sign
   → 接下来可以交给加密工具智能体分析签名逻辑
   ```

### 场景 3：实时代理抓包

**目标**：实时捕获 App 的 API 请求

**步骤**：

1. **启动代理**
   ```
   使用 proxy_start (
     port=8080,
     filter="api.target.com",
     hook_fetch=true
   )
   → 代理启动于 0.0.0.0:8080
   → CA 证书：~/.reptool/ca.pem
   ```

2. **配置设备代理**
   ```
   手机/模拟器设置 HTTP 代理：
   - 地址：电脑 IP
   - 端口：8080
   - 安装 CA 证书
   ```

3. **获取抓包数据**
   ```
   使用 proxy_get_sessions (filter="login", limit=10)
   → 输出最近 10 条包含 "login" 的请求
   ```

4. **分析并导出**
   ```
   使用 export_curl (
     url="https://api.target.com/login",
     method="POST",
     headers={"Content-Type": "application/json"},
     body='{"user":"test","pass":"123"}'
   )
   → curl -X POST 'https://api.target.com/login' -H 'Content-Type: application/json' -d '{"user":"test","pass":"123"}'
   ```

### 场景 4：WebSocket 调试

**目标**：分析 WebSocket 通信协议

**步骤**：

1. **连接 WebSocket**
   ```
   使用 crawl_ws_connect (
     url="wss://api.target.com/ws",
     message='{"type":"ping"}',
     listen=true
   )
   → 连接成功
   → 收到：{"type":"pong","ts":1234567890}
   ```

2. **发送认证消息**
   ```
   使用 crawl_ws_connect (
     url="wss://api.target.com/ws",
     message='{"type":"auth","token":"eyJ..."}'
   )
   → 收到：{"type":"auth_ok","user_id":"123"}
   ```

### 场景 5：批量请求重放

**目标**：从 HAR 中提取请求并批量重放验证

**步骤**：

1. **过滤目标请求**
   ```
   使用 har_filter (path="capture.har", method="POST", regex="/api/order")
   → 找到 5 个订单相关请求
   ```

2. **逐个重放**
   ```
   使用 crawl_http (
     url="https://api.target.com/api/order/create",
     method="POST",
     headers={"Authorization": "Bearer eyJ..."},
     body='{"item_id":"123","qty":1}'
   )
   → HTTP 200: {"order_id":"456","status":"created"}
   ```

## 流量分析工作流

### 典型逆向流程

```
┌─────────────────────────────────────────────────────────┐
│                    流量分析工作流                          │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  1. 抓包         proxy_start / 浏览器 DevTools          │
│       ↓                                                 │
│  2. 导出         保存 HAR 文件                           │
│       ↓                                                 │
│  3. 解析         har_parse → 提取 URL/Headers/Params    │
│       ↓                                                 │
│  4. 对比         har_diff → 找动态参数                   │
│       ↓                                                 │
│  5. 定位         har_filter → 筛选目标请求               │
│       ↓                                                 │
│  6. 分析         → 交给加密/JS智能体分析签名             │
│       ↓                                                 │
│  7. 验证         crawl_http → 重放验证                   │
│       ↓                                                 │
│  8. 导出         export_python → 生成复现代码            │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### 智能体协作

| 步骤 | 智能体 | 工具 |
|------|--------|------|
| 抓包/解析 | 流量分析 | proxy_*, har_* |
| JS 分析 | JS 分析 | js_* |
| 签名破解 | 加密工具 | crypto_*, calc_* |
| 代码生成 | 流量分析 | export_* |
| Hook 注入 | JS 分析 | js_hook_generate |

## 代理配置指南

### 基本使用

```bash
# 启动代理（默认 8080 端口）
reptool proxy --port 8080

# 仅抓取指定域名
reptool proxy --port 8080 --filter "api.example.com"

# Hook fetch 拦截
reptool proxy --port 8080 --hook-fetch

# 回调转发
reptool proxy --port 8080 --callback "http://localhost:3000/capture"
```

### CA 证书安装

```
证书位置：~/.reptool/ca.pem

iOS：
  1. 通过 Safari 下载 ca.pem
  2. 设置 → 通用 → VPN与设备管理 → 安装
  3. 设置 → 通用 → 关于 → 证书信任设置 → 开启

Android：
  1. 复制 ca.pem 到手机
  2. 设置 → 安全 → 加密与凭据 → 安装证书
  3. 或使用 Magisk 模块注入系统证书
```

## 最佳实践

### 1. HAR 文件管理
- 按功能模块分别抓包，避免一个 HAR 文件过大
- 命名规范：`{模块}_{日期}_{描述}.har`

### 2. 流量过滤
- 先用宽松过滤抓全量，再用 har_filter 精确筛选
- 排除静态资源（.js/.css/.png）减少噪音

### 3. 代理安全
- 代理仅在本地/受信网络使用
- 测试完毕及时 proxy_stop
- 不要在生产环境使用 MITM 代理

### 4. 凭证处理
- har_extract_creds 提取的凭证仅用于测试
- 不要将包含真实凭证的 HAR 文件提交到版本控制
