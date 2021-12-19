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

## Limitations

<...>

## Usage

<...>

[`Easy`]: http://thatwaseasy.example.com
[Squall]: https://github.com/mtag-dev/squall
[rustc_hash::FxHashMap]: https://docs.rs/rustc-hash/latest/rustc_hash/struct.FxHasher.html
