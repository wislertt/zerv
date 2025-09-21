#[cfg(test)]
mod tests {
    use crate::schema::create_zerv_version;
    use crate::version::zerv::{Component, ZervVars};

    #[test]
    fn test_zerv_schema_structure() {
        // Create a simple ZervVars for tier 1 (tagged, clean)
        let vars = ZervVars {
            major: Some(1),
            minor: Some(2),
            patch: Some(3),
            dirty: Some(false),
            distance: Some(0),
            ..Default::default()
        };

        let zerv = create_zerv_version(vars, Some("zerv-standard"), None).unwrap();

        // Test the actual schema structure
        println!("Core components: {:?}", zerv.schema.core);
        println!("Extra core components: {:?}", zerv.schema.extra_core);
        println!("Build components: {:?}", zerv.schema.build);

        // Verify core structure
        assert_eq!(zerv.schema.core.len(), 3);
        assert_eq!(
            zerv.schema.core[0],
            Component::VarField("major".to_string())
        );
        assert_eq!(
            zerv.schema.core[1],
            Component::VarField("minor".to_string())
        );
        assert_eq!(
            zerv.schema.core[2],
            Component::VarField("patch".to_string())
        );
    }

    #[test]
    fn test_zerv_ron_roundtrip_schema() {
        let vars = ZervVars {
            major: Some(1),
            minor: Some(2),
            patch: Some(3),
            dirty: Some(false),
            distance: Some(0),
            ..Default::default()
        };

        let original = create_zerv_version(vars, Some("zerv-standard"), None).unwrap();
        let ron_string = original.to_string();
        let parsed: crate::version::zerv::Zerv = ron_string.parse().unwrap();

        // Verify schema is preserved
        assert_eq!(parsed.schema.core.len(), 3);
        assert_eq!(
            parsed.schema.core[0],
            Component::VarField("major".to_string())
        );
        assert_eq!(
            parsed.schema.core[1],
            Component::VarField("minor".to_string())
        );
        assert_eq!(
            parsed.schema.core[2],
            Component::VarField("patch".to_string())
        );

        // Verify vars are preserved
        assert_eq!(parsed.vars.major, Some(1));
        assert_eq!(parsed.vars.minor, Some(2));
        assert_eq!(parsed.vars.patch, Some(3));
    }
}
