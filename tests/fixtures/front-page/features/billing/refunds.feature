Feature: Refunds

  Scenario: A customer is refunded
    Given a customer paid 50
    When the payment is refunded
    Then the customer's balance grows by 50
