Feature: Withdraw cash

  Scenario: Enough money in the account
    Given the account has 100
    When the holder withdraws 20
    Then the account has 80
