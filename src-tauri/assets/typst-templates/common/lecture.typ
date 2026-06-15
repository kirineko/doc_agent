#import "/doc-agent/typst/common/fonts.typ": *

/// 中文讲义：定义框。
#let definition-zh(title, body) = block(
  fill: rgb("#f0f7ff"),
  inset: 10pt,
  radius: 4pt,
  width: 100%,
)[
  #text(font: font-sans-zh, weight: "bold")[定义（#title）] #linebreak()
  #body
]

/// 中文讲义：例题框。
#let example-zh(title, body) = block(
  fill: rgb("#fffaf0"),
  inset: 10pt,
  radius: 4pt,
  width: 100%,
)[
  #text(font: font-sans-zh, weight: "bold")[例 #title] #linebreak()
  #body
]

/// 英文讲义：Definition block.
#let definition-en(title, body) = block(
  fill: rgb("#f0f7ff"),
  inset: 10pt,
  radius: 4pt,
  width: 100%,
)[
  #text(font: font-sans-en, weight: "bold")[Definition (#title)] #linebreak()
  #body
]

/// 英文讲义：Example block.
#let example-en(title, body) = block(
  fill: rgb("#fffaf0"),
  inset: 10pt,
  radius: 4pt,
  width: 100%,
)[
  #text(font: font-sans-en, weight: "bold")[Example #title] #linebreak()
  #body
]
