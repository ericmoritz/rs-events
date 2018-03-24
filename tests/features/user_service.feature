Feature: user service

Scenario: Create User
    Given a user registers at "oauth/register" using:
            | name      | new-test-user        |
            | password  | new-test-pass        |
            | email     | new-test-user@example.com |
    When they confirm their registration at "oauth/register/confirm"
    Then they can login with an oauth password grant at "oauth/token" using:
            | name     | new-test-user |
            | password | new-test-pass |
        And they can use the access token to look up their user info at "oauth/me"
        And they can refresh their access token at "oauth/token"
        And they can use the access token to look up their user info at "oauth/me"
