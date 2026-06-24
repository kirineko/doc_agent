# image-download Specification

## Purpose
TBD - created by archiving change add-image-download. Update Purpose after archive.
## Requirements
### Requirement: image_download 批量下载工具

系统 SHALL 提供 `image_download` 工具，从 1–20 个公网 http(s) 图片 URL 批量下载到项目沙箱内目录，并返回每张图片的本地相对路径与元数据。入参 `urls`（string 数组，必填）与可选 `dir`（项目相对输出目录，默认 `images`，不存在时自动创建）。成功项 MUST 返回 `{ url, path, bytes, width, height, format }`，并在结果中给出成功计数 `count`。

#### Scenario: 成功下载多张图片

- **WHEN** 模型调用 `image_download` 且 `urls` 含若干有效 http(s) 图片 URL
- **THEN** 系统将图片写入 `dir`（默认 `images/`）并返回 `downloaded[]`，每项含本地相对 `path` 与 `bytes`、`width`、`height`、`format`
- **AND** 这些 `path` MUST 出现在该 `tool_result` 的 `changed_paths` 中

#### Scenario: 空 urls 被拒绝

- **WHEN** 模型调用 `image_download` 且 `urls` 为空数组或缺失
- **THEN** 系统返回参数错误，不发起任何网络请求

#### Scenario: 数量越界被拒绝

- **WHEN** 模型传入超过 20 个 URL
- **THEN** 系统返回参数错误，不发起任何网络请求

### Requirement: 仅下载到项目沙箱内

系统 SHALL 将所有下载文件写入经沙箱校验的项目相对路径；输出目录 MUST 经 `resolve_for_write` 校验，越界路径（含 `..` 与绝对路径逃逸）MUST 被拒绝。输出目录 MUST NOT 为项目根（`.`）或位于 `.cache/` 之下：前者会锁定整个项目根，后者会被工作区静默过滤导致产物不可见。`dir` 缺省、空白或为空时 MUST 归一化为 `images`。外部图片 URL 本身 MUST NOT 被当作项目相对路径校验。

#### Scenario: 越界输出目录被拒绝

- **WHEN** 模型传入 `dir` 指向项目根目录之外（如 `../escape`）或为绝对路径
- **THEN** 系统返回错误，不写入任何文件

#### Scenario: 项目根或缓存目录被拒绝

- **WHEN** 模型传入 `dir` 为 `.` 或以 `.cache/` 开头（如 `.cache/imgs`）
- **THEN** 系统返回错误，不写入任何文件

#### Scenario: 空白 dir 归一化为默认目录

- **WHEN** 模型传入 `dir` 为空白、空串或省略
- **THEN** 系统使用默认 `images/` 作为输出目录，且文件锁与实际写盘目录一致

#### Scenario: 文件落在沙箱内

- **WHEN** `image_download` 成功
- **THEN** 每个返回的 `path` 都位于项目根目录之内，且为项目相对 POSIX 路径

#### Scenario: 符号链接目标名不被复用

- **WHEN** 输出目录中已存在与目标文件名相同的符号链接（含指向项目外或断链的符号链接）
- **THEN** 系统 MUST 跳过该名称、改用去重名，且 MUST NOT 沿符号链接写出项目根之外

### Requirement: 图片类型与体量校验

系统 SHALL 按下载到的**真实字节**判定图片类型（而非依赖 URL 扩展名或响应头），无法识别为受支持图片格式（png/jpeg/gif/webp/bmp/tiff）的响应 MUST 计入失败、MUST NOT 落地为图片文件；保存文件的扩展名 MUST 取自检测到的真实格式。单张图片体量超过上限（15 MiB）的下载 MUST 被中断并计入失败。

#### Scenario: 非图片响应不落地

- **WHEN** 某 URL 返回 HTML 错误页或非图片内容
- **THEN** 该 URL 计入 `failed[]`（注明非图片），不在 `dir` 中产生文件

#### Scenario: 扩展名取自真实格式

- **WHEN** URL 以 `.jpg` 结尾但实际字节是 PNG
- **THEN** 保存文件扩展名为 `.png`，`format` 字段为 `png`

#### Scenario: 超大图片被中断

- **WHEN** 某图片体量超过 15 MiB 上限
- **THEN** 该 URL 计入 `failed[]`，不保存部分文件

### Requirement: 外部地址安全约束

系统 SHALL 仅允许 `http` / `https` 协议 URL；其余协议（如 `data:`、`file:`、`ftp:`）MUST 被拒绝。系统 SHALL 对指向环回、私网、链路本地、未指定或唯一本地（ULA）地址的目标做尽力而为的 SSRF 拒绝（IP 字面量按地址段判定；非标准 IP 写法——十进制/短写/十六进制/IPv4-mapped IPv6 等——经 URL 归一化后等同判定；域名 `localhost` 与 `*.local` 拒绝）。系统 SHALL 对每一次 HTTP 重定向的目标 URL 复用同一套安全校验，重定向目标被拒时 MUST 停止跟随并把该 URL 计入失败。被拒 URL（含被拒的重定向目标）MUST 计入失败且 MUST NOT 把目标内容保存为图片。该校验为**尽力而为**：系统 MUST NOT 解析 DNS，因此「公网域名解析到私网/环回/云元数据 IP」（含 DNS rebinding）的首次连接在 MVP 内不做阻断（记为已知缺口）。

#### Scenario: 非 http(s) 协议被拒

- **WHEN** 模型传入 `file:///etc/passwd` 或 `data:` URL
- **THEN** 该 URL 计入 `failed[]`，不发起请求

#### Scenario: 内网/环回地址被拒

- **WHEN** 模型传入 `http://127.0.0.1/...`、`http://169.254.169.254/...` 或 `http://192.168.0.1/...`
- **THEN** 该 URL 计入 `failed[]`，不发起请求

#### Scenario: 重定向到内网地址被拒

- **WHEN** 一个公网 URL 返回 30x 重定向，目标指向私网/环回/链路本地/云元数据地址（如 `169.254.169.254`）
- **THEN** 系统停止跟随该重定向，该 URL 计入 `failed[]`，不把重定向目标内容保存为图片

### Requirement: 批量部分失败处理

系统 SHALL 在批量下载中独立处理每个 URL：除入参非法外，工具 MUST 返回成功结果（非中断错误），把逐 URL 失败放入 `failed[]`（含 `url` 与 `error`），成功项放入 `downloaded[]`。部分失败 MUST NOT 影响其余成功项的落地。

#### Scenario: 部分成功部分失败

- **WHEN** 一批 URL 中有的有效、有的失效
- **THEN** 有效图片写入 `dir` 并出现在 `downloaded[]` 与 `changed_paths`，失效 URL 出现在 `failed[]`，工具整体返回成功

### Requirement: 无条件可用且与文档生成解耦

系统 SHALL 将 `image_download` 无条件注册到 LLM 工具列表，不依赖任何 API Key，且 MUST NOT 受 Web 搜索开关（`include_web`）影响。该工具 MUST 独立于 docx/pptx/pdf 文档生成逻辑：自身不感知文档格式，文档生成工具（`skill_run` / `typst_to_pdf` / `html_to_pdf`）仅按本地相对路径引用已下载图片。系统 SHALL 在 system prompt 中提示：插入网络图片须先 `image_download` 落地、再按返回的本地路径引用。

#### Scenario: 未配置任何 Key 仍可用

- **WHEN** 用户未配置 Tavily 或任何模型外的 Key，模型请求下载图片
- **THEN** `image_download` 出现在工具列表中且可正常执行

#### Scenario: system prompt 含本地化指示

- **WHEN** Agent Loop 组装请求上下文
- **THEN** system prompt 包含"插入网络图片须先 image_download 落地、再按本地路径引用"的指示

