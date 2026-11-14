@<b>feature-tag</b> @ticket:"><script>alert('ticket')</script>
Feature: Hostile <script>alert("feature")</script> & 'friends'
  A description with <img src=x onerror="alert('description')"> & <b>bold</b> &amp; &#60;i&#62;

  Background: Setup <iframe src="javascript:alert('background')"></iframe>
    A background's <video src=x onerror="alert('background description')"></video>
    Given a user named "<script>alert('parameter')</script>"

  @<i>scenario-tag</i>
  Scenario: Scenario <img src=x onerror=alert('scenario')> & "quoted"
    Line one of <a href="javascript:alert('scenario description')">a link</a>
    Line two with <!-- a comment --> and ]]>
    When the text is <i>italic</i> & "<b>bold</b>" with 'apostrophes' and 3 <u>numbers</u>
    Then the table reads
      | <th>head</th>                       | name & 'more'         |
      | <script>alert('cell')</script>      | a \| b <br> "c"       |
    And the doc string reads
      """<i>media</i>
      <script>alert('doc string')</script>
      </pre><b>outside the pre</b>
      """

  Rule: Rule <svg onload="alert('rule')"></svg>
    <a href="javascript:alert('rule description')">A rule's link</a>

    Background: Rule's <style>body { display: none }</style>
      Given the style is "<style>body { display: none }</style>"

    @<em>outline-tag</em>
    Scenario Outline: Outline <object data="x"></object> for <who>
      An outline's <embed src="javascript:alert('outline description')">
      Given "<who>" signs in as <who>

      @<s>examples-tag</s>
      Examples: Examples <textarea>written</textarea>
        These examples' <details open ontoggle="alert('examples description')">
        | who                                 |
        | <script>alert('row')</script>       |
        | <img src=x onerror="alert('row')">  |
