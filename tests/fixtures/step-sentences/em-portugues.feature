# language: pt
Funcionalidade: Entrar

  Cenário: Ana entra
    Dado o usuário "ana" entra
    Quando ela abre a conta
    Então ela vê "Bem-vinda"
    E ela vê "Bem-vinda"
    Mas ela não vê "Erro"

  Cenário: Gil
    Dado the user "gil" exists
