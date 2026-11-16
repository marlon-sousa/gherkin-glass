@onboarding
Feature: Welcome
  Scenario: A new customer is greeted
    Given a new customer named "Ana"
    When she opens the bank's app
    Then she reads "Welcome, Ana"
