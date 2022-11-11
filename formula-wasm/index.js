// inspired by https://github.com/rustwasm/wasm-bindgen/tree/main/examples/
import { parse } from './pkg';

function addContent(content) {
  let result = parse(content);

  const newDiv = document.createElement("div");
  if (result === null) {
    result = "<em>null</em>";
  }
  newDiv.innerHTML = `<div style="display: inline-block; width: 300px">${content}</div>: <span style="font-weight: bold">${result}</span>`;
  document.body.appendChild(newDiv);
}

addContent('=UPPER("hello")')
addContent('=F.DIV(2, 0)')
addContent('=DATEVALUE(\'1/30/2020\')')
addContent('=NOW()')
addContent('={\'TEST\', SUM(1,2); 2, TRUE}')

// addContent('=NOT_EXISTING(2, 0)')
