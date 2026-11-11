Feature: Doc strings

  Background:
    Given a background step with a doc string
      """
      From the background.
      """

  Scenario: A media type, nested indentation and an escaped delimiter
    Given a request body
      """json
      {
        "quote": "\"\"\""
      }
      """

  Scenario: Backticks, with their own escape
    Given a snippet
      ```markdown
      Code fences look like \`\`\` in Markdown.
      ```

  Scenario: A line indented less than the delimiter
    Given uneven text
        """
        indented as the delimiter
      indented less
            indented more
        """

  Scenario: Blank lines, and a comment between the step and its doc string
    Given text with blank lines
      # A comment, even with """ in it, is not part of the doc string.
      """

      between blank lines

      """

  Scenario: Tabs, and a delimiter after a hash inside the content
    Given text with a tab and a hash
      """
	one tab, as indentation
      	a tab after the indentation
      # A line like a comment, with """ in it, is content.
      x = 1 # and from a hash on any line, """ does not close it either
      """

  Rule: Doc strings inside a rule
    Example: A doc string inside a rule
      Given a step in a rule
        """text
        Inside the rule.
        """
