Feature: Transfer money
  Scenario: Transfer to another account
    Given an account with 100
    When 30 is transferred
    Then the balance reads 70
