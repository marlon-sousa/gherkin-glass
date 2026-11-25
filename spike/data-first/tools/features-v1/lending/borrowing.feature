@lending @issue-19
Feature: Borrowing books
  Members of the library borrow books and take them home.
  Each member may hold a few books at a time, for a few weeks each.

  Background: A member at the desk
    Given a member named "Ana" with card number 1024
    And the library is open

  Scenario: Borrowing a book that is on the shelf
    A book on the shelf can be borrowed at once.

    Given the book "The Hobbit" is on the shelf
    When "Ana" borrows "The Hobbit"
    Then "The Hobbit" is due back in 21 days
    And "Ana" holds 1 book

  @slow
  Scenario: Borrowing several books at once
    Given these books are on the shelf:
      | title                  | author             |
      | Middlemarch            | George Eliot       |
      | Dom Casmurro           | Machado de Assis   |
      | Either \| Or           | Søren Kierkegaard  |
    When "Ana" borrows every book on the shelf
    Then "Ana" holds 3 books

  @outline
  Scenario Outline: How long a loan lasts
    Given "<title>" is on the shelf
    When "Ana" borrows "<title>"
    Then "<title>" is due back in <days> days

    @books
    Examples: Books
      | title        | days |
      | The Hobbit   | 21   |
      | Middlemarch  | 21   |

    @films
    Examples: Films
      | title        | days |
      | Metropolis   | 7    |

  Rule: A member holds at most 5 books
    The limit keeps books on the shelves for everyone.

    Background: A member near the limit
      Given "Ana" holds 5 books

    Scenario: Borrowing over the limit is refused
      Given the book "Emma" is on the shelf
      When "Ana" borrows "Emma"
      Then the loan is refused with "You already hold 5 books."
      And "Ana" holds 5 books

    Scenario: Returning a book makes room for another
      Given "Ana" returns "The Hobbit"
      And the book "Emma" is on the shelf
      When "Ana" borrows "Emma"
      Then "Ana" holds 5 books

  @holds
  Rule: A book on hold waits for the member who asked first

    @queue
    Scenario: A book on hold cannot be borrowed by someone else
      Given "Bea" asked to hold "Emma"
      And the book "Emma" is on the shelf
      When "Ana" borrows "Emma"
      Then the loan is refused with "Emma is on hold for another member."
