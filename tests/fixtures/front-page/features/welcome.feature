Feature: Welcome

  Scenario: A visitor opens the front page
    Given the site is served
    When a visitor opens its address
    Then the front page introduces the suite
