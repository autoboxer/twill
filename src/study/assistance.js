import { richDocumentHasContent } from '../rich-content/schema';

export function assistanceDocuments( content ) {
  return [
    { id: 'hint', label: 'Hint', document: content?.assistance?.hint },
    { id: 'reference', label: 'Reference', document: content?.assistance?.reference }
  ].filter( ( item ) => richDocumentHasContent( item.document ) );
}
