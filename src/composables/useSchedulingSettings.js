import { invoke } from '@tauri-apps/api/core';

export function useSchedulingSettings() {
  return {
    getSchedulingSettings: () => invoke( 'get_scheduling_settings' ),
    updateSchedulingSettings: ( desiredRetention, maximumIntervalDays ) => {
      return invoke( 'update_scheduling_settings', {
        input: {
          desiredRetention,
          maximumIntervalDays
        }
      });
    }
  };
}
