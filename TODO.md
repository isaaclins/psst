# TODO: Provide multi-account support allowing quick switching between Spotify profiles with isolated preferences and caches by building a single settings view that manages profiles, handles per-account credential vaulting, maps dedicated cache directories, and triggers a session handoff hook for player state.

# TODO: Offer advanced equalizer presets and custom EQ curve editor tied into the playback pipeline by extending the DSP graph, shipping a starter preset library, letting users save/load sound profiles from the settings view, persisting custom curves, and surfacing them in both CLI and GUI.

# TODO: Add social presence sharing (e.g., Discord Rich Presence, macOS Now Playing) with opt-in privacy controls. (see https://github.com/jpochyla/psst/pull/605)

# TODO: Support custom themes with the given theme editor under the Appearance settings view and implement export/import functionality for sharing color palettes and typography by defining a theming schema, adding live preview tooling (including font pickers limited to installed fonts), wiring persistence, and wiring export/import buttons that export the theme config to a user-chosen location OR load from a user-selected file.

# ✅ COMPLETED: Added comprehensive test suite with 70+ tests covering edge cases, error handling, unit tests, and integration tests. Tests can be run from ./scripts/run-tests.sh and are integrated into CI gating. See TESTING.md for details.

