Feature:

  Rule:

    @tagged
    Scenario Outline: Pay <amount>
      When the customer pays <amount>

      @tagged @paid
      Examples:
        | amount |
        | 10     |

  Rule: Nothing yet
