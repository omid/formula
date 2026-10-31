// inspired by https://github.com/rustwasm/wasm-bindgen/tree/main/examples/
import { parse } from './pkg';

function addContent(content, rows) {
  const result = parse(content, rows);

  // textContent, so formula results are never interpreted as HTML
  const formula = document.createElement("div");
  formula.style.display = "inline-block";
  formula.style.width = "300px";
  formula.textContent = rows ? `${content} with ${JSON.stringify(rows)}` : content;

  const value = document.createElement(result === null ? "em" : "span");
  value.style.fontWeight = "bold";
  value.textContent = result === null ? "null" : String(result);

  const newDiv = document.createElement("div");
  newDiv.append(formula, ": ", value);
  document.body.appendChild(newDiv);
}

addContent('=UPPER("hello")')
addContent('=F.DIV(2, 0)')
addContent('=DATEVALUE(\'1/30/2020\')')
addContent('=NOW()')
addContent('={\'TEST\', SUM(1,2); 2, TRUE}')
addContent('=SUM(A1:B2)', [[1, 2], [3, null]])
addContent('=TEXTJOIN(", ", TRUE, A:A)', [['apples'], [null], ['pears']])

// addContent('=NOT_EXISTING(2, 0)')
