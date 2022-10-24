let n1 = '0', n2 = '0', res, op = ''

const operations = {
  '+': () => Number.parseInt(n1) + Number.parseInt(n2),
  '-': () => Number.parseInt(n1) - Number.parseInt(n2),
  '/': () => Number.parseInt(n1) / Number.parseInt(n2),
  '*': () => Number.parseInt(n1) * Number.parseInt(n2),
  '': () => Number.parseInt(n1)
}

function registerNumber(n) {
  if(!op) {
    if(n != 0 || n1 != '0') {
      if(n1 == '0') n1 = ''
      n1 += n
      document.querySelector('textarea').innerText = n1
    }
  } else {
    if(n != 0 || n2 != '0') {
      if(n2 == '0') n2 = ''
      n2 += n
      document.querySelector('textarea').innerText = n2
    }
  }
}

function registerOperator(aop) {
  op = aop
  document.querySelector('textarea').innerText = op
}

function clearLast() {
  if(!op) {
    n1 = '0'
    document.querySelector('textarea').innerText = n1
  } else {
    n2 = '0'
    document.querySelector('textarea').innerText = n2
  }
}

function clearAll() {
  n1 = '0', n2 = '0', op = ''
  document.querySelector('textarea').innerText = '0'
}

function result() {
  res = operations[op]()
  clearAll()
  n1 = res.toString()
  document.querySelector('textarea').innerText = res
}