export const guideGroups = [
  { title: 'Getting started', topics: [ 'start', 'cards' ] },
  {
    title: 'Card types',
    topics: [ 'recall', 'type-answer', 'cloze', 'image-occlusion', 'explain', 'problem' ]
  },
  { title: 'Studying', topics: [ 'grading', 'feedback', 'sessions', 'card-quality' ] },
  { title: 'Working in Twill', topics: [ 'saving', 'shortcuts' ] }
];

export const guideTopics = [
  {
    id: 'start',
    title: 'Start studying',
    summary: 'Learn unfamiliar material, make a useful question, and return to it later.',
    sections: [
      {
        title: 'Your first concept',
        steps: [
          'Read an explanation and a worked example. Decide what you want to remember, explain, or do without looking at the source.',
          'Open Create. Write a specific question in Prompt and its expected response in Answer. Make it clear what a correct response should include. Start with Standard recall if you are unsure which card type to choose.',
          'Save the concept and open Study. Try to answer the question before you select Reveal answer.',
          'Compare your attempt with the answer and any feedback. Choose a grade based on your attempt before you saw the answer.',
          'Return on later days to review cards that are due. Practice with different examples if you also need to apply what you have learned.'
        ]
      },
      {
        title: 'Keep it manageable',
        paragraphs: [
          'Start with a few useful questions. Continue to read, study worked examples, and solve problems. You do not need a card for every sentence.',
          'Choose a card type for the ability you want to practice. You do not need to identify a learning style. Use the simplest card type that supports your goal.'
        ]
      }
    ],
    related: [ 'cards', 'grading', 'sessions' ]
  },
  {
    id: 'cards',
    title: 'Choose useful cards',
    summary: 'Use a concept to practice the same content with different card types.',
    sections: [
      {
        title: 'Concepts and cards',
        paragraphs: [
          'A concept contains a title, a Prompt, an Answer, and optional feedback. Select the card types you need. Use the setup tabs or Next to configure each type. The cards share the concept content, but each card has its own review schedule.',
          'Create separate concepts for questions with different prompts or answers. Select only the card types that provide useful practice. Each additional type adds to your review workload.',
          'Removing a card type also removes its review progress. Adding that type again creates a new card.'
        ]
      },
      {
        title: 'Match the task',
        bullets: [
          'Use Standard recall for vocabulary or short facts. Choose Type answer when spelling or notation matters.',
          'Use Cloze to recall missing text within a sentence.',
          'Use Image occlusion to hide labels or regions of a diagram.',
          'Use Explain to practice causes, connections, or differences.',
          'Use Problem to practice calculations, decisions, or technical procedures.'
        ]
      },
      {
        title: 'Write a clear question',
        paragraphs: [
          'Include enough context to answer the question. State any required units, assumptions, or reasoning. Decide which ability the question should test. Include several steps when you need to practice a whole procedure.',
          'Check that the title, images, and surrounding text do not give away the answer. If one card reveals the answer to another, practice those cards in separate sessions when possible.',
          'Use a custom template when a consistent layout helps you read or answer a question. A template changes how content appears, not what you practice.'
        ]
      }
    ],
    related: [ 'recall', 'explain', 'problem' ]
  },
  {
    id: 'recall',
    title: 'Standard recall',
    summary: 'Answer a question from memory before revealing the answer.',
    sections: [
      {
        title: 'Use it for',
        paragraphs: [
          'Use Standard recall for facts, vocabulary, or a specific question you can answer mentally or on paper. You do not have to type your response.'
        ],
        example: 'Prompt: What does TCP provide that UDP does not? Answer: TCP provides a reliable, ordered byte stream and retransmits lost data.',
        avoid: 'A question such as “Do I understand TCP?” does not specify what you should recall. Ask about a particular behavior or feature instead.'
      }
    ],
    related: [ 'type-answer', 'grading' ]
  },
  {
    id: 'type-answer',
    title: 'Type answer',
    summary: 'Write a short response and compare it with accepted answers.',
    sections: [
      {
        title: 'Use it for',
        paragraphs: [
          'Use Type answer to practice spelling, a short term, or notation. Add other valid answers in the card setup. During Study, enter your response before you reveal the answer.',
          'Twill highlights differences between your response and the accepted answers. It does not judge whether your response means the same thing. Compare the answers and choose your grade. Use Explain or Problem when you need to practice reasoning.'
        ],
        example: 'Prompt: Write the French word for coffee. The accepted answer is “café”.',
        avoid: 'A differently worded explanation can still be correct. Judge its meaning rather than whether its text matches exactly.'
      }
    ],
    related: [ 'recall', 'grading' ]
  },
  {
    id: 'cloze',
    title: 'Cloze',
    summary: 'Recall missing text with help from the surrounding sentence.',
    sections: [
      {
        title: 'Set up an omission',
        paragraphs: [
          'Select text in Prompt and select the cloze button to mark an omission. To mark a single word, place the cursor in that word and select the button. To remove an omission, place the cursor inside it and select the button again.',
          'You can also add or remove an omission with Ctrl+Shift+C on Linux or Command+Shift+C on macOS.',
          'Each omission group creates a card. Put omissions in the same group to hide them together. Use separate groups when you want to recall them independently.'
        ],
        example: 'In “TCP provides a reliable, ordered [byte stream]”, hide the bracketed phrase to practice recalling that term.',
        avoid: 'Hide text that matters to your learning goal. Check that the same answer is not visible elsewhere in the question.'
      }
    ],
    related: [ 'cards', 'grading' ]
  },
  {
    id: 'image-occlusion',
    title: 'Image occlusion',
    summary: 'Hide existing labels or relevant regions of an image.',
    sections: [
      {
        title: 'Set up a diagram',
        paragraphs: [
          'Insert an image from your device into Prompt. Draw and adjust rectangular masks over the areas you want to hide. Put masks in the same group to recall those areas together. Separate groups create separate cards.',
          'Check that captions, legends, and nearby labels do not reveal the answer. Practice with another accurate diagram if you need to recognize a structure in different images, rather than remember its location in one image.'
        ],
        example: 'Hide the “mitochondrion” label on a cell diagram and identify the marked structure. Use a separate Explain question if its function also matters.',
        avoid: 'Recalling a label does not show that you understand the structure’s function. If the image has no labels to hide, write a question about the image instead.'
      }
    ],
    related: [ 'explain', 'cards' ]
  },
  {
    id: 'explain',
    title: 'Explain',
    summary: 'Explain how something works, how ideas relate, or why they differ.',
    sections: [
      {
        title: 'Define a good explanation',
        paragraphs: [
          'Choose Why, How, Cause and effect, or Compare and contrast. Add the key points that a correct explanation should include. During Study, explain mentally or use the scratchpad. Compare the meaning of your explanation with the key points.',
          'Ask about a specific relationship or process. If you cannot explain it, review a clear example or any background knowledge you need before trying again.'
        ],
        example: 'Prompt: Why can packet loss delay later TCP data even when those later packets arrive? Key point: The ordered byte stream waits for missing earlier data to be recovered.',
        avoid: 'A question such as “Explain everything about networking” is too broad to judge. Ask a specific question and focus on the ideas needed to answer it.'
      }
    ],
    related: [ 'problem', 'feedback', 'grading' ]
  },
  {
    id: 'problem',
    title: 'Problem',
    summary: 'Solve a problem and compare your result and reasoning with a checked solution.',
    sections: [
      {
        title: 'Practice the ability you need',
        paragraphs: [
          'Write the problem and verify its solution. Add checkpoints for important steps. During Study, attempt the problem in the workpad or on paper. Compare your result and any required reasoning with the solution.',
          'Study a worked example first if the method is unfamiliar. Leave a formula in Prompt when you want to practice applying it. Ask for the formula from memory only if you also need to recall it. Allow enough time to work through the problem.'
        ],
        example: 'A rate rises from 10% to 15%. State the increase in percentage points and relative percent. Answer: The rate increases by 5 percentage points. The relative increase is 50%, calculated using the original 10% as the denominator.',
        avoid: 'Following a visible solution is useful instruction, but not an independent solution. Try a different case without the solution when you want to check application.'
      }
    ],
    related: [ 'explain', 'feedback', 'grading' ]
  },
  {
    id: 'grading',
    title: 'Grade the original attempt',
    summary: 'Choose a grade based on your attempt before you saw the answer.',
    sections: [
      {
        title: 'Simple and Advanced',
        bullets: [
          'In Simple mode, choose Forgot if you could not answer correctly or Remembered if you could. These grades use the same scheduling outcomes as Again and Good.',
          'In Advanced mode, choose Again if you could not answer correctly. Choose Hard if you answered correctly with difficulty, Good for ordinary success, or Easy for unusually easy success.'
        ],
        paragraphs: [
          'Use Hard only for a correct answer. Decide what a correct response should include before reviewing, such as required reasoning or units. Different wording is fine if it meets those requirements.',
          'Information included in the question is allowed context. If you need to look at the answer or an extra hint to finish, judge what you could answer before that help. Use Undo last grade to correct an accidental grade when the command is available.'
        ],
        example: 'If you needed the revealed answer to finish your response, choose Forgot or Again. Being able to repeat the answer afterward does not change that attempt.',
        avoid: 'Choose a grade that reflects your attempt. If a card repeatedly causes difficulty, inspect the question or revisit the material rather than choose Easy to delay it.'
      }
    ],
    related: [ 'feedback', 'card-quality', 'sessions' ]
  },
  {
    id: 'feedback',
    title: 'Feedback and retries',
    summary: 'Use feedback to understand mistakes and practice the correction.',
    sections: [
      {
        title: 'Check the correction',
        paragraphs: [
          'Add Explanation and context or Common mistakes when they help explain an answer. Read relevant feedback when you are unsure about a correct answer as well as when you make a mistake.',
          'After missed reviews, mastery practice offers an optional retry for each missed card. Choose Still missed or Recalled to record the retry. These responses do not change the review schedule. Return for later scheduled reviews to check what you remember.'
        ],
        example: 'After confusing percent with percentage points, inspect the denominator in a worked correction, then attempt another case without it.',
        avoid: 'An immediate retry can help you practice a correction, but the answer may still be fresh in your mind. Check it again later and practice related questions separately.'
      },
      {
        title: 'Optional pretesting',
        paragraphs: [
          'To try a question before studying its explanation, open Settings, select Study, and enable pretesting. For eligible concepts you have not studied, Twill offers a first attempt or Skip. After an attempt, Twill shows the explanation. The ordinary review takes place in a later session.',
          'Select Skip if you have no useful starting point. Keep the first attempt brief, then study the explanation and practice what you have learned.'
        ]
      }
    ],
    related: [ 'grading', 'problem', 'card-quality' ]
  },
  {
    id: 'sessions',
    title: 'Manage review sessions',
    summary: 'Choose a manageable set of cards that are due for review.',
    sections: [
      {
        title: 'Choose a session',
        paragraphs: [
          'Open Study to review cards that are due. For a focused session, filter by deck, tag, search text, card type, or learning state. You can also set a card limit or study matching results from Library. Only active cards that are due are included.',
          'Pause a session or visit another screen and return while Twill remains open. Select End session to stop. Your grades remain saved, but the selected cards and unfinished responses do not resume after an app restart.'
        ]
      },
      {
        title: 'Keep the workload useful',
        paragraphs: [
          'If reviews build up, add less new material and remove duplicate or unnecessary cards. Return to Study without filters regularly so you also review other topics. After a break, start with a manageable selection rather than reset your progress.',
          'To vary the order of questions that are due, open Settings, select Study, and enable mixed practice. Twill uses concept and tag relationships to select varied questions. Organize related topics so the differences are useful to practice.',
          'Start with the default scheduling settings unless you have a reason to change them. A higher target retention generally requires more reviews. It does not guarantee an exam score. Choose a card count and session length that you can sustain.'
        ],
        avoid: 'Keep your device clock correct and grade your attempts honestly. To prepare for an exam, also practice unfamiliar problems that represent what you will need to do.'
      }
    ],
    related: [ 'grading', 'card-quality', 'saving' ]
  },
  {
    id: 'card-quality',
    title: 'Fix recurring difficulties',
    summary: 'Check whether you need more practice or a clearer question.',
    sections: [
      {
        title: 'Inspect before adding more cards',
        paragraphs: [
          'Check the answer against a trusted source. Consider whether you need background knowledge, forgot the answer, used the wrong method, or found an unclear or incorrect question.',
          'Select Needs improvement to flag a card and add a note. Open the queue from Library to edit a card, resolve a report, or dismiss a suggestion. Twill may suggest cards that repeatedly cause difficulty. Inspect each suggestion before deciding what to change.',
          'During Study, select Edit now to make a correction immediately or Edit later to leave a note. Changes that affect the question remove its unfinished cards from the current session. Archive material you no longer need. Keep useful questions even when they feel easy today.'
        ],
        example: 'Replace “Explain TCP” with the particular behavior you need to understand and a few clear answer criteria.',
        avoid: 'Correct an inaccurate question before you continue practicing it. Adding duplicate cards or increasing target retention does not fix the content.'
      }
    ],
    related: [ 'cards', 'feedback', 'sessions' ]
  },
  {
    id: 'saving',
    title: 'Saved work and backups',
    summary: 'Know which work is saved and keep a separate backup.',
    sections: [
      {
        title: 'Saved and unfinished work',
        paragraphs: [
          'Twill saves concepts, settings, review history, and saved drafts on your device. When leaving an unfinished concept or template, choose Keep draft to return to it later or Discard to remove the draft.',
          'Study responses, scratchpads, selected cards, and unfinished retries remain available only while Twill is open. Your saved grades remain after you close the app. Keep a separate backup to protect saved content as well as drafts.'
        ]
      },
      {
        title: 'Back up or export',
        paragraphs: [
          'Open Settings and select Backups to create a .twill backup. It includes your library, images, templates, decks, tags, settings, saved drafts, and learning history. Store a separate copy somewhere private. Backups are not encrypted.',
          'Restoring a compatible backup replaces the library on this device and restarts Twill. A readable ZIP export contains JSON files and images for use outside Twill. Use a .twill backup, not a ZIP export, to restore your library.',
          'If Twill cannot open its local storage, use the recovery screen to try again or restore a backup.'
        ]
      }
    ],
    related: [ 'sessions', 'shortcuts' ]
  },
  {
    id: 'shortcuts',
    title: 'Shortcuts and customization',
    summary: 'Find available commands and adjust the reading experience.',
    sections: [
      {
        title: 'Find commands',
        paragraphs: [
          'Open Commands to search for an action. Open Shortcuts to see the available keyboard shortcuts. On Linux, press Ctrl+Shift+P for Commands or Ctrl+/ for Shortcuts. On macOS, use Command instead of Ctrl.',
          'Press Ctrl+S to save in an active editor. During Study, press Space to reveal an answer or a number key to choose an available grade. While you are entering a response, typing and editor shortcuts take priority.'
        ]
      },
      {
        title: 'Adjust appearance and organization',
        paragraphs: [
          'Open Settings to choose a theme, font, reading text size, and motion preferences. Use local CSS snippets if you want to customize the appearance further.',
          'Use decks and tags in Library to organize your topics. Use the visual template editor or supported HTML and CSS to create reusable layouts. Choose a card type for the learning activity and a template for its appearance.'
        ]
      }
    ],
    related: [ 'cards', 'saving' ]
  }
];

export function guideTopic( id ) {
  return guideTopics.find( ( topic ) => topic.id === id );
}

export function searchGuide( query ) {
  const terms = normalizedText( query ).split( /\s+/u ).filter( Boolean );

  return guideTopics.filter( ( topic ) => {
    const text = normalizedText([
      topic.title,
      topic.summary,
      ...topic.sections.flatMap( ( section ) => [
        section.title,
        ...( section.paragraphs ?? []),
        ...( section.steps ?? []),
        ...( section.bullets ?? []),
        section.example ?? '',
        section.avoid ?? ''
      ])
    ].join( ' ' ) );

    return terms.every( ( term ) => text.includes( term ) );
  });
}

function normalizedText( text ) {
  return text.normalize( 'NFKD' ).replace( /\p{M}/gu, '' ).toLowerCase();
}
