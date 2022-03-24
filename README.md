### Run Server
```
cargo run
```

### Generate New Address
Generate an address for an output descriptor: 

**Descriptor**: wpkh(tpubD6NzVbkrYhZ4Xferm7Pz4VnjdcDPFyjVu5K4iZXQ4pVN8Cks4pHVowTBXBKRhX64pkRyJZJN5xAKj4UDNnLPb5p2sSKXhewoYx5GbTdUFWq/*)

The descriptor parameter must be URL encoded, using a library or tool like this: https://www.urlencoder.org/

```
curl http://localhost:8000/gen_new_address/wpkh%28tpubD6NzVbkrYhZ4Xferm7Pz4VnjdcDPFyjVu5K4iZXQ4pVN8Cks4pHVowTBXBKRhX64pkRyJZJN5xAKj4UDNnLPb5p2sSKXhewoYx5GbTdUFWq%2F%2A%29
```

### Expected Result
```
tb1q7vzm6vyj493davdun6m52j3fdjxkpshar7jlur
```
