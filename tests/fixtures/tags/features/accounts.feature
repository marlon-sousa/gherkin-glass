@Slow @slow @index
Feature: Accounts
  Tags that share their name once lower-cased, and one named as the index.

  @issue-12 @needs:db
  Scenario: Open an account
    Given a customer

  Scenario Outline: Close <count> accounts
    Given <count> accounts

    @ticket:ç/1#2
    Examples: Many
      | count |
      | 2     |

    Examples: None
      | count |

  @issue-7
  Rule: Joint accounts
    @issue-12
    Scenario: Open a joint account
      Given two customers

    Scenario: Close a joint account
      Given two customers

  @needs:db
  Rule: Nothing yet
