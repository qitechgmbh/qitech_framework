use crate::schema::CommandKind;
use crate::schema::MachineSchema;
use crate::schema::ValueDefinition;
use crate::schema::ValueType;

/// The winder_v1 schema from the design checklist.
const WINDER_V1: &str = r#"
import si

name winder_v1

config {
    spool {
        direction enum {
            forward 
            reverse
        }

        speed_controller {
            algorithm enum {
                adaptive
                min_max 
            }

            speed_min float quantity=si.angular_velocity
            speed_max float quantity=si.angular_velocity

            adaptive {
                tension_target float
                radius_learning_rate float
                max_speed_multiplier float
                acceleration_factor float
                deacceleration_urgency_multiplier float
            }
        }
    }

    puller {
        direction enum {
            forward
            reverse
        }
        gear_ratio enum {
            one_to_one
            one_to_five
            one_to_ten
        }
        speed_controller {
            algorithm enum {
                direct
                adaptive
            }
            speed_desired float quantity=si.velocity

            adaptive {
                speed_delta_max float
                increase_per_step float
                tolerance_limit float quantity=si.length
                adjustment_distance float quantity=si.length
            }
        }
    }

    traverse {
        limit_inner float quantity=si.length
        limit_outer float quantity=si.length
        step_size float quantity=si.length
        padding float quantity=si.length
        edge_dwell_revolutions float
        start_dwell_revolutions float
    }

    spool_automatic {
        required_length float quantity=si.length
        action enum {
            no_action
            pull
            hold
        }
    }

    laser_pointer {
        enabled boolean
    }
}

state {
    mode enum {
        standby
        pull
        hold
        wind
    }
    spool {
        mode enum {
            standby
            hold
            wind
        }
        speed_controller {
            enabled boolean
        }
    }

    puller {
        mode enum {
            standby
            hold
            pull
        }
    }

    traverse {
        mode enum {
            standby
            hold
            traverse
        }
        state enum {
            idle
            going_in
            going_out
            initialize
            escape_endstop
            find_endstop_fine_distancing
            find_endstop_coarse
            find_endstop_fine
            validate
            dwelling_outer
            traversing_in
            dwelling_inner
            traversing_out
        }
        homed boolean
        endstop_triggered boolean
    }

    tension_arm {
        zero float quantity=si.angle optional=#true
    }
}

measurements {
    spool {
        speed float quantity=si.angular_velocity
        progress float quantity=si.length

        speed_controller {
            speed float quantity=si.angular_velocity
            filament_tension float optional=#true

            adaptive {
                speed_factor float
            }
        }
    }

    puller {
        speed float quantity=si.velocity

        speed_controller {
            speed_target float quantity=si.velocity

            adaptive {
                modulation float
                distance_since_adjustment float quantity=si.length
            }
        }
    }

    traverse {
        position float quantity=si.length
    }

    tension_arm {
        angle float quantity=si.angle
    }
}

commands {
    mode {
        standby kind=transition
        hold kind=transition
        pull kind=transition
        wind kind=transition
    }

    spool {
        reset_progress kind=action
    }

    traverse {
        goto_home kind=action
        goto_limit_inner kind=action {
            errors {
                not_homed
            }
        }
        goto_limit_outer kind=action {
            errors {
                not_homed
            }
        }
    }

    tension_arm {
        set_zero kind=action
    }
}

events {
    spool_automatic {
        required_length_reached {
            progress float quantity=si.length
        }
    }

    traverse {
        homing_failed
    }
}

inputs {
    puller {
        speed float quantity=si.velocity

        speed_controller {
            speed_target float quantity=si.velocity

            adaptive {
                modulation float
                distance_since_adjustment float quantity=si.length
            }
        }
    }

    laser_diameter float quantity=si.length
}
"#;

fn find<'a>(values: &'a [ValueDefinition], path: &str) -> &'a ValueDefinition {
    values
        .iter()
        .find(|v| v.path == path)
        .unwrap_or_else(|| panic!("missing `{path}`"))
}

fn messages(input: &str) -> Vec<String> {
    MachineSchema::parse_str(input)
        .unwrap_err()
        .issues
        .into_iter()
        .map(|i| i.message)
        .collect()
}

fn has(messages: &[String], needle: &str) -> bool {
    messages.iter().any(|m| m.contains(needle))
}

#[test]
fn parses_winder_v1() {
    let schema = MachineSchema::parse_str(WINDER_V1).unwrap();

    assert_eq!(schema.imports, ["si"]);
    assert_eq!(schema.name, "winder_v1");

    // nested groups become dotted paths
    let tension = find(
        &schema.config,
        "spool.speed_controller.adaptive.tension_target",
    );
    assert_eq!(tension.ty, ValueType::Float { quantity: None });

    let limit = find(&schema.config, "traverse.limit_inner");
    let ValueType::Float {
        quantity: Some(quantity),
    } = &limit.ty
    else {
        panic!("expected a float with quantity, got {}", limit.ty);
    };
    assert_eq!(quantity.to_string(), "si.length");

    // enums
    let ValueType::Enum { variants } = &find(&schema.config, "puller.gear_ratio").ty else {
        panic!("expected enum");
    };
    assert_eq!(variants.len(), 3);

    let ValueType::Enum { variants } = &find(&schema.state, "traverse.state").ty else {
        panic!("expected enum");
    };
    assert_eq!(variants.len(), 13);
    assert_eq!(variants[12].name, "traversing_out");
    assert_eq!(variants[12].value, 12);

    assert!(find(&schema.state, "tension_arm.zero").optional);
    assert!(!find(&schema.measurements, "traverse.position").optional);
    find(&schema.inputs, "laser_diameter");

    // commands
    let standby = schema
        .commands
        .iter()
        .find(|c| c.path == "mode.standby")
        .unwrap();
    assert_eq!(standby.kind, CommandKind::Transition);

    let inner = schema
        .commands
        .iter()
        .find(|c| c.path == "traverse.goto_limit_inner")
        .unwrap();
    assert_eq!(inner.kind, CommandKind::Action);
    assert_eq!(inner.errors[0].name, "not_homed");

    // events: typed children make an event, untyped children make a group
    let reached = schema
        .events
        .iter()
        .find(|e| e.path == "spool_automatic.required_length_reached")
        .unwrap();
    assert_eq!(reached.fields[0].path, "progress");

    let homing = schema
        .events
        .iter()
        .find(|e| e.path == "traverse.homing_failed")
        .unwrap();
    assert!(homing.fields.is_empty());
}

#[test]
fn enforces_header_order() {
    let m = messages(
        r#"
config {
    a float
}
name winder_v1
import si
"#,
    );
    assert!(has(&m, "`name` must come before all sections"));
    assert!(has(&m, "`import` must come before `name`"));
}

#[test]
fn requires_name() {
    let m = messages("config {\n    a float\n}\n");
    assert!(has(&m, "missing `name`"));
}

#[test]
fn rejects_invalid_values() {
    let m = messages(
        r#"
name test
config {
    a double
    b float colour="red"
    c float quantity=si.length
    d enum x y { z }
    e boolean {
        nested float
    }
    f quantity=si.length
    g {
    }
    h enum {
        x value=1
        y
    }
    i float
    i boolean
}
"#,
    );
    assert!(has(&m, "unknown type for `a`"));
    assert!(has(&m, "unknown property `colour`"));
    assert!(has(&m, "`si` is not imported"));
    assert!(has(&m, "list the variants of enum `d` as children"));
    assert!(has(&m, "boolean `e` cannot have children"));
    assert!(has(&m, "`f` has properties but no type"));
    assert!(has(&m, "`g` has neither a type nor children"));
    assert!(has(&m, "either all variants of enum `h`"));
    assert!(has(&m, "duplicate `i`"));
}

#[test]
fn rejects_invalid_commands() {
    let m = messages(
        r#"
name test
commands {
    stop kind=transition {
        params {
            speed float
        }
    }
    jump kind=teleport
    group_without_kind {
    }
    home kind=action {
        options {
        }
    }
}
"#,
    );
    assert!(has(&m, "transition `stop` cannot take parameters"));
    assert!(has(&m, "must be `action` or `transition`"));
    assert!(has(&m, "`group_without_kind` is missing `kind=action`"));
    assert!(has(&m, "unknown block `options`"));
}

#[test]
fn rejects_invalid_names() {
    let m = messages("name Winder\nconfig {\n    Speed float\n}\n");
    assert!(has(&m, "`name` must be a lowercase name"));
    assert!(has(&m, "`Speed` is not a valid name"));
}
