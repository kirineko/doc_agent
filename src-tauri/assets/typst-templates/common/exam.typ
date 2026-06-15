#import "/doc-agent/typst/common/fonts.typ": *

// 试卷辅助：计算题编号 + 答题空白。避免在 `+` 列表项之间插入 `#v()`，否则会重置题号为 1。

#let calc-counter = counter("calc-item")

/// 计算/证明题：自动递增题号，末尾留答题空白。
/// 用法：`#calc-item(8)[求极限 $...$。]`
#let calc-item(score, body) = context {
  calc-counter.step()
  block(breakable: true, below: 3.5cm)[
    #calc-counter.display(). （#score 分）#body
  ]
}

/// 重置计算题计数（新大题开始前可选调用）。
#let calc-counter-reset() = {
  calc-counter.update(0)
}

/// 选择题/填空题：用 `+` 连续列出，项内可换行；不要在两项之间单独写 `#v(3.5cm)`。
#let choice-item(body) = {
  + body
}
