Feature: user service

Scenario: Create User
    Given the service at "/status" is up
        And a user registers at "/oauth/register" using:
            | name      | new-test-user        |
            | password  | new-test-pass        |
            | email     | new-test-user@example.com |
    When they confirm their registration at "/oauth/register/confirm"
    Then they can login with an oauth password grant at "/oauth/access_token" using:
            | name     | new-test-user |
            | password | new-test-pass |
        And they can use the access token to look up their user info at "/oauth/me"

Scenario: Refresh token
    Given a user logins with an oauth password grant at "/oauth/access_token" using:
        | name     | test-user |
        | password | test-pass |
    When they refresh their access token at "/oauth/access_token"
    Then they can use the access token to look up their user info at "/oauth/me"
