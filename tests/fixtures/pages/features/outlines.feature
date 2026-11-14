Feature: Withdraw cash at the counter
  Outlines, and the scenarios their examples produce.

  Scenario Outline: Withdraw cash
    Given a balance of <balance>
    When <amount> is withdrawn by <who>
    Then the account shows
      | balance | <amount> |
      | <left>  | <none>   |
    And the receipt reads
      """<format>
      Withdrawn: <amount>
      """

    Examples: Small
      | amount | balance | left | format |
      | 10     | 100     | 90   | text   |
      | 20     | 100     | 80   | text   |

    Examples: Large
      | amount | balance | left | format |
      | 500    | 1000    | 500  | json   |

    Examples: None yet
      | amount | balance | left | format |

  Scenario Outline:
    Given <x> and <y>

    Examples:
      | x | y   | x |
      | 1 | <x> | 3 |

  Scenario Outline: Nothing to produce
    Given <x>

    Examples:
      | x |

    Examples:
