Feature: Refunds
  A refund gives the money back.

  Scenario: Card
    Given a paid order
    When I refund it
    Then the card is credited at once

  Scenario: Cash
    Given a paid order
    When I refund it in cash
    Then the till is short
