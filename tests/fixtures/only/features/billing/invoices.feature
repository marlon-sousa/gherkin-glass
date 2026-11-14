@billing
Feature: Invoices
  Scenario: An invoice for each order
    Given an order paid by card
    Then the customer gets an invoice
