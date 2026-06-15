// 中文学术论文模板
#import "/doc-agent/typst/common/fonts.typ": *
#import "/doc-agent/typst/common/page.typ": *

#show: apply-zh-body
#page-a4(margin: 2.5cm)
#footer-page-no()

#set heading(numbering: "1.1")

#align(center)[
  #text(size: 16pt, font: font-sans-zh, weight: "bold")[
    基于 Typst 的数学文档排版方法研究
  ]
  #v(0.6em)
  #text(size: 12pt)[
    张三#super[1]　李四#super[2]
  ]
  #v(0.4em)
  #text(size: 10pt)[
    #super[1]某某大学数学学院　#super[2]某某大学计算机学院
  ]
]

#v(1.2em)
#block(fill: luma(245), inset: 12pt, radius: 4pt)[
  #text(weight: "bold")[摘要]　
  本文讨论使用 Typst 进行含公式文档排版的实践，包括字体配置、定理环境与引用管理。
  实验表明，在试卷与论文场景下 Typst 可替代传统 LaTeX 工作流。
  #v(0.4em)
  #text(weight: "bold")[关键词]　Typst；数学排版；PDF；模板
]

#v(1em)
= 引言

数学类文档对公式、编号与版式一致性要求较高。Typst 提供现代语法与快速编译，适合桌面 Agent 离线生成 PDF。

= 方法

== 字体与语言

中文正文优先使用宋体与 Times New Roman 混排，数学公式使用 New Computer Modern Math：

$ integral_a^b f(x) dif x = F(b) - F(a) $

== 定理表述

#block(fill: luma(250), inset: 10pt, radius: 3pt)[
  *定理（拉格朗日中值定理）*　
  设 $f in C[a,b]$ 且在 $(a,b)$ 可导，则存在 $xi in (a,b)$ 使得
  $ f'(xi) = (f(b) - f(a)) / (b - a) $.
]

= 实验

在相同内容下对比 HTML 打印与 Typst 编译的版式稳定性，Typst 在分页与公式编号上表现更优。

= 结论

推荐在公式密集场景优先采用 Typst 导出能力。

= 参考文献

#pad(left: 2em)[
  [1] Typst Contributors. *Typst: A new markup-based typesetting system*. 2024.
]
