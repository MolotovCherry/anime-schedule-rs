# AnimeSchedule Api

A complete AnimeSchedule Api v3 client implemented in Rust

To get your api keys, go to https://animeschedule.net/users/<your_username>/settings/api

You can make an app without the animelist scope, but the animelists api will fail if you do so since the tokens will not have permission.

For an in depth review of their api and which endpoints require oauth2, see https://animeschedule.net/api/v3/documentation

When using an oauth2 endpoint, you must have created an oauth2 token for the user. You can do so using the included token api
```rust
// if you want to create your token with the full scope, add scope before generating a token
client.auth.add_scope("animelist");

// this requires a webserver to receive the oauth code+state for regenerate
// set your own custom callback for production usage
client.auth.set_callback(|url, state| async {
    // the url passed in is the one the client needs to navigate to

    // receive the state on your webserver, compare it to the received state above
    // to ensure it's valid and the right client. if you return wrong state, the
    // regenerate api will fail due to security check

    // get the code / state and return it
    (AuthorizationCode::new("".to_owned()), CsrfToken::new("".to_owned()))
});

// When dealing with access/refresh tokens, please be aware
// that the site api sets both to 3600
// if you pass this threashold, you are required to regenerate the tokens

// regenerate tokens from scratch
client.auth.regenerate().await;

// if you have a refresh key, you can exchange it for an access token
client.auth.refresh_token().await;

// you can automatically try to refresh it if access token expired
// if no refresh token exists, it will regenerate the whole thing
client.auth.try_refresh().await;

// you can also set the access/refresh token manually if you need to
client.auth.set_refresh_token(Some("token"));
client.auth.set_access_token(Some("token"));
// set the time from Instant::now() after which access token expires
client.auth.set_expires_in(Some(Duration::from_secs(3600)));

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
