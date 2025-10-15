use rstest::rstest;

use crate::version::{
    PEP440,
    SemVer,
    Zerv,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest]
    // Basic conversions
    #[case("1.2.3", "1.2.3")]
    #[case("1.2.3a1", "1.2.3-alpha.1")]
    #[case("1.2.3b2", "1.2.3-beta.2")]
    #[case("1.2.3rc1", "1.2.3-rc.1")]
    #[case("1.2.3.post1", "1.2.3-post.1")]
    #[case("1.2.3.dev0", "1.2.3-dev.0")]
    #[case("1.2.3+local", "1.2.3+local")]
    // Combined components
    #[case("1.2.3a1.post1", "1.2.3-alpha.1.post.1")]
    #[case("1.2.3.post1.dev0", "1.2.3-post.1.dev.0")]
    // Epoch handling
    #[case("1!1.2.3", "1.2.3-epoch.1")]
    #[case("2!1.2.3a1", "1.2.3-epoch.2.alpha.1")]
    #[case("3!1.2.3b2.post1", "1.2.3-epoch.3.beta.2.post.1")]
    #[case("1!1.2.3.post5.dev10", "1.2.3-epoch.1.post.5.dev.10")]
    #[case("5!2.0.0rc1+local.build", "2.0.0-epoch.5.rc.1+local.build")]
    #[case("0!1.2.3", "1.2.3")]
    #[case(
        "10!0.0.1a1.post2.dev3+build.456",
        "0.0.1-epoch.10.alpha.1.post.2.dev.3+build.456"
    )]
    // Additional combinations
    #[case("1.2.3rc1.dev1", "1.2.3-rc.1.dev.1")]
    #[case("1.2.3rc1.post2.dev1", "1.2.3-rc.1.post.2.dev.1")]
    // With build metadata
    #[case("1.2.3a1+build.456", "1.2.3-alpha.1+build.456")]
    #[case("1.2.3b2+build.456", "1.2.3-beta.2+build.456")]
    #[case("1.2.3rc3+build.456", "1.2.3-rc.3+build.456")]
    #[case("1.2.3a+build.456", "1.2.3-alpha.0+build.456")]
    #[case("1.2.3b+build.456", "1.2.3-beta.0+build.456")]
    #[case("1.2.3rc+build.456", "1.2.3-rc.0+build.456")]
    #[case("1.2.3.post1+build.456", "1.2.3-post.1+build.456")]
    #[case("1.2.3.dev1+build.456", "1.2.3-dev.1+build.456")]
    #[case("1.2.3a1.post1+build.456", "1.2.3-alpha.1.post.1+build.456")]
    #[case("1.2.3a1.dev1+build.456", "1.2.3-alpha.1.dev.1+build.456")]
    #[case("1.2.3a1.post1.dev1+build.456", "1.2.3-alpha.1.post.1.dev.1+build.456")]
    fn test_pep440_to_semver_via_zerv(#[case] pep440_str: &str, #[case] expected_semver: &str) {
        let pep440 = pep440_str.parse::<PEP440>().unwrap();
        let zerv: Zerv = pep440.into();
        let semver: SemVer = Into::<SemVer>::into(zerv);
        assert_eq!(semver.to_string(), expected_semver);
    }

    #[rstest]
    // Basic conversions
    #[case("1.2.3", "1.2.3")]
    #[case("1.2.3-alpha.1", "1.2.3a1")]
    #[case("1.2.3-beta.2", "1.2.3b2")]
    #[case("1.2.3-rc.1", "1.2.3rc1")]
    #[case("1.2.3-rc.1.dev.1", "1.2.3rc1.dev1")]
    #[case("1.2.3-rc.1.post.2.dev.1", "1.2.3rc1.post2.dev1")]
    #[case("1.2.3-rc.1.dev.1.post.2", "1.2.3rc1.post2.dev1")]
    #[case("1.2.3+build.123", "1.2.3+build.123")]
    #[case("1.2.3-alpha.1+build.456", "1.2.3a1+build.456")]
    // Epoch handling
    #[case("1.2.3-epoch.1", "1!1.2.3")]
    #[case("1.2.3-epoch.5.alpha.2", "5!1.2.3a2")]
    #[case("1.2.3-epoch.3.beta.1", "3!1.2.3b1")]
    #[case("1.2.3-epoch.2.rc.4", "2!1.2.3rc4")]
    // Multiple pre-release handling
    #[case("1.2.3-alpha.1.beta.2", "1.2.3a1+beta.2")]
    #[case("1.2.3-beta.3.alpha.1", "1.2.3b3+alpha.1")]
    #[case("1.2.3-rc.1.alpha.2.beta.3", "1.2.3rc1+alpha.2.beta.3")]
    #[case("1.2.3-foo.1.alpha.2", "1.2.3a2+foo.1")]
    #[case("1.2.3-alpha.bar.1", "1.2.3a0+bar.1")]
    #[case("1.2.3-1.2.3.alpha.1", "1.2.3a1+1.2.3")]
    #[case("1.2.3-custom.text.beta.5", "1.2.3b5+custom.text")]
    #[case("1.2.3-epoch.0.alpha.1", "1.2.3a1")]
    #[case("1.2.3-10.epoch.2.rc.3", "2!1.2.3rc3+10")]
    #[case("1.2.3-alpha", "1.2.3a0")]
    #[case("1.2.3-beta", "1.2.3b0")]
    #[case("1.2.3-rc", "1.2.3rc0")]
    // Complex edge cases
    #[case("1.2.3-10.epoch.epoch.rc.3", "1.2.3rc3+10.epoch")]
    #[case("1.2.3-10.a.rc.epoch.rc.3", "1.2.3a0+10.rc.rc.3")]
    #[case("1.2.3-10.dev.1.post.2.epoch.3", "3!1.2.3.post2.dev1+10")]
    #[case("1.2.3-epoch.epoch.alpha.1", "1.2.3a1+epoch")] //29
    #[case("1.2.3-alpha.alpha.beta.1", "1.2.3a0+alpha.beta.1")]
    #[case("1.2.3-dev.dev.rc.2", "1.2.3rc2+dev")]
    #[case("1.2.3-post.post.alpha.3", "1.2.3a3+post")]
    #[case("1.2.3-1.2.3.epoch.epoch.rc.1", "1.2.3rc1+1.2.3.epoch")]
    #[case("1.2.3-foo.bar.epoch.5.alpha.2", "5!1.2.3a2+foo.bar")]
    #[case("1.2.3-epoch.1.epoch.2.beta.3", "1!1.2.3b3+epoch.2")]
    fn test_semver_to_pep440_via_zerv(#[case] semver_str: &str, #[case] expected_pep440: &str) {
        let semver = semver_str.parse::<SemVer>().unwrap();
        let zerv: Zerv = semver.into();
        let pep440: PEP440 = Into::<PEP440>::into(zerv);
        assert_eq!(pep440.to_string(), expected_pep440);
    }
}
