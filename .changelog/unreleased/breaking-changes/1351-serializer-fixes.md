- Changed the serde schema produced by `serialize` functions in these
  helper modules ([\#1351](https://github.com/informalsystems/tendermint-
  rs/pull/1351)):

  * In `tendermint-proto`:
    - `serializers::nullable`
    - `serializers::optional`
  * In `tendermint`:
    - `serializers::apphash`
    - `serializers::hash`
    - `serializers::option_hash`

  If `serde_json` is used for serialization, the output schema does not change.
  But since serde is a generic framework, the changes may be breaking for
  other users. Overall, these changes should make the serialized data
  acceptable by the corresponding deserializer agnostically of the format.
