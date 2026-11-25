@catalogue @staff
Feature: Adding a book to the catalogue
  Staff add each new book to the catalogue, from its record.

  Scenario: Adding a book from its record
    Given a member of staff is signed in
    When they add a book with this record:
      """json
      {
        "title": "Emma",
        "author": "Jane Austen",
        "year": 1815
      }
      """
    Then the book "Emma" is on the shelf

  Scenario: A record in the library's own format
    Given a member of staff is signed in
    When they add a book with this record:
      ```text
      Title:  The Hobbit
      Author: J. R. R. Tolkien
        Note: the record keeps its own indentation.
      A line with three backticks: \`\`\`
      ```
    Then the book "The Hobbit" is on the shelf

  Scenario: A record with a mistake is refused
    Given a member of staff is signed in
    When they add a book with this record:
      """json
      { "title": "" }
      """
    Then the record is refused with "A book needs a title."
