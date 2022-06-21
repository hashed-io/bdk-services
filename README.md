**BDK Services**

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

### List transactions
Gets a list of transactions for an output descriptor.

Example request:
```
curl --location --request POST 'http://127.0.0.1:8000/list_trxs' \
--header 'Content-Type: application/json' \
--data-raw '{
    "descriptor": "wsh(sortedmulti(2,[20f24288/48'\''/0'\''/0'\''/2'\'']tpubD9zJG3Z4c9LLBCTeEcq64yFtVtfDHffWspDxKLY3apTbu4ocjFoD4vXz4XV2tfMAEQ8p9Km6CiEHBYqVhhG3qPPEcBZqPnwYuWx9RVmiVLz/0/*,[e9a0cf4a/48'\''/0'\''/0'\''/2'\'']tpubDA5kZcnunRMnATJYbo9ar5CR5zFCs5SsHmP69noNWEFwyhSPnCDmuwUND3qAvsqyBwUtm2BGurKz5nFvACpHkFzwvmupdsbznAFMNypghFB/0/*))#aakuctju"
}'
```

Example response:
```
[
    {
        "trx_id": "d09a8a41b743b242d84ad2295636b737b25fba3b773dcb82a71412943a935609",
        "received": 0,
        "sent": 30000,
        "fee": 1150,
        "confirmation_time": {
            "height": 2195889,
            "timestamp": 1650137221
        },
        "inputs": [
            {
                "previous_output_trx": "f9131474a143a3bf94d1c6a9ca39e0bf1196c6daf33209aae5611fd620f145ae:1"
            }
        ],
        "outputs": [
            {
                "value": 10000,
                "script_pubkey": "52210292e43b2c8b656c21dd04da745966a7bc77c6df9d3e0c8bfed438eebf9ec554622103bb52dfc3463e7804aad1d0a7fb738d9e9f27eb6d5e130d2a6c322aa17eb9d09b52ae",
                "address": null
            },
            {
                "value": 18850,
                "script_pubkey": "0020f4850c7e037fa450047b881639563906d8356200d9a23f4c4e04f8b1535c77e8",
                "address": "tb1q7jzsclsr07j9qprm3qtrj43eqmvr2csqmx3r7nzwqnutz56uwl5qmz802u"
            }
        ]
    },
    {
        "trx_id": "cabb8f56ca039a002ff6506216cb310d21ff14cd6b3927f3d6973028ebc2e535",
        "received": 74000,
        "sent": 0,
        "fee": 153,
        "confirmation_time": {
            "height": 2195897,
            "timestamp": 1650143127
        },
        "inputs": [
            {
                "previous_output_trx": "47c887f92fbddda6a1b261d565c2bc1ff3c3a1de5c46dd4822da62010c9ef2fd:1"
            }
        ],
        "outputs": [
            {
                "value": 74000,
                "script_pubkey": "00200b20d3a55b4bcd24163bd2ef1e4d16481ac0d72a9bddbc0cde471730dcc0b4a6",
                "address": "tb1qpvsd8f2mf0xjg93m6th3ungkfqdvp4e2n0wmcrx7gutnphxqkjnq0fwscv"
            },
            {
                "value": 1550385247,
                "script_pubkey": "001457a8cd35be516abcc64a81e5dd4e46cca4aa42da",
                "address": "tb1q275v6dd7294te3j2s8ja6njxejj25sk68ms5kq"
            }
        ]
    },
    {
        "trx_id": "0a8a6d20c79d1c9335563a3ee3defbc00d9c6e01401a5378ba32a627a254b43b",
        "received": 0,
        "sent": 74000,
        "fee": 965,
        "confirmation_time": {
            "height": 2195901,
            "timestamp": 1650146735
        },
        "inputs": [
            {
                "previous_output_trx": "cabb8f56ca039a002ff6506216cb310d21ff14cd6b3927f3d6973028ebc2e535:0"
            }
        ],
        "outputs": [
            {
                "value": 63035,
                "script_pubkey": "002059aad8a3feb0f1915ccb95d1aea8b16905c9abe95b1558d67b5a76684d70ebac",
                "address": "tb1qtx4d3gl7krcezhxtjhg6a293dyzun2lftv2434nmtfmxsntsawkqpugv3s"
            },
            {
                "value": 10000,
                "script_pubkey": "00206b5594c7aa84413b403a9e7ec0a37b7e99e804fce10b946f5a738f70cd28d637",
                "address": "tb1qdd2ef3a2s3qnksp6nelvpgmm06v7sp8uuy9egm66ww8hpnfg6cmsrncyp5"
            }
        ]
    },
    {
        "trx_id": "64266c0f71b3ef6f538f6d62b664a83568cc0f079c2c3cb93680ef142cb7b8a3",
        "received": 31000,
        "sent": 0,
        "fee": 7661,
        "confirmation_time": {
            "height": 2201743,
            "timestamp": 1651095100
        },
        "inputs": [
            {
                "previous_output_trx": "6a0d9538bfcbc4ad1c687de3ae4b8a9785cdc5a6b80195c2742dd1828ab85b97:1"
            }
        ],
        "outputs": [
            {
                "value": 31000,
                "script_pubkey": "00209ef20eb8d59edf05f1e30b093592fcb03d192bf5c50606e0295b5c390cb5f8d2",
                "address": "tb1qnmeqawx4nm0stu0rpvyntyhukq73j2l4c5rqdcpftdwrjr94lrfq7eln6u"
            },
            {
                "value": 1282561,
                "script_pubkey": "0014614a4137ab338a76a2ab6b8a8f2070564bbdb147",
                "address": "tb1qv99yzdatxw98dg4tdw9g7grs2e9mmv284662j4"
            }
        ]
    },
    {
        "trx_id": "f9131474a143a3bf94d1c6a9ca39e0bf1196c6daf33209aae5611fd620f145ae",
        "received": 30000,
        "sent": 0,
        "fee": 153,
        "confirmation_time": {
            "height": 2195813,
            "timestamp": 1650067887
        },
        "inputs": [
            {
                "previous_output_trx": "dc2de9bc52ee208b99cec0a9d4d535593104eec145559ad5b5f3eaa0edd10be7:0"
            }
        ],
        "outputs": [
            {
                "value": 46360154,
                "script_pubkey": "0014dfbd013093e7a1822d21aba6f5d9cab7c2c22c8f",
                "address": "tb1qm77szvynu7scytfp4wn0tkw2klpvyty07z8tkh"
            },
            {
                "value": 30000,
                "script_pubkey": "002082cfd4490b59cbd3435e6905d937a5a3300c8be095b506e854010689f7b1100c",
                "address": "tb1qst8agjgtt89axs67dyzajda95vcqezlqjk6sd6z5qyrgnaa3zqxq3xmvnn"
            }
        ]
    }
]
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
Finalize a transaction from the output descriptors and signed psbts, returns a trx id. A boolean broadcast parameter indicates whether the transaction should be broadcasted

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
    ],
    "broadcast": false
}'
```

Example response:
```
3ef5f9ceefc2405e55a7aac8d62fcef068b024f0d482f28c7ff1c8808bfafc3e
```
