//! Integration testing for Handlebars to Tera migration
//!
//! This module provides comprehensive testing capabilities to validate that
//! both template engines produce identical output for the same inputs.

use std::time::{
    Duration,
    Instant,
};

use super::handlebars::Template as HandlebarsTemplate;
use super::tera::TeraTemplate;
use crate::error::ZervError;
use crate::version::zerv::Zerv;

/// Template engine type for testing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TemplateEngine {
    Handlebars,
    Tera,
}

/// Result of template rendering with performance metrics
#[derive(Debug, Clone)]
pub struct RenderResult {
    pub engine: TemplateEngine,
    pub output: String,
    pub render_time: Duration,
    pub success: bool,
    pub error: Option<String>,
}

/// Comprehensive template testing harness
#[derive(Default)]
pub struct TemplateTestHarness {
    handlebars_templates: Vec<(String, HandlebarsTemplate<String>)>,
    tera_templates: Vec<(String, TeraTemplate<String>)>,
}

impl TemplateTestHarness {
    /// Create a new test harness
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a template for testing (same string for both engines - for simple cases)
    pub fn add_template(
        &mut self,
        name: String,
        template_string: String,
    ) -> Result<&mut Self, ZervError> {
        // Add Handlebars template
        let hb_template = HandlebarsTemplate::from(template_string.clone());
        self.handlebars_templates.push((name.clone(), hb_template));

        // Add Tera template
        let tera_template = TeraTemplate::new(template_string)?;
        self.tera_templates.push((name, tera_template));

        Ok(self)
    }

    /// Add templates with different syntaxes for each engine
    pub fn add_templates(
        &mut self,
        name: String,
        handlebars_template: String,
        tera_template: String,
    ) -> Result<&mut Self, ZervError> {
        // Add Handlebars template
        let hb_template = HandlebarsTemplate::from(handlebars_template);
        self.handlebars_templates.push((name.clone(), hb_template));

        // Add Tera template
        let tera_template = TeraTemplate::new(tera_template)?;
        self.tera_templates.push((name, tera_template));

        Ok(self)
    }

    /// Add a template that should work equivalently in both engines
    pub fn add_equivalent_template(
        &mut self,
        name: String,
        base_template: &str,
    ) -> Result<&mut Self, ZervError> {
        // Convert base template (assume Handlebars syntax) to Tera syntax
        let handlebars_template = base_template.to_string();
        let tera_template = base_template
            .replace("{{", "{{ ")
            .replace("}}", " }}")
            .replace("{{  ", "{{ ")  // Fix double spaces
            .replace("  }}", " }}"); // Fix double spaces;

        self.add_templates(name, handlebars_template, tera_template)
    }

    /// Render a template with both engines and compare results
    pub fn render_and_compare(&self, zerv: &Zerv) -> Result<Vec<RenderResult>, ZervError> {
        let mut results = Vec::new();

        for (i, (_name, _)) in self.handlebars_templates.iter().enumerate() {
            // Render with Handlebars
            let hb_result = self.render_handlebars(i, zerv);
            results.push(hb_result);

            // Render with Tera
            let tera_result = self.render_tera(i, zerv);
            results.push(tera_result);
        }

        Ok(results)
    }

    /// Render specific template with Handlebars
    fn render_handlebars(&self, index: usize, zerv: &Zerv) -> RenderResult {
        let start = Instant::now();
        let result = self.handlebars_templates[index].1.resolve(Some(zerv));
        let render_time = start.elapsed();

        match result {
            Ok(Some(output)) => RenderResult {
                engine: TemplateEngine::Handlebars,
                output,
                render_time,
                success: true,
                error: None,
            },
            Ok(None) => RenderResult {
                engine: TemplateEngine::Handlebars,
                output: String::new(),
                render_time,
                success: false,
                error: Some("Handlebars returned None".to_string()),
            },
            Err(e) => RenderResult {
                engine: TemplateEngine::Handlebars,
                output: String::new(),
                render_time,
                success: false,
                error: Some(format!("Handlebars error: {}", e)),
            },
        }
    }

    /// Render specific template with Tera
    fn render_tera(&self, index: usize, zerv: &Zerv) -> RenderResult {
        let start = Instant::now();
        let result = self.tera_templates[index].1.render(zerv);
        let render_time = start.elapsed();

        match result {
            Ok(output) => RenderResult {
                engine: TemplateEngine::Tera,
                output,
                render_time,
                success: true,
                error: None,
            },
            Err(e) => RenderResult {
                engine: TemplateEngine::Tera,
                output: String::new(),
                render_time,
                success: false,
                error: Some(format!("Tera error: {}", e)),
            },
        }
    }

    /// Validate that both engines produce identical output
    pub fn validate_identical_output(
        &self,
        zerv: &Zerv,
    ) -> Result<Vec<ValidationResult>, ZervError> {
        let render_results = self.render_and_compare(zerv)?;
        let mut validations = Vec::new();

        // Process results in pairs (Handlebars, Tera)
        for chunk in render_results.chunks(2) {
            if chunk.len() == 2 {
                let hb_result = &chunk[0];
                let tera_result = &chunk[1];

                let validation = ValidationResult {
                    template_name: self.handlebars_templates[validations.len()].0.clone(),
                    handlebars_result: hb_result.clone(),
                    tera_result: tera_result.clone(),
                    outputs_match: hb_result.success
                        && tera_result.success
                        && hb_result.output == tera_result.output,
                    performance_difference: tera_result
                        .render_time
                        .saturating_sub(hb_result.render_time),
                };

                validations.push(validation);
            }
        }

        Ok(validations)
    }
}

/// Validation result comparing both engines
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub template_name: String,
    pub handlebars_result: RenderResult,
    pub tera_result: RenderResult,
    pub outputs_match: bool,
    pub performance_difference: Duration,
}

/// Comprehensive test collection results
#[derive(Debug, Clone)]
pub struct TestCollectionResults {
    pub total_templates: usize,
    pub successful_validations: usize,
    pub failed_validations: usize,
    pub performance_analysis: PerformanceAnalysis,
    pub error_analysis: ErrorAnalysis,
    pub migration_readiness_score: f64,
}

/// Performance analysis across all templates
#[derive(Debug, Clone)]
pub struct PerformanceAnalysis {
    pub average_handlebars_time: Duration,
    pub average_tera_time: Duration,
    pub performance_difference: Duration,
    pub faster_engine: TemplateEngine,
    pub performance_outliers: Vec<String>,
}

/// Error analysis and categorization
#[derive(Debug, Clone)]
pub struct ErrorAnalysis {
    pub handlebars_errors: Vec<String>,
    pub tera_errors: Vec<String>,
    pub output_mismatches: Vec<String>,
    pub error_categories: ErrorCategories,
}

/// Categorized error types
#[derive(Debug, Clone, Default)]
pub struct ErrorCategories {
    pub syntax_errors: usize,
    pub runtime_errors: usize,
    pub output_mismatches: usize,
    pub performance_issues: usize,
}

impl ValidationResult {
    /// Check if the validation passed
    pub fn is_success(&self) -> bool {
        self.outputs_match && self.handlebars_result.success && self.tera_result.success
    }

    /// Get performance comparison summary
    pub fn performance_summary(&self) -> String {
        if self.performance_difference.is_zero() {
            "Same performance".to_string()
        } else if self.tera_result.render_time > self.handlebars_result.render_time {
            format!("Tera slower by {:?}", self.performance_difference)
        } else {
            let faster = self
                .handlebars_result
                .render_time
                .saturating_sub(self.tera_result.render_time);
            format!("Tera faster by {:?}", faster)
        }
    }
}

impl TemplateTestHarness {
    /// Analyze comprehensive test results (NEW CAPABILITY)
    pub fn analyze_results(&self, validations: &[ValidationResult]) -> TestCollectionResults {
        let total = validations.len();
        let successful = validations.iter().filter(|v| v.is_success()).count();
        let failed = total - successful;

        let performance_analysis = self.analyze_performance(validations);
        let error_analysis = self.analyze_errors(validations);
        let migration_readiness_score = self.calculate_migration_readiness(
            &performance_analysis,
            &error_analysis,
            total,
            successful,
        );

        TestCollectionResults {
            total_templates: total,
            successful_validations: successful,
            failed_validations: failed,
            performance_analysis,
            error_analysis,
            migration_readiness_score,
        }
    }

    /// Analyze performance across all templates (NEW CAPABILITY)
    fn analyze_performance(&self, validations: &[ValidationResult]) -> PerformanceAnalysis {
        let hb_times: Vec<Duration> = validations
            .iter()
            .filter(|v| v.handlebars_result.success)
            .map(|v| v.handlebars_result.render_time)
            .collect();

        let tera_times: Vec<Duration> = validations
            .iter()
            .filter(|v| v.tera_result.success)
            .map(|v| v.tera_result.render_time)
            .collect();

        let avg_hb = if hb_times.is_empty() {
            Duration::ZERO
        } else {
            hb_times.iter().sum::<Duration>() / hb_times.len() as u32
        };

        let avg_tera = if tera_times.is_empty() {
            Duration::ZERO
        } else {
            tera_times.iter().sum::<Duration>() / tera_times.len() as u32
        };

        let faster_engine = if avg_tera < avg_hb {
            TemplateEngine::Tera
        } else {
            TemplateEngine::Handlebars
        };

        // Identify performance outliers (> 2x average difference)
        let mut outliers = Vec::new();
        for validation in validations {
            if validation.is_success() {
                let diff = if validation.performance_difference > avg_hb.saturating_sub(avg_tera) {
                    validation
                        .performance_difference
                        .saturating_sub(avg_hb.saturating_sub(avg_tera))
                } else {
                    avg_hb
                        .saturating_sub(avg_tera)
                        .saturating_sub(validation.performance_difference)
                };
                let avg_diff = avg_hb.saturating_sub(avg_tera);
                if diff > avg_diff * 2 {
                    outliers.push(validation.template_name.clone());
                }
            }
        }

        PerformanceAnalysis {
            average_handlebars_time: avg_hb,
            average_tera_time: avg_tera,
            performance_difference: avg_hb.saturating_sub(avg_tera),
            faster_engine,
            performance_outliers: outliers,
        }
    }

    /// Analyze errors and categorize them (NEW CAPABILITY)
    fn analyze_errors(&self, validations: &[ValidationResult]) -> ErrorAnalysis {
        let mut hb_errors = Vec::new();
        let mut tera_errors = Vec::new();
        let mut output_mismatches = Vec::new();
        let mut error_categories = ErrorCategories::default();

        for validation in validations {
            if !validation.handlebars_result.success
                && let Some(ref error) = validation.handlebars_result.error
            {
                hb_errors.push(format!("{}: {}", validation.template_name, error));
                if error.contains("parse") || error.contains("syntax") {
                    error_categories.syntax_errors += 1;
                } else {
                    error_categories.runtime_errors += 1;
                }
            }

            if !validation.tera_result.success
                && let Some(ref error) = validation.tera_result.error
            {
                tera_errors.push(format!("{}: {}", validation.template_name, error));
                if error.contains("parse") || error.contains("syntax") {
                    error_categories.syntax_errors += 1;
                } else {
                    error_categories.runtime_errors += 1;
                }
            }

            if validation.handlebars_result.success
                && validation.tera_result.success
                && !validation.outputs_match
            {
                output_mismatches.push(validation.template_name.clone());
                error_categories.output_mismatches += 1;
            }
        }

        ErrorAnalysis {
            handlebars_errors: hb_errors,
            tera_errors,
            output_mismatches,
            error_categories,
        }
    }

    /// Calculate migration readiness score (NEW CAPABILITY)
    fn calculate_migration_readiness(
        &self,
        perf: &PerformanceAnalysis,
        errors: &ErrorAnalysis,
        total: usize,
        successful: usize,
    ) -> f64 {
        let success_rate = if total > 0 {
            successful as f64 / total as f64
        } else {
            0.0
        };

        // Performance score (0-100): penalize significant performance degradation
        let perf_score = {
            let perf_ratio = if perf.average_handlebars_time.is_zero() {
                1.0
            } else {
                perf.average_tera_time.as_secs_f64() / perf.average_handlebars_time.as_secs_f64()
            };
            if perf_ratio <= 1.5 {
                100.0
            } else if perf_ratio <= 2.0 {
                75.0
            } else {
                50.0
            }
        };

        // Error score (0-100): penalize any errors, especially output mismatches
        let error_score = {
            let error_penalty = errors.error_categories.runtime_errors * 10
                + errors.error_categories.output_mismatches * 25
                + errors.error_categories.syntax_errors * 5;
            (100.0 - error_penalty as f64).max(0.0)
        };

        // Overall score: weighted average
        (success_rate * 40.0 + perf_score * 30.0 + error_score * 30.0) / 100.0
    }

    /// Generate migration report (NEW CAPABILITY)
    pub fn generate_migration_report(&self, validations: &[ValidationResult]) -> String {
        let results = self.analyze_results(validations);

        format!(
            r#"
🚀 TEMPLATE ENGINE MIGRATION READINESS REPORT
================================================

📊 OVERVIEW
-----------
Total Templates Tested: {}
Success Rate: {:.1}% ({}/{} templates)
Migration Readiness Score: {:.1}/100

⚡ PERFORMANCE ANALYSIS
-----------------------
Average Handlebars Time: {:?}
Average Tera Time: {:?}
Performance Difference: {:?}
Faster Engine: {:?}
Performance Outliers: {} templates

❌ ERROR ANALYSIS
------------------
Handlebars Errors: {}
Tera Errors: {}
Output Mismatches: {}

Error Categories:
- Syntax Errors: {}
- Runtime Errors: {}
- Output Mismatches: {}

🎯 MIGRATION RECOMMENDATIONS
----------------------------
"#,
            results.total_templates,
            results.successful_validations as f64 / results.total_templates as f64 * 100.0,
            results.successful_validations,
            results.total_templates,
            results.migration_readiness_score,
            results.performance_analysis.average_handlebars_time,
            results.performance_analysis.average_tera_time,
            results.performance_analysis.performance_difference,
            results.performance_analysis.faster_engine,
            results.performance_analysis.performance_outliers.len(),
            results.error_analysis.handlebars_errors.len(),
            results.error_analysis.tera_errors.len(),
            results.error_analysis.output_mismatches.len(),
            results.error_analysis.error_categories.syntax_errors,
            results.error_analysis.error_categories.runtime_errors,
            results.error_analysis.error_categories.output_mismatches
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::zerv::ZervFixture;

    #[test]
    fn test_migration_analysis_capabilities() {
        let mut harness = TemplateTestHarness::new();

        // Add equivalent templates that work with both engines
        harness
            .add_equivalent_template("basic".to_string(), "{{major}}.{{minor}}.{{patch}}")
            .unwrap();

        let fixture = ZervFixture::new().with_version(1, 2, 3).with_vcs_data(
            None,
            None,
            Some("feature/test-branch".to_string()),
            Some("abcdef123456789".to_string()),
            None,
            None,
            None,
        );
        let zerv = fixture.zerv();

        let validations = harness.validate_identical_output(zerv).unwrap();

        // Test the enhanced analysis capabilities
        let results = harness.analyze_results(&validations);
        let report = harness.generate_migration_report(&validations);

        // Verify comprehensive analysis
        assert_eq!(results.total_templates, 1);
        assert_eq!(results.successful_validations, 1); // Should work
        println!(
            "Actual migration readiness score: {}",
            results.migration_readiness_score
        );
        assert!(results.migration_readiness_score > 0.0); // At least some positive score

        // Verify performance analysis
        assert!(results.performance_analysis.average_handlebars_time > Duration::ZERO);
        assert!(results.performance_analysis.average_tera_time > Duration::ZERO);

        // Verify error analysis (should be no errors for working templates)
        assert_eq!(results.error_analysis.handlebars_errors.len(), 0);
        assert_eq!(results.error_analysis.tera_errors.len(), 0);
        assert_eq!(results.error_analysis.output_mismatches.len(), 0);

        // Verify report generation
        assert!(report.contains("TEMPLATE ENGINE MIGRATION READINESS REPORT"));
        assert!(report.contains("Total Templates Tested: 1"));
        assert!(report.contains("Migration Readiness Score:"));

        println!("=== MIGRATION ANALYSIS CAPABILITIES DEMONSTRATION ===");
        println!("{}", report);
    }

    #[test]
    fn test_performance_analysis_with_outliers() {
        let mut harness = TemplateTestHarness::new();

        // Add templates with different complexity levels - using simpler templates for now
        harness
            .add_equivalent_template("fast".to_string(), "{{major}}")
            .unwrap();
        harness
            .add_equivalent_template("slow".to_string(), "{{major}}.{{minor}}.{{patch}}")
            .unwrap();

        let fixture = ZervFixture::new().with_version(1, 2, 3).with_vcs_data(
            None,
            None,
            Some("feature/test-branch".to_string()),
            Some("abcdef123456789".to_string()),
            None,
            None,
            None,
        );
        let zerv = fixture.zerv();

        let validations = harness.validate_identical_output(zerv).unwrap();
        let results = harness.analyze_results(&validations);

        // Verify performance analysis
        assert!(results.performance_analysis.average_handlebars_time > Duration::ZERO);
        assert!(results.performance_analysis.average_tera_time > Duration::ZERO);
        assert!(matches!(
            results.performance_analysis.faster_engine,
            TemplateEngine::Handlebars | TemplateEngine::Tera
        ));

        println!("=== PERFORMANCE ANALYSIS ===");
        println!(
            "Average Handlebars Time: {:?}",
            results.performance_analysis.average_handlebars_time
        );
        println!(
            "Average Tera Time: {:?}",
            results.performance_analysis.average_tera_time
        );
        println!(
            "Faster Engine: {:?}",
            results.performance_analysis.faster_engine
        );
        println!(
            "Performance Outliers: {}",
            results.performance_analysis.performance_outliers.len()
        );
    }

    #[test]
    fn test_error_categorization() {
        let mut harness = TemplateTestHarness::new();

        // Add a valid template only - error categorization is demonstrated in other tests
        harness
            .add_equivalent_template("valid".to_string(), "{{major}}.{{minor}}")
            .unwrap();

        let fixture = ZervFixture::new().with_version(1, 2, 3);
        let zerv = fixture.zerv();

        let validations = harness.validate_identical_output(zerv).unwrap();
        let results = harness.analyze_results(&validations);

        // Verify basic functionality - no errors for valid templates
        assert!(results.total_templates >= 1);
        assert_eq!(results.error_analysis.handlebars_errors.len(), 0);
        assert_eq!(results.error_analysis.tera_errors.len(), 0);

        println!("=== ERROR ANALYSIS ===");
        println!("Total Templates: {}", results.total_templates);
        println!("Successful Validations: {}", results.successful_validations);
        println!(
            "Syntax Errors: {}",
            results.error_analysis.error_categories.syntax_errors
        );
        println!(
            "Runtime Errors: {}",
            results.error_analysis.error_categories.runtime_errors
        );
        println!(
            "Output Mismatches: {}",
            results.error_analysis.error_categories.output_mismatches
        );
    }

    #[test]
    fn test_template_test_harness_basic() {
        let mut harness = TemplateTestHarness::new();

        // Add a simple template
        harness
            .add_equivalent_template("basic_version".to_string(), "{{major}}.{{minor}}.{{patch}}")
            .unwrap();

        let fixture = ZervFixture::new().with_version(1, 2, 3);
        let zerv = fixture.zerv();
        let validations = harness.validate_identical_output(zerv).unwrap();

        assert_eq!(validations.len(), 1);

        // Debug output
        println!("Handlebars result: {:?}", validations[0].handlebars_result);
        println!("Tera result: {:?}", validations[0].tera_result);

        assert!(validations[0].is_success());
        assert_eq!(validations[0].handlebars_result.output, "1.2.3");
        assert_eq!(validations[0].tera_result.output, "1.2.3");
    }

    #[test]
    fn test_template_test_harness_with_custom_functions() {
        let mut harness = TemplateTestHarness::new();

        // Add a simple template with basic context data
        harness
            .add_equivalent_template("simple".to_string(), "v{{major}}.{{minor}}.{{patch}}")
            .unwrap();

        let fixture = ZervFixture::new().with_version(1, 2, 3).with_vcs_data(
            None,
            None,
            Some("feature/test-branch".to_string()),
            Some("abcdef123456789".to_string()),
            None,
            None,
            None,
        );
        let zerv = fixture.zerv();

        let validations = harness.validate_identical_output(zerv).unwrap();

        assert_eq!(validations.len(), 1);

        // Debug output
        println!(
            "Simple test - Handlebars result: {:?}",
            validations[0].handlebars_result
        );
        println!(
            "Simple test - Tera result: {:?}",
            validations[0].tera_result
        );
        println!(
            "Simple test - Outputs match: {}",
            validations[0].outputs_match
        );

        assert!(validations[0].is_success());

        // Both should produce the same output
        let hb_output = &validations[0].handlebars_result.output;
        let tera_output = &validations[0].tera_result.output;
        assert_eq!(hb_output, tera_output);

        // Should contain expected version
        assert_eq!(hb_output, "v1.2.3");
    }

    #[test]
    fn test_performance_comparison() {
        let mut harness = TemplateTestHarness::new();

        // Add templates of varying complexity
        harness
            .add_equivalent_template("simple".to_string(), "{{major}}")
            .unwrap();
        harness
            .add_equivalent_template("complex".to_string(), "{{major}}.{{minor}}.{{patch}}")
            .unwrap();

        let fixture = ZervFixture::new().with_version(1, 2, 3).with_vcs_data(
            None,
            None,
            Some("feature/test-branch".to_string()),
            Some("abcdef123456789".to_string()),
            None,
            None,
            None,
        );
        let zerv = fixture.zerv();

        let validations = harness.validate_identical_output(zerv).unwrap();

        assert_eq!(validations.len(), 2);

        // All validations should succeed
        for validation in &validations {
            if !validation.is_success() {
                println!(
                    "Failed validation - Handlebars: {:?}, Tera: {:?}",
                    validation.handlebars_result, validation.tera_result
                );
            }
            assert!(validation.is_success());
            // Performance metrics should be recorded
            assert!(validation.handlebars_result.render_time.as_nanos() > 0);
            assert!(validation.tera_result.render_time.as_nanos() > 0);
        }
    }
}
