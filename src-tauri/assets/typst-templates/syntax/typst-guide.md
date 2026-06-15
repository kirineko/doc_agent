# Typst 语法手册（doc-agent 内置）

> 官方完整参考：<https://typst.app/docs/reference/>
>
> 本手册面向 Typst 0.13（与 doc-agent 嵌入引擎一致）。**不要臆造 LaTeX 语法**；不确定时查阅官方 Reference。

---

## 0. doc-agent 工作流

1. `typst_read_template` → `syntax/typst-guide`（本文件，**每次会话首次使用 Typst 前必读**）
2. 按需 `typst_read_template` → 场景模板（`report/report-zh`、`exam/exam-zh`、`paper/paper-zh`、`lecture/lecture-zh` 等）
3. `fs_write` / `fs_patch` 写入或修改项目内 `.typ`
4. `typst_to_pdf` 编译为 PDF

内置模块虚拟路径（`#import` 可用）：

| 路径 | 内容 |
|------|------|
| `/doc-agent/typst/common/fonts.typ` | 中英字体栈、`apply-zh-body` / `apply-en-body` |
| `/doc-agent/typst/common/page.typ` | `page-a4`、`footer-page-no` 等 |
| `/doc-agent/typst/common/exam.typ` | 试卷 `calc-item` 等辅助函数 |

---

## 1. 文件结构与标记模式

Typst 源文件为 `.typ` 纯文本。三种嵌入模式：

| 模式 | 语法 | 示例 |
|------|------|------|
| 标记（默认） | 直接书写 | `Hello *world*` |
| 代码 | `#函数(...)` `[内容块]` | `#align(center)[标题]` |
| 数学 | `$ ... $` | `$E = m c^2$` |

最小文档：

```typst
#set page(paper: "a4", margin: 2cm)
#set text(size: 11pt)

= 标题

正文段落。行内公式 $a^2 + b^2 = c^2$。

$ display(integral_0^1 x dif x = 1/2) $
```

---

## 2. 导入与模块化

```typst
// 导入整个模块
#import "/doc-agent/typst/common/fonts.typ": *

// 导入指定符号
#import "/doc-agent/typst/common/page.typ": page-a4, footer-page-no
#import "/doc-agent/typst/common/exam.typ": calc-item, calc-counter-reset

// 相对路径（同项目目录内）
#import "chapter1.typ": intro
```

项目内多文件目录：入口为 `main.typ`，`typst_to_pdf` 的 `path` 可指向该目录。

---

## 3. `#set` 与 `#show`

`#set` 修改后续默认值；`#show` 定义样式规则。

```typst
#set text(font: "Times New Roman", size: 12pt, lang: "zh")
#set par(justify: true, leading: 0.65em, first-line-indent: 2em)
#set heading(numbering: "1.1")

#show heading.where(level: 1): it => {
  pagebreak(weak: true)
  block(above: 1.5em, below: 1em)[
    #text(size: 16pt, weight: "bold")[#it.body]
  ]
}

#show: doc => {
  set par(justify: true)
  doc
}
```

常用 `#set`：

```typst
#set page(paper: "a4", margin: (x: 2.5cm, y: 2cm))
#set enum(numbering: "1.")
#set list(marker: [•])
```

---

## 4. 文本与段落

```typst
*粗体* _斜体_ #underline[下划线] #strike[删除线]
#text(size: 14pt, fill: blue)[彩色文字]
#text(font: "Arial")[指定字体]

换行用空行分段。强制换行：`linebreak()` 或 `\`

#align(left|center|right|justify)[对齐内容]

#pad(left: 2em)[左侧缩进]
#h(1cm)  // 水平空白
#v(1cm)  // 垂直空白
```

链接：

```typst
#link("https://typst.app")[Typst 官网]
```

---

## 5. 标题

```typst
= 一级标题
== 二级标题
=== 三级标题

// 无编号标题
#heading(outlined: false)[附录]

// 目录
#outline(title: [目录], indent: auto)
```

---

## 6. 列表

### 无序列表

```typst
- 第一项
- 第二项
  - 嵌套项
```

### 有序列表（`+`）

```typst
+ 第一步
+ 第二步
+ 第三步
```

编号列表项之间可换行续写，**同一项内**可插入 `#v()`：

```typst
+ 题目一
  续写说明
  #v(1cm)
+ 题目二
```

两个 `+` 之间插入**独立**的 `#v(3cm)` 块会打断列表，编号会重新从 1 开始。

### 定义列表

```typst
/ Term: 定义内容
/ API: 应用程序接口
```

### 枚举环境

```typst
#set enum(numbering: "(1)")

+ 条目 A
+ 条目 B
```

---

## 7. 数学公式

行内：`$x^2$`。独立显示：`$ ... $` 单独成段，或 `display(...)`。

### 基础

```typst
$x_i^2$                    // 上下标
$sqrt(x)$ $root(3, x)$     // 根号
$a / b$ $frac(a, b)$       // 分数
$bar(x)$ $hat(x)$ $vec(x)$ // 修饰
$abs(x)$ $norm(v)$          // 绝对值、范数
```

### 大型运算符

```typst
$sum_(i=1)^n i$
$product_(k=1)^n a_k$
$integral_a^b f(x) dif x$
$lim_(x -> 0) sin(x)/x$
$union_(i in cal(I)) A_i$
```

### 矩阵

```typst
$ mat(
    1, 2, 3;
    4, 5, 6;
  ) $

$ det mat(1, 2; 3, 4) $
```

### 分段函数 `cases`

分支之间用**逗号**分隔：

```typst
$ f(x) = cases(
    x^2, & x >= 0,
    -x, & x < 0,
  ) $

$ f(x) = cases(
    (2 x^3) / (1 + x^2), & x <= 1,
    1, & x > 1,
  ) $
```

### 对齐多行公式

```typst
$ f(x) &= x^2 + 1 \\
       &= (x+i)(x-i) + 1 $
```

### 常用符号

```typst
$ alpha, beta, gamma, delta, epsilon, theta, lambda, mu, pi, sigma, omega $
$ RR, NN, ZZ, QQ, CC $       // 数集
$ in, subset, union, intersect $
$ <=, >=, !=, approx, equiv $
$ oo, pm, times, div, cdot $
$ dif$                       // 微分 d
```

### 公式编号

```typst
$ E = m c^2 $ <eq:e=mc2>

如 @eq:e=mc2 所示。
```

---

## 8. 函数、变量与块

```typst
#let name = "Typst"
#let x = 3.14
#let add(a, b) = a + b

#let block-title(title, body) = block(
  fill: luma(240),
  inset: 10pt,
  radius: 4pt,
  width: 100%,
)[
  *#title* #linebreak()
  #body
]

#block-title[提示][这是一个可复用块。]

#let greeting(name) = [Hello, #name!]
#greeting("World")
```

带默认参数：

```typst
#let frame(content, fill: white) = box(fill: fill, inset: 8pt)[#content]
```

---

## 9. 布局

```typst
#grid(
  columns: (1fr, 1fr, 1fr),
  gutter: 12pt,
  [A], [B], [C],
  [1], [2], [3],
)

#stack(dir: ltr, spacing: 1em)[左][中][右]

#columns(2)[
  第一栏内容…
  #colbreak()
  第二栏内容…
]

#box(width: 100%, inset: 8pt, stroke: 0.5pt)[边框内容]
```

---

## 10. 表格

```typst
#table(
  columns: (auto, 1fr, 1fr),
  align: (left, center, right),
  inset: 8pt,
  stroke: 0.5pt,
  table.header([*列1*], [*列2*], [*列3*]),
  [A], [1], [2],
  [B], [3], [4],
)
```

`table` 与 `grid` 单元格均用 `[...]` 包裹内容。表头可用 `table.header(...)`。

---

## 11. 图片与图形

```typst
// 项目内相对路径
#image("figures/chart.png", width: 80%)

#figure(
  image("photo.jpg", width: 60%),
  caption: [示例图片],
) <fig:demo>

见 @fig:demo。
```

---

## 12. 页面与页眉页脚

```typst
#set page(
  paper: "a4",
  margin: (left: 2.5cm, right: 2.5cm, top: 2cm, bottom: 2cm),
  numbering: "1",
)

#set page(header: align(right)[章节标题])
#set page(footer: context {
  align(center)[#counter(page).display("1")]
})

#pagebreak()
#pagebreak(weak: true)  // 尽量分页
```

doc-agent 内置：

```typst
#import "/doc-agent/typst/common/page.typ": page-a4, footer-page-no
#page-a4()
#footer-page-no()
```

页码、计数器显示需在 `context { }` 内（内置 `footer-page-no` 已处理）。

---

## 13. 计数器

```typst
#let fig-counter = counter("figure")
#fig-counter.step()
#context fig-counter.display("Fig. 1")

#counter(page).display("I")  // 需在 context 中
```

试卷计算题（内置）：

```typst
#import "/doc-agent/typst/common/exam.typ": calc-item, calc-counter-reset

#calc-counter-reset()
#calc-item(8)[求 $f'(x)$，其中 $f(x)=x^3$。]
#calc-item(10)[计算 $integral_0^1 x^2 dif x$。]
```

---

## 14. 引用、标签与参考文献

```typst
= 引言 <sec:intro>

见 @sec:intro。见图 @fig:demo 与式 @eq:e=mc2。

#bibliography("refs.bib", style: "ieee")
```

无 `.bib` 文件时可手写：

```typst
#pad(left: 2em)[
  [1] Author. *Title*. Publisher, 2024.
]
```

---

## 15. 条件与循环

```typst
#if x > 0 [
  正数
] else [
  非正数
]

#for i in range(1, 4) [
  第 #i 项 #linebreak()
]

#for (k, v) in (a: 1, b: 2).pairs() [
  #k = #v
]
```

---

## 16. 代码与原始文本

展示代码块（围栏代码）：

````typst
```python
def hello():
    print("hi")
```
````

读取外部文件原文：

```typst
#raw("data/sample.txt", lang: "text")
```

---

## 17. 中英文混排

```typst
#import "/doc-agent/typst/common/fonts.typ": *

#show: apply-zh-body
#apply-zh-title([文档标题], [副标题])

中文正文 $mixed text$ 与公式混排。
```

英文文档：

```typst
#show: apply-en-body
#apply-en-title([Document Title], [Subtitle])
```

---

## 18. 场景模板索引

| id | 用途 |
|----|------|
| `syntax/typst-guide` | 本手册 |
| `report/report-zh` | 中文报告 |
| `report/report-en` | 英文报告 |
| `exam/exam-zh` | 中文试卷 |
| `exam/exam-en` | 英文试卷 |
| `paper/paper-zh` | 中文学术论文 |
| `paper/paper-en` | 英文学术论文 |
| `lecture/lecture-zh` | 中文讲义 |
| `lecture/lecture-en` | 英文讲义 |

---

## 19. 官方文档索引

| 主题 | URL |
|------|-----|
| 总览 | <https://typst.app/docs/reference/> |
| 语法 | <https://typst.app/docs/reference/syntax/> |
| 函数 | <https://typst.app/docs/reference/foundations/function/> |
| 数学 | <https://typst.app/docs/reference/math/> |
| 页面 | <https://typst.app/docs/reference/layout/page/> |
| 表格 | <https://typst.app/docs/reference/model/table/> |
| 引用 | <https://typst.app/docs/reference/model/bibliography/> |
