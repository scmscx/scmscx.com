use crate::{
    chk::{
        chk_mbrf::{ChkMbrfAction, ChkMbrfCondition, ChkMbrfIndividual},
        chk_trig::{ChkTrigAction, ChkTrigCondition, ChkTrigIndividual},
    },
    test::get_chk,
    ParsedChk,
};

#[tokio::test]
async fn test_specific_map_files_for_known_values() {
    let chk = get_chk("13e328f170aa38af1e6cd8f3245734cf18cdd815547acd351f54778f828aef54")
        .await
        .unwrap();
    let parsed_chk = ParsedChk::from_bytes(chk.as_slice());

    if let Ok(x) = parsed_chk.ver {
        assert_eq!(*x.file_format_version, 59);
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.iver {
        assert_eq!(*x.additional_file_format_version.unwrap(), 10);
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.vcod {
        assert_eq!(
            *x.hash,
            [1, 4, 5, 6, 2, 1, 5, 2, 0, 3, 7, 7, 5, 4, 6, 3],
            "{x:?}"
        );
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.iown {
        assert_eq!(
            x.player_owner,
            [6, 5, 3, 6, 7, 6, 6, 6, 0, 0, 0, 0],
            "{x:?}"
        );
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.ownr {
        assert_eq!(
            x.player_owner,
            [6, 5, 3, 6, 7, 6, 6, 0, 0, 0, 0, 0],
            "{x:?}"
        );
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.side {
        assert_eq!(x.player_side, [5, 5, 1, 5, 5, 5, 5, 5, 7, 7, 7, 7], "{x:?}");
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.era {
        assert_eq!(*x.tileset, 1, "{x:?}");
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.dim {
        assert_eq!(*x.width, 128, "{x:?}");
        assert_eq!(*x.height, 128, "{x:?}");
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.mtxm {
        assert_eq!(x.data.len(), 128 * 128, "{x:?}");
        assert_eq!(x.data[(126) + 128], 16, "{x:?}"); // 1 pink creep tile
        assert_eq!(x.data[(124) + 128 * (2)], 16385, "{x:?}"); // 2x2 doodad
        assert_eq!(x.data[(124) + 128 * (3)], 16401, "{x:?}"); // 2x2 doodad
        assert_eq!(x.data[(123) + 128 * (2)], 16384, "{x:?}"); // 2x2 doodad
        assert_eq!(x.data[(123) + 128 * (3)], 16400, "{x:?}"); // 2x2 doodad
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.tile {
        assert_eq!(x.data.len(), 128 * 128, "{x:?}");
        assert_eq!(x.data[(126) + 128], 16, "{x:?}"); // 1 pink creep tile
        assert_eq!(x.data[(124) + 128 * (2)], 64, "{x:?}"); // 2x2 doodad is not in TILE, so regular space platform
        assert_eq!(x.data[(124) + 128 * (3)], 67, "{x:?}"); // 2x2 doodad is not in TILE, so regular space platform
        assert_eq!(x.data[(123) + 128 * (2)], 81, "{x:?}"); // 2x2 doodad is not in TILE, so regular space platform
        assert_eq!(x.data[(123) + 128 * (3)], 83, "{x:?}"); // 2x2 doodad is not in TILE, so regular space platform
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.isom {
        x.data.iter().for_each(|&y| assert_eq!(y, 32, "{x:?}"));
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.unit {
        assert_eq!(x.units.len(), 8, "{x:?}");
        assert_eq!(
            x.units[0],
            crate::chk::chk_unit::ChkUnitIndividual {
                class_instance: 1,
                x: 3872,
                y: 162,
                unit_id: 0,
                type_of_relation_to_other_building: 0,
                properties_that_can_be_applied: 24,
                properties_that_can_be_changed: 3,
                owner: 0,
                hit_points_percent: 74,
                shield_points_percent: 100,
                energy_points_percent: 100,
                resource_amount: 0,
                number_of_units_in_hangar: 0,
                unit_state_flags: 24,
                unused: 0,
                class_instance_related_to: 0,
            },
            "{x:?}"
        );
        assert_eq!(
            x.units[1],
            crate::chk::chk_unit::ChkUnitIndividual {
                class_instance: 2,
                x: 3744,
                y: 80,
                unit_id: 214,
                type_of_relation_to_other_building: 0,
                properties_that_can_be_applied: 24,
                properties_that_can_be_changed: 3,
                owner: 2,
                hit_points_percent: 100,
                shield_points_percent: 100,
                energy_points_percent: 100,
                resource_amount: 0,
                number_of_units_in_hangar: 0,
                unit_state_flags: 0,
                unused: 0,
                class_instance_related_to: 0,
            },
            "{x:?}"
        );
        assert_eq!(
            x.units[2],
            crate::chk::chk_unit::ChkUnitIndividual {
                class_instance: 3,
                x: 3680,
                y: 80,
                unit_id: 214,
                type_of_relation_to_other_building: 0,
                properties_that_can_be_applied: 24,
                properties_that_can_be_changed: 3,
                owner: 0,
                hit_points_percent: 100,
                shield_points_percent: 100,
                energy_points_percent: 100,
                resource_amount: 0,
                number_of_units_in_hangar: 0,
                unit_state_flags: 0,
                unused: 0,
                class_instance_related_to: 0,
            },
            "{x:?}"
        );
        assert_eq!(
            x.units[3],
            crate::chk::chk_unit::ChkUnitIndividual {
                class_instance: 4,
                x: 3712,
                y: 80,
                unit_id: 214,
                type_of_relation_to_other_building: 0,
                properties_that_can_be_applied: 24,
                properties_that_can_be_changed: 3,
                owner: 1,
                hit_points_percent: 100,
                shield_points_percent: 100,
                energy_points_percent: 100,
                resource_amount: 0,
                number_of_units_in_hangar: 0,
                unit_state_flags: 0,
                unused: 0,
                class_instance_related_to: 0,
            },
            "{x:?}"
        );
        assert_eq!(
            x.units[4],
            crate::chk::chk_unit::ChkUnitIndividual {
                class_instance: 5,
                x: 3808,
                y: 80,
                unit_id: 214,
                type_of_relation_to_other_building: 0,
                properties_that_can_be_applied: 24,
                properties_that_can_be_changed: 3,
                owner: 3,
                hit_points_percent: 100,
                shield_points_percent: 100,
                energy_points_percent: 100,
                resource_amount: 0,
                number_of_units_in_hangar: 0,
                unit_state_flags: 0,
                unused: 0,
                class_instance_related_to: 0,
            },
            "{x:?}"
        );
        assert_eq!(
            x.units[5],
            crate::chk::chk_unit::ChkUnitIndividual {
                class_instance: 6,
                x: 3840,
                y: 80,
                unit_id: 214,
                type_of_relation_to_other_building: 0,
                properties_that_can_be_applied: 24,
                properties_that_can_be_changed: 3,
                owner: 4,
                hit_points_percent: 100,
                shield_points_percent: 100,
                energy_points_percent: 100,
                resource_amount: 0,
                number_of_units_in_hangar: 0,
                unit_state_flags: 0,
                unused: 0,
                class_instance_related_to: 0,
            },
            "{x:?}"
        );
        assert_eq!(
            x.units[6],
            crate::chk::chk_unit::ChkUnitIndividual {
                class_instance: 7,
                x: 3872,
                y: 80,
                unit_id: 214,
                type_of_relation_to_other_building: 0,
                properties_that_can_be_applied: 24,
                properties_that_can_be_changed: 3,
                owner: 5,
                hit_points_percent: 100,
                shield_points_percent: 100,
                energy_points_percent: 100,
                resource_amount: 0,
                number_of_units_in_hangar: 0,
                unit_state_flags: 0,
                unused: 0,
                class_instance_related_to: 0,
            },
            "{x:?}"
        );
        assert_eq!(
            x.units[7],
            crate::chk::chk_unit::ChkUnitIndividual {
                class_instance: 8,
                x: 3872,
                y: 80,
                unit_id: 214,
                type_of_relation_to_other_building: 0,
                properties_that_can_be_applied: 24,
                properties_that_can_be_changed: 3,
                owner: 6,
                hit_points_percent: 100,
                shield_points_percent: 100,
                energy_points_percent: 100,
                resource_amount: 0,
                number_of_units_in_hangar: 0,
                unit_state_flags: 0,
                unused: 0,
                class_instance_related_to: 0,
            },
            "{x:?}"
        );
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.puni {
        for i in 0..228 {
            // Global Availability
            assert_eq!(
                x.unit_global_availability[i],
                if i == 0 { 0 } else { 1 },
                "{i} {x:?}"
            );

            // Player defaults
            assert_eq!(
                x.unit_player_uses_defaults[0][i],
                if i == 0 { 1 } else { 1 },
                "{i} {x:?}"
            );
            assert_eq!(
                x.unit_player_uses_defaults[1][i],
                if i == 0 { 0 } else { 1 },
                "{i} {x:?}"
            );
            assert_eq!(
                x.unit_player_uses_defaults[2][i],
                if i == 0 { 0 } else { 1 },
                "{i} {x:?}"
            );
            assert_eq!(
                x.unit_player_uses_defaults[3][i],
                if i == 0 { 1 } else { 1 },
                "{i} {x:?}"
            );

            // Player availability
            assert_eq!(
                x.unit_player_availability[0][i],
                if i == 0 { 1 } else { 1 },
                "{i} {x:?}"
            );
            assert_eq!(
                x.unit_player_availability[1][i],
                if i == 0 { 1 } else { 1 },
                "{i} {x:?}"
            );
            assert_eq!(
                x.unit_player_availability[2][i],
                if i == 0 { 0 } else { 1 },
                "{i} {x:?}"
            );
            assert_eq!(
                x.unit_player_availability[3][i],
                if i == 0 { 1 } else { 1 },
                "{i} {x:?}"
            );
        }
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.unis {
        assert_eq!(x.armor_points[0], 39, "{x:?}");
        assert_eq!(x.base_weapon_damage[0], 77, "{x:?}");
        assert_eq!(x.build_time[0], 519, "{x:?}");
        assert_eq!(x.config[0], 0, "{x:?}");
        assert_eq!(x.gas_cost[0], 812, "{x:?}");
        assert_eq!(x.hit_points[0], 536849, "{x:?}");
        assert_eq!(x.shield_points[0], 291, "{x:?}");
        assert_eq!(x.string_number[0], 8, "{x:?}");
        assert_eq!(x.upgrade_bonus_weapon_damage[0], 15, "{x:?}");
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.upgr {
        assert_eq!(
            *x.global_default_maximum_upgrade_level,
            [
                3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 14, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
            ],
            "{x:?}"
        );
        assert_eq!(
            *x.global_default_starting_upgrade_level,
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ],
            "{x:?}"
        );
        assert_eq!(
            *x.player_uses_upgrade_defaults,
            [
                [
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
                ],
                [
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
                ],
                [
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
                ],
                [
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
                ],
                [
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
                ],
                [
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
                ],
                [
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
                ],
                [
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
                ],
                [
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
                ],
                [
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
                ],
                [
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
                ],
                [
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
                ]
            ],
            "{x:?}"
        );
        assert_eq!(
            *x.starting_upgrade_level,
            [
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ],
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ],
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ],
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ],
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ],
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ],
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ],
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ],
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ],
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ],
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ],
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ]
            ],
            "{x:?}"
        );
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.upgs {
        assert_eq!(
            *x.base_gas_cost,
            [
                100, 100, 150, 150, 150, 100, 150, 100, 100, 100, 100, 100, 100, 100, 100, 200,
                299, 100, 200, 150, 100, 150, 200, 150, 200, 150, 150, 100, 200, 150, 150, 150,
                150, 150, 150, 200, 200, 200, 150, 150, 150, 100, 200, 100, 150, 0
            ],
            "{x:?}"
        );
        assert_eq!(
            *x.base_mineral_cost,
            [
                100, 100, 150, 150, 150, 100, 150, 100, 100, 100, 100, 100, 100, 100, 100, 200, 77,
                100, 200, 150, 100, 150, 200, 150, 200, 150, 150, 100, 200, 150, 150, 150, 150,
                150, 150, 200, 200, 200, 150, 150, 150, 100, 200, 100, 150, 0
            ],
            "{x:?}"
        );
        assert_eq!(
            *x.base_time,
            [
                3990, 3990, 3990, 3990, 3990, 3990, 3990, 3990, 3990, 3990, 3990, 3990, 3990, 3990,
                3990, 3990, 19620, 1500, 2490, 2490, 2490, 2490, 2490, 2490, 2400, 1995, 1995,
                1500, 1500, 1500, 1500, 2490, 2490, 2490, 1995, 2490, 2490, 2490, 1995, 1995, 2490,
                2490, 2490, 1500, 2490, 0
            ],
            "{x:?}"
        );
        assert_eq!(
            *x.gas_cost_factor,
            [
                75, 75, 75, 75, 75, 75, 75, 75, 75, 50, 50, 50, 75, 50, 75, 100, 15, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ],
            "{x:?}"
        );
        assert_eq!(
            *x.mineral_cost_factor,
            [
                75, 75, 75, 75, 75, 75, 75, 75, 75, 50, 50, 50, 75, 50, 75, 100, 13, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ],
            "{x:?}"
        );
        assert_eq!(
            *x.time_factor,
            [
                480, 480, 480, 480, 480, 480, 480, 480, 480, 480, 480, 480, 480, 480, 480, 480,
                285, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0
            ],
            "{x:?}"
        );
        assert_eq!(
            *x.upgrade_uses_default_setings,
            [
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
            ],
            "{x:?}"
        );
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.dd2 {
        assert_eq!(x.doodads.len(), 1, "{x:?}");
        assert_eq!(
            x.doodads[0],
            crate::chk::chk_dd2::ChkDd2Individual {
                doodad_number: 0,
                x: 3968,
                y: 96,
                owner: 0,
                disabled: 0
            },
            "{x:?}"
        );
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.thg2 {
        assert_eq!(x.sprites.len(), 3, "{x:?}");
        assert_eq!(
            x.sprites[0],
            crate::chk::chk_thg2::ChkThg2Individual {
                sprite_number: 59,
                x: 3968,
                y: 96,
                owner: 0,
                unused: 0,
                flags: 4992,
            },
            "{x:?}"
        );
        assert_eq!(
            x.sprites[1],
            crate::chk::chk_thg2::ChkThg2Individual {
                sprite_number: 0,
                x: 3970,
                y: 163,
                owner: 0,
                unused: 0,
                flags: 8832,
            },
            "{x:?}"
        );
        assert_eq!(
            x.sprites[2],
            crate::chk::chk_thg2::ChkThg2Individual {
                sprite_number: 287,
                x: 3872,
                y: 61,
                owner: 0,
                unused: 0,
                flags: 4736,
            },
            "{x:?}"
        );
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.mask {
        for i in 0..x.fog.len() {
            assert_eq!(x.fog[i], if i == 890 { 254 } else { 255 }, "{x:?} {i}");
        }
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.mrgn {
        assert_eq!(x.locations.len(), 64, "{x:?}");

        for i in 0..x.locations.len() {
            match i {
                0 => assert_eq!(
                    x.locations[0],
                    crate::chk::chk_mrgn::ChkMrgnIndividual {
                        left: 3744,
                        top: 32,
                        right: 3808,
                        bottom: 96,
                        name_string_number: 10,
                        elevation_flags: 0
                    },
                    "{x:?}"
                ),
                63 => assert_eq!(
                    x.locations[i],
                    crate::chk::chk_mrgn::ChkMrgnIndividual {
                        left: 0,
                        top: 0,
                        right: 4096,
                        bottom: 4096,
                        name_string_number: 3,
                        elevation_flags: 0
                    },
                    "{x:?}"
                ),
                _ => assert_eq!(
                    x.locations[i],
                    crate::chk::chk_mrgn::ChkMrgnIndividual {
                        left: 0,
                        top: 0,
                        right: 0,
                        bottom: 0,
                        name_string_number: 0,
                        elevation_flags: 0
                    },
                    "{x:?}"
                ),
            };
        }
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.str {
        // offset 2050 is offset of null byte
        assert_eq!(x.string_data.len(), 2236, "{x:?}");
        // assert_eq!(x.string_offsets[0], 2051, "{x:?}");
        // assert_eq!(x.string_offsets[1], 2077, "{x:?}");
        // assert_eq!(x.string_offsets[2], 2105, "{x:?}");
        // assert_eq!(x.string_offsets[3], 2114, "{x:?}");
        // assert_eq!(x.string_offsets[4], 2127, "{x:?}");
        // assert_eq!(x.string_offsets[5], 2140, "{x:?}");
        // assert_eq!(x.string_offsets[6], 2148, "{x:?}");
        // assert_eq!(x.string_offsets[7], 2156, "{x:?}");
        // assert_eq!(x.string_offsets[8], 2168, "{x:?}");
        // assert_eq!(x.string_offsets[9], 2198, "{x:?}");
        // assert_eq!(x.string_offsets[10], 2219, "{x:?}");
        // assert_eq!(x.string_offsets[11], 2050, "{x:?}");

        assert_eq!(
            encoding_rs::WINDOWS_1252
                .decode(&x.string_data[2051..(2051 + 26)],)
                .0
                .into_owned(),
            "test case scenario string\0".to_owned(),
            "{x:?}"
        );
        assert_eq!(
            encoding_rs::WINDOWS_1252
                .decode(&x.string_data[(2051 + 26)..(2051 + 54)],)
                .0
                .into_owned(),
            "test case scenario string 3\0".to_owned(),
            "{x:?}"
        );
        assert_eq!(
            encoding_rs::WINDOWS_1252
                .decode(&x.string_data[(2051 + 63)..(2051 + 76)],)
                .0
                .into_owned(),
            "test force 1\0".to_owned(),
            "{x:?}"
        );
        assert_eq!(
            encoding_rs::WINDOWS_1252
                .decode(&x.string_data[(2051 + 76)..(2051 + 89)],)
                .0
                .into_owned(),
            "test force 2\0".to_owned(),
            "{x:?}"
        );
        assert_eq!(
            encoding_rs::WINDOWS_1252
                .decode(&x.string_data[(2051 + 89)..(2051 + 97)],)
                .0
                .into_owned(),
            "Force 3\0".to_owned(),
            "{x:?}"
        );
        assert_eq!(
            encoding_rs::WINDOWS_1252
                .decode(&x.string_data[(2051 + 97)..(2051 + 105)],)
                .0
                .into_owned(),
            "Force 4\0".to_owned(),
            "{x:?}"
        );
        assert_eq!(
            encoding_rs::WINDOWS_1252
                .decode(&x.string_data[(2051 + 105)..(2051 + 117)],)
                .0
                .into_owned(),
            "Test Case 1\0".to_owned(),
            "{x:?}"
        );
        assert_eq!(
            encoding_rs::WINDOWS_1252
                .decode(&x.string_data[(2051 + 117)..(2051 + 147)],)
                .0
                .into_owned(),
            "sound\\Zerg\\Drone\\ZDrErr00.WAV\0".to_owned(),
            "{x:?}"
        );
        assert_eq!(
            encoding_rs::WINDOWS_1252
                .decode(&x.string_data[(2051 + 147)..(2051 + 168)],)
                .0
                .into_owned(),
            "location test string\0".to_owned(),
            "{x:?}"
        );
        assert_eq!(
            encoding_rs::WINDOWS_1252
                .decode(&x.string_data[(2051 + 168)..(2051 + 185)],)
                .0
                .into_owned(),
            "test switch name\0".to_owned(),
            "{x:?}"
        );
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.sprp {
        assert_eq!(*x.scenario_name_string_number, 1, "{x:?}");
        assert_eq!(*x.description_string_number, 2, "{x:?}");
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.forc {
        assert_eq!(x.force_name, [4, 5, 6, 7], "{x:?}");
        assert_eq!(x.force_properties, [15, 9, 15, 15], "{x:?}");
        assert_eq!(x.player_forces, [0, 1, 2, 3, 0, 0, 0, 0], "{x:?}");
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.wav {
        assert_eq!(x.wav_string_number.len(), 512, "{x:?}");
        assert_eq!(x.wav_string_number[0], 9, "{x:?}");
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.ptec {
        assert_eq!(
            *x.already_researched,
            [
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
            ],
            "{x:?}"
        );
        assert_eq!(
            *x.global_already_researched_defaults,
            [0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1],
            "{x:?}"
        );
        assert_eq!(
            *x.player_availability,
            [
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
            ],
            "{x:?}"
        );
        assert_eq!(
            *x.player_uses_default,
            [
                [1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
            ],
            "{x:?}"
        );
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.tecs {
        assert_eq!(
            *x.energy_cost_to_cast,
            [
                0, 100, 100, 0, 50, 0, 100, 75, 4723, 25, 25, 0, 0, 150, 100, 150, 0, 75, 75, 75,
                100, 150, 100, 0
            ],
            "{x:?}"
        );
        assert_eq!(
            *x.gas_cost,
            [
                100, 200, 200, 100, 0, 150, 150, 200, 1256, 150, 100, 100, 100, 100, 100, 200, 100,
                100, 100, 200, 150, 150, 150, 150
            ],
            "{x:?}"
        );
        assert_eq!(
            *x.mineral_cost,
            [
                100, 200, 200, 100, 0, 150, 150, 200, 956, 150, 100, 100, 100, 100, 100, 200, 100,
                100, 100, 200, 150, 150, 150, 150
            ],
            "{x:?}"
        );
        assert_eq!(
            *x.technology_uses_default_settings,
            [1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            "{x:?}"
        );
        assert_eq!(
            *x.time,
            [
                1200, 1500, 1800, 1200, 0, 1200, 1500, 1200, 3510, 1500, 1200, 1200, 1200, 1200,
                1200, 1500, 1500, 1200, 1200, 1800, 1200, 1800, 1500, 1500
            ],
            "{x:?}"
        );
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.mbrf {
        assert_eq!(x.triggers.len(), 1, "{x:?}");
        assert_eq!(x.triggers, [ChkMbrfIndividual { conditions: [ChkMbrfCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 13, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkMbrfCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 13, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkMbrfCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 13, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkMbrfCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 13, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkMbrfCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 13, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkMbrfCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 13, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkMbrfCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 13, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkMbrfCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 13, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkMbrfCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 13, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkMbrfCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 13, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkMbrfCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 13, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkMbrfCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 13, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkMbrfCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 13, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkMbrfCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 13, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkMbrfCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 13, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkMbrfCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 13, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }], actions: [ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 9, seconds_or_milliseconds: 980, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 2, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkMbrfAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }], execution_flags: 0, executed_for_player: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], current_action: 0 }], "{x:?}");
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.trig {
        assert_eq!(x.triggers.len(), 1, "{x:?}");
        assert_eq!(x.triggers, [ChkTrigIndividual { conditions: [ChkTrigCondition { location: 1, group: 0, qualified_number: 17, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 3, resource_type_or_score_type_or_switch_number: 0, flags: 16, mask_flag: 0 }, ChkTrigCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 0, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkTrigCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 0, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkTrigCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 0, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkTrigCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 0, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkTrigCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 0, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkTrigCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 0, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkTrigCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 0, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkTrigCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 0, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkTrigCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 0, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkTrigCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 0, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkTrigCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 0, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkTrigCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 0, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkTrigCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 0, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkTrigCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 0, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }, ChkTrigCondition { location: 0, group: 0, qualified_number: 0, unit_id: 0, numeric_comparison_or_switch_state: 0, condition: 0, resource_type_or_score_type_or_switch_number: 0, flags: 0, mask_flag: 0 }], actions: [ChkTrigAction { location: 1, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 1, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 11, number_of_units_or_action_state_or_unit_order_or_number_modifier: 1, flags: 24, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }, ChkTrigAction { location: 0, string_number: 0, wav_string_number: 0, seconds_or_milliseconds: 0, first_or_only_group_or_player_affected: 0, second_group_affected_or_secondary_location_or_cuwp_number_or_number_or_ai_script_or_switch_number: 0, unit_type_or_score_type_or_resource_type_or_alliance_status: 0, action: 0, number_of_units_or_action_state_or_unit_order_or_number_modifier: 0, flags: 0, padding: 0, mask_flag: 0 }], execution_flags: 0, executed_for_player: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], current_action: 0 }], "{x:?}");
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.uprp {
        assert_eq!(x.cuwp_slots.len(), 64, "{x:?}");

        for i in 0..x.cuwp_slots.len() {
            match i {
                0 => assert_eq!(
                    x.cuwp_slots[i],
                    crate::chk::chk_uprp::ChkUprpIndividual {
                        flag_of_special_properties: 31,
                        which_elements_of_unit_data_are_valid: 63,
                        owner: 0,
                        hit_points_percent: 255,
                        shield_points_percent: 255,
                        energy_points_percent: 35,
                        resource_amount: 123,
                        number_of_units_in_hangar: 52,
                        flags: 13,
                        padding: 0
                    },
                    "{x:?} {i}"
                ),
                _ => assert_eq!(
                    x.cuwp_slots[i],
                    crate::chk::chk_uprp::ChkUprpIndividual {
                        flag_of_special_properties: 0,
                        which_elements_of_unit_data_are_valid: 63,
                        owner: 0,
                        hit_points_percent: 100,
                        shield_points_percent: 100,
                        energy_points_percent: 100,
                        resource_amount: 0,
                        number_of_units_in_hangar: 0,
                        flags: 0,
                        padding: 0
                    },
                    "{x:?} {i}"
                ),
            }
        }
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.upus {
        assert_eq!(
            *x.cuwp_slot_is_used,
            [
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0
            ],
            "{x:?}"
        );
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.swnm {
        assert_eq!(
            *x.switch_name_string_number,
            [
                11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0
            ],
            "{x:?}"
        );
    } else {
        unreachable!();
    }

    if let Ok(x) = parsed_chk.crgb {
        assert_eq!(x.player_color_option, [3, 2, 3, 3, 3, 3, 3, 3], "{x:?}");
        assert_eq!(
            x.rgb,
            [
                [0, 0, 15],
                [91, 46, 226],
                [0, 0, 2],
                [0, 0, 3],
                [0, 0, 4],
                [0, 0, 5],
                [0, 0, 6],
                [0, 0, 7]
            ],
            "{x:?}"
        );
    } else {
        unreachable!();
    }
}
