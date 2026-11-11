Feature:

  Background:
    Given an unnamed background, which is allowed

  Scenario:
    Given an unnamed scenario

  Scenario Outline:
    Given <n>

    Examples:
      | n |
      | 1 |

  Rule:
    Scenario: Named
      Given a named scenario in an unnamed rule
