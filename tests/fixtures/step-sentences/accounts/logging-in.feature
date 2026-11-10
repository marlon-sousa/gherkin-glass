Feature: Logging in

  Background:
    Given the user "ana" exists

  Scenario: Ana logs in
    Given the user "ana" logs in
    When she opens her account
    Then she sees "Welcome"
    And she sees "Welcome"

  Scenario: Bia logs in
    * the user "bia" logs in
    Then She sees "Welcome"

  Scenario Outline: Anyone logs in
    Given the user <name> logs in
    And the user "<name>" exists
    When I withdraw <amount>
    Then the balance is < 0 and <unknown>

    Examples:
      | name | amount |
      | cleo | 10     |
