import {
  cloneTemplateContent,
  createDefaultTemplateContent
} from '../templates/defaults';

export function createTemplateEditorState( template = null ) {
  return {
    content: cloneTemplateContent( template?.content ),
    name: template?.name ?? ''
  };
}

export function cloneTemplateEditorState( state ) {
  if ( !state || typeof state !== 'object' || Array.isArray( state ) ) {
    return createTemplateEditorState();
  }

  return {
    content: state.content?.schemaVersion === createDefaultTemplateContent().schemaVersion
      ? cloneTemplateContent( state.content )
      : createDefaultTemplateContent(),
    name: typeof state.name === 'string' ? state.name : ''
  };
}

export function templateEditorStateKey( state ) {
  return JSON.stringify( cloneTemplateEditorState( state ) );
}
