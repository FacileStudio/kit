# kit: Parity

Each test of the Go `facile` installer and the Rust test in kit that carries it over.

The Go list is every `func Test` in `FacileStudio/facile` at `origin/main` commit `c581629`,
from `git grep -h "^func Test" origin/main -- '*_test.go'`: 93 tests. Each appears exactly once
below. The Rust names come from `cargo test --workspace -- --list`. A name in the `kit` crate
lives in `crates/kit/src`; a name in `kit-core` lives in `crates/kit-core/src`.

| Status | Go tests |
|---|---|
| Ported | 66 |
| Out of scope: login | 27 |
| Not ported | 0 |

Login stays in `facile`: `facile login` and `facile logout` still serve porte's device
exchange, so `internal/authflow` and `internal/credstore` have no Rust counterpart.

## `cmd/doctor_sources_test.go`

| Go test | Rust test |
|---|---|
| `TestCheckSourceShadowing_DetectsCollision` | kit-core `catalog::tests::doctor_sources::check_source_shadowing_detects_collision` |
| `TestCheckSourceShadowing_NoFalsePositive` | kit-core `catalog::tests::doctor_sources::check_source_shadowing_no_false_positive` |
| `TestCheckSourceShadowing_CaseInsensitive` | kit-core `catalog::tests::doctor_sources::check_source_shadowing_case_insensitive` |

## `cmd/doctor_test.go`

| Go test | Rust test |
|---|---|
| `TestRealPathSeesThroughASymlinkedDirectory` | kit-core `facile::tests::version::real_path_sees_through_a_symlinked_directory` |

## `cmd/list_test.go`

| Go test | Rust test |
|---|---|
| `TestStateOf` | kit `cmd::tests::state_of_reads_like_the_listing` |

## `cmd/login_test.go`

| Go test | Rust test |
|---|---|
| `TestOrderPutsBrowserFlowsFirst` | out of scope: login |
| `TestTokenPageIsBuiltFromTheResolvedServer` | out of scope: login |

## `cmd/root_test.go`

| Go test | Rust test |
|---|---|
| `TestSplitNames` | kit `cmd::tests::split_names_splits_commas_and_whitespace` |

## `cmd/self_test.go`

| Go test | Rust test |
|---|---|
| `TestFromHomebrew` | kit-core `facile::tests::version::from_homebrew_matches_install_paths_only` |
| `TestOutdated` | kit-core `facile::tests::version::outdated_is_an_ordering_never_an_inequality` |
| `TestSplitSelf` | kit `cmd::tests::split_self_takes_kit_out_of_an_update` |
| `TestUnknownToolAnswersForFacile` | kit `cmd::tests::unknown_tool_answers_for_kit` |
| `TestRootVersionInheritedFromRoot` | kit `cmd::tests::root_version_is_inherited_by_every_subcommand` |

## `cmd/update_test.go`

| Go test | Rust test |
|---|---|
| `TestUpToDateSkipsOnlyAnExactVersionMatch` | kit-core `facile::tests::update::up_to_date_skips_only_an_exact_version_match` |
| `TestUpToDateChecksSourceOnlyToolsAgainstTheLatestTag` | kit-core `facile::tests::update::up_to_date_checks_source_only_tools_against_the_latest_tag` |
| `TestStaleKeepsTheOutdatedToolsInCatalogOrder` | kit-core `facile::tests::update::stale_keeps_the_outdated_tools_in_catalog_order` |
| `TestUpToDateWithVersionPattern` | kit-core `facile::tests::update::up_to_date_with_version_pattern` |
| `TestUpToDateWithVersionCmd` | kit-core `facile::tests::update_more::up_to_date_with_version_cmd` |
| `TestStaleKeepsAnInstalledToolWhoseLatestTagIsUnreadable` | kit-core `facile::tests::update_more::stale_keeps_an_installed_tool_whose_latest_tag_is_unreadable` |

## `internal/authflow/authflow_test.go`

| Go test | Rust test |
|---|---|
| `TestAwaitCallback` | out of scope: login |
| `TestAwaitCallbackIgnoresStrayRequests` | out of scope: login |
| `TestNonceIsHexAndUnique` | out of scope: login |
| `TestCookieScraping` | out of scope: login |
| `TestStartURLCarriesPortStateAndExtras` | out of scope: login |

## `internal/authflow/choose_test.go`

| Go test | Rust test |
|---|---|
| `TestChooseFlow` | out of scope: login |
| `TestCatalogFlowsAreComplete` | out of scope: login |

## `internal/authflow/devicegrant_test.go`

| Go test | Rust test |
|---|---|
| `TestDeviceGrantPollsUntilApproved` | out of scope: login |
| `TestDeviceGrantBacksOffOnSlowDown` | out of scope: login |
| `TestDeviceGrantReportsEachRefusalDifferently` | out of scope: login |
| `TestDeviceGrantStopsWhenTheCodeExpires` | out of scope: login |

## `internal/authflow/oidcdevice_test.go`

| Go test | Rust test |
|---|---|
| `TestDiscoveryDecidesWhetherTheGrantRuns` | out of scope: login |
| `TestDeviceSignInStoresTheToolsOwnToken` | out of scope: login |
| `TestDeviceSignInSaysSoWhenTheServerHasNoExchangeYet` | out of scope: login |
| `TestAnUnregisteredClientIsReportedInTheProvidersWords` | out of scope: login |
| `TestAnUnusableAuthorizationIsRefused` | out of scope: login |

## `internal/authflow/server_test.go`

| Go test | Rust test |
|---|---|
| `TestResolveServerPrecedence` | out of scope: login |
| `TestResolveServerFallsBackToTheCatalogDefault` | out of scope: login |

## `internal/credstore/credstore_test.go`

| Go test | Rust test |
|---|---|
| `TestWritePreservesUnknownKeys` | out of scope: login |
| `TestWriteCreatesFileAtItsTargetMode` | out of scope: login |
| `TestWriteTightensAnAlreadyLooseFile` | out of scope: login |
| `TestWriteStoresExtraFields` | out of scope: login |
| `TestClearKeepsTheServerURL` | out of scope: login |

## `internal/credstore/expand_test.go`

| Go test | Rust test |
|---|---|
| `TestExpand` | out of scope: login |
| `TestExpandUserConfigIsPlatformNative` | out of scope: login |

## `internal/credstore/keychain_test.go`

| Go test | Rust test |
|---|---|
| `TestAKeychainlessBoxKeepsTheTokenAndSaysWhere` | out of scope: login |
| `TestAKeychainlessBoxStillRecordsTheServer` | out of scope: login |

## `internal/installer/installer_test.go`

| Go test | Rust test |
|---|---|
| `TestVerifyChecksumRejectsATamperedArchive` | kit-core `facile::tests::installer::verify_checksum_rejects_a_tampered_archive` |
| `TestBuildRefusesASourceFallbackAfterAnIntegrityFailure` | kit-core `facile::tests::build_fallback::build_refuses_a_source_fallback_after_an_integrity_failure` |
| `TestBuildStillFallsBackWhenAReleaseIsMerelyUnreachable` | kit-core `facile::tests::build_fallback::build_still_falls_back_when_a_release_is_merely_unreachable` |
| `TestExtractFindsTheBinaryAndIgnoresTheRest` | kit-core `facile::tests::installer::extract_finds_the_binary_and_ignores_the_rest` |
| `TestAtomicInstallLeavesNoStagedFileBehind` | kit-core `facile::tests::installer::atomic_install_leaves_no_staged_file_behind` |
| `TestRunKeepsTheCompilerErrorsAndTheSummary` | kit-core `facile::tests::installer::run_keeps_the_compiler_errors_and_the_summary` |
| `TestLastLinesKeepsTheTailOfABuildLog` | kit-core `facile::tests::installer::last_lines_keeps_the_tail_of_a_build_log` |

## `internal/installer/latest_test.go`

| Go test | Rust test |
|---|---|
| `TestLatestResolvesTagsConcurrently` | kit-core `facile::tests::release::latest_resolves_tags_concurrently` |
| `TestLatestHandlesEmptyRepoList` | kit-core `facile::tests::release::latest_handles_an_empty_repo_list` |

## `internal/installer/release_test.go`

| Go test | Rust test |
|---|---|
| `TestReleaseAssetFallsBackToTheAPIOnServerErrors` | kit-core `facile::tests::release::release_asset_falls_back_to_the_api_on_server_errors` |
| `TestReleaseAssetDoesNotRetryAMissingAsset` | kit-core `facile::tests::release::release_asset_does_not_retry_a_missing_asset` |
| `TestLatestTagFallsBackToTheAPI` | kit-core `facile::tests::release::latest_tag_falls_back_to_the_api` |
| `TestIgnoreRepoMiseAddsTheCheckout` | kit-core `facile::tests::release::ignore_repo_mise_adds_the_checkout` |
| `TestAPIHeadersSendTheTokenOnlyWhenPresent` | kit-core `facile::tests::release::api_headers_send_the_token_only_when_present` |

## `internal/installer/skill_test.go`

| Go test | Rust test |
|---|---|
| `TestRegisterSkillWritesThePiTarget` | kit-core `facile::tests::skill::register_skill_writes_the_pi_target` |
| `TestRegisterSkillFallsBackToLegacyPiTarget` | kit-core `facile::tests::skill::register_skill_falls_back_to_legacy_pi_target` |
| `TestNoMyceliumSkillsDirDoesNotCreateOne` | kit-core `facile::tests::skill::no_mycelium_skills_dir_does_not_create_one` |

## `internal/manifest/integration_test.go`

| Go test | Rust test |
|---|---|
| `TestIntegrationSourcesFileLoading` | kit-core `catalog::tests::integration_load::sources_file_loading`, plus `sources_path_priority` and `config_yml_with_sources_key` |
| `TestIntegrationCatalogMergingWithSingleSources` | kit-core `catalog::tests::integration_load::single_sources_shadow_base_and_append_new` |
| `TestIntegrationListSourceParsing` | kit-core `catalog::tests::integration_merge::list_source_parsing` |
| `TestIntegrationFacileTomlParsing` | kit-core `catalog::tests::integration_parse::facile_toml_parsing` |
| `TestIntegrationMiseTomlParsing` | kit-core `catalog::tests::integration_parse::mise_toml_parsing` |
| `TestIntegrationMerge` | kit-core `catalog::tests::integration_merge::merge_layers` |
| `TestIntegrationVersionPatternMatching` | kit-core `catalog::tests::integration_parse::version_pattern_matching` |
| `TestIntegrationSingleSourceToTool` | kit-core `catalog::tests::integration_parse::single_source_to_tool_fills_defaults_and_keeps_explicit_values` and `single_source_to_tool_rejects_bad_entries` |
| `TestIntegrationLoadBaseVsLoad` | kit-core `catalog::tests::integration_load::load_merges_user_sources_and_load_base_does_not` |

## `internal/manifest/manifest_test.go`

| Go test | Rust test |
|---|---|
| `TestCacheIsStaleOnceTheBinaryIsNewer` | kit-core `catalog::tests::manifest::cache_is_stale_once_the_binary_is_newer` |
| `TestFacileCatalogOverridesEverything` | kit-core `catalog::tests::manifest::kit_catalog_overrides_everything`, plus `kit_catalog_wins_over_facile_catalog` |
| `TestAMissingOverrideFallsBackToEmbedded` | kit-core `catalog::tests::manifest::a_missing_override_falls_back_to_embedded` |
| `TestEmbeddedSSOCodeFlowsAreOnThePorteContract` | kit-core `catalog::tests::manifest::embedded_sso_code_flows_are_on_the_porte_contract` |
| `TestSonarTheFirstMCPToolDeclaresTheInstallHook` | kit-core `catalog::tests::manifest::sonar_the_first_mcp_tool_declares_the_install_hook` |
| `TestOneManifestDeclaresEveryBinaryOfARepo` | kit-core `catalog::tests::manifest::one_manifest_declares_every_binary_of_a_repo` |
| `TestASingleBinStillProducesOneEntry` | kit-core `catalog::tests::manifest::a_single_bin_still_produces_one_entry` |

## `internal/manifest/sources_parse_test.go`

| Go test | Rust test |
|---|---|
| `TestParseSources_RootLevelMap` | kit-core `catalog::tests::sources_parse::root_level_map` |
| `TestParseSources_BranchPin` | kit-core `catalog::tests::sources_parse::branch_pin` |
| `TestParseSources_BareSequence` | kit-core `catalog::tests::sources_parse::bare_sequence` |
| `TestParseSources_ToolsSection` | kit-core `catalog::tests::sources_parse::tools_section` |
| `TestParseSources_ExtendedObject` | kit-core `catalog::tests::sources_parse::extended_object` |
| `TestParseSources_LegacyFormat` | kit-core `catalog::tests::sources_parse_more::legacy_format` |
| `TestParseSources_SourcesSectionMap` | kit-core `catalog::tests::sources_parse_more::sources_section_map_and_slice` (map half) |
| `TestParseSources_SourcesSectionSlice` | kit-core `catalog::tests::sources_parse_more::sources_section_map_and_slice` (slice half) |
| `TestParseSources_SourcesSectionStructured` | kit-core `catalog::tests::sources_parse_more::sources_section_structured` |
| `TestParseSources_CaseInsensitiveKeys` | kit-core `catalog::tests::sources_parse_more::case_insensitive_keys` |

## `internal/store/store_test.go`

| Go test | Rust test |
|---|---|
| `TestSourcesPaths_OrderAndFallback` | kit-core `facile::tests::store::sources_paths_order_and_fallback` |
| `TestSourcesPaths_RespectsXDG` | kit-core `facile::tests::store::sources_paths_respects_xdg` |
| `TestSourcesPath_PrefersConfigYmlWhenNoDotfile` | kit-core `facile::tests::store::sources_path_prefers_config_yml_when_no_dotfile` |
| `TestSourcesPath_FindsYamlVariant` | kit-core `facile::tests::store::sources_path_finds_yaml_variant` |

## Tests new in kit

These Rust tests have no Go counterpart. They cover the list file, the Flathub and Nix
backends, name resolution and the new commands.

| Module | Tests |
|---|---|
| `catalog::tests::discover` | `an_incomplete_single_source_reads_the_repo_facile_toml`, `an_unreachable_list_is_a_load_error`, `a_repo_with_no_config_falls_back_to_defaults_or_errors_without_a_name`, `list_sources_are_discovered_and_cached`, `live_fetch_reaches_the_catalog_on_main`, `mise_toml_is_the_fallback_on_master` |
| `catalog::tests::integration_load` | `load_keeps_a_parse_error_in_load_errors` |
| `catalog::tests::load` | `a_catalog_with_an_unknown_key_still_parses`, `a_catalog_with_no_tools_is_an_error`, `a_fetched_catalog_is_cached_and_used`, `a_fresh_cache_skips_the_network`, `a_stale_cache_beats_the_embedded_copy_when_offline`, `refresh_fails_loudly_when_offline_and_load_does_not`, `the_auth_block_survives_a_write`, `the_embedded_catalog_lists_every_suite_tool` |
| `catalog::tests::sources_parse_more` | `empty_and_broken_files` |
| `cmd::tests` | `help_lines_follow_the_wording_rules` |
| `crates/kit/tests` (end to end) | `a_bare_name_without_nix_names_kit_setup_nix`, `add_takes_comma_lists_and_dedupes`, `add_then_del_round_trips_the_list_file`, `check_exits_one_on_drift_and_zero_when_clean`, `check_reports_an_unlisted_flatpak_as_extra`, `clean_asks_first_and_refuses_without_a_terminal`, `del_of_an_unlisted_name_says_so`, `kit_refuses_to_add_or_remove_itself`, `list_json_has_one_section_per_source_with_kit_first`, `many_matches_without_a_terminal_fail_with_the_candidates`, `sync_installs_only_what_is_missing`, `sync_quiet_skips_missing_backends_with_one_hint_each`, `undo_is_nix_only`, `version_prints_one_line_both_ways` |
| `facile::tests::cache` | `doctor_reports_a_bin_dir_missing_from_path`, `doctor_reports_a_shadowed_binary_and_stray_copies`, `doctor_reports_interrupted_installs`, `latest_cache_answers_fresh_entries_without_the_network`, `latest_cache_refreshes_after_a_day_and_never_caches_a_failure` |
| `facile::tests::end_to_end` | `install_aborts_on_a_checksum_mismatch`, `install_places_verifies_and_registers_mcp`, `update_refuses_a_binary_another_manager_owns`, `update_skips_a_current_tool_and_installs_a_missing_one` |
| `facile::tests::release` | `latest_tag_follows_the_redirect_and_names_a_missing_release` |
| `facile::tests::self_update` | `installed_runs_the_binaries_and_skips_broken_ones`, `self_update_leaves_a_current_kit_and_a_source_build_alone`, `self_update_refuses_under_a_package_manager` |
| `facile::tests::skill` | `inject_block_replaces_only_the_marked_section` |
| `facile::tests::store` | `on_path_matches_whole_entries`, `tilde_shortens_the_home_directory` |
| `facile::tests::update` | `up_to_date_reinstalls_when_it_cannot_tell` |
| `facile::tests::version` | `foreign_owners_name_their_update_command`, `owner_of_names_who_placed_a_binary`, `version_of_reads_both_line_shapes` |
| `flathub::cli_tests` | `every_call_sends_the_exact_argv`, `invalid_or_empty_ids_never_run_flatpak`, `list_installed_parses_ids`, `missing_flatpak_is_not_available`, `non_zero_exit_returns_code_and_stderr_tail`, `search_sends_argv_and_parses_pairs` |
| `flathub::id_tests` | `accepts_valid_ids`, `rejects_invalid_ids` |
| `listfile::tests` | `dedupes_on_add_and_on_read`, `missing_file_copies_seed`, `rejects_unknown_key_and_version`, `round_trip_keeps_order`, `second_editor_blocks_until_lock_released`, `source_prefix_parse_and_display` |
| `nix::attr_tests` | `env_expr_is_exact`, `exists_expr_is_exact`, `normalize_sorts_dedups_and_rejects`, `parse_rev_reads_locked_then_revision`, `validate_attr_table` |
| `nix::cli_tests` | `exists_rejects_injection_without_running_nix`, `exists_runs_nix_eval_and_reads_bool`, `failure_carries_the_stderr_tail`, `missing_binary_is_not_available`, `real_nix_resolves_and_syncs`, `registry_rev_runs_flake_metadata` |
| `nix::sync_tests` | `failed_rollback_keeps_state`, `needs_resync_tracks_rev_and_attrs`, `sync_rejects_bad_attr_before_running_anything`, `sync_runs_build_then_prune_and_records`, `undo_rolls_back_and_reads_the_generation_record`, `undo_without_a_record_marks_state_stale` |
| `resolve::error_tests` | `backend_failures_propagate`, `catalog_failure_propagates`, `rejects_bad_specs` |
| `resolve::tests` | `resolves_every_case`, `serializes_for_json_output` |

`nix::cli_tests::real_nix_resolves_and_syncs` and
`catalog::tests::discover::live_fetch_reaches_the_catalog_on_main` are `#[ignore]`. CI's `nix` job
runs them.

Back to the [README](../README.md).
