@banking @cash
Feature: Withdraw cash
  Customers take money out of their accounts.

  Background:
    Given the customer has an account

  @smoke
  Scenario: Withdraw within the balance
    Given the balance is 100
    When the customer withdraws 40
      | banknote | count |
      | 20       | 2     |
    Then the receipt says
      """
      Dispensed: 40
      """

  Scenario Outline: Withdraw an amount
    When the customer withdraws <amount>
    Then the balance is <balance>

    @regression
    Examples: Small amounts
      | amount | balance |
      | 10     | 90      |

    @regression @slow
    Examples: Large amounts
      | amount | balance |
      | 90     | 10      |

  @limits
  Rule: The daily limit applies

    Background:
      Given the daily limit is 500

    Scenario: Withdraw over the limit
      When the customer withdraws 600
      Then the withdrawal is refused

    Scenario:
      When the customer withdraws 500
      Then the withdrawal is accepted
