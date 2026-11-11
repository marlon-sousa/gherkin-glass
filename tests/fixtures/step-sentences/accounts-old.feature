Feature: Old accounts

  Scenario: Star steps
    Given the user "fay" exists
    When the old account is opened
    Then it is shown
    * a star step
    And a step after a star
    Given a step with a table:
      | a |
    And a step with a doc string:
      """
      text
      """

  Scenario Outline: Two examples
    Given the user <first> and <second> meet

    Examples:
      | first |
      | ana   |

    Examples:
      | second |
      | bia    |
