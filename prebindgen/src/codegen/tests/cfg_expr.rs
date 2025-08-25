use crate::{
    codegen::{cfg_expr::CfgExpr, CfgExprRules},
    SourceLocation,
};

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
    let src = SourceLocation::default();

    // With no selection, keep predicates as-is
    let expr = CfgExpr::TargetOs("macos".into());
    assert_eq!(
        expr.apply_rules(&CfgExprRules::default(), &src),
        Some(CfgExpr::TargetOs("macos".into()))
    );

    // No selection for arch/vendor/env should also keep predicates as-is
    assert_eq!(
        CfgExpr::TargetArch("x86_64".into()).apply_rules(&CfgExprRules::default(), &src),
        Some(CfgExpr::TargetArch("x86_64".into()))
    );
    assert_eq!(
        CfgExpr::TargetVendor("apple".into()).apply_rules(&CfgExprRules::default(), &src),
        Some(CfgExpr::TargetVendor("apple".into()))
    );
    assert_eq!(
        CfgExpr::TargetEnv("gnu".into()).apply_rules(&CfgExprRules::default(), &src),
        Some(CfgExpr::TargetEnv("gnu".into()))
    );

    // Select OS = macos: becomes true (None)
    assert_eq!(
        CfgExpr::TargetOs("macos".into()).apply_rules(
            &CfgExprRules {
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
                enabled_target_vendor: Some("apple".into()),
                ..Default::default()
            },
            &src,
        ),
        None
    );
    // Non-matching vendor becomes False
    assert_eq!(
        CfgExpr::TargetVendor("unknown".into()).apply_rules(
            &CfgExprRules {
                enabled_target_vendor: Some("apple".into()),
                ..Default::default()
            },
            &src,
        ),
        Some(CfgExpr::False)
    );
    assert_eq!(
        CfgExpr::TargetEnv("gnu".into()).apply_rules(
            &CfgExprRules {
                enabled_target_env: Some("gnu".into()),
                ..Default::default()
            },
            &src,
        ),
        None
    );
    // Non-matching env becomes False
    assert_eq!(
        CfgExpr::TargetEnv("msvc".into()).apply_rules(
            &CfgExprRules {
                enabled_target_env: Some("gnu".into()),
                ..Default::default()
            },
            &src,
        ),
        Some(CfgExpr::False)
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
    // Test enabled feature - should be removed (None = always true)
    let expr = CfgExpr::Feature("feature1".to_string());
    assert_eq!(
        expr.apply_rules(
            &CfgExprRules {
                enabled_features: vec!["feature1".to_string()].into_iter().collect(),
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
                disabled_features: vec!["feature2".to_string()].into_iter().collect(),
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
                feature_mappings: vec![("old_feature".to_string(), "new_feature".to_string())]
                    .into_iter()
                    .collect(),
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
                enabled_features: vec!["feature1".to_string()].into_iter().collect(),
                disabled_features: vec!["feature2".to_string()].into_iter().collect(),
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
                enabled_features: vec!["feature1".to_string()].into_iter().collect(),
                disabled_features: vec!["feature2".to_string()].into_iter().collect(),
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
                disabled_features: vec!["feature2".to_string()].into_iter().collect(),
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
                enabled_features: vec!["feature1".to_string()].into_iter().collect(),
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
    // Test unmapped feature - should panic
    let expr = CfgExpr::Feature("unknown".to_string());
    expr.apply_rules(&CfgExprRules::default(), &SourceLocation::default());
}

#[test]
#[should_panic(expected = "unmapped feature: unknown")]
fn test_strict_feature_processing_unmapped_in_any_panic() {
    // Test unmapped feature in any() - should panic
    let expr = CfgExpr::Any(vec![
        CfgExpr::Feature("feature1".to_string()),
        CfgExpr::Feature("unknown".to_string()),
    ]);
    expr.apply_rules(
        &CfgExprRules {
            enabled_features: vec!["feature1".to_string()].into_iter().collect(),
            ..Default::default()
        },
        &SourceLocation::default(),
    );
}
