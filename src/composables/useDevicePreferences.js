import { invoke } from '@tauri-apps/api/core';

export function useDevicePreferences() {
  return {
    getDevicePreferences: () => invoke( 'get_device_preferences' ),
    setGradingMode: ( gradingMode ) => invoke( 'set_grading_mode', {
      input: { gradingMode }
    }),
    setPretestingEnabled: ( enabled ) => invoke( 'set_pretesting_enabled', {
      input: { enabled }
    }),
    setStartupDestination: ( startupDestination ) => invoke(
      'set_startup_destination',
      { input: { startupDestination } }
    ),
    setAppearancePreferences: ( appearance ) => invoke(
      'set_appearance_preferences',
      { input: { appearance } }
    )
  };
}
