import { richDocumentHasContent } from '../rich-content/schema';
import { reconstructionPrompts } from '../explain/prompts';

export function assistanceDocuments( content, explain = null ) {
  const documents = [
    { id: 'hint', label: 'Hint', icon: 'i-lucide-lightbulb', document: content?.assistance?.hint },
    { id: 'reference', label: 'Reference', icon: 'i-lucide-book-open', document: content?.assistance?.reference }
  ].filter( ( item ) => richDocumentHasContent( item.document ) );
  const prompts = reconstructionPrompts( explain );

  if ( prompts.length ) {
    documents.push({
      id: 'structure',
      label: 'Structure',
      icon: 'i-lucide-list-tree',
      document: {
        type: 'doc',
        content: [{
          type: 'bulletList',
          content: prompts.map( prompt => ({
            type: 'listItem',
            content: [{ type: 'paragraph', content: [{ type: 'text', text: prompt }] }]
          }) )
        }]
      }
    });
  }

  return documents;
}
