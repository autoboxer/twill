import katex from 'katex';

import { highlightCode } from '../rich-content/schema';

export function createStudyTemplateFields( card, mediaUrls ) {
  return {
    title: `<div class="twill-field twill-title">${ escapeHtml( card.conceptTitle ) }</div>`,
    prompt: richFieldHtml( card.content.prompt, 'prompt', mediaUrls ),
    answer: richFieldHtml( card.content.answer, 'answer', mediaUrls )
  };
}

function richFieldHtml( document, name, mediaUrls ) {
  const content = document.content.map( ( node ) => {
    return renderNode( node, mediaUrls );
  }).join( '' );

  return `<div class="twill-field twill-${ name }">${ content }</div>`;
}

function renderNode( node, mediaUrls ) {
  switch ( node.type ) {
  case 'paragraph':
    return `<p>${ renderChildren( node, mediaUrls ) }</p>`;

  case 'heading':
    return renderHeading( node, mediaUrls );

  case 'blockquote':
    return `<blockquote>${ renderChildren( node, mediaUrls ) }</blockquote>`;

  case 'bulletList':
    return renderBulletList( node, mediaUrls );

  case 'orderedList':
    return renderOrderedList( node, mediaUrls );

  case 'listItem':
    return `<li>${ renderChildren( node, mediaUrls ) }</li>`;

  case 'codeBlock':
    return renderCodeBlock( node );

  case 'text':
    return renderText( node );

  case 'hardBreak':
    return '<br>';

  case 'horizontalRule':
    return '<hr>';

  case 'inlineMath':
    return renderMath( node.attrs.latex, false );

  case 'blockMath':
    return renderMath( node.attrs.latex, true );

  case 'mediaImage':
    return renderMedia( node, mediaUrls );

  default:
    throw new TypeError( `Unsupported rich-content node: ${ node.type }` );
  }
}

function renderChildren( node, mediaUrls ) {
  return ( node.content ?? []).map( ( child ) => {
    return renderNode( child, mediaUrls );
  }).join( '' );
}

function renderHeading( node, mediaUrls ) {
  const level = Math.min( Math.max( node.attrs.level, 1 ), 3 );

  return `<h${ level }>${ renderChildren( node, mediaUrls ) }</h${ level }>`;
}

function renderOrderedList( node, mediaUrls ) {
  const start = node.attrs?.start;
  const startAttribute = Number.isInteger( start ) && start !== 1
    ? ` start="${ start }"`
    : '';
  const typeAttribute = listTypeAttribute( node );

  return `<ol${ startAttribute }${ typeAttribute }>${ renderChildren( node, mediaUrls ) }</ol>`;
}

function renderBulletList( node, mediaUrls ) {
  return `<ul${ listTypeAttribute( node ) }>${ renderChildren( node, mediaUrls ) }</ul>`;
}

function listTypeAttribute( node ) {
  return node.attrs?.type
    ? ` type="${ escapeHtml( node.attrs.type ) }"`
    : '';
}

function renderCodeBlock( node ) {
  const language = node.attrs?.language;
  const languageClass = language
    ? ` class="language-${ escapeHtml( language ) }"`
    : '';
  const code = ( node.content ?? []).map( ( child ) => child.text ).join( '' );
  const content = highlightCode( code, language )
    .map( renderHighlightedNode )
    .join( '' );

  return `<pre><code${ languageClass }>${ content }</code></pre>`;
}

function renderHighlightedNode( node ) {
  if ( node.type === 'text' ) {
    return escapeHtml( node.value );
  }

  const classes = ( node.properties?.className ?? [])
    .map( escapeHtml )
    .join( ' ' );
  const classAttribute = classes ? ` class="${ classes }"` : '';
  const content = ( node.children ?? []).map( renderHighlightedNode ).join( '' );

  return `<span${ classAttribute }>${ content }</span>`;
}

function renderText( node ) {
  return ( node.marks ?? []).reduce( ( content, mark ) => {
    return renderMark( mark, content );
  }, escapeHtml( node.text ) );
}

function renderMark( mark, content ) {
  switch ( mark.type ) {
  case 'bold':
    return `<strong>${ content }</strong>`;

  case 'italic':
    return `<em>${ content }</em>`;

  case 'underline':
    return `<u>${ content }</u>`;

  case 'strike':
    return `<s>${ content }</s>`;

  case 'code':
    return `<code>${ content }</code>`;

  case 'link':
    return renderLink( mark, content );

  default:
    throw new TypeError( `Unsupported rich-content mark: ${ mark.type }` );
  }
}

function renderLink( mark, content ) {
  const title = mark.attrs?.title
    ? ` title="${ escapeHtml( mark.attrs.title ) }"`
    : '';

  return `<a aria-disabled="true"${ title }>${ content }</a>`;
}

function renderMath( latex, displayMode ) {
  const html = katex.renderToString( latex, {
    displayMode,
    maxExpand: 1_000,
    maxSize: 20,
    output: 'mathml',
    strict: 'warn',
    throwOnError: false,
    trust: false
  });
  const className = displayMode
    ? 'twill-math twill-math--block'
    : 'twill-math';

  return `<span class="${ className }">${ html }</span>`;
}

function renderMedia( node, mediaUrls ) {
  const mediaUrl = mediaUrls.get( node.attrs.mediaId );

  if ( !mediaUrl ) {
    return [
      '<figure class="twill-media-image twill-media-image--error">',
      'Image could not be loaded.',
      '</figure>'
    ].join( '' );
  }

  const alt = escapeHtml( node.attrs.alt ?? '' );
  const title = node.attrs.title
    ? ` title="${ escapeHtml( node.attrs.title ) }"`
    : '';

  return [
    '<figure class="twill-media-image">',
    `<img src="${ mediaUrl }" alt="${ alt }"${ title }>`,
    '</figure>'
  ].join( '' );
}

function escapeHtml( value ) {
  return String( value ).replace( /[&<>"']/g, ( character ) => ({
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#039;'
  })[ character ]);
}
