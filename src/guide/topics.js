export const guideGroups = [
  { title: 'Getting started', topics: [ 'start', 'cards' ] },
  {
    title: 'Card types',
    topics: [ 'recall', 'type-answer', 'cloze', 'image-occlusion', 'image-labeling', 'explain', 'problem' ]
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
          'Check that the title, images, and surrounding text do not give away the answer. Twill defers remaining cards that share revealed content or a direct practice link. Use separate sessions when other questions give away an answer.',
          'Use a custom template when a consistent layout helps you read or answer a question. A template changes how content appears, not what you practice.'
        ]
      },
      {
        title: 'Compare linked cases',
        steps: [
          'Open either concept from Library. Beside its linked case, select Add comparison.',
          'Write a question about the difference that matters and an expected explanation. Select Save comparison.',
          'Select Compare to read both case prompts. Explain the difference mentally or enter an optional response, then select Show explanation to check your reasoning.',
          'In Study, reveal the current answer and select Linked practice. Choose a case with an authored comparison, then select Compare cases. Select Return to session when you are ready to grade your original answer.'
        ],
        paragraphs: [
          'The comparison is shared by both linked concepts. Edit it from either side. Removing the comparison keeps the link and both concepts.',
          'Comparisons do not record a review or change review dates. Closing clears the comparison response. Pausing Study keeps it until you resume or return to the session.',
          'Your original response and checks stay unchanged. Grade that original attempt, not how well you answered after seeing the comparison.'
        ]
      },
      {
        title: 'Practice with answer choices',
        steps: [
          'Edit a linked comparison and select Add answer choices. Write two to six different options and feedback explaining why each option fits or does not fit. Mark one option as the correct answer.',
          'Leave Keep option order unchecked to shuffle each new attempt. Select it when the order is meaningful, such as a sequence or a range. Select Save comparison.',
          'Open the comparison, choose an answer and optionally explain your choice. You can change your selection before selecting Check choice.',
          'Read the feedback and expected explanation. If your choice was incorrect, check why the correct option fits. Return to your original Study attempt when you are ready to grade it.'
        ],
        paragraphs: [
          'Use plausible alternatives that help you distinguish the cases. Avoid trick wording or options that give away the answer.',
          'Choice practice is supported practice, not a free recall review. Its result does not change review dates or count as a completed review. Pausing keeps your selection and option order; closing starts a fresh attempt next time.'
        ]
      },
      {
        title: 'Link related cases',
        steps: [
          'Create a separate concept for each case, with its own Prompt, Answer and any required key points or checkpoints.',
          'Open a concept from Library. In Linked practice, select Link concept, describe the learning objective and choose another concept.',
          'Select the linked title to revisit the other case. Use Edit learning objective to change the connection, or Remove link to keep both concepts without the connection.',
          'In Study, reveal your current answer, then select Linked practice. Choose a related case and the card you want to attempt. Select Recalled or Still missed after checking its answer, or select Return to session at any time.'
        ],
        paragraphs: [
          'Link cases that apply a shared principle in different situations, or cases whose differences help you choose the right principle. A link is visible from either concept. Each card keeps its own grade and review schedule.',
          'Linked practice is an extra attempt. It does not change review dates or count as a completed review. Your original response and checks are restored when you return to the session.',
          'Revealing an answer, an answer part, a hint or a reference defers remaining cards from that concept and its directly linked cases. Cloze and Image occlusion also defer related cards when shown, since visible context can reveal other answers. Select the deferred count to see those cards. They remain due, and a new session makes them available again.',
          'Pausing, hiding help or undoing a grade does not remove this boundary. Linked practice and mastery retries remain available as supported attempts.',
          'Archived cases remain labeled in the links but are not available for study. Restore them before adding new links. Deleting a concept removes its connections.'
        ],
        example: 'Compare a file transfer that requires reliable ordered delivery with a live video stream where late frames are useless. Ask which transport fits each case and why.',
        avoid: 'Changing a title or a few words without changing the decision does not test a new application. Changing numbers can help you practice a procedure, but does not necessarily test when to use it.'
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
        avoid: 'Recalling a label does not show that you understand the structure’s function. Practice its function separately when that matters.'
      }
    ],
    related: [ 'image-labeling', 'explain', 'cards' ]
  },
  {
    id: 'image-labeling',
    title: 'Image labeling',
    summary: 'Identify marked structures on an unlabeled image.',
    sections: [
      {
        title: 'Prepare an unlabeled image',
        paragraphs: [
          'Select Image labeling in the concept editor. Select Add labels below a Prompt image. Add a point, place it on the image, and enter its expected label. Enter accepted alternatives on separate lines. A position description can explain where to look without naming the answer.',
          'Use the arrow keys or percentage fields to adjust a point. Use Image zoom for small details. Select Apply labels, then save the concept. In the Library, select a numbered target to locate it and select Show labels to inspect its answers.'
        ],
        avoid: 'Use an image without printed answers. Check that the position description does not give away the label.'
      },
      {
        title: 'Recall one target at a time',
        paragraphs: [
          'Each target creates a separate card with its own review date. During Study, identify the numbered point before selecting Reveal answer. The image stays visible while its expected label and alternatives appear below it. Other labels remain hidden.',
          'You can type in Your label before revealing, or leave it empty and answer from memory. A typed response is compared with the expected label and accepted alternatives. Capitalization and extra spaces do not affect the match. Accents and punctuation do. Compare the meaning yourself and choose your grade. If you use help, the comparison uses your response before help.',
          'After you reveal an answer or use help, other cards from the same concept wait for another session. Their review dates do not change. Start another session when you want to practice another due target.',
          'Choose Forgot or Remembered for that target, not for the whole diagram. Hints and references count as help. Editing a label or moving its point keeps its review progress. Removing a target removes its card. Adding it again starts a new card.'
        ],
        avoid: 'Knowing a name does not show that you understand its function. Use Explain or linked practice for relationships and functions when they matter.'
      }
    ],
    related: [ 'image-occlusion', 'explain', 'grading' ]
  },
  {
    id: 'explain',
    title: 'Explain',
    summary: 'Explain an idea or reconstruct how several ideas fit together.',
    sections: [
      {
        title: 'Define a good explanation',
        paragraphs: [
          'Choose Why, How, Cause and effect, or Compare and contrast. Add the key points that a correct explanation should include. During Study, explain mentally or use the scratchpad. Compare the meaning of your explanation with the key points.',
          'After revealing the answer, you can mark each key point as Remembered, Partially remembered, or Forgot. If you wrote an explanation, the choices are Present, Incomplete, and Missing. Different wording can express the same idea. Checks are optional, and you still choose one grade for the whole card.',
          'Ask about a specific relationship or process. If you cannot explain it, review a clear example or any background knowledge you need before trying again.'
        ],
        example: 'Prompt: Why can packet loss delay later TCP data even when those later packets arrive? Key point: The ordered byte stream waits for missing earlier data to be recovered.',
        avoid: 'A question such as “Explain everything about networking” is too broad to judge. Ask a specific question and focus on the ideas needed to answer it.'
      },
      {
        title: 'Reconstruct a topic',
        steps: [
          'Select Explain in the concept editor. In Prompt focus, choose Reconstruct an outline, Reconstruct a sequence, or Reconstruct relationships.',
          'Write a bounded task in Prompt and a checked reference in Answer. Add the essential ideas, ordered steps, or relationships that a good response needs. Use Move step up or Move step down to change the reference sequence.',
          'During Study, reconstruct the topic before selecting Compare reconstruction. Write in Your reconstruction, answer mentally, or use paper. You can use line breaks and indentation to organize your response.',
          'Compare your original response with the reference and its essential structure. Mark important gaps with the optional checks.',
          'Select Add correction to write what you would change. Revisit lists the ideas you marked as missing or partial. You can also correct your response mentally or on paper. Choose one grade for the original attempt, not the correction.'
        ],
        paragraphs: [
          'For a diagram, add an image without answers to Prompt and describe the connections you want to reconstruct. A written description or paper sketch is enough. You do not need to draw inside Twill.',
          'Judge the ideas, order, and connections that matter to the task. Different wording or another valid layout can be correct. A map or outline is an alternative to prose, not a required extra step.',
          'To offer a starting point, select Add structural prompts in the reconstruction setup. Write a heading or guiding question on each line, without its answer. You can add up to 12 prompts of 280 characters each. Select Structure in the preview to check them.',
          'During Study, select Structure if you need those prompts. Revealing them counts as help. Hiding them does not undo that help. Your original response stays available for comparison, separate from anything you write afterward. Grade the attempt you made before help.',
          'To practice a connection you missed, use Linked practice in the card header and choose a relevant linked case. If the question or reference needs work, use Card issue or Edit later. A gap in your recall does not necessarily mean the card needs changing.',
          'Your correction stays available when you hide it, pause, visit another screen, or return from linked practice. Undo restores it with the original attempt. Corrections are not saved after you restart the app.'
        ],
        example: 'Ask for the three messages in a TCP handshake. Add one reference step for each message. Include why each acknowledgement follows the preceding message when that relationship matters.',
        avoid: 'Keep the task small enough to check fairly. Reconstructing an entire chapter in one card makes it difficult to identify and grade important gaps.'
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
          'After revealing the answer, you can mark each checkpoint as Remembered, Partially remembered, or Forgot. If you wrote in the workpad, the choices are Present, Incomplete, and Missing. Accept another valid method when the question allows it. These optional checks do not choose a grade.',
          'Study a worked example first if the method is unfamiliar. Leave a formula in Prompt when you want to practice applying it. Ask for the formula from memory only if you also need to recall it. Allow enough time to work through the problem.'
        ],
        example: 'A rate rises from 10% to 15%. State the increase in percentage points and relative percent. Answer: The rate increases by 5 percentage points. The relative increase is 50%, calculated using the original 10% as the denominator.',
        avoid: 'Following a visible solution is useful instruction, but not an independent solution. Try a different case without the solution when you want to check application.'
      },
      {
        title: 'Prepare a worked example',
        paragraphs: [
          'In the Problem setup, select Add worked example. Give each step a prompt and a checked solution. You can add a principle question to an important step. Choose which solutions should stay visible in completion practice.',
          'Use Full example and Completion preview to check your steps. The preview is also available under Worked example in the concept details. It does not change your study progress.'
        ],
        avoid: 'Use worked steps for a procedure you need to apply. You do not need a sequence of steps for every fact. Check that the solutions agree with the Answer.'
      },
      {
        title: 'Choose the help you need',
        paragraphs: [
          'During Study, use Workpad to solve normally. Select Complete steps to fill the gaps in a worked solution, or Worked example to inspect all steps. In completion practice, you can show or hide any missing solution without losing your writing. Principle responses are optional.',
          'Your work stays available when you change support, pause, or visit another view. Select Check solution to compare with the full Answer and checkpoints. Grade the attempt you made before help. Hiding help again does not turn supported work into an independent attempt.',
          'After checking the solution, select Try another problem when a linked Problem card is available. Solve that different problem without revealing its solution. Extra practice keeps its own response and does not change review dates. Return to session to continue with your original attempt.'
        ],
        avoid: 'Completing a step after seeing its solution does not show independent recall. Try a different case, and practice again later. An app restart clears unfinished study work, but completed reviews remain saved.'
      },
      {
        title: 'Find and correct a mistake',
        paragraphs: [
          'In the Problem setup, add an error example to your worked example. Write a plausible mistake, identify it, explain why it is wrong, and provide corrected work. Check the correction against a reliable source and the correct worked example.',
          'During Study, select Find the error. Locate the error, explain it, and correct the work before selecting Show correction. Twill shows the authored correction but does not judge your writing. Your responses stay available when you hide the correction, change support, pause, or return from a linked problem.',
          'Select Worked example whenever you need to review the correct method. Error practice counts as help for the current problem. Grade your original attempt before help, then try a different linked problem without its solution.'
        ],
        avoid: 'If the method is new, study the correct example first. An erroneous example should always be clearly labeled and paired with a checked correction.'
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
          'Information included in the question is allowed context. If you need to look at the answer or an extra hint to finish, judge what you could answer before that help. Use Undo last grade to correct an accidental grade when the command is available.',
          'If you open Hint or Reference before revealing the answer, Twill marks the review as assisted. Grade your original attempt, not what you could answer afterward. Choose Forgot or Again if you needed help to answer.'
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
        title: 'Use optional help',
        steps: [
          'When creating or editing a concept, select Add help. Write a Hint for a useful cue or memory association, or add a Reference with a formula, source or example.',
          'Try the question first. Select Hint or Reference when you need it, or reveal the full answer without opening either.',
          'After revealing the answer, compare your response from before help. If you continued writing after opening help, that later response appears separately.'
        ],
        paragraphs: [
          'Help is shared by the concept’s card types. Put context that should always be available in Prompt instead. Hint and Reference are withheld during a first pretest attempt.',
          'Opening help before the answer marks the review as assisted. Hiding it does not remove that mark. Opening a reference after revealing the answer is feedback for that attempt.'
        ]
      },
      {
        title: 'Reveal an answer in parts',
        steps: [
          'In Answer, place the cursor in a block or select several blocks. Select Answer part in the toolbar to make those blocks a part. Select it again inside a part to remove the part without deleting its content.',
          'Below the editor, group a part with another part when you want to reveal them together. Without authored groups, all parts belong to the default reveal group.',
          'On a built-in study card, try the whole question first. Select Reveal for one part or use its group control to reveal the named parts together. Without authored groups, select Reveal all parts. You can still reveal the full answer at any time.'
        ],
        paragraphs: [
          'Keep context that should always be visible in Prompt. Other Answer content stays hidden until you reveal the full answer. Template cards use their own layouts and show parts as ordinary answer content.',
          'Revealing a part early marks the review as assisted. Grade your whole original attempt, not just the parts you remembered. Parts do not receive separate grades or review schedules.',
          'After full reveal, Explain and Problem cards use their key points or checkpoints for optional checks. Other built-in cards let you check the parts themselves. Use Remembered, Partially remembered, or Forgot for mental recall. Written answers use Present, Incomplete, and Missing. A part may provide extra explanation rather than a requirement. If a required idea was missing or incomplete, choose Forgot or Again for the whole answer. Checks remain available when you pause or visit another view, but not after you restart the app.'
        ]
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
