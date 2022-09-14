// copied from https://github.com/rustwasm/wasm-bindgen/tree/main/examples/hello_world
const formula = import('./pkg');

formula
  .then(m => console.log(m.parse('=UPPER("hello")')))
  .catch(console.error);

formula
  .then(m => console.log(m.parse('=F.DIV(2, 0)')))
  .catch(console.error);

formula
  .then(m => console.log(m.parse('=DATEVALUE(\'1/30/2020\')')))
  .catch(console.error);

formula
  .then(m => console.log(m.parse('=NOW()')))
  .catch(console.error);

formula
  .then(m => console.log(m.parse('={\'TEST\', SUM(1,2); 2, TRUE}')))
  .catch(console.error);

// formula
//   .then(m => console.log(m.parse('=NOT_EXISTING(2, 0)')))
//   .catch(console.error);
