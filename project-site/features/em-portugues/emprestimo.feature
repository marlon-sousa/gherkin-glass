# language: pt
@empréstimo
Funcionalidade: Empréstimo de livros
  Os sócios da biblioteca levam livros para casa.

  Contexto: Uma sócia no balcão
    Dado uma sócia chamada "Ana" com o cartão 1024

  Cenário: Empréstimo de um livro na estante
    Dado que o livro "Dom Casmurro" está na estante
    Quando "Ana" leva "Dom Casmurro"
    Então "Dom Casmurro" deve voltar em 21 dias

  Esquema do Cenário: Quanto tempo dura um empréstimo
    Dado que o livro "<título>" está na estante
    Quando "Ana" leva "<título>"
    Então "<título>" deve voltar em <dias> dias

    Exemplos: Livros e filmes
      | título       | dias |
      | Dom Casmurro | 21   |
      | Metrópolis   | 7    |
