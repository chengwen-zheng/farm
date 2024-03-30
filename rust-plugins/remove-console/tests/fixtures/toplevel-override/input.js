let console = {
  log: (msg) => {},
}

function func1() {
  console.log('remove console tests in function')
}

console.log('remove console tests at top level')
