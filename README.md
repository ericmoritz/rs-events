# rs-events

A web service for events

## development install

```
sudo apt-get install libssl-dev libpq-dev libmysqlclient-dev libsqlite3-0 libsqlite3-dev
cargo install diesel_cli
```

Create a local rs_events database

```
$ sudo -u postgres createuser rs_events
$ sudo -u postgres createdb 
$ sudo -u postgres psql
psql=# alter user rs_events with encrypted password 'rs_events';
psql=# grant all privileges on database rs_events to rs_events;
$ diesel migration run


```



