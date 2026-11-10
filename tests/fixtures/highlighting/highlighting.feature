Feature: Highlighting
  Every part highlighting finds, beside text that only looks like one.

  Background:
    Given an account with 100 and a "<amount>" note

  Scenario: Parameters
    When the customer types "hello 42" and ""
    And the customer types "a \"quoted\" word"
    But a "lone quote stays plain
    * the customer's 'name' is plain

  Scenario: Numbers
    Then the balance is -3.5, then 1.000,50 or $7 on 2026-11-10.
    And R2D2 is 3rd in v1.2, 2.x, x_1 and item-4
    And the quote in "a\\" b" closes after b

  Scenario: Not an outline
    Given <amount> is plain here

  Scenario Outline: Placeholders
    Given <amount> in "<currency> 3" and <unknown>
    When the customer asks for < 5 >

    Examples:
      | amount | currency |
      | 10     | EUR      |

  Rule: Steps inside a rule

    Scenario: In a rule
      Given 2 "items"
