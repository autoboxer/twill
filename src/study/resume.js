let resumableSession = null;

export function preserveStudySession( session ) {
  resumableSession = session;
}

export function markStudyConceptChanged( conceptId ) {
  if ( !resumableSession || !resumableSession.recall.cards.some( ( card ) => card.conceptId === conceptId ) ) {
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

  for ( const card of resumableSession.recall.cards ) {
    if ( card.templateId === templateId ) {
      changedConceptIds.add( card.conceptId );
    }
  }

  resumableSession.changedConceptIds = [ ...changedConceptIds ];
}

export function takeStudySession() {
  const session = resumableSession;

  resumableSession = null;

  return session;
}
