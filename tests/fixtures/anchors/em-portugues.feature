# language: pt
Funcionalidade: Contas

  Contexto:
    Dado um cliente

  Cenário: Saque em conta poupança
    Dado uma conta

  Esquema do Cenário:
    Dado <valor>

    Exemplos:
      | valor |
      | 10    |
