# herb

CLI [BitTorrent](https://www.bittorrent.org/beps/bep_0003.html) client in Rust.

Jesse Li wrote this [outstanding blog post](https://blog.jse.li/posts/torrent/) on how to create a bittorrent client in Golang.
This project follows his work but in Rust instead of Go. Praise Jesse Li!

## Run

You need a torrent file, like [this one](https://cdimage.debian.org/debian-cd/current/amd64/bt-cd/debian-10.4.0-amd64-netinst.iso.torrent).

```sh
cargo run < debian-10.4.0-amd64-netinst.iso.torrent
```

## Implementation progress

* [x] read torrent files
* [x] connect to tracker
* [x] communicate bencoded messages with tracker
* [x] concurrent TCP connections with peers
* [x] handshake peers
* [x] bitfields
* [x] messages
* [ ] mpmc message passing between peer connection processes
* [ ] saving to disk
* [ ] seeding
* [ ] non-HTTP trackers
* [ ] multi-file torrents
* [ ] magnet links
* [ ] distributed peer discovery

## License

MIT
