use super::PEP440;
use super::utils::LocalSegment;
use crate::error::ZervError;
use crate::version::zerv::{
    Component,
    PreReleaseVar,
    Zerv,
    ZervSchema,
    ZervVars,
};

impl From<PEP440> for Zerv {
    fn from(pep440: PEP440) -> Self {
        let schema = ZervSchema::pep440_default().expect("PEP440 default schema should be valid");
        pep440
            .to_zerv_with_schema(&schema)
            .expect("PEP440 default conversion should work")
    }
}

impl PEP440 {
    pub fn to_zerv_with_schema(&self, schema: &ZervSchema) -> Result<Zerv, ZervError> {
        // Only support default PEP440 schema for now
        if *schema != ZervSchema::pep440_default()? {
            return Err(ZervError::NotImplemented(
                "Custom schemas not yet implemented for PEP440 conversion".to_string(),
            ));
        }

        let vars = ZervVars {
            major: self.release.first().copied().map(|n| n as u64),
            minor: self.release.get(1).copied().map(|n| n as u64),
            patch: self.release.get(2).copied().map(|n| n as u64),
            epoch: (self.epoch > 0).then_some(self.epoch as u64),
            post: self.post_number.map(|n| n as u64),
            dev: self.dev_number.map(|n| n as u64),
            pre_release: self.pre_label.map(|label| PreReleaseVar {
                label,
                number: self.pre_number.map(|n| n as u64),
            }),
            ..Default::default()
        };

        // Handle excess release parts beyond major.minor.patch
        let mut schema = schema.clone();
        for &part in self.release.iter().skip(3) {
            schema.push_core(Component::Int(part as u64))?;
        }

        // Handle local segments - add to build
        if let Some(local_segments) = &self.local {
            for segment in local_segments {
                match segment {
                    LocalSegment::Str(s) => {
                        schema.push_build(Component::Str(s.clone()))?;
                    }
                    LocalSegment::UInt(n) => {
                        schema.push_build(Component::Int(*n as u64))?;
                    }
                }
            }
        }

        Ok(Zerv { vars, schema })
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_utils::zerv::zerv_pep440::to;

    #[rstest]
    // Basic conversions
    #[case("1.2.3", to::v1_2_3().build())]
    #[case("2!1.2.3", to::v1_2_3_e2().build())]
    #[case("1.2.3a1", to::v1_2_3_a1().build())]
    #[case("1.2.3.post1", to::v1_2_3_post1().build())]
    #[case("1.2.3.dev1", to::v1_2_3_dev1().build())]
    #[case("1.2.3+ubuntu.20.4", to::v1_2_3_ubuntu_build().build())]
    #[case("2!1.2.3a1.post1.dev1+local.1", to::v1_2_3_e2_a1_post1_dev1_local().build())]
    // Epoch handling
    #[case("1!1.0.0", to::v1_0_0_e1().build())]
    #[case("5!1.0.0", to::v1_0_0_e5().build())]
    #[case("999!1.0.0", to::v1_0_0_e999().build())]
    // Post handling
    #[case("1.0.0.post5", to::v1_0_0_post5().build())]
    #[case("1.0.0.post0", to::v1_0_0_post0().build())]
    // Dev handling
    #[case("1.0.0.dev0", to::v1_0_0_dev0().build())]
    #[case("1.0.0.dev10", to::v1_0_0_dev10().build())]
    // Epoch + pre-release combinations
    #[case("2!1.0.0a1", to::v1_0_0_e2_a1().build())]
    #[case("3!1.0.0b2", to::v1_0_0_e3_b2().build())]
    #[case("1!1.0.0rc5", to::v1_0_0_e1_rc5().build())]
    #[case("4!1.0.0a0", to::v1_0_0_e4_a0().build())]
    // Post + dev combinations
    #[case("1.0.0.post1.dev2", to::v1_0_0_post1_dev2().build())]
    // Pre-release + post combinations
    #[case("1.0.0a1.post2", to::v1_0_0_a1_post2().build())]
    #[case("1.0.0b3.post1", to::v1_0_0_b3_post1().build())]
    #[case("1.0.0rc2.post5", to::v1_0_0_rc2_post5().build())]
    // Pre-release + dev combinations
    #[case("1.0.0a1.dev2", to::v1_0_0_a1_dev2().build())]
    #[case("1.0.0b2.dev1", to::v1_0_0_b2_dev1().build())]
    #[case("1.0.0rc1.dev3", to::v1_0_0_rc1_dev3().build())]
    // Triple combinations
    #[case("1.0.0a1.post2.dev3", to::v1_0_0_a1_post2_dev3().build())]
    #[case("1.0.0b2.post3.dev1", to::v1_0_0_b2_post3_dev1().build())]
    #[case("1.0.0rc1.post1.dev1", to::v1_0_0_rc1_post1_dev1().build())]
    // Epoch + post + dev combinations
    #[case("2!1.0.0.post1.dev3", to::v1_0_0_e2_post1_dev3().build())]
    #[case("1!1.0.0.post1.dev2", to::v1_0_0_e1_post1_dev2().build())]
    // All components together
    #[case("3!1.0.0a1.post2.dev1", to::v1_0_0_e3_a1_post2_dev1().build())]
    #[case("1!1.0.0b2.post1.dev3", to::v1_0_0_e1_b2_post1_dev3().build())]
    // With build metadata
    #[case("1!1.0.0+build.123", to::v1_0_0_e1_build().build())]
    #[case("1.0.0.post1+build.456", to::v1_0_0_post1_build().build())]
    #[case("1.0.0.dev2+build.789", to::v1_0_0_dev2_build().build())]
    #[case("2!1.0.0a1+build.abc", to::v1_0_0_e2_a1_build().build())]
    // Complex local version identifiers
    #[case("1.0.0+foo.bar.123", to::v1_0_0_complex_build().build())]
    #[case(
        "1!1.0.0a1.post1.dev1+complex.local.456",
        to::v1_0_0_e1_a1_post1_dev1_complex().build()
    )]
    fn test_pep440_to_zerv_conversion(#[case] pep440_str: &str, #[case] expected: Zerv) {
        let pep440: PEP440 = pep440_str.parse().unwrap();
        let zerv: Zerv = pep440.into();
        assert_eq!(zerv, expected);
    }

    #[rstest]
    #[case("1.0.0")]
    #[case("2!1.2.3")]
    #[case("1.0.0a1")]
    #[case("1.0.0.post1")]
    #[case("1.0.0.dev1")]
    #[case("1.0.0+local.1")]
    #[case("2!1.2.3a1.post1.dev1+local.1")]
    fn test_round_trip_conversion(#[case] version_str: &str) {
        let original: PEP440 = version_str.parse().unwrap();
        let zerv: Zerv = original.clone().into();
        let converted: PEP440 = zerv.into();

        assert_eq!(original.to_string(), converted.to_string());
    }
}
