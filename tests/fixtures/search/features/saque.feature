# language: pt
Funcionalidade: Saque em conta poupança

  Cenário: Saque com saldo
    Dado que o saldo é 100
    Quando o cliente saca 40
    Então o saldo é 60
