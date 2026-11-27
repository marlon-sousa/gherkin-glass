Feature: Returning books

  Scenario:
    Given a book I borrowed
    When I return it
    Then it is off my card
