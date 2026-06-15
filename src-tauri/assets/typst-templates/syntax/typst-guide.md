# Typst 语法手册（doc-agent 内置）

> 官方完整参考：<https://typst.app/docs/reference/>
>
> 本手册面向 Typst 0.13（与 doc-agent 嵌入引擎一致）。**不要臆造 LaTeX 语法**；不确定时查阅官方 Reference。

---

## 0. doc-agent 工作流

### 0.1 标准流程（新建文档）

1. `typst_read_template` → `syntax/typst-guide`（**每会话首次使用 Typst 前必读**）
2. `typst_list_templates` → 选择场景模板 id
3. `typst_read_template` → 读取该场景模板（如 `exam/exam-zh`）作为起点
4. `fs_write` / `fs_patch` → 写入项目内 `.typ`（可复制模板后改）
5. `typst_to_pdf` → 编译 PDF

**仅重编译**已有 `.typ`、做小改动时：可跳过步骤 2–3，但仍须本会话已读过 `syntax/typst-guide`。

### 0.2 内置模块（`#import "/doc-agent/typst/..."`）

| 路径 | 导出 |
|------|------|
| `common/fonts.typ` | `apply-zh-body`、`apply-en-body`、`apply-zh-title`、`font-*` |
| `common/page.typ` | `page-a4`、`page-a4-compact`、`page-exam`、`footer-page-no` |
| `common/exam.typ` | `fill-blank`、`calc-item`、`calc-counter-reset`、`exam-header-zh/en`、`mc-options`、`field-line` |
| `common/lecture.typ` | `definition-zh/en`、`example-zh/en` |

### 0.3 场景模板索引

| id | 用途 | 复制后优先改什么 |
|----|------|------------------|
| `exam/exam-zh` | 中文试卷 | `#exam-header-zh` 元信息、各大题题干 |
| `exam/exam-en` | 英文试卷 | `#exam-header-en`；计算题加 `lang: "en"` |
| `report/report-zh` | 中文报告 | 标题、摘要、表格数据 |
| `report/report-en` | 英文报告 | 同上 |
| `paper/paper-zh` | 中文学术论文 | 题名、作者、摘要、正文 |
| `paper/paper-en` | 英文学术论文 | 同上 |
| `lecture/lecture-zh` | 中文讲义 | 定义/例题块、章节标题 |
| `lecture/lecture-en` | 英文讲义 | 同上 |

---

## 1. 文件结构与标记模式

| 模式 | 语法 | 示例 |
|------|------|------|
| 标记（默认） | 直接书写 | `Hello *world*` |
| 代码 | `#函数(...)`、`[内容块]` | `#align(center)[标题]` |
| 数学 | `$ ... $` | `$E = m c^2$` |

```typst
#set page(paper: "a4", margin: 2cm)
#set text(size: 11pt)

= 标题
正文。行内公式 $a^2 + b^2 = c^2$。
$ display(integral_0^1 x dif x = 1/2) $
```

---

## 2. 导入

```typst
#import "/doc-agent/typst/common/fonts.typ": *
#import "/doc-agent/typst/common/page.typ": page-exam, footer-page-no
#import "/doc-agent/typst/common/exam.typ": *
#import "/doc-agent/typst/common/lecture.typ": definition-zh, example-zh

#import "chapter.typ": section-a   // 项目内相对路径
```

多文件目录以 `main.typ` 为入口；`typst_to_pdf` 的 `path` 可指向该目录。

---

## 3. `#set` 与 `#show`

```typst
#set text(font: "Times New Roman", size: 12pt, lang: "zh")
#set par(justify: true, leading: 0.65em, first-line-indent: 2em)
#set heading(numbering: "1.1")
#set page(paper: "a4", margin: (x: 2.5cm, y: 2cm))

#show: apply-zh-body   // 推荐：用内置字体栈
```

---

## 4. 文本与段落

```typst
*粗体* _斜体_ #underline[下划线]
#align(left|center|right|justify)[…]
#pad(left: 2em)[缩进]
#v(1cm)    // 垂直空白
#h(1cm)    // 水平空白（不画线！选择题选项间距可用）
```

**填空答题线**用 `#fill-blank()`，**不要**用 `#h()` 或句号 `。` 代替（见 §18、§19）。

---

## 5. 标题与目录

```typst
= 一级标题
== 二级标题
#outline(title: [目录], indent: auto)
#pagebreak()
```

---

## 6. 列表

### 无序 `-`

```typst
- 第一项
  - 嵌套
```

### 有序 `+`（试卷填空/选择常用）

```typst
+ 第一题 …
+ 第二题 …
```

**同一 `+` 项内**可换行、`#pad`、`#mc-options`；**不要在两个 `+` 之间**单独写 `#v(3.5cm)`，否则下一题编号会从 1 重新开始。

计算/证明大题改用 `#calc-item`，不要用 `+` 列表。

---

## 7. 数学公式

行内：`$x^2$`。独立一行：整段 `$ ... $` 或 `display(...)`。

### 基础

```typst
$frac(a,b)$  $sqrt(x)$  $x_i^2$  $bar(x)$  $abs(x)$
$sum_(i=1)^n i$  $integral_a^b f(x) dif x$  $lim_(x->0) f(x)$
$ mat(1,2;3,4) $    // 矩阵
```

### 分段函数：用 `cases`，勿用 `mat(delim: "{", …)`

```typst
$ f(x) = cases(
    x^2, & x >= 0,
    -x, & x < 0,
  ) $
```

### 常用符号

```typst
$ alpha, beta, pi, epsilon, theta, oo, pm, in, subset $
$ RR, NN, ZZ, QQ, CC $    // 数集
$ dif $                    // 微分 d
```

### 公式编号

```typst
$ E = m c^2 $ <eq:emc2>
见 @eq:emc2。
```

---

## 8. 函数与变量

```typst
#let name = "Typst"
#let add(a, b) = a + b
#let frame(content, fill: white) = box(fill: fill, inset: 8pt)[#content]
```

带名参数调用：`#fill-blank(width: 2.5cm)`（**勿**写 `#fill-blank(2.5cm)`）。

---

## 9. 布局

```typst
#grid(columns: (1fr, 1fr, 1fr), gutter: 12pt, [A], [B], [C])
#box(width: 100%, inset: 8pt, stroke: 0.5pt)[边框]
```

---

## 10. 表格

```typst
#table(
  columns: (auto, 1fr, 1fr),
  inset: 8pt,
  stroke: 0.5pt,
  table.header([*列1*], [*列2*]),
  [A], [1],
)
```

---

## 11. 图片

```typst
#image("figures/chart.png", width: 80%)
#figure(image("photo.jpg", width: 60%), caption: [图注]) <fig:demo>
```

---

## 12. 页面

```typst
#import "/doc-agent/typst/common/page.typ": page-a4, page-exam, footer-page-no

#page-a4()        // 报告、论文
#page-exam()      // 试卷（紧凑边距）
#footer-page-no()
```

---

## 13. 计数器

```typst
#let c = counter("fig")
#c.step()
#context c.display("1.1")
```

试卷题号由 `calc-item` 内置计数器处理，大题开始前调用 `#calc-counter-reset()`。

---

## 14. 引用与文献

```typst
= 引言 <sec:intro>
见 @sec:intro。
#bibliography("refs.bib", style: "ieee")
```

---

## 15. 条件与循环

```typst
#if x > 0 [正] else [非正]
#for i in range(1, 4) [第 #i 项 #linebreak()]
```

---

## 16. 代码块

````typst
```python
def f(): pass
```
````

---

## 17. 中英文混排

```typst
#import "/doc-agent/typst/common/fonts.typ": *
#show: apply-zh-body
#apply-zh-title([标题], [副标题])
中文正文与 $x^2$ 混排。
```

英文：`#show: apply-en-body`、`#apply-en-title([...])`。

---

## 18. 试卷排版（`common/exam.typ`）

### 页眉与版心

```typst
#import "/doc-agent/typst/common/exam.typ": *
#import "/doc-agent/typst/common/page.typ": page-exam, footer-page-no

#show: apply-zh-body
#page-exam()
#footer-page-no()

#exam-header-zh(
  [高等数学期中考试],
  [2025–2026 学年第一学期],
  [120 分钟],
  [100],
)
```

### 填空题

```typst
+ $lim_(x->0) (sin x)/x = $ #fill-blank()
+ 导数 $f'(1) = $ #fill-blank(width: 2.5cm)
```

### 选择题

```typst
+ 题目题干 … \
  #mc-options(
    [选项 A 文字或 $公式$],
    [选项 B],
    [选项 C],
    [选项 D],
  )
```

### 计算/证明题

```typst
#calc-counter-reset()
#calc-item(10)[求 $f(x)=x^3$ 的极值点]
#calc-item(15, lang: "en")[Evaluate the integral $integral x dif x$]
```

- 题干**句末不必加句号**；答题区由 `calc-item` 的 `below: 3.5cm` 预留。
- 英文卷：`lang: "en"` → 显示 `(10 pts)`。

---

## 19. 讲义排版（`common/lecture.typ`）

```typst
#import "/doc-agent/typst/common/lecture.typ": definition-zh, example-zh

#definition-zh[数列极限][
  设 ${a_n}$ … $lim_(n->oo) a_n = A$。
]

#example-zh[1][
  证明 $lim_(n->oo) 1/n = 0$。
]
```

英文用 `definition-en`、`example-en`。

---

## 20. 常见错误

| 错误写法 | 后果 | 正确做法 |
|----------|------|----------|
| 填空用 `#h(3cm)` | 只有空白，**无横线** | `#fill-blank()` |
| 填空用句号 `。` | 显示句号而非答题区 | `#fill-blank()` |
| `#fill-blank(2.5cm)` | 编译报错 unexpected argument | `#fill-blank(width: 2.5cm)` |
| `+` 项间插入 `#v(3.5cm)` | 下一题号变 1 | 用 `#calc-item` 或同一 `+` 项内排版 |
| `mat(delim: "{", …)` 做分段 | 版式错误 | `cases(..., & cond,)` |
| 计算题题干末尾加 `。` | 与答题空白叠在一起难看 | 省略句号 |
| 凭记忆写 Typst | 编译失败 | 先读本手册与场景模板 |

---

## 21. 官方文档索引

| 主题 | URL |
|------|------|
| 总览 | <https://typst.app/docs/reference/> |
| 数学 | <https://typst.app/docs/reference/math/> |
| 页面 | <https://typst.app/docs/reference/layout/page/> |
| 表格 | <https://typst.app/docs/reference/model/table/> |
