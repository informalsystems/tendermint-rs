//! Macros to facilitate protobuf conversions

macro_rules! tendermint_pb_modules {
    {
        $($contents:item)*
    } => {
        mod v0_34 {
            use tendermint_proto::v0_34 as pb;
            #[allow(unused_imports)]
            use tendermint_proto::Protobuf;

            $($contents)*
        }
        mod v0_37 {
            use tendermint_proto::v0_37 as pb;
            #[allow(unused_imports)]
            use tendermint_proto::Protobuf;

            $($contents)*
        }
        mod v0_38 {
            use tendermint_proto::v0_38 as pb;
            #[allow(unused_imports)]
            use tendermint_proto::Protobuf;

            $($contents)*
        }
    };
}
