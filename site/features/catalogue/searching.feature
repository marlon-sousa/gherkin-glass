@catalogue
Feature: Searching the catalogue
  Anyone can search the catalogue, members or not.

  Background:
    Given the catalogue holds these books:
      | title          | author           | year |
      | The Hobbit     | J. R. R. Tolkien | 1937 |
      | Middlemarch    | George Eliot     | 1871 |
      | Dom Casmurro   | Machado de Assis | 1899 |

  Scenario: Searching by title
    When someone searches for "hobbit"
    Then the results are:
      | title      |
      | The Hobbit |

  Scenario: Searching by author
    When someone searches for "Eliot"
    Then the results are:
      | title       |
      | Middlemarch |

  Scenario: A search that finds nothing
    When someone searches for "dragons"
    Then the results are empty
    And the page says "Nothing matches dragons."
