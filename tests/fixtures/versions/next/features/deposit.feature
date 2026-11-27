Feature: Deposit cash
  Scenario: Deposit into savings
    Given an account with 100
    When 50 is deposited
    Then the balance reads 150
