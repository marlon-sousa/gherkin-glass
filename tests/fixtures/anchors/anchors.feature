Feature: Anchors

  Background:
    Given a customer

  Scenario: Withdraw  Cash, quickly!
    Given an account

  Scenario: The customer's card
    Given a card

  Scenario: The customer’s card
    Given another card

  Scenario: ???
    Given nothing to name it by

  Scenario: Step 2: pay
    Given a payment

  Scenario Outline: Withdraw cash
    Given <amount>

    Examples:
      | amount |
      | 10     |
      | 20     |

    Examples: Large amounts
      | amount |
      | 1000   |

  Scenario: Withdraw cash row 2
    Given a name like a row's anchor

  Rule:
    Background:
      Given a rule's background

    Scenario: Withdraw cash
      Given a repeated name

  Rule: Étape SPÉCIALE
    Scenario: Scenario
      Given a name like the keyword
