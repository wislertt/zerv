# SemVer to Zerv Conversion - Clean Implementation

```rust
use super::{
    BuildMetadata,
    PreReleaseIdentifier,
    SemVer,
};
use crate::error::ZervError;
use crate::version::zerv::core::PreReleaseLabel;
use crate::version::zerv::{
    Component,
    PreReleaseVar,
    Var,
    Zerv,
    ZervSchema,
    ZervVars,
};

struct PreReleaseProcessor<'a> {
    vars: &'a mut ZervVars,
    schema: &'a mut ZervSchema,
    pending_var: Option<Var>,
}

impl<'a> PreReleaseProcessor<'a> {
    fn new(vars: &'a mut ZervVars, schema: &'a mut ZervSchema) -> Self {
        Self {
            vars,
            schema,
            pending_var: None,
        }
    }

    fn is_var_set(&self, var: &Var) -> bool {
        match var {
            Var::PreRelease => self.vars.pre_release.is_some() || self.pending_var == Some(Var::PreRelease),
            Var::Epoch => self.vars.epoch.is_some() || self.pending_var == Some(Var::Epoch),
            Var::Post => self.vars.post.is_some() || self.pending_var == Some(Var::Post),
            Var::Dev => self.vars.dev.is_some() || self.pending_var == Some(Var::Dev),
            _ => false,
        }
    }

    fn handle_pending_prerelease(&mut self, identifier: &PreReleaseIdentifier) -> Result<bool, ZervError> {
        if let Some(Var::PreRelease) = self.pending_var {
            if let PreReleaseIdentifier::String(_) = identifier {
                self.schema.push_extra_core(Component::Var(Var::PreRelease))?;
                self.pending_var = None;
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn handle_duplicate(&mut self, var: Var, s: &str) -> Result<bool, ZervError> {
        if let Some(pending) = self.pending_var {
            if let Some(current_var) = Var::try_from_secondary_label(s) {
                if current_var == pending || self.is_var_set(&current_var) {
                    self.schema.push_extra_core(Component::Var(pending))?;
                    self.pending_var = None;
                    self.schema.push_extra_core(Component::Str(s.to_string()))?;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn set_var_value(&mut self, var: Var, value: Option<u64>) -> Result<(), ZervError> {
        match var {
            Var::Epoch => self.vars.epoch = value,
            Var::Post => self.vars.post = value,
            Var::Dev => self.vars.dev = value,
            Var::PreRelease => {
                if let Some(ref mut pr) = self.vars.pre_release {
                    pr.number = value;
                }
            }
            _ => {}
        }
        self.schema.push_extra_core(Component::Var(var))?;
        self.pending_var = None;
        Ok(())
    }

    fn process_string(&mut self, s: &str) -> Result<(), ZervError> {
        if let Some(var) = Var::try_from_secondary_label(s) {
            if self.is_var_set(&var) {
                self.schema.push_extra_core(Component::Str(s.to_string()))?;
            } else if var == Var::PreRelease {
                if let Some(label) = PreReleaseLabel::try_from_str(s) {
                    self.vars.pre_release = Some(PreReleaseVar { label, number: None });
                    self.pending_var = Some(var);
                } else {
                    self.schema.push_extra_core(Component::Str(s.to_string()))?;
                }
            } else {
                self.pending_var = Some(var);
            }
        } else {
            self.schema.push_extra_core(Component::Str(s.to_string()))?;
        }
        Ok(())
    }

    fn process_uint(&mut self, n: u64) -> Result<(), ZervError> {
        if let Some(var) = self.pending_var {
            self.set_var_value(var, Some(n))
        } else {
            self.schema.push_extra_core(Component::Int(n))
        }
    }

    fn finalize(&mut self) -> Result<(), ZervError> {
        if let Some(var) = self.pending_var {
            self.schema.push_extra_core(Component::Var(var))?;
        }
        Ok(())
    }
}

impl From<SemVer> for Zerv {
    fn from(semver: SemVer) -> Self {
        let schema = ZervSchema::semver_default().expect("SemVer default schema should be valid");
        semver
            .to_zerv_with_schema(&schema)
            .expect("SemVer default conversion should work")
    }
}

impl SemVer {
    pub fn to_zerv_with_schema(&self, schema: &ZervSchema) -> Result<Zerv, ZervError> {
        if *schema != ZervSchema::semver_default()? {
            return Err(ZervError::NotImplemented(
                "Custom schemas not yet implemented for SemVer conversion".to_string(),
            ));
        }

        let mut vars = ZervVars {
            major: Some(self.major),
            minor: Some(self.minor),
            patch: Some(self.patch),
            ..Default::default()
        };

        let mut schema = schema.clone();
        let mut processor = PreReleaseProcessor::new(&mut vars, &mut schema);

        // Process pre-release identifiers
        if let Some(pre_release) = &self.pre_release {
            for identifier in pre_release {
                // Handle pending PreRelease var
                if processor.handle_pending_prerelease(identifier)? {
                    continue;
                }

                // Handle pending var with potential duplicates
                if let PreReleaseIdentifier::String(s) = identifier {
                    if processor.handle_duplicate(processor.pending_var.unwrap_or(Var::Major), s)? {
                        continue;
                    }
                }

                // Process pending var value
                if processor.pending_var.is_some() {
                    let value = match identifier {
                        PreReleaseIdentifier::UInt(n) => Some(*n),
                        _ => None,
                    };
                    processor.set_var_value(processor.pending_var.unwrap(), value)?;
                    continue;
                }

                // Process new identifier
                match identifier {
                    PreReleaseIdentifier::String(s) => processor.process_string(s)?,
                    PreReleaseIdentifier::UInt(n) => processor.process_uint(*n)?,
                }
            }
        }

        processor.finalize()?;

        // Handle build metadata
        if let Some(build_metadata) = &self.build_metadata {
            for metadata in build_metadata {
                let component = match metadata {
                    BuildMetadata::String(s) => Component::Str(s.clone()),
                    BuildMetadata::UInt(n) => Component::Int(*n),
                };
                schema.push_build(component)?;
            }
        }

        Ok(Zerv { vars, schema })
    }
}
```
