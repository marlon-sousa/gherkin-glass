@lending
Feature: Returning books
  Books come back to the desk or to the book drop, and go back on the shelf.

  Scenario: Returning a book on time
    Given "Ana" borrowed "The Hobbit" 10 days ago
    When "Ana" returns "The Hobbit"
    Then "The Hobbit" is on the shelf
    And "Ana" owes nothing

  Scenario: Returning a book late
    Given "Ana" borrowed "The Hobbit" 30 days ago
    When "Ana" returns "The Hobbit"
    Then "Ana" owes 0.90 in fines
    And the receipt reads:
      """
      Ana, card 1024
      The Hobbit: 9 days late, 0.90
      Total: 0.90
      Three quotes, as they are printed: \"\"\"
      """

  Scenario Outline: Fines for each kind of item
    Given "Ana" borrowed the <kind> "<title>" <days> days ago
    When "Ana" returns "<title>"
    Then "Ana" owes <fine> in fines

    Examples:
      | kind | title      | days | fine |
      | book | Emma       | 22   | 0.10 |
      | film | Metropolis | 9    | 1.00 |
