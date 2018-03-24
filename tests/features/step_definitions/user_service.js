const { Given, When, Then } = require('cucumber');
const assert = require('assert');
const Promise = require('bluebird');
const rp = require('request-promise');
const retry = require('retry-as-promised');
const OAuth2 = require('oauth').OAuth2;

const PREFIX = "http://web:8080/";

Given('a user registers at {string} using:', (url, table) => {
    let world = this;
    let data = table.rowsHash();
    return rp({
        url: PREFIX + url,
        method: 'POST',
        body: data,
        json: true
    }).then((doc) => {
        world.confirm_token = doc.confirm_token;
    })
});

When('they confirm their registration at {string}', url => {
    let world = this;
    return rp({
        url: PREFIX + url + '?confirm_token=' + escape(world.confirm_token),
        json: true
    })
})
function getOAuthAccessToken(oauth2, code, params) {
    return new Promise((resolve, reject) => {
        oauth2.getOAuthAccessToken(code, params, (e, a, r) => {
            if( !e ) resolve({access_token: a, refresh_token: r});
            else reject(e);
        })
    });
}

Then('they can login with an oauth password grant at {string} using:', (token_url, table) => {
    let world = this;
    let data = table.rowsHash();
    let oauth2 = new OAuth2('', '', PREFIX, null, token_url);

    return getOAuthAccessToken(oauth2, '', 
        {
            'grant_type': 'password',
            'username': data.name,
            'password': data.password
        }).then(doc => {
            world.access_token = doc.access_token;
            world.refresh_token = doc.refresh_token;
        });
});

Then('they can use the access token to look up their user info at {string}', url => {
    let world = this;
    let oauth2 = new OAuth2('', '', PREFIX);
    return rp({
        url: PREFIX + url,
        headers: {
            'Authorization': oauth2.buildAuthHeader(world.access_token)
        },
        json: true
    }).then(doc => {
        assert.equal(doc.name, 'new-test-user');
        assert.equal(doc.email, 'new-test-user@example.com');
    })
})

Then('they can refresh their access token at {string}', token_url => {
    let world = this;
    let oauth2 = new OAuth2('', '', PREFIX, null, token_url);
    return getOAuthAccessToken(oauth2, 
            world.refresh_token, 
            {'grant_type': 'refresh_token'}
        ).then(doc => {
            world.access_token = doc.access_token;
            world.refresh_token = doc.refresh_token;
        });
})
