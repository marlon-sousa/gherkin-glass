# Refunds, with comments and blank lines that move every block.

Feature: Refunds
  A refund gives the money back.


  # By card.
  Scenario: Card
    Given a paid order

    When I refund it
    Then the card is credited



  Scenario: Cash
    # The till.
    Given a paid order
    When I refund it in cash
    Then the till is short
