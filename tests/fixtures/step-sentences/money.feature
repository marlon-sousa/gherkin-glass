Feature: Money

  Scenario: Withdraw
    When I withdraw 10
    And I withdraw 20
    Then the note says "" and "it's done"
    And the user's card is "kept
    And the screen shows “hi”
    And the café is paid

  Scenario: Starting with and
    And the first step is an and
    Then the café is paid

  Rule: Every user
    Background:
      Given the user "dan" exists

    Scenario: Eve logs in
      When the user "eve" logs in
      Then the user <name> logs in
