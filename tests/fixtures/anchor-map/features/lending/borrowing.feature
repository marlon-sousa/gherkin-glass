Feature: Borrowing books

  Background:
    Given the library is open

  Scenario: Borrow a book
    Given a book on the shelf
    When I borrow it
    Then it is on my card

  Scenario: Borrow a book
    Given a book someone holds
    When I borrow it
    Then I am told it is held

  Rule:

    Background:
      Given I owe nothing

    Scenario Outline: Borrow up to a limit
      Given I have <count> books
      When I borrow one more
      Then I am <told>

      Examples: Under the limit
        | count | told          |
        | 1     | allowed       |
        # Two books is the edge of the limit.

        | 2     | allowed       |

      Examples:
        | count | told          |
        | 5     | refused       |
