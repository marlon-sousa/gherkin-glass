@billing @slow
Feature: Refunds & <returns>
  A refund gives the money back,
  in the way it was paid.

  Background:
    Given a customer named "Ana"

  Scenario: Refund a card payment
    The plain case.

    Given an order with these lines:
      | item   | price |
      | pen    | 1.50  |
      | a \| b | 2     |
    When the order is refunded
    Then the card gets
      """json
      {"amount": 3.50}
      """
    And a note is kept
      """

      Refunded in full.
        Indented.
      """

  @outline
  Scenario Outline: Refund <count> items
    Given <count> items
    Then the total is <total>

    @cheap
    Examples: Cheap
      | count | total |
      | 1     | 2     |

    Examples:
      | count | total |

  Rule: Refunds after thirty days
    Only store credit.

    Background: An old order
      Given an order from forty days ago

    @in-rule
    Example: Store credit
      Then store credit is given

    Scenario Outline: Refund again <count> times
      Given <count> refunds

      Examples: Again
        | count |
        | 2     |

  Rule:
    Scenario: Refund a refund
      Then it is refused
