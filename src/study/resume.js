let resumableSession = null;

export function preserveStudySession( session ) {
  resumableSession = session;
}

export function markStudyConceptChanged( conceptId ) {
  if ( !resumableSession ) {
    return;
  }

  const changedConceptIds = new Set(
    resumableSession.changedConceptIds ?? []
  );

  changedConceptIds.add( conceptId );
  resumableSession.changedConceptIds = [ ...changedConceptIds ];
}

export function takeStudySession() {
  const session = resumableSession;

  resumableSession = null;

  return session;
}
