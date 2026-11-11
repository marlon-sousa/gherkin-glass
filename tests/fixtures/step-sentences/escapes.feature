Feature: Escapes

  Scenario Outline: Quotes after a backslash
    Then the screen shows \"hi\" and "a \" b" for <name>
    And the screen shows <> for <name>

    Examples:
      | name |  |
      | ana  |  |
