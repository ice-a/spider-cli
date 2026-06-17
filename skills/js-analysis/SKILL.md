# JS 分析智能体 (JavaScript Analysis Agent)

## 技能概述

专注于 JavaScript 代码的深度分析与反混淆，识别加密算法、提取密钥、生成 Hook 脚本，支持现代混淆技术（obfuscator.io、JScrambler、Webpack）的自动化处理。

## 核心能力

### 1. 代码反混淆
- **控制流平坦化还原** - 识别 switch-case 控制流混淆，还原原始逻辑
- **字符串解密** - 自动解密 base64/hex/unicode 编码的字符串常量
- **死代码消除** - 移除永远不执行的分支和无用代码
- **eval 展开** - 自动执行安全的 eval/Function 动态代码生成
- **变量重命名** - 将 `_0x1a2b3c` 类变量名映射为语义化名称

### 2. 加密算法识别
- **内置库检测** - 识别 CryptoJS、jsrsasign、forge、spark-md5 等库
- **自定义实现分析** - 通过特征码识别手写的 AES/RSA/HMAC 实现
- **混合加密链** - 追踪多层加密流程（如 RSA 加密 AES 密钥 → AES 加密数据）

### 3. 签名算法分析
- **参数排序规则** - 识别字典序/自定义排序
- **拼接格式** - 识别 key=value&key2=value2 或 key+value+key2
- **盐值定位** - 查找固定盐值、时间戳、随机数
- **哈希算法** - 识别 MD5/SHA1/SHA256/HMAC

### 4. 密钥提取
- **硬编码密钥** - 提取字符串常量中的 AES key/IV、RSA 公钥
- **动态生成密钥** - 分析密钥生成函数（如从 timestamp 派生）
- **分片密钥** - 识别分散在多个变量中的密钥片段

### 5. Hook 脚本生成
- **函数拦截** - 生成 Proxy/defineProperty Hook
- **XHR/Fetch Hook** - 拦截网络请求，打印参数和签名
- **WebSocket Hook** - 监听 WS 消息
- **加密函数 Hook** - 拦截 CryptoJS.AES.encrypt 等调用

## MCP 工具清单

### 代码处理
- `js_format` - 格式化/美化/压缩 JS 代码
- `js_deobfuscate` - 反混淆（5 种技术：auto/decrypt/control_flow/eval/dead_code）

### 静态分析
- `js_extract_apis` - 提取所有 API 端点（支持模板字符串、拼接）
- `js_analyze_sign` - 分析签名函数调用链，输出依赖图
- `js_extract_keys` - 提取密钥/盐值/IV/AppID/AppSecret

### Hook 生成
- `js_hook_generate` - 生成 Hook 脚本（sign/fetch/auto 三种模式）

### 辅助工具
- `crypto_hash` - 验证哈希结果
- `crypto_encrypt` / `crypto_decrypt` - 验证加密逻辑
- `calc_step` - 分步模拟加密流程

## 使用场景

### 场景 1：反混淆 obfuscator.io 代码

**挑战**：代码经过控制流平坦化 + 字符串加密

**步骤**：

1. **下载混淆代码**
   ```bash
   curl https://example.com/app.js > app.obf.js
   ```

2. **第一步：字符串解密**
   ```
   使用 js_deobfuscate (technique=decrypt) 解密字符串数组
   → 生成 app.step1.js
   ```

3. **第二步：控制流还原**
   ```
   使用 js_deobfuscate (technique=control_flow) 还原 switch-case
   → 生成 app.step2.js
   ```

4. **第三步：死代码消除**
   ```
   使用 js_deobfuscate (technique=dead_code) 移除无用分支
   → 生成 app.step3.js
   ```

5. **第四步：格式化**
   ```
   使用 js_format (minify=false) 美化代码
   → 获得可读的 app.final.js
   ```

**提示**：如果单次反混淆效果不佳，分步执行多次，每次应用一种技术。

### 场景 2：分析签名算法

**目标**：某 APP 的 API 签名为 `sign=md5(params_sorted + timestamp + "secretKey")`

**步骤**：

1. **定位签名函数**
   ```
   使用 js_extract_apis 提取所有请求 URL
   → 在代码中搜索 "sign" 关键字
   → 使用 js_analyze_sign (functions="generateSign,md5") 分析调用链
   ```

2. **提取密钥**
   ```
   使用 js_extract_keys 查找所有字符串常量
   → 输出：{ "secretKey": "abc123...", "salt": "xyz..." }
   ```

3. **验证签名流程**
   ```
   使用 calc_sign_sort (params='{"user":"test","id":"1"}', salt="abc123", algorithm="md5")
   → 输出：待签名字符串 + 最终签名
   → 对比抓包中的签名值
   ```

4. **生成 Hook 脚本**
   ```
   使用 js_hook_generate (function_name="generateSign", hook_type="sign")
   → 生成 console.log Hook
   → 在浏览器控制台执行，实时打印签名过程
   ```

### 场景 3：识别加密库

**目标**：判断代码使用的加密库和算法

**步骤**：

1. **特征码扫描**
   ```javascript
   // 使用 js_extract_keys 输出结果中查找特征
   CryptoJS.AES  → 使用 CryptoJS 库的 AES
   JSEncrypt     → 使用 JSEncrypt 库的 RSA
   forge.md.sha256 → 使用 node-forge 库的 SHA256
   ```

2. **自定义实现识别**
   ```javascript
   // 查找 AES 特征码
   S-box 常量：0x63, 0x7c, 0x77...
   轮密钥扩展：KeyExpansion
   加密模式：CBC/ECB/CTR

   // 查找 RSA 特征码
   modPow, BigInteger
   e=65537 (常见公钥指数)
   ```

3. **算法参数提取**
   ```
   使用 js_extract_keys 提取：
   - AES key (16/24/32 字节)
   - AES IV (16 字节)
   - RSA public key (PEM 格式)
   - HMAC key
   ```

### 场景 4：WebSocket 协议逆向

**目标**：分析 WS 消息的加密格式

**步骤**：

1. **查找 WebSocket 代码**
   ```
   使用 js_extract_apis 搜索 "ws://" 或 "wss://"
   → 定位 WebSocket 初始化代码
   ```

2. **分析消息处理**
   ```javascript
   // 搜索 ws.send / ws.onmessage
   使用 js_analyze_sign 分析消息加密函数
   ```

3. **生成 Hook 脚本**
   ```
   使用 js_hook_generate (function_name="ws.send", hook_type="auto")
   → 拦截所有发送的消息
   → 打印原始数据和加密后数据
   ```

4. **连接测试**
   ```
   使用 crawl_ws_connect (url="wss://example.com/ws", message="test", listen=true)
   → 发送测试消息
   → 观察服务器响应格式
   ```

## 反混淆技术详解

### 技术 1：字符串解密 (decrypt)

**适用场景**：代码中存在字符串数组 + 解密函数

**示例**：
```javascript
// 混淆前
var api = "https://api.example.com/login";

// 混淆后
var _0x1a2b = ['aHR0cHM6Ly9hcGkuZXhhbXBsZS5jb20vbG9naW4='];
var api = atob(_0x1a2b[0]);

// 反混淆后
var api = "https://api.example.com/login";
```

**原理**：识别 base64/hex/unicode 模式，自动解码字符串数组。

### 技术 2：控制流平坦化还原 (control_flow)

**适用场景**：代码被转换为 switch-case 状态机

**示例**：
```javascript
// 混淆前
console.log("step1");
console.log("step2");

// 混淆后
var state = 0;
while (true) {
  switch (state) {
    case 0: console.log("step1"); state = 1; break;
    case 1: console.log("step2"); state = -1; break;
    default: return;
  }
}

// 反混淆后
console.log("step1");
console.log("step2");
```

**原理**：分析状态转移图，重建原始控制流。

### 技术 3：死代码消除 (dead_code)

**适用场景**：存在永远不执行的分支

**示例**：
```javascript
// 混淆前
var x = 1;

// 混淆后
var x;
if (false) {
  x = 999; // 永远不执行
}
x = 1;

// 反混淆后
var x = 1;
```

**原理**：常量折叠 + 无用分支剪枝。

### 技术 4：Eval 展开 (eval)

**适用场景**：使用 eval/Function 动态生成代码

**示例**：
```javascript
// 混淆前
function add(a, b) { return a + b; }

// 混淆后
var add = new Function('a', 'b', 'return a + b');

// 反混淆后
function add(a, b) { return a + b; }
```

**原理**：在沙箱中安全执行 eval，提取生成的代码。

### 技术 5：自动模式 (auto)

**原理**：按顺序尝试所有技术，直到代码不再变化。

**流程**：
```
原始代码
  → decrypt (字符串解密)
  → eval (动态代码展开)
  → control_flow (控制流还原)
  → dead_code (死代码消除)
  → format (格式化)
```

## Hook 脚本模板

### 模板 1：函数签名 Hook

```javascript
// 使用 js_hook_generate (function_name="generateSign", hook_type="sign")
(function() {
  var original = window.generateSign;
  window.generateSign = function(...args) {
    console.log('[Hook] generateSign 输入:', args);
    var result = original.apply(this, args);
    console.log('[Hook] generateSign 输出:', result);
    return result;
  };
})();
```

### 模板 2：Fetch Hook

```javascript
// 使用 js_hook_generate (function_name="fetch", hook_type="fetch")
(function() {
  var originalFetch = window.fetch;
  window.fetch = function(url, options) {
    console.log('[Hook] Fetch 请求:', url, options);
    return originalFetch.apply(this, arguments).then(response => {
      console.log('[Hook] Fetch 响应:', response);
      return response;
    });
  };
})();
```

### 模板 3：CryptoJS Hook

```javascript
// Hook AES 加密
(function() {
  var original = CryptoJS.AES.encrypt;
  CryptoJS.AES.encrypt = function(message, key, cfg) {
    console.log('[Hook] AES.encrypt 输入:', {
      message: message.toString(),
      key: key.toString(),
      cfg: cfg
    });
    var result = original.apply(this, arguments);
    console.log('[Hook] AES.encrypt 输出:', result.toString());
    return result;
  };
})();
```

## 智能体协作模式

### 单智能体模式

适用于单文件快速分析：

```
用户：分析这个 JS 文件的签名算法
智能体：
  1. js_format 格式化代码
  2. js_extract_keys 提取密钥
  3. js_analyze_sign 分析签名函数
  4. 输出完整签名流程 + Python 复现代码
```

### 多智能体模式

适用于多文件复杂项目：

**角色**：
- **反混淆专家** - 处理混淆代码，输出可读版本
- **算法分析师** - 识别加密算法，提取密钥
- **Hook 生成器** - 生成验证用的 Hook 脚本
- **验证工程师** - 编写 Python/Go 复现代码并测试

**协作流程**：
```
协调者：收到 10 个混淆 JS 文件
  → 并行分配给 3 个反混淆专家
反混淆专家：输出可读代码
  → 传递给算法分析师
算法分析师：识别签名算法 + 提取密钥
  → 传递给 Hook 生成器 + 验证工程师
Hook 生成器：生成浏览器验证脚本
验证工程师：编写 Python 复现代码
协调者：汇总结果，生成最终报告
```

## 最佳实践

### 1. 分阶段反混淆
不要一次性使用 `technique=auto`，逐步应用每种技术并检查结果。

### 2. 保留中间产物
每次反混淆后保存文件：`app.step1.js`, `app.step2.js`，便于回溯。

### 3. 结合动态调试
Hook 脚本 + Chrome DevTools 断点，静态分析 + 动态验证。

### 4. AI 辅助阅读
对于复杂逻辑，使用 `reptool ai analyze --file app.js` 让 AI 解释代码。

### 5. 自动化批量处理
编写脚本批量调用 MCP 工具：
```bash
for file in *.js; do
  reptool js deobfuscate --file "$file" --technique auto
done
```

## 故障排查

### 问题 1：反混淆后代码仍然不可读
- 尝试不同技术顺序
- 手动分析字符串数组解密函数
- 使用 AST 工具（如 Babel）进一步处理

### 问题 2：密钥提取结果为空
- 检查是否为动态生成密钥
- 搜索 `key`、`secret`、`salt`、`iv` 关键字
- 使用正则搜索 16/24/32 字节的 hex 字符串

### 问题 3：Hook 脚本不生效
- 确认函数名拼写正确
- 检查函数是否在 window 对象上
- 使用 `Object.getOwnPropertyNames(window)` 列出所有全局函数

## 参考资料

- [obfuscator.io 混淆技术](https://obfuscator.io/)
- [AST Explorer](https://astexplorer.net/)
- [CryptoJS 文档](https://cryptojs.gitbook.io/)
