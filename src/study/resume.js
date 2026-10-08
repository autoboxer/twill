let resumableSession = null;

export function preserveStudySession( session ) {
  resumableSession = session;
}

export function markStudyConceptChanged( conceptId ) {
  if ( !resumableSession || ( !resumableSession.recall.cards.some( ( card ) => card.conceptId === conceptId )
    && !resumableSession.recall.deferredCards?.some( ( item ) => item.card.conceptId === conceptId )
    && resumableSession.linkedPractice?.active?.card.conceptId !== conceptId ) ) {
    return;
  }

  const changedConceptIds = new Set(
    resumableSession.changedConceptIds ?? []
  );

  changedConceptIds.add( conceptId );
  resumableSession.changedConceptIds = [ ...changedConceptIds ];
}

export function markStudyTemplateChanged( templateId ) {
  if ( !resumableSession ) {
    return;
  }

  const changedConceptIds = new Set( resumableSession.changedConceptIds ?? []);

  const cards = [
    ...resumableSession.recall.cards,
    ...( resumableSession.recall.deferredCards ?? []).map( ( item ) => item.card )
  ];

  for ( const card of cards ) {
    if ( card.templateId === templateId ) {
      changedConceptIds.add( card.conceptId );
    }
  }

  const practiceCard = resumableSession.linkedPractice?.active?.card;

  if ( practiceCard?.templateId === templateId ) {
    changedConceptIds.add( practiceCard.conceptId );
  }

  resumableSession.changedConceptIds = [ ...changedConceptIds ];
}

export function takeStudySession() {
  const session = resumableSession;

  resumableSession = null;

  return session;
}
