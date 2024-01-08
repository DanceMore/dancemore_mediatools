# rust-koditool

born from the ashes of a Ruby-based tool that was using a [10 year out-of-date library](https://github.com/dr-impossible/xbmc-client).

when I looked into JSON-RPC support in Ruby, it seemed like everything has become abandoned or out-of-date for at least 3+ years.

for some reason, even tho software is made from words and logic and light, it is still susceptible to rot and decay. JSON-RPC itself should allow auto-discovery and introspection and Ruby libraries used to love `method_missing()` in order to support cool auto-discovery to `.method()` functionality. the Kodi JSON-RPC API is so stable, they haven't needed to update their documentation in 6 versions!

there's no logical reason that my 10 year old Ruby code shouldn't keep working, but it's become brittle and unreliable.

in order to fight back, I've been porting my favorite utilities to Rust so that they can compile from now until the foreseeable future and solve problems for longer than 10 years, because I'm now old enough to have seen multiple solutions last that long in production deployments.
