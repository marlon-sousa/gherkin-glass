Feature: Pay a bill
  Scenario: A bill is paid in full
    Given a bill of 10 euros
    When it is paid
    Then nothing is owed
