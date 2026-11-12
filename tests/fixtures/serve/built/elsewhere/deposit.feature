Feature: Deposit cash

  Scenario: Cash paid in
    Given the account has 100
    When the holder deposits 20
    Then the account has 120
