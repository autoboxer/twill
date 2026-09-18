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
  const { custom, mode, schemaVersion, visual } = state.content;

  // Restored JSON can have a different property order than the saved template
  return JSON.stringify({
    name: state.name,
    schemaVersion,
    mode,
    alignment: visual.appearance.alignment,
    showFieldLabels: visual.appearance.showFieldLabels,
    front: visual.front.blocks.map( blockKey ),
    answer: visual.answer.blocks.map( blockKey ),
    frontHtml: custom.frontHtml,
    answerHtml: custom.answerHtml,
    css: custom.css
  });
}

function blockKey( block ) {
  return {
    type: block.type,
    field: block.field,
    text: block.text
  };
}
