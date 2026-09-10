## ADDED Requirements

### Requirement: 原生二进制与 base64 转换

运行时暴露给脚本的二进制转换 API（`atob`、`btoa`、`Buffer.from(str, 'base64')`、`buf.toString('base64')`、`TextEncoder.encode`、`TextDecoder.decode`、无 encoding 的 `fs.readFileSync`）MUST 由宿主原生实现完成字节级转换，脚本侧 MUST NOT 逐字节循环。`fs.readFileSync(path)`（无 encoding）MUST 返回 `Uint8Array` 实例且带 `toString('base64')` 方法；`fs.readFileSync(path, 'base64')` 与 `doc_read(path)` 行为不变。API 名称、参数与返回类型 MUST 与既有 polyfill 兼容，既有脚本无需修改。

#### Scenario: 读取 4MB 图片为字节

- **WHEN** 脚本对沙箱内 4MB JPEG 调用 `fs.readFileSync(path)`
- **THEN** 1 秒内返回长度等于文件字节数的 `Uint8Array`
- **AND** `Buffer.from(bytes).toString('base64')` 与 `fs.readFileSync(path, 'base64')` 结果一致

#### Scenario: atob / btoa 全字节往返

- **WHEN** 脚本对包含 0x00–0xFF 全部字节值的 binary string 调用 `btoa` 再 `atob`
- **THEN** 结果与原字符串逐字符相等

#### Scenario: UTF-8 编解码

- **WHEN** 脚本用 `new TextEncoder().encode("中文 emoji 😀")` 再 `new TextDecoder().decode(...)`
- **THEN** 结果与原字符串相等

#### Scenario: 非法 base64 报错

- **WHEN** 脚本调用 `Buffer.from("!!!", "base64")`
- **THEN** 抛出包含 `invalid base64` 的异常，而非静默返回空数组

### Requirement: 图片元数据与缩放 op

运行时 SHALL 提供 `doc_image_info(path)` 返回 `{ width, height, mime, bytes }`，仅读取文件头，MUST 支持 PNG / JPEG / GIF / WebP；SHALL 提供 `doc_image_resize(path, { maxEdge, quality, out })` 按长边缩放并写入沙箱（默认输出 `.cache/images/<stem>-<maxEdge>-<hash>.jpg`，带透明通道的 PNG 输出 PNG），返回 `{ path, width, height, bytes }`。缩放写入 MUST 经沙箱校验与文件锁 write gate，且计入本次 `skill_run` 的 `written_paths`。长边已 ≤ `maxEdge` 且格式为 JPEG/PNG 且未指定 `out` 时 MUST 直接返回原路径不重编码。

#### Scenario: 取尺寸不解码

- **WHEN** 脚本对 4.3MB JPEG 调用 `doc_image_info`
- **THEN** 100ms 内返回 `width`、`height`、`mime: "image/jpeg"`、`bytes`

#### Scenario: 大图缩放

- **WHEN** 脚本对 4000×3000 JPEG 调用 `doc_image_resize(path, { maxEdge: 1600 })`
- **THEN** 返回路径位于 `.cache/images/`，`width` 为 1600，`height` 为 1200，输出文件可被 `image_read` 与 pptxgenjs 正常使用
- **AND** 该路径出现在 `skill_run` 结果的 `written_paths`

#### Scenario: 小图直通

- **WHEN** 脚本对 800×600 PNG 调用 `doc_image_resize(path, { maxEdge: 1600 })`
- **THEN** 返回原路径且文件未被修改

#### Scenario: 非图片文件

- **WHEN** 脚本对 `.docx` 调用 `doc_image_info`
- **THEN** 抛出包含 `unsupported image format` 的异常

#### Scenario: 越界输出被拒

- **WHEN** 脚本调用 `doc_image_resize(path, { out: "../evil.jpg" })`
- **THEN** op 返回沙箱错误，文件未被创建

### Requirement: 超时分类与上限告知

`skill_run` 的 `timeout_secs` 参数 MUST 在工具 schema 中声明 `minimum: 1`、`maximum: 120`；请求值超出范围时 MUST 截断到范围内，且结果（成功或失败）MUST 携带 `timeout_clamped: { requested, applied }`。脚本超时的错误结果 `error` 字段 MUST 为 `"script timeout"`（MUST NOT 归类为 `"JavaScript runtime error"`），`hint` MUST 说明超时秒数与上限、指出本运行时对大二进制逐字节处理极慢、并给出「用 `doc_image_info` 取尺寸 / 用 `doc_image_resize` 缩图 / 减少单次处理文件数」的修复方向；MUST NOT 建议加大 `timeout_secs` 作为首选。

#### Scenario: 超时独立分类

- **WHEN** 脚本在 `timeout_secs` 内未完成
- **THEN** 工具错误结果 `error` 为 `"script timeout"`，`detail` 为 `"script timeout"`，`hint` 包含 `doc_image_resize`

#### Scenario: 超出上限被告知

- **WHEN** Agent 传 `timeout_secs: 180`
- **THEN** 实际以 120 秒执行，结果包含 `timeout_clamped: { "requested": 180, "applied": 120 }`

#### Scenario: 范围内不附加字段

- **WHEN** Agent 传 `timeout_secs: 60`
- **THEN** 结果不包含 `timeout_clamped`

### Requirement: 运行时循环迭代上限

运行时 MUST 为单个循环设置迭代次数上限（默认 2×10⁷），超出时脚本抛错并终止，错误结果 `error` 为 `"loop iteration limit exceeded"`。宿主判定超时后 MUST 置位取消标志，此后脚本任何 native op 调用 MUST 立即抛出 `script cancelled by host` 使脚本尽早退出。

#### Scenario: 死循环确定性终止

- **WHEN** 脚本执行 `while (true) {}`
- **THEN** 脚本在迭代上限处抛错终止，工具返回 `error: "loop iteration limit exceeded"`，且不等待到 `timeout_secs`

#### Scenario: 超时后 op 调用被拒

- **WHEN** 宿主已因超时返回错误，脚本线程随后调用 `doc_read`
- **THEN** 该调用抛出 `script cancelled by host`，脚本线程结束

### Requirement: 图片处理指引文档

`skill_read {"skill":"runtime"}` 返回的能力矩阵 MUST 包含「图片与二进制」章节，说明 `doc_image_info` / `doc_image_resize` 用法、`fs.readFileSync` 三种返回形态、以及「多 MB 图片直接 base64 进 pptxgenjs 会超时」的限制；`skill_read {"skill":"pptx"}` 的 pptxgenjs Images 一节 MUST 以「`doc_image_info` 取尺寸 → 大图 `doc_image_resize` → `fs.readFileSync(resized, 'base64')` → `addImage`」为推荐流程并给出保持宽高比的示例代码，MUST NOT 再示范手写 JPEG/PNG 文件头解析。

#### Scenario: runtime 文档包含图片章节

- **WHEN** Agent 调用 `skill_read {"skill":"runtime"}`
- **THEN** 返回内容包含 `doc_image_info`、`doc_image_resize` 与 `maxEdge` 关键字

#### Scenario: pptx 文档推荐缩图流程

- **WHEN** Agent 调用 `skill_read {"skill":"pptx", "doc":"pptxgenjs.md"}`
- **THEN** Images 一节示例先调用 `doc_image_resize` 再 `addImage`，且不含 `0xFFC0` / `IHDR` 等手写解析代码

### Requirement: 默认缩图文件名区分来源与参数

默认输出文件名 MUST 包含源相对路径和缩放参数的哈希后缀。

#### Scenario: 同名图片与不同质量互不覆盖

- **WHEN** 不同目录或扩展名的同名图片，或同一图片使用不同 quality，依次缩图且未指定 out
- **THEN** 返回不同路径，先前生成的文件内容不变
- **AND** 相同输入参数重复调用返回相同路径

## MODIFIED Requirements

### Requirement: 运行时沙箱

运行时 MUST 禁用网络访问；文件读写 MUST 仅通过宿主注入的自定义 op 进行，且每次访问经现有 `Sandbox` 路径校验；单次执行 MUST 有超时上限（默认 30 秒，最大 120 秒，超出请求值截断并告知），超时即向调用方返回错误。写入类 op 在落盘前 MUST 通过文件锁系统申请目标路径 Write lock，冲突时 MUST 返回错误且不写入磁盘。

#### Scenario: 越界写被拒

- **WHEN** 脚本尝试写入项目根目录之外的路径
- **THEN** op 返回沙箱错误，文件未被创建

#### Scenario: 写锁冲突被拒

- **WHEN** 脚本尝试写入已被其他 session 占用的 `out.pptx`
- **THEN** op 返回 file busy 错误，`out.pptx` 不被覆盖

#### Scenario: 网络不可用

- **WHEN** 脚本尝试发起 fetch 请求
- **THEN** 运行时报错（无网络扩展），执行不挂起

#### Scenario: 超时上限

- **WHEN** Agent 请求 `timeout_secs` 大于 120
- **THEN** 实际超时为 120 秒，结果携带 `timeout_clamped`
