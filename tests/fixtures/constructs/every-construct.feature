# A comment, which is not kept.
@billing @slow
Feature: Every construct
  A description of the feature,
  # A comment inside the description, which is not kept.
  over two lines.

  Background: A logged in customer
    Given a customer named "Ana"

  Scenario: A plain scenario
    A description of the scenario.

    Given an order with these lines:
      | item  | price |
      | pen   | 1.50  |
      | a \| b | 2     |
    # A comment between steps, which is not kept.
    When the order is placed
    Then a receipt is sent
    And no error is shown
    But nothing is charged twice
    * the order is saved

  @outline
  Scenario Outline: An outline with two named examples
    Given <count> items
    Then the total is <total>

    @cheap
    Examples: Cheap
      | count | total |
      | 1     | 2     |

    Examples: Expensive
      | count | total |
      | 9     | 90    |

  Rule: A rule with its own background
    The rule's description.

    Background: Inside the rule
      Given the rule applies

    @in-rule
    Example: A scenario inside the rule
      Given a step with a doc string
        """
        Some text.
        """
