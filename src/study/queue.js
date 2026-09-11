export function resolveStudyQueue( queue ) {
  const concepts = new Map( queue.concepts.map( ( concept ) => [ concept.id, concept ]) );
  const templates = new Map( queue.templates.map( ( template ) => [ template.id, template ]) );
  const cards = queue.cards.map( ( card ) => {
    const concept = concepts.get( card.conceptId );
    const template = card.templateId ? templates.get( card.templateId ) : null;

    if ( !concept || ( card.templateId && !template ) ) {
      throw new Error( 'Study content is incomplete. Reload the study queue.' );
    }

    // Each form shares its concept and template documents throughout the active session
    return {
      ...card,
      conceptLastChangeId: concept.lastChangeId,
      conceptTitle: concept.title,
      content: concept.content,
      template
    };
  });

  return { ...queue, cards };
}
