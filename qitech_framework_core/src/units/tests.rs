use crate::units::Conversion;
use crate::units::UnitCatalog;

const EXAMPLE: &str = r#"
length {
    meter      symbol="m"  base=#true
    millimeter symbol="mm" factor=1e-3
    inch       symbol="in" factor=0.0254
}

temperature {
    kelvin  symbol="K"  base=#true
    celsius symbol="°C" offset=273.15
}

sound_pressure {
    pascal symbol="Pa" base=#true
    db_spl symbol="dB" log10_multiplier=20 log10_reference=2e-5
}

hardness_shore_a {
    shore_a symbol="Shore A"
}
"#;

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0)
}

#[test]
fn parses_example() {
    let catalog = UnitCatalog::parse_str(EXAMPLE).unwrap();
    assert_eq!(catalog.quantities.len(), 4);

    let (quantity, mm) = catalog.unit("length.millimeter").unwrap();
    assert_eq!(quantity.base().unwrap().name, "meter");
    assert_eq!(mm.symbol, "mm");
    assert!(approx(mm.conversion.to_base(1500.0).unwrap(), 1.5));

    let (_, celsius) = catalog.unit("temperature.celsius").unwrap();
    assert!(approx(celsius.conversion.to_base(20.0).unwrap(), 293.15));
    assert!(approx(celsius.conversion.from_base(273.15).unwrap(), 0.0));

    let (_, db) = catalog.unit("sound_pressure.db_spl").unwrap();
    assert!(approx(
        db.conversion.to_base(94.0).unwrap(),
        1.0023744672545445
    ));
    assert!(approx(db.conversion.from_base(2e-5).unwrap(), 0.0));

    let shore = catalog.quantity("hardness_shore_a").unwrap();
    assert!(!shore.is_convertible());
    assert_eq!(shore.units[0].conversion, Conversion::None);
}

#[test]
fn collects_all_issues() {
    let input = r#"
length {
    meter symbol="m" base=#true
    foot symbol="ft" base=#true
    inch factor=0.0254
    yard symbol="yd" factor=0.9144 log10_reference=1.0
    rod symbol="rd" colour="red"
}
length {
    meter symbol="m" base=#true
}
"#;
    let error = UnitCatalog::parse_str(input).unwrap_err();
    let messages: Vec<&str> = error.issues.iter().map(|i| i.message.as_str()).collect();

    assert!(
        messages
            .iter()
            .any(|m| m.contains("more than one base unit"))
    );
    assert!(messages.iter().any(|m| m.contains("missing `symbol`")));
    assert!(messages.iter().any(|m| m.contains("mixes")));
    assert!(
        messages
            .iter()
            .any(|m| m.contains("unknown property `colour`"))
    );
    assert!(
        messages
            .iter()
            .any(|m| m.contains("duplicate quantity `length`"))
    );
    assert_eq!(error.issues[0].line, 4);
}

#[test]
fn rejects_conversion_without_base() {
    let input = r#"
scale {
    a symbol="a"
    b symbol="b" factor=2
}
"#;
    let error = UnitCatalog::parse_str(input).unwrap_err();
    assert!(error.issues[0].message.contains("has no base unit"));
}

#[test]
fn reports_syntax_errors_with_position() {
    let error = UnitCatalog::parse_str("length {\n    meter symbol=\"m\n").unwrap_err();
    assert!(!error.issues.is_empty());
}
