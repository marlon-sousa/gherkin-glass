Feature: Cards
  Scenario: Pay
    Given a card
    When I pay 5
    Then 5 is charged
