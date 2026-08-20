const segmenter = typeof Intl.Segmenter === 'function'
  ? new Intl.Segmenter( undefined, { granularity: 'grapheme' })
  : null;

export function normalizeTypeAnswer( value ) {
  return displayTypeAnswer( value ).toLowerCase();
}

export function compareTypeAnswer( response, acceptedAnswers ) {
  const responseText = displayTypeAnswer( response );
  const normalizedResponse = normalizeTypeAnswer( response );
  const exactAnswer = acceptedAnswers.find( ( answer ) => (
    normalizeTypeAnswer( answer ) === normalizedResponse
  ) );

  if ( exactAnswer ) {
    return {
      acceptedAnswer: exactAnswer,
      acceptedSegments: [],
      exact: true,
      responseSegments: [{ different: false, text: responseText }]
    };
  }

  const acceptedAnswer = closestAcceptedAnswer( responseText, acceptedAnswers );
  const { acceptedSegments, responseSegments } = differenceSegments(
    responseText,
    acceptedAnswer
  );

  return {
    acceptedAnswer,
    acceptedSegments,
    exact: false,
    responseSegments
  };
}

function displayTypeAnswer( value ) {
  return value.normalize( 'NFC' ).trim().replace( /\s+/gu, ' ' );
}

function closestAcceptedAnswer( response, acceptedAnswers ) {
  const responseTokens = comparisonTokens( response );
  let closest = acceptedAnswers[ 0 ] ?? '';
  let closestDistance = Number.POSITIVE_INFINITY;
  let closestLengthDifference = Number.POSITIVE_INFINITY;

  for ( const answer of acceptedAnswers ) {
    const answerTokens = comparisonTokens( answer );
    const distance = editDistance( responseTokens, answerTokens );
    const lengthDifference = Math.abs( responseTokens.length - answerTokens.length );

    if (
      distance < closestDistance
      || (
        distance === closestDistance
        && lengthDifference < closestLengthDifference
      )
    ) {
      closest = answer;
      closestDistance = distance;
      closestLengthDifference = lengthDifference;
    }
  }

  return displayTypeAnswer( closest );
}

function editDistance( first, second ) {
  let previous = Array.from({ length: second.length + 1 }, ( _, index ) => index );

  for ( let firstIndex = 0; firstIndex < first.length; firstIndex += 1 ) {
    const current = [ firstIndex + 1 ];

    for ( let secondIndex = 0; secondIndex < second.length; secondIndex += 1 ) {
      const substitutionCost = first[ firstIndex ] === second[ secondIndex ] ? 0 : 1;

      current.push( Math.min(
        current[ secondIndex ] + 1,
        previous[ secondIndex + 1 ] + 1,
        previous[ secondIndex ] + substitutionCost
      ) );
    }

    previous = current;
  }

  return previous[ second.length ];
}

function differenceSegments( response, acceptedAnswer ) {
  const responseGraphemes = graphemes( response );
  const acceptedGraphemes = graphemes( acceptedAnswer );
  const responseTokens = responseGraphemes.map( comparisonToken );
  const acceptedTokens = acceptedGraphemes.map( comparisonToken );
  const commonLengths = commonSubsequenceLengths( responseTokens, acceptedTokens );
  const responseSegments = [];
  const acceptedSegments = [];
  let responseIndex = 0;
  let acceptedIndex = 0;

  while (
    responseIndex < responseGraphemes.length
    || acceptedIndex < acceptedGraphemes.length
  ) {
    if (
      responseIndex < responseGraphemes.length
      && acceptedIndex < acceptedGraphemes.length
      && responseTokens[ responseIndex ] === acceptedTokens[ acceptedIndex ]
    ) {
      appendSegment( responseSegments, responseGraphemes[ responseIndex ], false );
      appendSegment( acceptedSegments, acceptedGraphemes[ acceptedIndex ], false );
      responseIndex += 1;
      acceptedIndex += 1;
      continue;
    }

    const responseRemainder = commonLengths[ responseIndex + 1 ]?.[ acceptedIndex ] ?? -1;
    const acceptedRemainder = commonLengths[ responseIndex ]?.[ acceptedIndex + 1 ] ?? -1;

    if (
      responseIndex < responseGraphemes.length
      && (
        acceptedIndex >= acceptedGraphemes.length
        || responseRemainder >= acceptedRemainder
      )
    ) {
      appendSegment( responseSegments, responseGraphemes[ responseIndex ], true );
      responseIndex += 1;
    } else {
      appendSegment( acceptedSegments, acceptedGraphemes[ acceptedIndex ], true );
      acceptedIndex += 1;
    }
  }

  return { acceptedSegments, responseSegments };
}

function commonSubsequenceLengths( first, second ) {
  const lengths = Array.from(
    { length: first.length + 1 },
    () => new Uint16Array( second.length + 1 )
  );

  for ( let firstIndex = first.length - 1; firstIndex >= 0; firstIndex -= 1 ) {
    for ( let secondIndex = second.length - 1; secondIndex >= 0; secondIndex -= 1 ) {
      lengths[ firstIndex ][ secondIndex ] = first[ firstIndex ] === second[ secondIndex ]
        ? lengths[ firstIndex + 1 ][ secondIndex + 1 ] + 1
        : Math.max(
          lengths[ firstIndex + 1 ][ secondIndex ],
          lengths[ firstIndex ][ secondIndex + 1 ]
        );
    }
  }

  return lengths;
}

function comparisonTokens( value ) {
  return graphemes( displayTypeAnswer( value ) ).map( comparisonToken );
}

function comparisonToken( value ) {
  return value.toLowerCase();
}

function graphemes( value ) {
  if ( !segmenter ) {
    return Array.from( value );
  }

  return Array.from( segmenter.segment( value ), ( part ) => part.segment );
}

function appendSegment( segments, text, different ) {
  const previous = segments.at( -1 );

  if ( previous?.different === different ) {
    previous.text += text;
    return;
  }

  segments.push({ different, text });
}
