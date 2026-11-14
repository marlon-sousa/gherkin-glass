Ability: Kinds written with other words
  Each heading names its block's kind; the word the file writes,
  when it is another, is said at the top of the block.

  Background:
    Given a shelf

  Example: Written as an example
    Given a book

  Scenario Template: Written as a template
    Given <count> books

    Scenarios: Shelves
      | count |
      | 2     |

  Scenario: Written as a scenario with examples
    Given <count> books

    Examples:
      | count |
      | 3     |

  Scenario Outline: Written as an outline without examples
    Given a book

  Example:
    Given nothing named

  Scenario: Outside any rule
    Given a scenario named as the group

  Rule: Kinds inside a rule
    Example: Inside the rule
      Given a book
