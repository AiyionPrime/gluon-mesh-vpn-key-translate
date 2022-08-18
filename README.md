# gluon-mesh-vpn-key-translate

Translates fastd to WireGuard keys.

## usage

### public key translation

```bash
gluon-mesh-vpn-key-translate 12760ee9088d7b737a11c21c587ab73b64be3c2834eaef6185ec66c3c05c1d78
```

translates the provided fastd public key to it's WireGuard pendant:

```console
Gluon+LYDa2ib6EDnWlRELwrst1s1Ut7WKNR5LMI83c=
```

Reading it from `stdin` does work as well and provides the same output.

```bash
echo 12760ee9088d7b737a11c21c587ab73b64be3c2834eaef6185ec66c3c05c1d78 | gluon-mesh-vpn-key-translate
```

As `gluon-mesh-vpn-key-translate` is primarily expected to run on Freifunk servers,
that translate public keys of routers it is therefore defaulting to public key translation.

### private key translation

In order to translate private keys, provide the `--private` flag or its alias `--secret`.

Careful: calling `gluon-mesh-vpn-key-translate` with a private key as argument is not supported in order to keep privates out of your shell history. Use `stdin` instead[^1]:

```bash
cat fastd_secret | gluon-mesh-vpn-key-translate --private
```

```console
aMLMgPlQVLbt1wuzVRQq3FTZAsaX/hztyJjfcbeeT3Y=
```

### `--if` and `--of`

Instead of using `stdin` and `stdout`, input- and and output-files can be specified using the options `--if` and `--of` that each take a path of a file as argument. To translate a public key[^2]:

```bash
gluon-mesh-vpn-key-translate --if fastd_public
```

```console
Gluon+LYDa2ib6EDnWlRELwrst1s1Ut7WKNR5LMI83c=
```

Note: It is recommended for private keys to be only read- and writable by the owner.
It is recommended to set appropriate `umask` beforehand, whenever secret keys are stored.

```bash
mkdir -p generated_keys
(umask 0077; gluon-mesh-vpn-key-translate --private --if fastd_secret --of generated_keys/wg_secret.key)
ls -l generated_keys/
```

```console
total 4
-rw------- 1 user user 45 Jan 01 00:00 wg_secret.key
```

### Furthermore

`--version` emits the current version[^3]

`--help` provides a short usage info.

[^1]: `fastd_secret` is a file that contains the exemplary private key `68c2cc80f95054b6edd70bb355142adc54d902c697fe1cedc898df71b79e4f76`.

[^2]: `fastd_public` is a file that either contains the raw hex string or the fastd-config-form instead: `key "12760ee9088d7b737a11c21c587ab73b64be3c2834eaef6185ec66c3c05c1d78";`.

[^3]: This project follows [SemVer](https://semver.org/) and won't introduce backwards incompatible changes without incrementation of the major version.
