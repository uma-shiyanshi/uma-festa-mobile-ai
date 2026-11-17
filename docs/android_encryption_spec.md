# 安卓 2.32.5 无头加解密规范

> **来源**：外部已解通并在安卓 2.32.5 实机跑通无头请求的协作者同步（2026-11-18）。
> **验证状态**：公式与布局来自对方实测；本仓尚未逐条复现。标 `[待验]` 的项需本地跑一次确认。
> **取代关系**：本文结论**取代** `docs/android_session_findings.md` 第 3、5 节中关于
> `Cryptographer` / `AES256Crypt` 参与 HTTP 发包加密的推断——那套只用于本地存档等处。

## 1. 关键纠偏（此前判断作废）

| 原判断 | 实际 |
|---|---|
| A4 是 34 字节 | **A4 是 50 字节**。前 34B（`[120:154]`）进头部 AES-256-CTR 块，**后 16B（`[154:170]`）参与正文 `MD5a` 和响应 AES-GCM 的 key/AAD**，少一字节都解不开 |
| `AES256Crypt.KEYSTR` / `GenerateKeyString()` / `ComputeHash` 参与发包加密 | **不参与**。C# 的 `HttpHelper.CompressRequest` 本身不做加密，只把 170B 前缀与 msgpack 拼块，P/Invoke 给 `libnative.so` 的 `LZ4_compress_default_ext`，再转 Base64 |
| 借 PC 抓包能拿到 A1/A4 | **拿不到**。A1 前 32B 是服务端 X25519 公钥（大端），后 20B 跟平台/版本走（安卓与 PC 的 2.32.5 不同）；**A4 是账号级凭据**，PC 只能拿到 PC 上那个号的 |

## 2. 前缀布局

### 170B（正常请求）

| 偏移 | 长度 | 内容 |
|---|---|---|
| `0..4` | 4B | `u32_le(166)` = 前缀总长 − 4 |
| `4..56` | 52B | **A1**：`[4:36]` 服务端 X25519 公钥（大端）+ `[36:56]` 版本/平台常量 |
| `56..72` | 16B | `Hex2bin(SID)`，与 HTTP 头 `SID` 同值 |
| `72..88` | 16B | `Hex2bin(Udid.Replace("-",""))` |
| `88..120` | 32B | `Random(32)`，决定本次临时密钥，可任意填 |
| `120..170` | 50B | **A4**（账号级凭据，注册后才有） |
| `170..` | N | msgpack 明文 |

### 120B（首次注册 / pre_signup / signup）

| 偏移 | 长度 | 内容 |
|---|---|---|
| `0..4` | 4B | `u32_le(116)` |
| `4..56` | 52B | A1 |
| `56..72` | 16B | SID |
| `72..88` | 16B | UDID |
| `88..120` | 32B | Random(32) |
| `120..` | N | msgpack（**无 A4**） |

## 3. SID 滚动

```
MakeMd5(s) = md5_hex_lower( UTF8( s + SALT ) )     // SALT = 11 位 ASCII，日服安卓/PC 共用
SID[0]     = MakeMd5( str(viewer_id) + udid_with_dashes )   // udid 带横杠；注册前 viewer_id = "0"
SID[n+1]   = MakeMd5( 上一响应的 data_headers.sid )          // 42 位字符串原样拼，不截取、不 hex 解码
```

> 本仓实测佐证：`/api/md5log` 的 `output` 与请求头 SID 一致，`input` 与响应
> `data_headers.sid` 一致。缺的正是 `SALT` 这 11 字节——拿到即可本地复算。

## 4. 纯手机取 A1 / A4

### 路径 A：hook `libnative.so!LZ4_compress_default_ext`（推荐）

```
LZ4_compress_default_ext(const uint8_t* src, uint8_t* dst, int src_len, int dst_cap)
```

- 标准 4 参数 C 导出，比 hook 13 参数的 `Send` 简单得多
- 定位：读 `/proc/self/maps` 找 `libnative.so` 基址 + 解析 ELF `.dynsym`；**不要 `dlopen`**（Hachimi / 转译环境易崩）
- `CompressRequest` 拼好 `u32_le(166)+170B+msgpack` 后才调它 → hook 入口直接读
  `src[0..170]`（`declared==166`）或 `src[0..120]`（`declared==116`）
- **一次性同时拿到 A1、UDID、A4 与 msgpack 明文**

### 路径 B：已有 A1 + SALT 后，向服务端要 A4

| 场景 | 接口 | 取法 |
|---|---|---|
| 新号注册 | `tool/pre_signup` → `tool/signup`（120B 分支） | 响应 `data.auth_key`（68 字符 base64）解码 = **50B A4** |
| 老号切号 | `account/chain_by_transition_code`（引继码） | 响应 `data.auth_key` 解码 = 50B A4 |

## 5. 完整加解密链

请求方向**不做 LZ4 压缩**。以下全部从 170B 前缀确定性派生：

**① PCG-64**
```
initstate = BE64(src[0x58:0x60])
initseq   = BE64(src[0x60:0x68])
标准 pcg_setseq_64 初始化后，额外推进 (src[0x38] & 0x0F) 次，每次取 xsh_rr_32 低 8 位
```

**② X25519（两处非标准）**
```
取 RNG 前 32B 按大端读入 → 整体无条件右移 1 位 → 置 bit254、清低 3 位 = 私钥标量 d（小端）
dst[4:36] = 小端(d·G)                          // 客户端临时公钥
对端公钥  = src[4:36] 整体字节反转（前缀里是大端）→ 共享密钥 K32（32B 小端）
```

**③ 头部块 AES-256-CTR**
```
key   = K32
nonce = dst[4:20]
明文  = 0x01 0x00 || src[0x78:0x9A]                    // 170B 分支
       0x00 0x00 || udid || PCG流[96..114]             // 120B 分支
密文 → dst[36:72]
```

**④ 正文 AES-128-GCM（16 字节 IV）**
```
MD5a    = MD5( src[0x38:0x48] || src[0x9A:0xAA] || src[0x48:0x58] )   // 120B 分支用 udid 代替
B       = mix(MD5a, K32, src[0x24:0x34])                              // 16 轮特定字节置换，非简单 XOR
gcm_key = MD5( B || K32 || MD5a || src[0x24:0x38] )
IV      = MD5( dst[4:36] || src[0x38:0x48] || src[0x48:0x58] )
AAD     = src[0x24:0x58]      // 52B
tag → dst[72:88]；密文 → dst[88:]
dst[0:4] = u32_le(msgpack_len + 84)；整体 Base64 发送
```

**⑤ 响应解密（`LZ4_decompress_safe_ext`）**
```
key  = MD5( K32 || req[0x38:0x48] || req[0x9A:0xAA] )    // 120B 分支末段换 udid
AAD  = req[0x38:0x48] || req[0x9A:0xAA]                  // 只这两个字段
resp Base64 解码后：out_len = u32_le(resp[0:4])，IV = resp[4:20]，tag = resp[20:36]，密文 = resp[36:]
GCM 解密后若 out_len != 0 → 再做标准 LZ4 block 解压 → msgpack 明文
```

## 6. 待补常量

| 常量 | 长度 | 性质 | 状态 |
|---|---|---|---|
| `A1` | 52B | 版本/平台公共 | 对方可提供，**待接收** |
| `SALT` | 11B | 安卓/PC 共用 | 对方可提供，**待接收** |
| `A4` | 50B | **账号级**，各号不同 | 自己提：路径 A hook，或路径 B 要 `auth_key` |
| `UDID` | 16B | 安装级 | 已可从 `start_session` 明文 + 内存取得 |

## 7. 本仓已有的、可直接用于交叉验证的样本

- `/api/md5log`：`input` = 响应 `sid`（42 位），`output` = 请求头 SID → 拿到 SALT 后可逐条验算
- `/api/sniff/metadata`：`start_session` 请求/响应明文（含全部设备字段）
- `so/endpoints_3.29.6.txt`：156 个端点清单

拿到 A1 + SALT 后，第一步就能做：**用 `md5(sid_raw + SALT)` 复算请求头 SID，与 md5log 的 `output` 逐条比对**。
只要这一步对上，SID 就完全自主，不必再依赖插件。
