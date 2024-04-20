# AnimeSchedule Api

A complete AnimeSchedule Api v3 client implemented in Rust

To get your api keys, go to https://animeschedule.net/users/<your_username>/settings/api

You can make an app without the animelist scope, but the animelists api will fail if you do so since the tokens will not have permission.

For an in depth review of their api and which endpoints require oauth2, see https://animeschedule.net/api/v3/documentation

When using an oauth2 endpoint, you must have created an oauth2 token for the user. You can do so using the included token api
```rust
// if you want to create your token with the full scope, add scope before generating a token
client.token.add_scope("animelist");

// this requires a webserver to receive the oauth code+state for regenerate
// this crate comes with a built in localhost webserver you can use with feature `callback_server`
// by default this listens on localhost:8888, but you can change it
//
// but if you are using this in production, you will need to use your own server
client.token.set_callback_server("127.0.0.1", 1234);
// set your own custom callback for production usage
client.token.set_callback(|url| async {
    // get the code / state and return it
    (Code("".to_owned()), State("".to_owned()))
});

// regenerate tokens from scratch
client.token.regenerate().await;

// if you have a refresh key, you can exchange it for an access token
client.token.refresh_token().await;

// you can automatically try to refresh it if access token expired
// if no refresh token exists, it will regenerate the whole thing
client.token.try_refresh().await;

// you can also set the access/refresh token manually if you need to
client.token.set_refresh_token(Some("token"));
client.token.set_access_token(Some("token"));
// set the time from Instant::now() after which access token expires
client.token.set_expires_in(Some(Duration::from_secs(3600)));

// use the api
client.anime().get().q("query").genres(&["action"]).send().await;
// if using an oauth protected endpoint, make sure you have set your token
// with correct oauth tokens and scope first!
client.animelists().get().await;
client.animelists().put().route("foo").etag("etag").episodes_seen(5).note("I love anime").send().await;

// for more information on the api, see their api docs:
// https://animeschedule.net/api/v3/documentation
//
// this api follows a builder pattern, and follows their api
// it should be relatively intuitive to use
```

If you see any bugs, please report them. Mostly these may be instances "null" appears in the api but was assumed to always exist in the deserialized types. While this was quickly tested with a bit of data to hopefully get a complete picture, this is not guaranteed to be always correct. Other than that, it should be feature complete and working.

Usage of this crate is subject to the [api tos](https://animeschedule.net/api-terms-of-use)
