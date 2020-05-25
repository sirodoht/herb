# herb

CLI [BitTorrent](https://www.bittorrent.org/beps/bep_0003.html) client in Rust.

Jesse Li wrote this [outstanding blog post](https://blog.jse.li/posts/torrent/) on how to create a bittorrent client in Golang.
This project follows his work but in Rust instead of Go. Praise Jesse Li!

## Run

You need a torrent file, like [this one](https://cdimage.debian.org/debian-cd/current/amd64/bt-cd/debian-10.4.0-amd64-netinst.iso.torrent).

```sh
cargo run < debian-10.4.0-amd64-netinst.iso.torrent
```

## Implementation

* [x] read torrent files
* [x] connect to tracker
* [x] read bencoded tracker responses
* [x] start concurrent TCP connections with peers
* [x] handshake peers
* [ ] communicate bitfield
* [ ] communicate messages
* [ ] seeding
* [ ] non-HTTP trackers
* [ ] multi-file torrents
* [ ] magnet links
* [ ] distributed peer discovery

## License

MIT
