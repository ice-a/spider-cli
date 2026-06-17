# 加密工具智能体 (Crypto Toolkit Agent)

## 技能概述

专注于加密算法的识别、验证和破解，提供哈希计算、对称/非对称加密、HMAC 签名、Base64/URL 编解码等常用密码学工具，支持分步计算和签名差异对比。

## 核心能力

### 1. 哈希算法
- **MD5** - 128 位哈希（已不安全，仅用于校验）
- **SHA-1** - 160 位哈希（已不安全）
- **SHA-256** - 256 位哈希（推荐）
- **SHA-512** - 512 位哈希（高安全）
- **输出格式** - hex/base64/bytes

### 2. 对称加密
- **AES** - CBC/ECB/GCM 模式，支持 128/192/256 位密钥
- **DES/3DES** - 传统块加密（已过时）
- **RC4** - 流加密（已不安全）
- **填充模式** - PKCS7/Zero/NoPadding

### 3. 非对称加密
- **RSA** - 公钥加密/私钥解密
- **密钥格式** - PEM/DER/PKCS1/PKCS8
- **填充模式** - PKCS1/OAEP

### 4. HMAC 签名
- **算法** - HMAC-MD5/SHA1/SHA256/SHA512
- **应用场景** - API 签名、消息认证码

### 5. 编码转换
- **Base64** - 标准/URL-safe 编解码
- **URL 编码** - percent-encoding
- **Hex** - 十六进制转换

### 6. 辅助工具
- **时间戳** - 10 位/13 位时间戳生成与解析
- **随机 User-Agent** - 模拟不同浏览器
- **参数签名** - 字典排序 + 拼接 + 哈希

## MCP 工具清单

### 哈希工具
- `crypto_hash` - 哈希计算（md5/sha1/sha256/sha512）
  - 参数：`algorithm`, `input`, `output_format`（hex/base64）

### 对称加密
- `crypto_encrypt` - 加密（aes-cbc/aes-ecb/aes-gcm/des/3des/rc4）
  - 参数：`algorithm`, `key`, `data`, `iv`（可选）
- `crypto_decrypt` - 解密
  - 参数：`algorithm`, `key`, `data`, `iv`（可选）

### HMAC 签名
- `crypto_hmac` - HMAC 签名
  - 参数：`algorithm`, `key`, `data`

### 编码工具
- `crypto_base64` - Base64 编解码
  - 参数：`action`（encode/decode）, `data`
- `crypto_urlencode` - URL 编解码
  - 参数：`action`（encode/decode）, `data`

### 签名工具
- `calc_sign_sort` - 参数字典排序签名
  - 参数：`params`（JSON）, `salt`, `algorithm`
- `calc_step` - 分步加密计算器
  - 参数：`steps`（数组）, `data`
- `calc_diff_sign` - 签名差异对比
  - 参数：`src_str`, `src_sign`, `dst_str`, `dst_sign`

### 辅助工具
- `crypto_timestamp` - 时间戳工具
  - 参数：`bits`（10/13）, `offset`（秒偏移）
- `crypto_random_ua` - 随机 User-Agent
  - 参数：`browser`（chrome/firefox/safari/all）, `count`

## 使用场景

### 场景 1：识别哈希算法

**问题**：抓包发现签名字段 `sign=5d41402abc4b2a76b9719d911017c592`，不知道用的什么算法

**步骤**：

1. **长度判断**
   ```
   32 个字符 → MD5 (128 bit = 16 byte = 32 hex)
   40 个字符 → SHA-1 (160 bit = 20 byte = 40 hex)
   64 个字符 → SHA-256 (256 bit = 32 byte = 64 hex)
   128 个字符 → SHA-512 (512 bit = 64 byte = 128 hex)
   ```

2. **验证假设**
   ```
   使用 crypto_hash (algorithm="md5", input="hello")
   → 输出：5d41402abc4b2a76b9719d911017c592
   → 确认是 MD5
   ```

3. **查找待签名字符串**
   ```
   使用 js_extract_keys 查找盐值
   → 发现盐值 "secretKey123"
   使用 crypto_hash (algorithm="md5", input="params_string+secretKey123")
   → 对比签名
   ```

### 场景 2：破解参数签名

**目标**：API 签名格式为 `sign=md5(key1=value1&key2=value2&salt=abc123)`

**步骤**：

1. **抓包提取参数**
   ```
   请求参数：{"user":"test", "timestamp":1234567890}
   签名：e10adc3949ba59abbe56e057f20f883e
   ```

2. **尝试字典排序**
   ```
   使用 calc_sign_sort (
     params='{"user":"test","timestamp":"1234567890"}',
     salt="abc123",
     algorithm="md5"
   )
   → 输出：
     待签名串：timestamp=1234567890&user=test&salt=abc123
     签名：e10adc3949ba59abbe56e057f20f883e
   → 匹配成功！
   ```

3. **生成复现代码**
   ```python
   import hashlib
   import json

   def generate_sign(params, salt):
       sorted_params = sorted(params.items())
       sign_str = '&'.join([f'{k}={v}' for k, v in sorted_params])
       sign_str += f'&salt={salt}'
       return hashlib.md5(sign_str.encode()).hexdigest()
   ```

### 场景 3：验证 AES 加密

**问题**：JS 代码使用 AES-CBC 加密，需要验证加密逻辑

**步骤**：

1. **提取加密参数**
   ```
   使用 js_extract_keys 提取：
   - key: "1234567890123456" (16 字节)
   - iv: "0000000000000000" (16 字节)
   - 明文：{"user":"test"}
   ```

2. **验证加密**
   ```
   使用 crypto_encrypt (
     algorithm="aes-cbc",
     key="1234567890123456",
     iv="0000000000000000",
     data='{"user":"test"}'
   )
   → 输出（hex）：a1b2c3d4e5f6...
   ```

3. **对比抓包结果**
   ```
   抓包中的加密数据：a1b2c3d4e5f6...
   → 匹配成功！确认加密逻辑正确
   ```

4. **验证解密**
   ```
   使用 crypto_decrypt (
     algorithm="aes-cbc",
     key="1234567890123456",
     iv="0000000000000000",
     data="a1b2c3d4e5f6..."
   )
   → 输出：{"user":"test"}
   ```

### 场景 4：分步验证复杂签名

**目标**：签名流程为 `base64(md5(sha256(params) + timestamp))`

**步骤**：

1. **使用分步计算器**
   ```
   使用 calc_step (
     steps=["sha256", "md5", "base64"],
     data='{"user":"test"}'
   )
   → 输出每一步的结果：
     Step 1 (sha256): 9f86d081884c7d659a2feaa0c55ad015...
     Step 2 (md5): e10adc3949ba59abbe56e057f20f883e
     Step 3 (base64): ZTEwYWRjMzk0OWJhNTlhYmJlNTZlMDU3...
   ```

2. **对比抓包签名**
   ```
   抓包签名：ZTEwYWRjMzk0OWJhNTlhYmJlNTZlMDU3...
   → 匹配成功！
   ```

3. **添加时间戳参数**
   ```
   使用 calc_step (
     steps=["sha256", "+timestamp", "md5", "base64"],
     data='{"user":"test"}'
   )
   → 在 sha256 和 md5 之间插入时间戳拼接
   ```

### 场景 5：签名差异对比

**问题**：两次请求参数相同，但签名不同，怀疑有隐藏参数

**步骤**：

1. **提取两次请求数据**
   ```
   请求 1：params={"user":"test"} → sign=abc123
   请求 2：params={"user":"test"} → sign=def456
   ```

2. **使用差异对比**
   ```
   使用 calc_diff_sign (
     src_str='{"user":"test"}',
     src_sign="abc123",
     dst_str='{"user":"test"}',
     dst_sign="def456"
   )
   → 输出：
     差异分析：签名不同但输入相同
     可能原因：
       1. 存在时间戳/随机数参数
       2. 盐值动态生成
       3. 签名算法包含隐藏字段
   ```

3. **查找隐藏参数**
   ```
   使用 proxy_get_sessions 获取完整请求头
   → 发现 X-Timestamp: 1234567890
   → 将时间戳加入签名字符串重新计算
   ```

### 场景 6：HMAC 签名验证

**目标**：API 使用 HMAC-SHA256 签名

**步骤**：

1. **提取签名密钥**
   ```
   使用 js_extract_keys 查找 HMAC key
   → 发现：hmacKey = "secret123"
   ```

2. **计算 HMAC 签名**
   ```
   使用 crypto_hmac (
     algorithm="sha256",
     key="secret123",
     data='{"user":"test"}'
   )
   → 输出（hex）：a1b2c3d4e5f6...
   ```

3. **对比抓包签名**
   ```
   请求头 Authorization: HMAC-SHA256 a1b2c3d4e5f6...
   → 匹配成功！
   ```

## 加密算法识别指南

### 1. 根据输出长度识别哈希

| 输出长度 | 可能算法 |
|---------|---------|
| 32 字符 (hex) | MD5 |
| 40 字符 (hex) | SHA-1 |
| 64 字符 (hex) | SHA-256 |
| 128 字符 (hex) | SHA-512 |
| 22 字符 (base64) | MD5 (base64) |
| 28 字符 (base64) | SHA-1 (base64) |
| 44 字符 (base64) | SHA-256 (base64) |

### 2. 根据密钥长度识别对称加密

| 密钥长度 | 可能算法 |
|---------|---------|
| 8 字节 | DES |
| 16 字节 | AES-128 |
| 24 字节 | 3DES, AES-192 |
| 32 字节 | AES-256 |
| 任意长度 | RC4 |

### 3. 根据输出格式识别

| 特征 | 算法 |
|-----|------|
| 输出长度 = 输入长度 + 16 (AES block size) | AES-ECB/CBC |
| 输出长度 = 输入长度 + 12 (tag) | AES-GCM |
| 输出以 `0x` 开头 | Hex 编码 |
| 输出以 `=` 结尾 | Base64 编码 |
| 输出包含 `%` | URL 编码 |

### 4. 根据 JS 代码特征识别

```javascript
// MD5
md5(str), CryptoJS.MD5(str)

// SHA-256
sha256(str), CryptoJS.SHA256(str)

// AES
CryptoJS.AES.encrypt(data, key), AES.encrypt(...)

// RSA
JSEncrypt.encrypt(...), forge.pki.publicKeyFromPem(...)

// HMAC
CryptoJS.HmacSHA256(data, key)

// Base64
btoa(str), atob(str), Buffer.from(str).toString('base64')
```

## 常见签名格式

### 格式 1：字典排序 + MD5

```
输入：{"user":"test","id":"123"}
排序：id=123&user=test
加盐：id=123&user=test&salt=abc
签名：md5("id=123&user=test&salt=abc")
```

**验证工具**：`calc_sign_sort`

### 格式 2：时间戳 + SHA256

```
输入：{"user":"test"}
时间戳：1234567890
拼接：{"user":"test"}1234567890
签名：sha256('{"user":"test"}1234567890')
```

**验证工具**：`crypto_hash` + `crypto_timestamp`

### 格式 3：HMAC-SHA256

```
输入：GET&/api/user&timestamp=123
密钥：secret123
签名：hmac_sha256("GET&/api/user&timestamp=123", "secret123")
```

**验证工具**：`crypto_hmac`

### 格式 4：多层加密

```
步骤 1：JSON → Base64
步骤 2：Base64 → AES-CBC 加密
步骤 3：AES 输出 → Base64
```

**验证工具**：`calc_step`

### 格式 5：RSA + AES 混合

```
步骤 1：生成随机 AES key
步骤 2：用 AES key 加密数据
步骤 3：用 RSA 公钥加密 AES key
输出：{"data": aes_encrypted, "key": rsa_encrypted}
```

**验证工具**：`crypto_encrypt` (AES) + 手动 RSA 验证

## 分步计算器语法

### 支持的步骤

| 步骤名 | 说明 | 示例 |
|-------|------|------|
| `md5` | MD5 哈希 | `md5` |
| `sha1` | SHA-1 哈希 | `sha1` |
| `sha256` | SHA-256 哈希 | `sha256` |
| `sha512` | SHA-512 哈希 | `sha512` |
| `base64` | Base64 编码 | `base64` |
| `base64_decode` | Base64 解码 | `base64_decode` |
| `urlencode` | URL 编码 | `urlencode` |
| `+<string>` | 拼接字符串 | `+timestamp` |
| `aes-cbc` | AES-CBC 加密 | `aes-cbc:key:iv` |

### 示例

**例 1：简单哈希链**
```json
{
  "steps": ["sha256", "md5", "base64"],
  "data": "hello"
}
```

**例 2：带拼接**
```json
{
  "steps": ["sha256", "+secretKey", "md5"],
  "data": "params_string"
}
```

**例 3：加密后编码**
```json
{
  "steps": ["aes-cbc:1234567890123456:0000000000000000", "base64"],
  "data": '{"user":"test"}'
}
```

## 智能体协作模式

### 单智能体模式

适用于简单加密验证：

```
用户：验证这个 MD5 签名
智能体：
  1. crypto_hash 计算 MD5
  2. 对比用户提供的签名
  3. 输出匹配结果
```

### 多智能体模式

适用于复杂签名逆向：

**角色**：
- **算法识别专家** - 根据输出长度/格式识别算法
- **密钥提取专家** - 从 JS 代码中提取密钥/盐值
- **签名验证专家** - 使用 calc_step 分步验证
- **代码生成专家** - 生成 Python/Go 复现代码

**协作流程**：
```
算法识别专家：分析签名格式 → 识别为 SHA256
密钥提取专家：从 JS 提取盐值 → "abc123"
签名验证专家：calc_sign_sort 验证 → 匹配成功
代码生成专家：生成 Python 代码 → 完成交付
```

## 最佳实践

### 1. 先识别算法，再验证密钥
不要盲目尝试所有算法，根据输出长度和格式缩小范围。

### 2. 使用分步计算器调试
复杂签名流程用 `calc_step` 逐步验证，不要一次性计算。

### 3. 保存中间结果
每一步的输出都保存下来，便于调试和对比。

### 4. 注意编码格式
同样的哈希值，hex 和 base64 输出不同，确认抓包使用的格式。

### 5. 时间戳同步
如果签名包含时间戳，确保本地时间与服务器同步。

## 故障排查

### 问题 1：签名始终不匹配
- 检查参数排序规则（字典序 vs 原始顺序）
- 检查拼接符号（`&` vs `+` vs 空格）
- 检查编码格式（UTF-8 vs GBK）
- 检查输出格式（hex vs base64 vs uppercase）

### 问题 2：AES 解密失败
- 确认 key 长度（16/24/32 字节）
- 确认 IV 长度（16 字节）
- 确认加密模式（CBC/ECB/GCM）
- 确认填充模式（PKCS7/Zero）
- 确认输入格式（hex/base64）

### 问题 3：分步计算器报错
- 检查步骤名拼写
- 确认步骤顺序合理（先解码再哈希）
- 检查参数格式（AES 需要 key:iv）

## 参考资料

- [CryptoJS 文档](https://cryptojs.gitbook.io/)
- [OWASP 密码学备忘单](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html)
- [RFC 2104 - HMAC](https://tools.ietf.org/html/rfc2104)
- [AES 模式详解](https://en.wikipedia.org/wiki/Block_cipher_mode_of_operation)
