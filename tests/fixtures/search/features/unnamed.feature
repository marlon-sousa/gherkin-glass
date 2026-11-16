Feature:

  Rule:

    @tagged
    Scenario Outline: Pay <amount>
      When the customer pays <amount>

      @tagged @paid
      Examples:
        | amount |
        | 10     |

      Examples:
        | amount | payer |
        | 20     | Emma  |

  Rule: Nothing yet
