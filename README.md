### Run Server
```
cargo run
```
### Generate Output Descriptors
Generate the output descriptors for a multisig wallet, which is a json object with the threshold and the cosigners. 

Example request:
```
curl --location --request POST 'http://127.0.0.1:8000/gen_output_descriptor' \
--header 'Content-Type: application/json' \
--data-raw '{
    "threshold": 2,
    "cosigners" :[{
        "xfp":"20F24288",
        "xpub":"Vpub5grEFi7zATrHdP3w4NjjGx5KYdJvdPs3pEEtKFxfrfnMfm5Mv81GmUQoanSYvnJyrgSGuP4DdW5dqxjXAfjjVxgQeNY5wr7LfqWKUGjwhyT",
        "derivation_path":"m/48'\''/0'\''/0'\''/2'\''"
    },
    {
        "xfp":"E9A0CF4A",
        "xpub":"Vpub5gwgZHMqLjsjcdtqRZ4E441r8itvCoeQEBQ29iDzn5ahkPi8y4RqcVMBjJngxzonpDnMX5UQLeBLkC9wdBHyQqJ7xxt5BvmnYUoXRiUGLQM",
        "derivation_path":"m/48'\''/0'\''/0'\''/2'\''"
    }]
}'
```
Example response:
```
{
    "descriptor": "wsh(sortedmulti(2,[20f24288/48'/0'/0'/2']tpubD9zJG3Z4c9LLBCTeEcq64yFtVtfDHffWspDxKLY3apTbu4ocjFoD4vXz4XV2tfMAEQ8p9Km6CiEHBYqVhhG3qPPEcBZqPnwYuWx9RVmiVLz/0/*,[e9a0cf4a/48'/0'/0'/2']tpubDA5kZcnunRMnATJYbo9ar5CR5zFCs5SsHmP69noNWEFwyhSPnCDmuwUND3qAvsqyBwUtm2BGurKz5nFvACpHkFzwvmupdsbznAFMNypghFB/0/*))#aakuctju",
    "change_descriptor": "wsh(sortedmulti(2,[20f24288/48'/0'/0'/2']tpubD9zJG3Z4c9LLBCTeEcq64yFtVtfDHffWspDxKLY3apTbu4ocjFoD4vXz4XV2tfMAEQ8p9Km6CiEHBYqVhhG3qPPEcBZqPnwYuWx9RVmiVLz/1/*,[e9a0cf4a/48'/0'/0'/2']tpubDA5kZcnunRMnATJYbo9ar5CR5zFCs5SsHmP69noNWEFwyhSPnCDmuwUND3qAvsqyBwUtm2BGurKz5nFvACpHkFzwvmupdsbznAFMNypghFB/1/*))#yw9ckc8f"
}
```

### Generate New Address
Generate an address for an output descriptor.

Example request:
```
curl --location --request POST 'http://127.0.0.1:8000/gen_new_address' \
--header 'Content-Type: application/json' \
--data-raw '{
    "descriptor": "wsh(sortedmulti(2,[20f24288/48'\''/0'\''/0'\''/2'\'']tpubD9zJG3Z4c9LLBCTeEcq64yFtVtfDHffWspDxKLY3apTbu4ocjFoD4vXz4XV2tfMAEQ8p9Km6CiEHBYqVhhG3qPPEcBZqPnwYuWx9RVmiVLz/0/*,[e9a0cf4a/48'\''/0'\''/0'\''/2'\'']tpubDA5kZcnunRMnATJYbo9ar5CR5zFCs5SsHmP69noNWEFwyhSPnCDmuwUND3qAvsqyBwUtm2BGurKz5nFvACpHkFzwvmupdsbznAFMNypghFB/0/*))#aakuctju"
}'
```

Example response:
```
tb1qrn285mxq2usmct66pwuct7cc07f8g7c8eemvll4k88v2s97t5d6q3ta5lq
```


### Get Multisig Wallet
Get Multisig Wallet from the an output descriptor

Example request:
```
curl --location --request POST 'http://127.0.0.1:8000/get_multisig' \
--header 'Content-Type: application/json' \
--data-raw '{
    "descriptor": "wsh(sortedmulti(2,[20f24288/48'\''/0'\''/0'\''/2'\'']tpubD9zJG3Z4c9LLBCTeEcq64yFtVtfDHffWspDxKLY3apTbu4ocjFoD4vXz4XV2tfMAEQ8p9Km6CiEHBYqVhhG3qPPEcBZqPnwYuWx9RVmiVLz/0/*,[e9a0cf4a/48'\''/0'\''/0'\''/2'\'']tpubDA5kZcnunRMnATJYbo9ar5CR5zFCs5SsHmP69noNWEFwyhSPnCDmuwUND3qAvsqyBwUtm2BGurKz5nFvACpHkFzwvmupdsbznAFMNypghFB/0/*))#aakuctju"
}'
```

Example response:
```
{
    "threshold": 2,
    "cosigners": [
        {
            "xfp": "20f24288",
            "xpub": "Vpub5grEFi7zATrHdP3w4NjjGx5KYdJvdPs3pEEtKFxfrfnMfm5Mv81GmUQoanSYvnJyrgSGuP4DdW5dqxjXAfjjVxgQeNY5wr7LfqWKUGjwhyT",
            "derivation_path": "m/48'/0'/0'/2'"
        },
        {
            "xfp": "e9a0cf4a",
            "xpub": "Vpub5gwgZHMqLjsjcdtqRZ4E441r8itvCoeQEBQ29iDzn5ahkPi8y4RqcVMBjJngxzonpDnMX5UQLeBLkC9wdBHyQqJ7xxt5BvmnYUoXRiUGLQM",
            "derivation_path": "m/48'/0'/0'/2'"
        }
    ]
}
```

### Get Balance
Get Balance in sats for a wallets output descriptor

Example request:
```
curl --location --request POST 'http://127.0.0.1:8000/get_balance' \
--header 'Content-Type: application/json' \
--data-raw '{
    "descriptor": "wsh(sortedmulti(2,[20f24288/48'\''/0'\''/0'\''/2'\'']tpubD9zJG3Z4c9LLBCTeEcq64yFtVtfDHffWspDxKLY3apTbu4ocjFoD4vXz4XV2tfMAEQ8p9Km6CiEHBYqVhhG3qPPEcBZqPnwYuWx9RVmiVLz/0/*,[e9a0cf4a/48'\''/0'\''/0'\''/2'\'']tpubDA5kZcnunRMnATJYbo9ar5CR5zFCs5SsHmP69noNWEFwyhSPnCDmuwUND3qAvsqyBwUtm2BGurKz5nFvACpHkFzwvmupdsbznAFMNypghFB/0/*))#aakuctju"
}'
```

Example response:
```
32000
```

### Generate PSBT
Generate a PSBT from the output descriptors and transaction details, it returns a base64 encoded psbt

Example request:
```
curl --location --request POST 'http://127.0.0.1:8000/gen_psbt' \
--header 'Content-Type: application/json' \
--data-raw '{
    "descriptors": {
        "descriptor": "wsh(sortedmulti(2,tpubD9zJG3Z4c9LLBCTeEcq64yFtVtfDHffWspDxKLY3apTbu4ocjFoD4vXz4XV2tfMAEQ8p9Km6CiEHBYqVhhG3qPPEcBZqPnwYuWx9RVmiVLz/0/*,tpubDA5kZcnunRMnATJYbo9ar5CR5zFCs5SsHmP69noNWEFwyhSPnCDmuwUND3qAvsqyBwUtm2BGurKz5nFvACpHkFzwvmupdsbznAFMNypghFB/0/*))#3xvsph9g",
        "change_descriptor": "wsh(sortedmulti(2,tpubD9zJG3Z4c9LLBCTeEcq64yFtVtfDHffWspDxKLY3apTbu4ocjFoD4vXz4XV2tfMAEQ8p9Km6CiEHBYqVhhG3qPPEcBZqPnwYuWx9RVmiVLz/1/*,tpubDA5kZcnunRMnATJYbo9ar5CR5zFCs5SsHmP69noNWEFwyhSPnCDmuwUND3qAvsqyBwUtm2BGurKz5nFvACpHkFzwvmupdsbznAFMNypghFB/1/*))#jxu9yn3m"
    },
    "to_address":"tb1qdd2ef3a2s3qnksp6nelvpgmm06v7sp8uuy9egm66ww8hpnfg6cmsrncyp5",
    "amount": 10000,
    "fee_sat_per_vb": 5.0
}'
```

Example response:
```
cHNidP8BAIkBAAAAAaO4tywU74A2uTwsnAcPzGg1qGS2Ym2PU2/vs3EPbCZkAAAAAAD9////AkNOAAAAAAAAIgAgXvpJmTbDmA7cBYYipzkdFUyTxsvhHQYzvDIQjPIIjA8QJwAAAAAAACIAIGtVlMeqhEE7QDqefsCje36Z6AT84QuUb1pzj3DNKNY3AAAAAAABAOoCAAAAAAEBl1u4ioLRLXTClQG4psXNhZeKS67jfWgcrcTLvziVDWoBAAAAAP7///8CGHkAAAAAAAAiACCe8g641Z7fBfHjCwk1kvywPRkr9cUGBuApW1w5DLX40gGSEwAAAAAAFgAUYUpBN6szinaiq2uKjyBwVku9sUcCRzBEAiAGn/m5aFp8jlT7opwbNfCWTlSUH4gTSgQDWXEul8+wgQIgIUVW/QCcsD9Ip4OM4ugKQFwzGaoKXJmukIEjibxT2AkBIQORx/xBx+neNfEY1BQGP2mM53uWsIQZbg2nFQWWAyCq54SYIQABASsYeQAAAAAAACIAIJ7yDrjVnt8F8eMLCTWS/LA9GSv1xQYG4ClbXDkMtfjSAQVHUiECNeXtaFoBQxW587h32UhB0gWVExXXQlEP66TynFqTRcAhAvP2jBCtxiVWJuvXSSTyQvGb/55C5gvG6ywZG+y0o7iNUq4iBgI15e1oWgFDFbnzuHfZSEHSBZUTFddCUQ/rpPKcWpNFwAzKoWFWAAAAAAIAAAAiBgLz9owQrcYlVibr10kk8kLxm/+eQuYLxussGRvstKO4jQywKQw3AAAAAAIAAAAAIgICRZWxspC7OZUCjQgeh0xBiu8BmetWHp/+BeXow+lbOfsMyqFhVgEAAAACAAAAIgIDiq8cqvwjuzxp7MOjWxQqosTkXI5cC6vWisncaZPX8z0MsCkMNwEAAAACAAAAAAA=
```

### Finalize Transaction
Finalize a transaction from the output descriptors and signed psbts, returns a trx id

Example request:
```
curl --location --request POST 'http://127.0.0.1:8000/finalize_trx' \
--header 'Content-Type: application/json' \
--data-raw '{
    "descriptors": {
        "descriptor": "wsh(sortedmulti(2,tpubD9zJG3Z4c9LLBCTeEcq64yFtVtfDHffWspDxKLY3apTbu4ocjFoD4vXz4XV2tfMAEQ8p9Km6CiEHBYqVhhG3qPPEcBZqPnwYuWx9RVmiVLz/0/*,tpubDA5kZcnunRMnATJYbo9ar5CR5zFCs5SsHmP69noNWEFwyhSPnCDmuwUND3qAvsqyBwUtm2BGurKz5nFvACpHkFzwvmupdsbznAFMNypghFB/0/*))#3xvsph9g",
        "change_descriptor": "wsh(sortedmulti(2,tpubD9zJG3Z4c9LLBCTeEcq64yFtVtfDHffWspDxKLY3apTbu4ocjFoD4vXz4XV2tfMAEQ8p9Km6CiEHBYqVhhG3qPPEcBZqPnwYuWx9RVmiVLz/1/*,tpubDA5kZcnunRMnATJYbo9ar5CR5zFCs5SsHmP69noNWEFwyhSPnCDmuwUND3qAvsqyBwUtm2BGurKz5nFvACpHkFzwvmupdsbznAFMNypghFB/1/*))#jxu9yn3m"
    },
    "psbts":[
        "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"
    ]
}'
```

Example response:
```
3ef5f9ceefc2405e55a7aac8d62fcef068b024f0d482f28c7ff1c8808bfafc3e
```
