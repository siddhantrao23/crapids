# crapids

Challenges done as a part of the [Fly.io Distributed Systems Challenge](https://fly.io/dist-sys). 

The first Implementation was inspired by Jon Gjengset but fell short due to certain limitations
- required writing the library code to bridge to maelstorm
- using pre-written async libs seemed filled with errors
- using go with the intended library from the authors is the best way to go

The first 3 challenges are successful in Rust.

### Build and Run
```sh
cargo b
./maelstrom test -w broadcast --bin target/debug/broadcast --node-count 5 --time-limit 20 --rate 10 --nemesis partition
```

## Go implementation
*TODO*
