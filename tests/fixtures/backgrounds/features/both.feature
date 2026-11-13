Feature: Transfers
  Background:
    Given an account with these balances:
      | account | balance |
      | current | 100     |
    And a note
      """text
      Opened today.
      """

  Scenario: Outside any rule
    When 10 is moved
    Then the balance is 90

  Rule: Limits
    Background: Over the limit
      Given a limit of 50

    Scenario Outline: Move <amount>
      When <amount> is moved

      Examples:
        | amount |
        | 60     |

  Rule: No background of its own
    Scenario: Only the feature's background
      Then nothing changes

    Scenario: Nothing of its own
