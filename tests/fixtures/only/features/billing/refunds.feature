@billing
Feature: Refunds
  Scenario: Refund a card payment
    Given an order paid by card
    When the order is refunded
    Then the card gets the money back

  Scenario: Refund in store credit
    Given an order paid by card
    When the customer asks for store credit
    Then the customer gets store credit
