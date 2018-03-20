var {setWorldConstructor} = require('cucumber');

function World() {
    this.access_token = null;
    this.refresh_token = null;
}

setWorldConstructor(World)
