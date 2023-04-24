mod v0_34 {
    use crate::v0_34::types::{evidence, Evidence};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl<'de> Deserialize<'de> for Evidence {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let sum = Option::<evidence::Sum>::deserialize(deserializer)?;
            Ok(Self { sum })
        }
    }

    impl Serialize for Evidence {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.sum.serialize(serializer)
        }
    }
}

mod v0_37 {
    use crate::v0_37::types::{evidence, Evidence};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl<'de> Deserialize<'de> for Evidence {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let sum = Option::<evidence::Sum>::deserialize(deserializer)?;
            Ok(Self { sum })
        }
    }

    impl Serialize for Evidence {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.sum.serialize(serializer)
        }
    }
}

mod v0_38 {
    use crate::v0_38::types::{evidence, Evidence};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl<'de> Deserialize<'de> for Evidence {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let sum = Option::<evidence::Sum>::deserialize(deserializer)?;
            Ok(Self { sum })
        }
    }

    impl Serialize for Evidence {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.sum.serialize(serializer)
        }
    }
}
