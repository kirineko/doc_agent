#import "/doc-agent/typst/common/tokens.typ": *

#let page-a4(margin: margin-report) = {
  set page(paper: "a4", margin: margin)
}

#let page-a4-compact(margin: (x: 2cm, y: 2cm)) = {
  set page(paper: "a4", margin: margin)
}

#let page-a4-landscape(margin: (x: 2cm, y: 2cm)) = {
  set page(paper: "a4", margin: margin, flipped: true)
}

#let page-letter(margin: 1in) = {
  set page(paper: "us-letter", margin: margin)
}

#let page-exam(margin: margin-exam) = {
  set page(paper: "a4", margin: margin)
}

#let page-paper(margin: margin-paper) = {
  set page(paper: "a4", margin: margin)
}

#let page-lecture(margin: margin-lecture) = {
  set page(paper: "a4", margin: margin)
}

#let footer-page-no() = {
  set page(footer: context {
    align(center)[
      #text(size: fs-footnote, fill: color-muted)[#counter(page).display("1")]
    ]
  })
}
