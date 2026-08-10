import { invoke } from '@tauri-apps/api/core';

export function useStudyPreferences() {
  return {
    getStudyPreferences: () => invoke( 'get_study_preferences' ),
    setGradingMode: ( gradingMode ) => invoke( 'set_grading_mode', {
      input: { gradingMode }
    })
  };
}
