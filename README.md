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

### List signers
Gets a list of xpubs who signed a psbt, the psbt and wallet descriptors should be provided

Example request:
```
curl --location --request POST 'http://127.0.0.1:8000/list_signers' \
--header 'Content-Type: application/json' \
--data-raw '{
    "descriptors": {
        "descriptor": "wsh(sortedmulti(3,[c0b82c68/48'\''/1'\''/0'\''/2'\'']tpubDDvtX53NKpPSfmtDXPraDXztq9UJSPoTqUq6LJ9emR4XyfXkkBoGM3LPvNh74qkysLNeZzWsNw6JXpRMjWn8492o7nsVw3cryZExM9Ax13G/0/*,[5e6b6a06/48'\''/1'\''/0'\''/2'\'']tpubDEcAPU7V865L7KtcWu3hUFuzqGs5AH7mwCPWuHqXEHxqRbQiXuhMZWsHTVZYzAaTPdYK44jtzBycfXwGHdZzqC7LmNoLyT74nDRDPK9vwLr/0/*,[4f82bcb7/48'\''/1'\''/0'\''/2'\'']tpubDE7g2MjNUnQX2gV3VEXixreNBp6WNEKkqnJ8nBpFGSbKdWLpa7ACdanEDsrpbZLSpqR93XApEJvK3MtijrMZQyGEggQd3Dbs1fq6wcqeJEB/0/*,[3b83b09c/48'\''/1'\''/0'\''/2'\'']tpubDFWbJ3wcyot3h5wdHquyiJRAArmhZytk7LMWdLP6VPEd7vBNiSuHkHwcY6fmCxDUYBgjtvSizr7hvTfWDj9Nq78dUc6WRrVGyn4Jf9bJYaN/0/*))#78jys7qq",
    "change_descriptor": "wsh(sortedmulti(3,[c0b82c68/48'\''/1'\''/0'\''/2'\'']tpubDDvtX53NKpPSfmtDXPraDXztq9UJSPoTqUq6LJ9emR4XyfXkkBoGM3LPvNh74qkysLNeZzWsNw6JXpRMjWn8492o7nsVw3cryZExM9Ax13G/1/*,[5e6b6a06/48'\''/1'\''/0'\''/2'\'']tpubDEcAPU7V865L7KtcWu3hUFuzqGs5AH7mwCPWuHqXEHxqRbQiXuhMZWsHTVZYzAaTPdYK44jtzBycfXwGHdZzqC7LmNoLyT74nDRDPK9vwLr/1/*,[4f82bcb7/48'\''/1'\''/0'\''/2'\'']tpubDE7g2MjNUnQX2gV3VEXixreNBp6WNEKkqnJ8nBpFGSbKdWLpa7ACdanEDsrpbZLSpqR93XApEJvK3MtijrMZQyGEggQd3Dbs1fq6wcqeJEB/1/*,[3b83b09c/48'\''/1'\''/0'\''/2'\'']tpubDFWbJ3wcyot3h5wdHquyiJRAArmhZytk7LMWdLP6VPEd7vBNiSuHkHwcY6fmCxDUYBgjtvSizr7hvTfWDj9Nq78dUc6WRrVGyn4Jf9bJYaN/1/*))#sda5dl7e"
    },
    "psbt":
        "cHNidP8BAIkBAAAAAfx15Ttmz6elm9LHqX2jVvqboFTMUrD3OVilRE0RH3HNAQAAAAD9////AhAnAAAAAAAAIgAgapL4iNK+iOvUjmi74v5KOdJq0+brS2MsQt8bZu/jvVy37QAAAAAAACIAIDGu4FBMXgV+irxy6Vz78NrpoH/ezv1eabyuP2wfZkIWAAAAAAABAH0CAAAAAT+oqsgQzz8UeaO8LJJbqVkfwWeYbirVFcC2brDjkKl6AQAAAAD+////AqsLKDAAAAAAFgAUhEkMOtrFy4DnVXpDGJ/qgMsGz5NAGQEAAAAAACIAIJr0KH+bUEFUEaEMV+hkqL0I7NJwuXLireNKjH/A6A7lVNgiAAEBK0AZAQAAAAAAIgAgmvQof5tQQVQRoQxX6GSovQjs0nC5cuKt40qMf8DoDuUiAgNHCPu3PMi2RSVEJl3Hn6FdhYoWxDTEdvVvg0I2V+/JK0cwRAIgQevv55jDdhvw6pJAhjXknVP4JzISWX6RxjqjG3ACYGUCIDGoyjaEXMPEgCDnFti6dhClIHcHXzZpR20XPwDgOzXFAQEFi1MhArnezPqp/4uC8h2tGFRVE3r3OhwQphISKoDOVVAAyWLmIQNHCPu3PMi2RSVEJl3Hn6FdhYoWxDTEdvVvg0I2V+/JKyEDZu4oYzq12z164t18nKDxSXquyKRTdzIMfaa5Hc8E8eghA/WJNsbQW4ZpqQc4wgJXx76N0J9BXtmtn8KS6CGnnVZPVK4iBgK53sz6qf+LgvIdrRhUVRN69zocEKYSEiqAzlVQAMli5hw7g7CcMAAAgAEAAIAAAACAAgAAgAAAAAAAAAAAIgYDRwj7tzzItkUlRCZdx5+hXYWKFsQ0xHb1b4NCNlfvySscwLgsaDAAAIABAACAAAAAgAIAAIAAAAAAAAAAACIGA2buKGM6tds9euLdfJyg8Ul6rsikU3cyDH2muR3PBPHoHF5ragYwAACAAQAAgAAAAIACAACAAAAAAAAAAAAiBgP1iTbG0FuGaakHOMICV8e+jdCfQV7ZrZ/Ckughp51WTxxPgry3MAAAgAEAAIAAAACAAgAAgAAAAAAAAAAAACICAlw+3OyS5P2wtDCQ+c6YDC9ri+MDDBBVzkUOLAUHVVZvHE+CvLcwAACAAQAAgAAAAIACAACAAAAAAAEAAAAiAgMYBouAT9fLseZTj+djEPT0T314LQoz3HAuRyA10SMtqhxea2oGMAAAgAEAAIAAAACAAgAAgAAAAAABAAAAIgIDeNuinQCUmzQDgtpHNeF9SAC2ISHAnHX8Tpe/WnGU/LscO4OwnDAAAIABAACAAAAAgAIAAIAAAAAAAQAAACICA8fn4BH+XPTkMQAGbQV/EQ7KauiBrGnjH0xycD8eXAtQHMC4LGgwAACAAQAAgAAAAIACAACAAAAAAAEAAAAAIgICUvXn7M3omQ21dQszr+Oaz0UdY4dS96R081fW5cFlMi8cO4OwnDAAAIABAACAAAAAgAIAAIABAAAAAAAAACICApQ5AtQa4B7rstVnAh5DkB+ChCmGbqPbMBTw19hvnrodHE+CvLcwAACAAQAAgAAAAIACAACAAQAAAAAAAAAiAgKw44wCccnmj+Tw8KO/Vxuz/TDdSfZXIJgzMXArlTLo6hxea2oGMAAAgAEAAIAAAACAAgAAgAEAAAAAAAAAIgIDayJ1LN9LpJuBEQVR47CUHxm2G9XZ1cudLZP8pv4clEAcwLgsaDAAAIABAACAAAAAgAIAAIABAAAAAAAAAAA="
}'
```

Example response:
```
[
    {
        "xfp": "c0b82c68",
        "xpub": "Vpub5knpWjcHt8uQ7xUWM9mDRWpKst81n7zzmtr2LDaH3GPHkMoVw41L3bDDSded6xioVcg7L3ozoiwfCEKPCVFoiiKy9yqkV6nejso8Puy7Mvf",
        "derivation_path": "m/48'/1'/0'/2'"
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


### Create Proof of Reserves
Generates a non spendable PSBT that serves as proof of reserves for the specified vault

Example request:
```
curl --location --request POST 'http://127.0.0.1:8000/create_proof' \
--header 'Content-Type: application/json' \
--data-raw '{
    "descriptors": {
        "descriptor": "wsh(sortedmulti(2,tpubD9zJG3Z4c9LLBCTeEcq64yFtVtfDHffWspDxKLY3apTbu4ocjFoD4vXz4XV2tfMAEQ8p9Km6CiEHBYqVhhG3qPPEcBZqPnwYuWx9RVmiVLz/0/*,tpubDA5kZcnunRMnATJYbo9ar5CR5zFCs5SsHmP69noNWEFwyhSPnCDmuwUND3qAvsqyBwUtm2BGurKz5nFvACpHkFzwvmupdsbznAFMNypghFB/0/*))#3xvsph9g",
        "change_descriptor": "wsh(sortedmulti(2,tpubD9zJG3Z4c9LLBCTeEcq64yFtVtfDHffWspDxKLY3apTbu4ocjFoD4vXz4XV2tfMAEQ8p9Km6CiEHBYqVhhG3qPPEcBZqPnwYuWx9RVmiVLz/1/*,tpubDA5kZcnunRMnATJYbo9ar5CR5zFCs5SsHmP69noNWEFwyhSPnCDmuwUND3qAvsqyBwUtm2BGurKz5nFvACpHkFzwvmupdsbznAFMNypghFB/1/*))#jxu9yn3m"
    },
    "message": "Generating proof"
}'
```

Example response:
```
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
```


### Finalize Proof of Reserves
Takes in the signed proof of reserves by the vault owners and finalizes the proof which can then be verified

Example request:
```
curl --location --request POST 'http://127.0.0.1:8000/finalize_proof' \
--header 'Content-Type: application/json' \
--data-raw '{
    "descriptors": {
        "descriptor": "wsh(sortedmulti(2,tpubD9zJG3Z4c9LLBCTeEcq64yFtVtfDHffWspDxKLY3apTbu4ocjFoD4vXz4XV2tfMAEQ8p9Km6CiEHBYqVhhG3qPPEcBZqPnwYuWx9RVmiVLz/0/*,tpubDA5kZcnunRMnATJYbo9ar5CR5zFCs5SsHmP69noNWEFwyhSPnCDmuwUND3qAvsqyBwUtm2BGurKz5nFvACpHkFzwvmupdsbznAFMNypghFB/0/*))#3xvsph9g",
        "change_descriptor": "wsh(sortedmulti(2,tpubD9zJG3Z4c9LLBCTeEcq64yFtVtfDHffWspDxKLY3apTbu4ocjFoD4vXz4XV2tfMAEQ8p9Km6CiEHBYqVhhG3qPPEcBZqPnwYuWx9RVmiVLz/1/*,tpubDA5kZcnunRMnATJYbo9ar5CR5zFCs5SsHmP69noNWEFwyhSPnCDmuwUND3qAvsqyBwUtm2BGurKz5nFvACpHkFzwvmupdsbznAFMNypghFB/1/*))#jxu9yn3m"
    },
    "psbts": ["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"]
}'
```

Example response:
```
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
```


### Verify Proof of Reserves
Takes in the finalized proof of reserves validates it and returns the amount contained in the vault

Example request:
```
curl --location --request POST 'http://127.0.0.1:8000/verify_proof' \
--header 'Content-Type: application/json' \
--data-raw '{
    "descriptors": {
        "descriptor": "wsh(sortedmulti(2,tpubD9zJG3Z4c9LLBCTeEcq64yFtVtfDHffWspDxKLY3apTbu4ocjFoD4vXz4XV2tfMAEQ8p9Km6CiEHBYqVhhG3qPPEcBZqPnwYuWx9RVmiVLz/0/*,tpubDA5kZcnunRMnATJYbo9ar5CR5zFCs5SsHmP69noNWEFwyhSPnCDmuwUND3qAvsqyBwUtm2BGurKz5nFvACpHkFzwvmupdsbznAFMNypghFB/0/*))#3xvsph9g",
        "change_descriptor": "wsh(sortedmulti(2,tpubD9zJG3Z4c9LLBCTeEcq64yFtVtfDHffWspDxKLY3apTbu4ocjFoD4vXz4XV2tfMAEQ8p9Km6CiEHBYqVhhG3qPPEcBZqPnwYuWx9RVmiVLz/1/*,tpubDA5kZcnunRMnATJYbo9ar5CR5zFCs5SsHmP69noNWEFwyhSPnCDmuwUND3qAvsqyBwUtm2BGurKz5nFvACpHkFzwvmupdsbznAFMNypghFB/1/*))#jxu9yn3m"
    },
    "psbt":"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",
    "message": "Generating proof"
}'
```

Example response:
```
200965
```

