# High performance HTTP router

Initially created as routing subsystem for Python's [Squall] API Framework.

Designed to mitigate the overhead of large-scale routing tables.

Squall router avoids massive Regex processing and performs validation only in those places where it is strictly necessary

The primary concept is finding relevant handlers using the HashMap-based database.
Then perform computing only suitable entities and not the entire table.
Regex matching is performed only for those fields which have defined data validators.

It is suitable for:
- API Gateways
- API services where routing cache not an option because of a lot of different parameters values
- Just to ensure that routing is not a point of performance drop after adding a batch of new endpoints


## Performance 
Benchmark code based on actix-router, so it also here for reference.

<div>
<img src="https://raw.githubusercontent.com/mtag-dev/rs-squall-router/main/assets/violin.svg" />
</div>


## HashDoS safety note

Crate's routing database based on [rustc_hash::FxHashMap] which is non-cryptographic hash.

It is applicable and safe here because the database is filled only by endpoints registration and not during requests handling.

## Usage

<...>


## Limitations

At the moment, there are two well known limitations

#### Equal length routes with mixed `dynamic` and `static` parameters in the same position

`/api/v1/user/{user_id}/info/full`

`/api/v1/user/parameter/{prm}/details`

In case of such API design request to `/api/v1/user/parameter/info/full` will not find the handler.
It is applicable only for routes with at least one dynamic parameter and with the same amount of octets.
We have checked how many APIs have such behavior, it is less than 1% which can be easily aligned with this contract.

In the next releases, it will be covered by assertion to prevent a bad user experience.

There are a few ways under discussion how to make such limitations not applicable.

#### Wildcard route suffix

`/static/{path:.*}` - Dynamic routing part depends on octets splitting so it is not applicable.

Instead, you should use location api:

```Rust
let handler_id = 8;
router.add_http_location("GET".to_string(), "/static".to_string(), handler_id);
```


[`Easy`]: http://thatwaseasy.example.com
[Squall]: https://github.com/mtag-dev/squall
[rustc_hash::FxHashMap]: https://docs.rs/rustc-hash/latest/rustc_hash/struct.FxHasher.html
