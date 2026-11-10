# language: pt
Funcionalidade: Só a regra tem contexto
  Cenário: Fora da regra
    Então nada muda

  Regra: Com contexto
    Contexto:
      Dado um cliente

    Cenário: Dentro da regra
      Então algo muda
