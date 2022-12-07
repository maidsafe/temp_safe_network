# sn_client

| [![](https://img.shields.io/crates/v/sn_client)](https://crates.io/crates/sn_client) | [![Documentation](https://docs.rs/sn_client/badge.svg)](https://docs.rs/sn_client) |
|:----------:|:----------:|

## Compatability

`sn_node`: 0.58.X

## Overview

`sn_client` interface against the sn_node crate.

## Features

### "default": This is our standard setup
(Helps with detecting bugs)
- Msg _all elders_ for cmds + elders wait on _all acks_ from adults
- Msg _three elders_ for queries _to one adult_ + wait on _all responses_ from elders

### "check-replicas": this is used for all CI runs right now.
(Verifies data was stored)
- Msg _three elders_ for queries _to all adults_ + wait on _all responses_

### "msg-happy-path" [client side]: this is used for sn_client e2e CI runs right now.
- Msg _one elder_ for cmds/queries

### "msg-happy-path" [node side]: this is to be eventually added to CI runs.
- Elder waits on _one ack/response_ from adults

----

## Testing

- `"check-replicas"` which is as `default` but makes sure each adult is storing data.
- `"msg-happy-path"` the least reliable setup, but also least resource demanding.
- `"msg-happy-path"` churn testing, i.e. are we slowly losing more data (todo).

Eventually (todo), the `"check-replicas"` feat will only run under the `"msg-happy-path"` (makes sense to check replicas after using the happy path, and if check replicas works fine with happy path, shouldn't need testing without it).


## Crate Dependencies
Crate dependencies graph:

![sn_client Safe Network dependencies](https://github.com/maidsafe/sn_client/blob/png_generator/sn_client-sn-dependencies.png)


### Legend
Dependencies are coloured depending on their kind:
* **Black:** regular dependency
* **Purple:** build dependency
* **Blue:** dev dependency
* **Red:** optional dependency

A dependency can be of more than one kind. In such cases, it is coloured with the following priority:
`Regular -> Build -> Dev -> Optional`

<details>
<summary> View all sn_client dependencies</summary>
<p>

![sn_client all dependencies](https://github.com/maidsafe/sn_client/blob/png_generator/sn_client-all-dependencies.png)

</p>
</details>

Click [here](https://maidsafe.github.io/interdependency-svg-generator/) for an overview of the interdependencies of all the main MaidSafe components.

## License

This Safe Network library is licensed under the General Public License (GPL), version 3 ([LICENSE](LICENSE) http://www.gnu.org/licenses/gpl-3.0.en.html).

### Linking exception

sn_client is licensed under GPLv3 with linking exception. This means you can link to and use the library from any program, proprietary or open source; paid or gratis. However, if you modify sn_client, you must distribute the source to your modified version under the terms of the GPLv3.

See the [LICENSE](LICENSE) file for more details.
