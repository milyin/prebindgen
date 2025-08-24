use std::collections::HashSet;

use crate::{codegen::cfg_expr::CfgExpr, codegen::CfgExprRules, SourceLocation};

#[test]
fn test_simple_feature() {
    let expr = CfgExpr::parse_from_string(r#"feature = "test""#).unwrap();
    assert_eq!(expr, CfgExpr::Feature("test".to_string()));
}

#[test]
fn test_target_arch() {
    let expr = CfgExpr::parse_from_string(r#"target_arch = "x86_64""#).unwrap();
    assert_eq!(expr, CfgExpr::TargetArch("x86_64".to_string()));
}

#[test]
fn test_target_vendor_os_env_parse() {
    let vendor = CfgExpr::parse_from_string(r#"target_vendor = "apple""#).unwrap();
    assert_eq!(vendor, CfgExpr::TargetVendor("apple".to_string()));

    let os = CfgExpr::parse_from_string(r#"target_os = "macos""#).unwrap();
    assert_eq!(os, CfgExpr::TargetOs("macos".to_string()));

    let env = CfgExpr::parse_from_string(r#"target_env = "gnu""#).unwrap();
    assert_eq!(env, CfgExpr::TargetEnv("gnu".to_string()));
}

#[test]
fn test_target_filters_processing() {
    use std::collections::HashMap;
    let enabled_features = HashSet::new();
    let disabled_features = HashSet::new();
    let feature_mappings: HashMap<String, String> = HashMap::new();
    let src = SourceLocation::default();

    // With no selection, keep predicates as-is
    let expr = CfgExpr::TargetOs("macos".into());
    assert_eq!(
        expr.apply_rules(
            &CfgExprRules {
                enabled_features: enabled_features.clone(),
                disabled_features: disabled_features.clone(),
                feature_mappings: feature_mappings.clone(),
                ..Default::default()
            },
            &src,
        ),
        Some(CfgExpr::TargetOs("macos".into()))
    );

    // Select OS = macos: becomes true (None)
    assert_eq!(
        CfgExpr::TargetOs("macos".into()).apply_rules(
            &CfgExprRules {
                enabled_features: enabled_features.clone(),
                disabled_features: disabled_features.clone(),
                feature_mappings: feature_mappings.clone(),
                enabled_target_os: Some("macos".into()),
                ..Default::default()
            },
            &src,
        ),
        None
    );

    // Non-matching becomes False
    assert_eq!(
        CfgExpr::TargetOs("linux".into()).apply_rules(
            &CfgExprRules {
                enabled_features: enabled_features.clone(),
                disabled_features: disabled_features.clone(),
                feature_mappings: feature_mappings.clone(),
                enabled_target_os: Some("macos".into()),
                ..Default::default()
            },
            &src,
        ),
        Some(CfgExpr::False)
    );

    // Arch selection
    assert_eq!(
        CfgExpr::TargetArch("x86_64".into()).apply_rules(
            &CfgExprRules {
                enabled_features: enabled_features.clone(),
                disabled_features: disabled_features.clone(),
                feature_mappings: feature_mappings.clone(),
                enabled_target_arch: Some("x86_64".into()),
                ..Default::default()
            },
            &src,
        ),
        None
    );
    assert_eq!(
        CfgExpr::TargetArch("aarch64".into()).apply_rules(
            &CfgExprRules {
                enabled_features: enabled_features.clone(),
                disabled_features: disabled_features.clone(),
                feature_mappings: feature_mappings.clone(),
                enabled_target_arch: Some("x86_64".into()),
                ..Default::default()
            },
            &src,
        ),
        Some(CfgExpr::False)
    );

    // Vendor and Env selection
    assert_eq!(
        CfgExpr::TargetVendor("apple".into()).apply_rules(
            &CfgExprRules {
                enabled_features: enabled_features.clone(),
                disabled_features: disabled_features.clone(),
                feature_mappings: feature_mappings.clone(),
                enabled_target_vendor: Some("apple".into()),
                ..Default::default()
            },
            &src,
        ),
        None
    );
    assert_eq!(
        CfgExpr::TargetEnv("gnu".into()).apply_rules(
            &CfgExprRules {
                enabled_features: enabled_features.clone(),
                disabled_features: disabled_features.clone(),
                feature_mappings: feature_mappings.clone(),
                enabled_target_env: Some("gnu".into()),
                ..Default::default()
            },
            &src,
        ),
        None
    );
}

#[test]
fn test_not_expression() {
    let expr = CfgExpr::parse_from_string(r#"not(feature = "test")"#).unwrap();
    match expr {
        CfgExpr::Not(inner) => {
            assert_eq!(*inner, CfgExpr::Feature("test".to_string()));
        }
        _ => panic!("Expected Not expression, got: {expr:?}"),
    }
}

#[test]
fn test_any_expression() {
    let expr = CfgExpr::parse_from_string(r#"any(feature = "a", feature = "b")"#).unwrap();
    match expr {
        CfgExpr::Any(exprs) => {
            assert_eq!(exprs.len(), 2);
            assert_eq!(exprs[0], CfgExpr::Feature("a".to_string()));
            assert_eq!(exprs[1], CfgExpr::Feature("b".to_string()));
        }
        _ => panic!("Expected Any expression"),
    }
}

#[test]
fn test_all_expression() {
    let expr = CfgExpr::parse_from_string(r#"all(feature = "a", feature = "b")"#).unwrap();
    match expr {
        CfgExpr::All(exprs) => {
            assert_eq!(exprs.len(), 2);
            assert_eq!(exprs[0], CfgExpr::Feature("a".to_string()));
            assert_eq!(exprs[1], CfgExpr::Feature("b".to_string()));
        }
        _ => panic!("Expected All expression"),
    }
}

#[test]
fn test_strict_feature_processing() {
    use std::collections::HashMap;

    let mut enabled_features = HashSet::new();
    enabled_features.insert("feature1".to_string());

    let mut disabled_features = HashSet::new();
    disabled_features.insert("feature2".to_string());

    let mut feature_mappings = HashMap::new();
    feature_mappings.insert("old_feature".to_string(), "new_feature".to_string());

    // Test enabled feature - should be removed (None = always true)
    let expr = CfgExpr::Feature("feature1".to_string());
    assert_eq!(
        expr.apply_rules(
            &CfgExprRules {
                enabled_features: enabled_features.clone(),
                disabled_features: disabled_features.clone(),
                feature_mappings: feature_mappings.clone(),
                ..Default::default()
            },
            &SourceLocation::default()
        ),
        None
    );

    // Test disabled feature - should become False
    let expr = CfgExpr::Feature("feature2".to_string());
    assert_eq!(
        expr.apply_rules(
            &CfgExprRules {
                enabled_features: enabled_features.clone(),
                disabled_features: disabled_features.clone(),
                feature_mappings: feature_mappings.clone(),
                ..Default::default()
            },
            &SourceLocation::default()
        ),
        Some(CfgExpr::False)
    );

    // Test mapped feature - should be renamed
    let expr = CfgExpr::Feature("old_feature".to_string());
    assert_eq!(
        expr.apply_rules(
            &CfgExprRules {
                enabled_features: enabled_features.clone(),
                disabled_features: disabled_features.clone(),
                feature_mappings: feature_mappings.clone(),
                ..Default::default()
            },
            &SourceLocation::default()
        ),
        Some(CfgExpr::Feature("new_feature".to_string()))
    );

    // Test any() with enabled feature - should be removed (None = always true)
    let expr = CfgExpr::Any(vec![
        CfgExpr::Feature("feature1".to_string()),
        CfgExpr::Feature("feature2".to_string()),
    ]);
    assert_eq!(
        expr.apply_rules(
            &CfgExprRules {
                enabled_features: enabled_features.clone(),
                disabled_features: disabled_features.clone(),
                feature_mappings: feature_mappings.clone(),
                ..Default::default()
            },
            &SourceLocation::default()
        ),
        None
    );

    // Test all() with disabled feature - should become False
    let expr = CfgExpr::All(vec![
        CfgExpr::Feature("feature1".to_string()),
        CfgExpr::Feature("feature2".to_string()),
    ]);
    assert_eq!(
        expr.apply_rules(
            &CfgExprRules {
                enabled_features: enabled_features.clone(),
                disabled_features: disabled_features.clone(),
                feature_mappings: feature_mappings.clone(),
                ..Default::default()
            },
            &SourceLocation::default()
        ),
        Some(CfgExpr::False)
    );

    // Test not() with disabled feature - should be removed (not(false) = true)
    let expr = CfgExpr::Not(Box::new(CfgExpr::Feature("feature2".to_string())));
    assert_eq!(
        expr.apply_rules(
            &CfgExprRules {
                enabled_features: enabled_features.clone(),
                disabled_features: disabled_features.clone(),
                feature_mappings: feature_mappings.clone(),
                ..Default::default()
            },
            &SourceLocation::default()
        ),
        None
    );

    // Test not() with enabled feature - should become False (not(true) = false)
    let expr = CfgExpr::Not(Box::new(CfgExpr::Feature("feature1".to_string())));
    assert_eq!(
        expr.apply_rules(
            &CfgExprRules {
                enabled_features: enabled_features.clone(),
                disabled_features: disabled_features.clone(),
                feature_mappings: feature_mappings.clone(),
                ..Default::default()
            },
            &SourceLocation::default()
        ),
        Some(CfgExpr::False)
    );
}

#[test]
#[should_panic(expected = "unmapped feature: unknown")]
fn test_strict_feature_processing_unmapped_panic() {
    use std::collections::HashMap;

    let enabled_features = HashSet::new();
    let disabled_features = HashSet::new();
    let feature_mappings = HashMap::new();

    // Test unmapped feature - should panic
    let expr = CfgExpr::Feature("unknown".to_string());
    expr.apply_rules(
        &CfgExprRules {
            enabled_features,
            disabled_features,
            feature_mappings,
            ..Default::default()
        },
        &SourceLocation::default(),
    );
}

#[test]
#[should_panic(expected = "unmapped feature: unknown")]
fn test_strict_feature_processing_unmapped_in_any_panic() {
    use std::collections::HashMap;

    let mut enabled_features = HashSet::new();
    enabled_features.insert("feature1".to_string());

    let disabled_features = HashSet::new();
    let feature_mappings = HashMap::new();

    // Test unmapped feature in any() - should panic
    let expr = CfgExpr::Any(vec![
        CfgExpr::Feature("feature1".to_string()),
        CfgExpr::Feature("unknown".to_string()),
    ]);
    expr.apply_rules(
        &CfgExprRules {
            enabled_features,
            disabled_features,
            feature_mappings,
            ..Default::default()
        },
        &SourceLocation::default(),
    );
}
