const { Given, When, Then } = require('cucumber');
const assert = require('assert');
const Promise = require('bluebird');
const rp = require('request-promise');
const retry = require('retry-as-promised');

const PREFIX = "http://web:8080";

Given('the service at {string} is up', (url) => {
    let retry_options = {
        max: 10,
        timeout: 60,
        name: "Service " + url + " up?"
    }
    return retry(options => 
        rp({
            url: PREFIX + url,
            json: true
        }).then( (doc) => {
            assert.equal(doc.status, 'up');
        }),
        retry_options
    );
});

Given('a user registers at {string} using:', (url, table) => {
    let world = this;
    let data = table.rowsHash();
    return rp({
        url: PREFIX + url,
        method: 'POST',
        body: data,
        json: true
    }).then((doc) => {
        console.dir(doc);
        world.confirm_token = doc.confirm_token;
    })
});
