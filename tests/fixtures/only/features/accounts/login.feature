Feature: Logging in
  Scenario: Log in with a password
    Given a customer with a password
    When the customer logs in
    Then the customer sees the accounts
