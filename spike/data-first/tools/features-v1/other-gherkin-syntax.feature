@rooms
Feature: Booking a study room
  Gherkin accepts other words for some of its keywords. This file uses
  them: "Example" for a scenario, "Scenario Template" for a scenario
  outline and "Scenarios" for an outline's examples. They mean exactly
  what "Scenario", "Scenario Outline" and "Examples" mean.

  Background: A member at the desk
    Given a member named "Ana" with card number 1024

  Example: Booking a free room
    Given the study room "Garden" is free at 10:00
    When "Ana" books "Garden" at 10:00
    Then "Garden" is booked for "Ana" at 10:00

  Example: Booking a room someone else holds
    Given "Bea" booked the study room "Garden" at 10:00
    When "Ana" books "Garden" at 10:00
    Then the booking is refused with "Garden is booked at 10:00."

  Scenario Template: How long a booking lasts
    Given the study room "<room>" is free at 10:00
    When "Ana" books "<room>" at 10:00
    Then "<room>" is booked until <until>

    Scenarios: Small rooms
      | room   | until |
      | Garden | 11:00 |
      | Attic  | 11:00 |

    Scenarios: Large rooms
      | room   | until |
      | Hall   | 12:00 |
