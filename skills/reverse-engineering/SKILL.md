# 逆向工程智能体 (Reverse Engineering Agent)

## 技能概述

专注于 Web/App 逆向分析的智能体技能，整合 MITM 代理、流量分析、JS 反混淆、加密算法识别等能力，提供端到端的逆向工程工作流。

## 核心能力

### 1. 流量拦截与分析
- MITM 代理启动与管理
- HAR 文件解析与凭证提取
- 请求/响应差异对比
- API 端点自动发现

### 2. JavaScript 逆向
- JS 代码反混淆（控制流平坦化、字符串解密、死代码消除）
- 签名算法定位与分析
- 密钥/盐值/IV 自动提取
- Hook 脚本生成（fetch/XHR/WebSocket）

### 3. 加密分析
- 常见加密算法识别（AES/DES/RSA/RC4/HMAC）
- 分步加密计算器（逐步验证签名流程）
- 签名差异对比（参数排序/拼接/哈希）
- 时间戳/随机数生成器

### 4. 移动端逆向
- 微信小程序 wxapkg 解包
- APK URL/密钥提取
- Protobuf 解码
- Frida Hook 脚本生成（SSL Pinning 绕过、存储 Hook）

## MCP 工具清单

此技能依赖以下 MCP 工具（通过 `reptool mcp` 启动服务）：

### 流量分析工具
- `proxy_start` - 启动 MITM 代理，自动生成 CA 证书
- `proxy_get_sessions` - 获取抓包会话，支持正则过滤
- `har_parse` - 解析 HAR 文件，提取 cookies/headers/params
- `har_diff` - 对比两份 HAR 的请求差异
- `har_extract_creds` - 自动识别登录凭证（token/session/apikey）
- `har_filter` - 按方法/正则过滤请求

### JS 逆向工具
- `js_format` - JS 格式化/美化/压缩
- `js_deobfuscate` - 反混淆（auto/decrypt/control_flow/eval/dead_code）
- `js_extract_apis` - 提取所有 API 端点
- `js_analyze_sign` - 分析签名函数调用链
- `js_extract_keys` - 提取密钥/盐值/IV/AppID
- `js_hook_generate` - 生成 Hook 脚本（sign/fetch/auto）

### 加密工具
- `crypto_hash` - 哈希计算（md5/sha1/sha256/sha512）
- `crypto_hmac` - HMAC 签名
- `crypto_encrypt` / `crypto_decrypt` - 对称加密/解密
- `calc_sign_sort` - 参数字典排序签名
- `calc_step` - 分步加密计算器
- `calc_diff_sign` - 签名差异对比

### 移动端工具
- `mini_wxapkg_parse` - 微信小程序解包
- `mini_apk_extract` - APK 资源提取
- `proto_decode` - Protobuf 解码

## 使用场景

### 场景 1：Web 签名逆向

**目标**：破解某电商平台的 API 签名算法

**工作流**：

1. **启动代理抓包**
   ```
   使用 proxy_start 启动 MITM 代理（端口 8080）
   → 配置浏览器代理
   → 手动触发目标请求
   ```

2. **提取签名参数**
   ```
   使用 proxy_get_sessions 获取抓包记录
   → 使用 har_parse 解析请求参数
   → 识别签名字段（如 sign, signature, token）
   ```

3. **定位签名代码**
   ```
   下载目标网站的 JS 文件
   → 使用 js_extract_keys 查找密钥/盐值
   → 使用 js_analyze_sign 分析签名函数
   ```

4. **反混淆代码**
   ```
   使用 js_deobfuscate (technique=auto) 反混淆
   → 使用 js_format 格式化代码
   → 人工审计签名逻辑
   ```

5. **验证签名算法**
   ```
   使用 calc_sign_sort 测试参数排序
   → 使用 calc_step 分步验证加密流程
   → 使用 calc_diff_sign 对比两次签名差异
   ```

6. **生成 Hook 脚本**
   ```
   使用 js_hook_generate (hook_type=sign) 生成拦截脚本
   → 在浏览器控制台注入
   → 实时打印签名输入/输出
   ```

### 场景 2：小程序逆向

**目标**：分析某小程序的 API 接口和加密方式

**工作流**：

1. **解包小程序**
   ```
   从手机提取 .wxapkg 文件
   → 使用 mini_wxapkg_parse 解包
   → 获得完整源码
   ```

2. **提取 API 端点**
   ```
   使用 js_extract_apis 扫描所有 .js 文件
   → 获得完整接口列表
   ```

3. **分析加密逻辑**
   ```
   使用 js_extract_keys 查找 AppKey/AppSecret
   → 使用 js_analyze_sign 定位签名函数
   → 使用 crypto_hash/crypto_hmac 验证签名
   ```

4. **重放请求**
   ```
   使用 crawl_http 重放接口请求
   → 使用 export_python 生成调用代码
   ```

### 场景 3：APP SSL Pinning 绕过

**目标**：绕过 Android APP 的证书锁定

**工作流**：

1. **生成 Frida Hook 脚本**
   ```
   reptool hook ssl-pinning --platform android --output bypass.js
   ```

2. **注入 Hook**
   ```
   frida -U -f com.example.app -l bypass.js
   → 观察证书验证流程
   ```

3. **配合 MITM 代理抓包**
   ```
   使用 proxy_start 启动代理
   → 手机配置代理 + 安装 CA 证书
   → 成功抓取 HTTPS 流量
   ```

## 智能体协作模式

### 模式 1：单智能体模式（简单任务）

适用于单一目标的快速逆向（如单个 API 签名）：

```
用户输入：分析这个 HAR 文件的签名算法

智能体执行：
1. har_parse 解析请求
2. js_extract_keys 查找密钥
3. calc_sign_sort 验证签名
4. 输出完整签名算法
```

### 模式 2：多智能体协作（复杂项目）

适用于多接口、多模块的完整逆向：

**角色分配**：
- **流量分析师** - 负责 HAR 解析、API 梳理、凭证提取
- **JS 逆向专家** - 负责反混淆、代码审计、Hook 生成
- **加密专家** - 负责算法识别、签名验证、密钥破解
- **协调者** - 整合结果、生成最终报告

**协作流程**：
```
协调者：收到任务 → 分解为 3 个子任务
  ├─ 流量分析师：解析 HAR + 提取 API 列表
  ├─ JS 逆向专家：反混淆 + 定位签名函数
  └─ 加密专家：验证算法 + 生成复现代码

协调者：汇总结果 → 生成完整逆向报告
```

## 最佳实践

### 1. 分步验证，避免盲目猜测
- 先用 `calc_step` 分步验证每一步加密
- 使用 `calc_diff_sign` 对比差异而非猜测参数

### 2. 善用 AI 辅助
- 使用 `ai` 命令让 AI 分析复杂混淆代码
- 配置 Claude/GPT 提供反混淆建议

### 3. 保留原始数据
- 所有 HAR 文件保存为证据
- Hook 脚本版本化管理

### 4. 自动化重复任务
- 批量接口用 `har_export` 生成代码
- 签名算法封装为 Python 模块

## 配置要求

### MCP Server 配置

**Claude Code** (`~/.claude/config.json`):
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

**VS Code / Cursor** (`.vscode/settings.json`):
```json
{
  "github.copilot.chat.codeGeneration.instructions": [
    {
      "text": "Use reptool MCP tools for reverse engineering tasks"
    }
  ]
}
```

### 前置依赖

- Rust 二进制：从源码编译 `cargo build --release`
- CA 证书：首次使用 `proxy_start` 时自动生成
- Frida（可选）：用于移动端 Hook

## 故障排查

### 问题 1：MCP 工具无响应
```bash
# 测试 MCP Server
reptool mcp
# 输入：{"jsonrpc":"2.0","method":"tools/list","id":1}
# 应返回 41 个工具列表
```

### 问题 2：代理证书不受信任
```bash
# 导出 CA 证书
reptool proxy start --port 8080
# 证书位于：~/.reptool/ca.crt
# 手动导入到系统/浏览器信任列表
```

### 问题 3：JS 反混淆效果不佳
```bash
# 尝试不同技术
reptool js deobfuscate --file app.js --technique decrypt
reptool js deobfuscate --file app.js --technique control_flow
reptool js deobfuscate --file app.js --technique eval
```

## 参考资料

- [MCP 协议规范](https://spec.modelcontextprotocol.io/)
- [Frida 官方文档](https://frida.re/docs/)
- [OWASP 移动安全测试指南](https://owasp.org/www-project-mobile-security-testing-guide/)
