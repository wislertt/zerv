use crate::version::{PEP440, SemVer, Zerv};
use rstest::rstest;

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest]
    #[case("1.2.3", "1.2.3")]
    #[case("1.2.3a1", "1.2.3-alpha.1")]
    #[case("1.2.3b2", "1.2.3-beta.2")]
    #[case("1.2.3rc1", "1.2.3-rc.1")]
    #[case("1.2.3.post1", "1.2.3")] // Post-releases are dropped in SemVer conversion
    #[case("1.2.3.dev0", "1.2.3")] // Dev releases are dropped in SemVer conversion
    #[case("1.2.3+local", "1.2.3+local")]
    #[case("1.2.3a1.post1", "1.2.3-alpha.1")] // Post part is dropped, only pre-release kept
    #[case("1.2.3.post1.dev0", "1.2.3")]
    // Both post and dev are dropped
    #[case("1!1.2.3", "1.2.3-epoch.1")] // Epochs are preserved in pre-release section
    // Complex edge cases
    #[case("2!1.2.3a1", "1.2.3-epoch.2.alpha.1")] // Epoch + pre-release
    #[case("3!1.2.3b2.post1", "1.2.3-epoch.3.beta.2")] // Epoch + pre-release + post (post dropped)
    #[case("1!1.2.3.post5.dev10", "1.2.3-epoch.1")] // Epoch + post + dev (both dropped)
    #[case("5!2.0.0rc1+local.build", "2.0.0-epoch.5.rc.1+local.build")] // Epoch + pre-release + local
    #[case("0!1.2.3", "1.2.3")] // Zero epoch (default, not included)
    #[case("10!0.0.1a1.post2.dev3+build.456", "0.0.1-epoch.10.alpha.1+build.456")] // All components
    fn test_pep440_to_semver_via_zerv(#[case] pep440_str: &str, #[case] expected_semver: &str) {
        let pep440 = pep440_str.parse::<PEP440>().unwrap();
        let zerv: Zerv = pep440.into();
        let semver: SemVer = Into::<SemVer>::into(zerv);
        assert_eq!(semver.to_string(), expected_semver);
    }

    #[rstest]
    #[case("1.2.3", "1.2.3")]
    #[case("1.2.3-alpha.1", "1.2.3a1")]
    #[case("1.2.3-beta.2", "1.2.3b2")]
    #[case("1.2.3-rc.1", "1.2.3rc1")]
    #[case("1.2.3-rc.1.dev.1", "1.2.3rc1.dev.1")] // fix this case
    #[case("1.2.3-rc.1.post.2.dev.1", "1.2.3rc1.post2.dev1")] // fix this case
    #[case("1.2.3-rc.1.post.2.dev.1", "1.2.3rc1.post2.dev1")] // fix this case
    #[case("1.2.3-rc.1.dev.1.post.2", "1.2.3rc1.dev1+post.2")] // fix this case
    #[case("1.2.3+build.123", "1.2.3+build.123")]
    #[case("1.2.3-alpha.1+build.456", "1.2.3a1+build.456")]
    // Complex pre-release combinations and orders
    #[case("1.2.3-epoch.1", "1!1.2.3")] // Epoch recognized and converted
    #[case("1.2.3-epoch.5.alpha.2", "5!1.2.3a2")] // Epoch + alpha
    #[case("1.2.3-epoch.3.beta.1", "3!1.2.3b1")] // Epoch + beta
    #[case("1.2.3-epoch.2.rc.4", "2!1.2.3rc4")] // Epoch + rc
    #[case("1.2.3-alpha.1.beta.2", "1.2.3a1+beta.2")] // Multiple pre-release (first wins)
    #[case("1.2.3-beta.3.alpha.1", "1.2.3b3+alpha.1")] // Multiple pre-release (first wins)
    #[case("1.2.3-rc.1.alpha.2.beta.3", "1.2.3rc1+alpha.2.beta.3")] // Multiple pre-release (first wins)
    #[case("1.2.3-foo.1.alpha.2", "1.2.3a2+foo.1")] // Non-standard + standard (standard wins)
    #[case("1.2.3-alpha.bar.1", "1.2.3a+bar.1")] // Alpha with non-numeric
    #[case("1.2.3-1.2.3.alpha.1", "1.2.3a1+1.2.3")] // Numbers + alpha
    #[case("1.2.3-custom.text.beta.5", "1.2.3b5+custom.text")] // Custom text + beta
    #[case("1.2.3-epoch.0.alpha.1", "1.2.3a1")] // Zero epoch (dropped)
    #[case("1.2.3-10.epoch.2.rc.3", "2!1.2.3rc3+10")] // Numbers + epoch + rc
    #[case("1.2.3-alpha", "1.2.3a")] // Alpha without number
    #[case("1.2.3-beta", "1.2.3b")] // Beta without number
    #[case("1.2.3-rc", "1.2.3rc")] // RC without number
    #[case("1.2.3-10.epoch.epoch.rc.3", "1.2.3rc3+10.epoch.epoch")]
    #[case("1.2.3-10.a.rc.epoch.rc.3", "1.2.3a+10.rc.epoch.rc.3")]
    #[case("1.2.3-10.dev.1.post.2.epoch.3", "3!1.2.3+10.dev.1.post.2")]
    #[case("1.2.3-epoch.epoch.alpha.1", "1.2.3a1+epoch.epoch")]
    #[case("1.2.3-alpha.alpha.beta.1", "1.2.3a+alpha.beta.1")]
    #[case("1.2.3-dev.dev.rc.2", "1.2.3rc2+dev.dev")]
    #[case("1.2.3-post.post.alpha.3", "1.2.3a3+post.post")]
    #[case("1.2.3-1.2.3.epoch.epoch.rc.1", "1.2.3rc1+1.2.3.epoch.epoch")]
    #[case("1.2.3-foo.bar.epoch.5.alpha.2", "5!1.2.3a2+foo.bar")]
    #[case("1.2.3-epoch.1.epoch.2.beta.3", "1!1.2.3b3+epoch.2")]
    fn test_semver_to_pep440_via_zerv(#[case] semver_str: &str, #[case] expected_pep440: &str) {
        let semver = semver_str.parse::<SemVer>().unwrap();
        let zerv: Zerv = semver.into();
        let pep440: PEP440 = Into::<PEP440>::into(zerv);
        assert_eq!(pep440.to_string(), expected_pep440);
    }
}
