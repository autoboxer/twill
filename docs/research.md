# Research and design rationale

This register explains which sources informed Twill’s learning features and
where the evidence limits a proposed benefit. It includes original experiments,
research reviews published in academic journals, and separately identified
guidance from practitioners. It is a targeted review, not an exhaustive
systematic review with a protocol registered in advance.

The strongest general foundation is recalling information from memory, returning
to it later, and receiving useful feedback. Other activities depend on the task,
the learner’s prior knowledge, the activity used for comparison, and the time
required. Success immediately after learning is different from remembering
later. Applying learning to a new task is also a separate outcome. A difference
that is not statistically significant does not prove that two methods are
equivalent. Combining useful methods does not necessarily combine their benefits.

The access column describes what was inspected. **Full** means an accessible
paper was read. **Abstract** means the summary is limited to the paper’s primary
record or the excerpts specified. Details that could not be verified are not
treated as established facts. Some links require publisher access. Reading
Twill’s help does not require a connection.

Reported values of *d* and *g* are standardized effect sizes. They describe the
size of a difference between groups, not a percentage improvement in retention.

## How evidence informs the product

The table includes both existing features and proposed directions. Optional
hints and references, answer parts with selective reveal, and idea checks are
available. Linked applications, activities for comparing examples or choosing
between answers, guided sequences, image labeling, reconstruction controls,
generated examples, and displays of learning across sessions remain planned. See
[current boundaries](studying-effectively.md#current-boundaries) for availability.

| Implemented or planned choice | Main research basis | What remains a product inference |
| --- | --- | --- |
| Recall, Type answer, Cloze, and multiple card types | Rowland 2014; Yang 2021; Pan & Rickard 2018; Yu 2025 | The editors, six card types, text comparison, and separate schedules are Twill design choices. |
| FSRS scheduling and spaced review | Cepeda 2006/2008; Bahrick 1993; Latimier 2021; Rawson & Dunlosky 2011 | These studies do not validate FSRS itself, its parameters, or the default 90% target. |
| Explain and Problem | Bisra 2018; Rittle-Johnson 2017; Barbieri 2023; Murray 2025 | The scratchpad, checkpoints, and grading by the learner have not been experimentally validated as Twill features. |
| Answer feedback and mastery retries | Butler 2008/2013; Rawson & Dunlosky 2011 | One retry in the same session is a practical limit for practicing a correction. It is not the spaced practice used in the research. |
| Optional pretesting | St. Hilaire 2024; King-Shepard 2025 | Twill’s explanation sequence and deferral of reviews to a later session are design choices. |
| Optional mixed practice | Brunmair & Richter 2019; Rittle-Johnson 2009 | Twill uses shared tags and queue rules to approximate related practice. This exact approach has not been validated experimentally. |
| Image occlusion and planned image labeling | General evidence on recall and application; Rohrer 2010; research on visual presentation below | No direct comparison located in this review shows that Twill labeling is better than occlusion or validates its grouping controls. |
| Practical guide | Dunlosky 2013 and the sources for particular subjects below | The guide’s structure, examples, search, and introduction to the app need usability testing. |
| Optional hints and references, answer parts, and idea checks | Finn 2010; Dunlosky 2011; Froese 2022; Fiechter/van den Broek 2019; Roelle 2017 | Optional help, reveal groups, comparison with the original response, and the assisted flag are product choices. Idea checks record the learner’s judgments for the current session. They do not determine a grade or schedule parts independently. Grades still assess the original attempt. No study establishes a precise conversion from hint use to an FSRS grade. |
| Planned linked applications, comparisons, and choices | Gick & Holyoak 1983; Alfieri 2013; Butler 2017; Smith/McDermott 2014; Little 2015 | The way Twill will link questions and let users create choices is a design interpretation of the evidence. |
| Planned guided problems and correction activities | Atkinson 2003; Kalyuga 2003; Adams 2014; Sinha & Kapur 2021; contrary evidence on removing help | A short sequence for one learner does not reproduce a complete classroom teaching intervention. |
| Planned reconstruction and generated examples | Blunt 2014; Cromley 2020; Rawson 2016; Obergassel 2025/2026; contrary findings from Zamary 2018 and O'Day 2021 | Maps, sketches, outlines, and creating examples are optional activities. They are not consistently better for every task. |
| Planned records of learning across sessions and optional uncertainty | Rawson 2011/2020; Nelson 1991; Butler 2008; Double 2025 | A display of results or confidence control has not itself been shown to improve retention in Twill. |
| Card quality reports, search, sessions, drafts, undo, accessibility, and interface quality | User requirements, observed usability problems, code review, and correctness checks | These are product and engineering decisions. No learning study validates Twill’s particular flags, filters, colors, corner radii, or splash screens. |
| Local storage, privacy, sync, backup, platforms, and technology choices | Explicit product constraints and technical design work | These choices follow Twill’s requirements. They are not claims about learning science. |

## Practical guidance from Anki and SuperMemo

These sources were reviewed on October 5, 2026. The official Anki documentation
and the SuperMemo authoring guide linked from it inform Twill’s practical design.
They complement the academic research, but are not controlled learning studies.

| Source | Useful contribution and boundary | Integration |
| --- | --- | --- |
| [Anki: Getting started](https://docs.ankiweb.net/getting-started.html) and [Adding/editing](https://docs.ankiweb.net/editing.html) | Share content while testing different recall directions separately. Write clear questions and add cards deliberately. These pages do not establish that creating all material yourself is always better than using suitable provided material. | This informs Twill’s concepts, cards, answer parts, and guide, as well as planned linked practice. |
| [Anki: Studying](https://docs.ankiweb.net/studying.html) | Offer simple or more detailed grading and defer related cards. The exact delay and controls are product decisions. | This informs existing grading, the guide, and planned linked practice. |
| [Anki: Deck options](https://docs.ankiweb.net/deck-options.html) | Introducing new material adds future reviews. Higher desired retention requires more work. Numerical examples and defaults are guidance, not measured rules for every learner. | This informs existing settings, the guide, and planned records of learning across sessions. |
| [Anki: Leeches](https://docs.ankiweb.net/leeches.html) | Repeated mistakes are a reason to inspect wording, understanding, and whether the material is still useful. Twill already suggests cards for inspection, so it does not need to copy Anki’s suspension threshold. | This informs the existing quality queue, the guide, and planned learning records. |
| [Anki: Statistics](https://docs.ankiweb.net/stats.html) | Reporting the first review separately distinguishes it from later repetitions. Card statistics alone do not establish general understanding or the ability to apply learning to a new task. | This informs planned learning records and optional uncertainty reporting. |
| [Anki: Field replacements](https://docs.ankiweb.net/templates/fields.html) and [Filtered decks](https://docs.ankiweb.net/filtered-decks.html) | Treat hints as help and text comparison as advice. Explain whether extra practice changes scheduling. Current Twill sessions include only cards that are due. Studying cards early requires a separate decision. | This informs Type answer, optional help, session limits, and the guide. Linked practice remains planned. |
| [SuperMemo: Twenty rules](https://super-memory.com/articles/20rules.htm), linked by Anki | Write focused questions, check background knowledge, include useful examples, and record sources. This is practitioner guidance. Keep several reasoning steps together when the whole procedure is the intended skill. | This informs the guide and answer parts, as well as planned guided problems and source references. |

## Historical evidence expansion

The review spans several decades rather than only recent publications. These
sources include results measured soon after learning and results measured much
later. The access notes and limitations describe each source separately.

| Source and access | Finding and limits | Consequence for Twill |
| --- | --- | --- |
| [Slamecka & Graf, 1978](https://doi.org/10.1037/0278-7393.4.6.592); [Bertsch et al., 2007](https://doi.org/10.3758/BF03193441), full research synthesis | Across 86 studies and 445 effects, generating responses rather than reading them had an average benefit of about 0.40 standard deviations. Only 30 effects measured delays beyond one day. Most tasks constrained the response rather than ask learners to create educational examples freely. | Provide requirements for creating examples. Do not claim that making cards or writing any response ensures lasting understanding. |
| [Gick & Holyoak, 1983](https://doi.org/10.1016/0010-0285(83)90002-6), primary abstract and author paper | Comparing analogies helped learners identify a shared structure and apply it to a problem. This is not evidence of retention over several months. | Planned linked and comparison practice should emphasize meaningful relationships and when they apply, not just reword questions. |
| [Nelson & Dunlosky, 1991](https://doi.org/10.1111/j.1467-9280.1991.tb00147.x), full | In 30 learners studying paired words, judgments made after a delay predicted recall better. This measured monitoring within a session, not whether collecting confidence improved learning. | Encourage learners to check recall later with the answer hidden. Keep planned uncertainty reporting optional. |
| [Bahrick et al., 1993](https://doi.org/10.1111/j.1467-9280.1993.tb00571.x), primary abstract | In a study lasting nine years, vocabulary tests one to five years after training favored wider spacing. Only four people participated. | Keep the evidence of lasting retention and its small sample limitation visible. Do not prescribe the study’s exact schedule of 56 days. |
| [Rosenshine et al., 1996](https://doi.org/10.3102/00346543066002181), review abstract | Teaching learners to generate questions improved comprehension of new material. Effects differed greatly between standardized tests and tests written by researchers. | Model useful questions and clear answer requirements. Do not treat the number of questions created as a retention goal. |
| [O'Reilly, Symons & MacLatchy-Gaudet, 1998](https://doi.org/10.1006/ceps.1997.0977), primary abstract | In 55 students learning cardiovascular facts, explaining meaning and connections worked better than generic questions about why or repetition. The abstract did not establish the exact test delay. | Use Explain, the guide, and planned guided problems to practice meaningful connections. Do not require a generic “why?” after every statement. |
| [Atkinson et al., 2003](https://doi.org/10.1037/0022-0663.95.4.774), full in initial research pass | In two experiments, gradually removing worked steps while explaining principles improved application to similar and more different problems without extra time. Removing steps alone had less reliable benefits for more different problems. Tests occurred within the session. | Offer optional, limited help in planned guided problems. Do not remove steps automatically for every task. |
| [Kalyuga et al., 2003](https://doi.org/10.1207/S15326985EP3801_4), review abstract | Help that benefits beginners can become unnecessary or harmful as expertise grows. | Let learners inspect, skip, or restore help. Do not require a beginner sequence for familiar material. |
| [Thiede et al., 2003](https://doi.org/10.1037/0022-0663.95.1.66), full | In 66 students, generating keywords after a delay improved judgments of understanding, selection of material to restudy, and performance after restudying. The results were measured in the same session, not as an isolated gain in lasting memory. | Planned reconstruction activities should help learners choose what to revisit, rather than only collect ratings or keywords. |
| [Roediger & Karpicke, 2006](https://doi.org/10.1111/j.1467-9280.2006.01693.x), abstract | In prose experiments without feedback, restudying performed better after five minutes. Recall practice performed better after two days and one week. | Distinguish initial instruction, immediate ease, and later retention in the guide. |
| [Cepeda et al., 2008](https://doi.org/10.1111/j.1467-9280.2008.02209.x), author paper | More than 1,350 people participated, with final tests up to one year later. Useful review gaps depended on how long learners needed to retain the material. | Explain spacing and workload without claiming that this validates FSRS parameters or one review gap for everyone. |
| [Kornell & Bjork, 2008](https://doi.org/10.1080/09658210701763899), full | Dropping flashcards after they seemed learned was generally unhelpful in four experiments, including tests after one week. Actual exposure to the material could differ. | Encourage learners to archive unwanted material, not useful material that feels easy today. Continue reviews on later days. |
| [Butler et al., 2008](https://doi.org/10.1037/0278-7393.34.4.918), full | Two experiments with 30 participants each found feedback useful for correct answers that learners were unsure about. Testing included a delay of two days. | Offer feedback after uncertain correct answers. Collecting confidence was not itself the tested benefit. |
| [Finn & Metcalfe, 2010](https://doi.org/10.3758/MC.38.7.951), full | Feedback with incremental hints helped after 30 minutes and about one day. Small samples and unequal time across conditions limit claims about efficiency. | Keep hints optional and retain easy access to the full answer. Later hint studies also found no benefit or poorer results. |
| [Rohrer et al., 2010](https://doi.org/10.1037/a0017678), institutional primary abstract | Practicing map recall improved schoolchildren’s responses to repeated and new questions one day later. | Include spatial relationships in planned labeling and reconstruction activities. This evidence does not identify a superior canvas or drag interaction. |
| [Rawson & Dunlosky, 2011](https://doi.org/10.1037/a0023956), primary abstract | Across three experiments with 533 students, spaced relearning improved retention one to four months later with relatively few extra attempts. | Distinguish success across sessions from an immediate correction in planned learning records. Neither the tested criterion of three successes nor the number of attempts saved is a universal efficiency rule. |
| [Dunlosky et al., 2013](https://doi.org/10.1177/1529100612453266), review abstract and excerpts | A broad review rated practice testing and distributed practice highly. Many popular techniques had narrower evidence or applicability. | Keep recall and spacing as a simple foundation. Do not claim that highlighting, summarizing, or mnemonics are always useless. |
| [Alfieri et al., 2013](https://doi.org/10.1080/00461520.2013.775712), research synthesis | Across 57 experiments and 336 tests, comparing cases had an overall effect of d=.50. Later effects were smaller than immediate effects. Associations with study characteristics do not establish causes. | Plan comparison practice without prescribing one order for everyone or guaranteeing retention over several months. |
| [Adams et al., 2014](https://doi.org/10.1016/j.chb.2014.03.053), full; [Große & Renkl, 2007](https://doi.org/10.1016/j.learninstruc.2007.09.008), abstract | Finding and correcting decimal errors helped 208 schoolchildren after one week, but not immediately. Initial group differences and the specific subject limit the result. Earlier probability studies also found that prior knowledge mattered. | Offer optional correction activities with verified solutions and help for beginners. Do not routinely present false solutions without correction. |
| [Little & Bjork, 2015](https://doi.org/10.3758/s13421-014-0452-8), full | Plausible competing choices improved recall of related information. The final test followed a filler task lasting four minutes, not a delay of 48 hours. | Use choices to practice distinctions. Use the separate classroom studies and tests after one week for claims about later retention. |
| [Rawson & Dunlosky, 2016](https://doi.org/10.1007/s10648-016-9377-z), abstract; [Zamary & Rawson, 2018](https://doi.org/10.1007/s10648-016-9396-9), abstract | In the first study, creating examples worked better than restudying definitions for the same amount of time. In the second, supplied examples worked better than creating examples or combining activities for learning after two days and efficiency. | Include accurate supplied examples and optional generation. A more active task is not necessarily the best choice. |
| [Zamary & Rawson, 2018](https://doi.org/10.1007/s10648-018-9433-y), abstract | In two experiments on declarative concepts with 146 and 131 learners, supplied examples and examples with help gradually removed had similar results after two days. Supplied examples took much less study time. | Do not extend findings about removing procedural help to every concept. A difference that is not significant does not establish equivalence. |
| [Roelle & Berthold, 2017](https://doi.org/10.1016/j.learninstruc.2017.01.008), primary abstract and excerpts | In 192 chemistry learners, removing reference access was less helpful for complex inferences than for simpler summaries. The exact test delay was not verified here. | Leave background information visible when a question tests reasoning. Practice recalling that information separately when needed. |
| [Bisra et al., 2018](https://doi.org/10.1007/s10648-018-9434-x); [Rittle-Johnson et al., 2017](https://doi.org/10.1007/s11858-017-0834-z), research synthesis abstracts and excerpts | Explaining ideas in one’s own words has broad average benefits. In mathematics, evidence from later tests and classrooms is more limited than evidence from immediate tests. | Practice useful processes and connections with Explain. Do not require a long explanation for every fact. |
| [Fritz et al., 2007](https://doi.org/10.1002/acp.1287); [Miyatsu & McDaniel, 2019](https://doi.org/10.3758/s13421-019-00936-2), abstracts | Combining memory aids and recall practice helped in some vocabulary conditions and tests after one week. It did not help in every condition or in a comparison after 48 hours. | Offer optional association notes in Hint or answer feedback. This does not require a dedicated memory palace system. |
| [Chan, 2009](https://doi.org/10.1016/j.jml.2009.04.004), abstract; [Oliva & Storm, online 2022/print 2023](https://doi.org/10.1007/s00426-022-01729-0), full | Related information that was not tested sometimes benefited. A larger replication with delays of a day or a week found a smaller overall benefit, with no reliable benefit in some individual comparisons. | Assess important related questions directly in planned linked practice. Do not assign success to every related question after one correct answer. |
| [Sinha & Kapur, 2021](https://doi.org/10.3102/00346543211019105), full research synthesis | Across 53 studies and 166 comparisons, solving problems before instruction had a moderate average advantage. Age, subject, and study design limited the result. Many interventions included collaborative generation and deliberate instruction to bring the ideas together. | A brief attempt followed by instruction and comparison is reasonable to offer. Pretesting alone does not reproduce the full studied intervention. |

Newer work remains relevant as a check on these foundations, not as the exclusive
source of evidence. The sections below include sources from 2020 through 2026
used in the original review, including contrary findings. The following limits
are particularly important.

- Recalling the overall structure can use prose or maps. Research with tests
  after one week supports both, not the superiority of maps. Adding a mapping
  step before recall did not improve a later outcome in a controlled comparison.
  [Blunt & Karpicke, 2014](https://doi.org/10.1037/a0035934),
  [O'Day & Karpicke, 2021](https://doi.org/10.1037/edu0000486).
- Recall practice has a smaller advantage over activities that develop ideas
  actively than over passive activities. Feedback and task format matter.
  [Gonçalves et al., 2025](https://doi.org/10.1007/s10648-025-10076-6).
- Mathematics evidence supports spacing, but the smaller set of comparisons
  between testing and restudying is inconclusive. Use Problem cards alongside
  instruction and varied problem solving.
  [Murray et al., 2025](https://doi.org/10.1007/s10648-025-10035-1).
- Creating responses and recalling information can support different outcomes,
  but combining them takes time and is not consistently better. Recent evidence
  does not establish one best order for everyone.
  [Obergassel et al., 2025](https://doi.org/10.1037/edu0000949),
  [Obergassel et al., 2026](https://doi.org/10.1002/acp.70188).
- There is no established best feedback delay for every task. Requiring confidence
  judgments can interfere with some learning.
  [Kandemir et al., 2026](https://doi.org/10.1007/s10648-026-10117-8),
  [Double et al., 2025](https://doi.org/10.1007/s11409-025-09413-5).

Before implementing an activity, identify its intended benefit. It might help
learners remember later, apply knowledge, distinguish similar ideas, find errors,
or use a more accessible format. Compare it with the best existing Twill workflow.
Optional personal trials can compare similar content, total time including
authoring, later independent answers, and new examples. These are not controlled
studies of effectiveness. This review does not justify telemetry, one ideal
session duration or card count, or a retention guarantee for Twill.

## Additional sources for existing decisions and safeguards

| Source | Short findings and limits | Access | Twill decision |
| --- | --- | --- | --- |
| [Rowland, 2014](https://doi.org/10.1037/a0037559) | Testing worked better than restudying overall. Initial recall showed larger average benefits than recognition. This does not rank every well written question format. | Primary abstract | Keep recall, Type answer, Cloze, and review. Also allow useful choice questions in planned comparison practice. |
| [Cepeda et al., 2006](https://doi.org/10.1037/0033-2909.132.3.354) | A review of 317 experiments in 184 articles found that the time between study sessions and the time until testing jointly affect recall. There is no single ideal review gap. | Author manuscript abstract and excerpts | Explain spaced scheduling in the guide without treating the result as proof of a particular FSRS setting. |
| [St. Hilaire, Chan & Ahn, 2024](https://doi.org/10.3758/s13423-023-02353-8) | A research synthesis registered in advance found that questions before instruction benefited targeted material, with g=.54. There was virtually no general benefit for untested content, with g=.04. It appeared online in 2023 and in the 2024 volume. | Publisher abstract | Keep pretesting optional, describe its benefits narrowly, and provide the explanation after an attempt. |
| [Brunmair & Richter, 2019](https://doi.org/10.1037/bul0000209) | Across 59 studies, alternating between categories had a moderate average benefit that depended strongly on the material and category similarity. Words sometimes benefited more from practice grouped by category. | Primary abstract | Use meaningful tags for mixed practice and useful distinctions for planned comparisons. Do not treat random shuffling as the goal. |
| [Pashler et al., 2008](https://doi.org/10.1111/j.1539-6053.2009.01038.x) | The review found insufficient evidence for matching instruction to a diagnosed learning style. Real preferences and aptitudes do not establish that matching claim. | Primary abstract | Recommend activities by the task and accessibility needs, not by a classification such as visual or auditory learner. |
| [Karpicke et al., 2014](https://learninglab.psych.purdue.edu/downloads/2014/2014_Karpicke_etal_JARMAC.pdf) | Elementary school learners benefited from recall practice with sufficient support. Recall without guidance was not consistently successful. | Full, initial research pass | Offer optional structural prompts in planned reconstruction activities. Do not require an empty canvas. |
| [Zamary, Rawson & Dunlosky, 2016](https://doi.org/10.1016/j.learninstruc.2016.08.002) | Students often overestimated the quality of examples they created. The tested feedback did not reliably remove the problem. | Abstract and results excerpt | Provide requirements and reference examples for planned generation activities. Do not automatically treat a learner’s example as verified content. |
| [Rawson et al., 2018](https://doi.org/10.1037/xap0000146) | Later relearning reduced the effects of initial learning conditions. This does not establish one repetition requirement for everyone. | Primary abstract | Interpret success across separated sessions in planned learning records. Distinguish initial ease from lasting learning in the guide. |

## Foundations and important limits

| Source | Short findings and limits | Access | Twill decision |
| --- | --- | --- | --- |
| [Yang et al., 2021](https://pubmed.ncbi.nlm.nih.gov/33683913/) | A synthesis of classroom testing research included 222 independent studies and 48,478 students, with an overall effect of g=.499. Comparison activities and outcomes varied. | Abstract. This supports recall practice, not a particular interface. | Keep recall and review as the guide’s foundation. |
| [Latimier et al., 2021](https://doi.org/10.1007/s10648-020-09572-8) | Across 29 studies, spacing recall practice rather than grouping it together had an effect of g=.74. Expanding the gaps rather than keeping them uniform had an effect of g=.034, which was not significant. | Abstract. This was not a direct test of FSRS and does not justify replacing it. | Retain spaced review and explain it in the guide and planned learning records. |
| [Pan & Rickard, 2018](https://pubmed.ncbi.nlm.nih.gov/29733621/) | Across 122 experiments and 192 effects, benefits for new tasks depended on initial success, how responses aligned between tasks, and how ideas were developed. | Abstract and manuscript excerpts. Application benefits are conditional. | Use different card types and planned linked or comparison practice to address meaningful tasks. |
| [Gonçalves et al., 2025](https://doi.org/10.1007/s10648-025-10076-6) | Across 44 studies and 142 comparisons, recall practice had a small advantage over active activities that developed ideas, with g=.14. The result varied with feedback and the comparison activity. | Abstract. This does not show that all active methods are equivalent. | Keep ordinary review efficient. Offer planned reconstruction and generation when they address a useful goal. |
| [Murray et al., 2025](https://eprints.whiterose.ac.uk/id/eprint/229807/) | Spacing in mathematics had an effect of g=.28 across 27 studies. Testing compared with restudying had an effect of g=.18 across seven studies, with a confidence interval that included zero. | Abstract. The uncertain result does not show that recall practice never works in mathematics. | Keep Problem cards alongside instruction and planned guided practice. |
| [King-Shepard et al., 2025](https://doi.org/10.1007/s10648-025-10075-7) | Questions before instruction benefited targeted content, with g=.66. There was no general benefit for other content, with g=.01. | Abstract. This supports narrow claims for pretesting. | Explain the limits of optional pretesting in the guide. |
| [Corral & Carpenter, 2025](https://doi.org/10.1016/j.learninstruc.2025.102219) | For concepts about research methods, three rounds of recall practice produced retention and application benefits at a test after one week. Evidence from one immediate round was less convincing. | Full publisher methods and results were inspected. This is not a claim about retention over several months. | Include later application checks in planned linked and comparison practice. |

## Variants, comparisons, and supported problem solving

| Source | Short findings and limits | Access | Twill decision |
| --- | --- | --- | --- |
| [Butler et al., 2017](https://doi.org/10.1037/xap0000142) | In four geology experiments, varied applications worked better than repeating the same example on new questions after two days. Varied examples during study also helped. | Abstract | Offer meaningful variations in planned linked practice. |
| [Butowska et al., 2024](https://doi.org/10.1073/pnas.2413511121) | Across seven experiments, mainly on vocabulary, varied context helped recall, including after 24 hours. The benefit depended on spacing conditions. | Full | Use contextual variations deliberately in planned linked practice. |
| [Cao & Carvalho, 2026](https://doi.org/10.1007/s10648-026-10169-w) | In two experiments on learning rules, the effects of variation depended on instruction. They were not significant when rules were explicitly supplied. Tests followed short delays. | Full. This is narrower evidence than lasting retention. | Vary planned linked practice for a purpose, not just novelty. |
| [Rittle-Johnson et al., 2009](https://doi.org/10.1037/a0016026) | In 236 school students, benefits from comparing alternative methods depended on prior algebra knowledge. Beginners benefited from simpler comparisons or sequential examples. | Abstract | Account for prior knowledge in planned comparisons and guided problems. |
| [Barbieri et al., 2023](https://doi.org/10.1007/s10648-023-09745-1) | Across 55 studies and 181 effects, worked examples benefited mathematics performance, with g=.48. Added explanation prompts were associated with smaller effects. This does not estimate later retention from gradually removing help. | Abstract | Include instruction before independent solving in planned guided problems. |
| [Miller-Cotto & Auxter](https://doi.org/10.1080/01443410.2019.1646411) | Across four weekly algebra homework assignments with 135 undergraduates, gradually removing help did not work better than ordinary practice. Combining that approach with explanation prompts worked less well than ordinary problem solving. | Full. Published online in 2019 and in the 2021 issue. | Keep this contrary evidence visible when designing guided practice. |

## Generative and reconstruction activities

| Source | Short findings and limits | Access | Twill decision |
| --- | --- | --- | --- |
| [Endres et al., 2024](https://doi.org/10.1016/j.learninstruc.2024.101974) | In 152 undergraduates tested after one week, creating examples aided comprehension, unlike simply recalling supplied examples. This does not establish that combining methods is always better. | Full | Offer planned example creation for an appropriate learning goal. |
| [Obergassel et al., 2025](https://doi.org/10.1037/edu0000949) | This study was registered in advance, with 340 people recruited and 306 analyzed. Combining tasks improved later retention compared with generation alone and comprehension compared with recall alone. It took extra time and showed no advantage for either order. | Full. Tests were immediate or after one week. | Make combined activities optional in planned reconstruction and generation. |
| [Obergassel et al., 2026](https://doi.org/10.1002/acp.70188) | With 231 people recruited and 208 analyzed, neither order of generation and recall showed a reliable advantage at the test after one week. The delay between tasks was immediate or two days. | Full. The methods clarify that the comparison included restudying before generation, despite inconsistent abstract wording. | Do not prescribe one order for planned reconstruction and generation. |
| [Blunt & Karpicke, 2014](https://doi.org/10.1037/a0035934) | Two experiments separated the choice of maps or prose from whether the source was visible. Recall benefited results after one week in both formats, without reliable superiority for either format. | Full | Offer different response formats for planned reconstruction without claiming that maps are always better. |
| [O'Day & Karpicke, 2021](https://doi.org/10.1037/edu0000486) | Adding concept mapping with the source available before recall did not improve results after one week, despite taking more time. | Full. This shows that benefits need not add together. | Avoid compulsory extra stages in planned reconstruction. |
| [Cromley et al., 2020](https://doi.org/10.1007/s10956-019-09807-6) | Across 53 studies and 8,111 participants, drawing effects varied for facts, inferences, and application to new tasks. Different comparison activities prevent treating the overall effect as the benefit of a particular control. | Abstract | Consider drawing in planned labeling and reconstruction without promising a specific interface benefit. |
| [Leutner & Biele, 2025](https://doi.org/10.1007/s10648-025-10067-7) | Across 14 studies and 16 comparisons, drawing support did not help overall. Help with integrating ideas showed promise for comprehension in a subgroup. The subgroup analysis was small and exploratory, with little evidence on application to new tasks. | Full | Keep drawing support restrained in planned reconstruction. |
| [Yu et al., 2025](https://doi.org/10.1007/s10648-025-10024-4) | Across 18 studies and 2,560 participants, recalling answers mentally benefited learning, with g=.23. Speaking or writing responses had a small additional average advantage, with g=.17. | Full. This does not justify requiring typing. | Retain mental recall and explain it in the guide and planned reconstruction. |
| [Harders & Ebersbach, 2026](https://doi.org/10.1002/acp.70174) | This study was registered in advance and matched time across activities, with 208 people analyzed. Explanation did not work better than rereading fictional factual material immediately or after two weeks. | Full. The narrow task and potentially high performance limit interpretation. This is not a general rejection of explanation. | Keep this contrary evidence visible for Explain, the guide, and planned generation activities. |

## Feedback, hints, choice, and presentation

| Source | Short findings and limits | Access | Twill decision |
| --- | --- | --- | --- |
| [Butler et al., 2013](https://doi.org/10.1037/a0031026) | Explanatory feedback improved responses to new inference questions two days later. Results on repeated questions were comparable to feedback with only the answer. One experiment fixed the time available for feedback. | Full | Use explanations in answer feedback and references when reasoning matters. Planned comparison activities can reuse them. |
| [Dunlosky et al., 2011](https://doi.org/10.1080/17470218.2010.502239) | Judging essential ideas reduced overconfident evaluation of one’s own answers. This is not direct proof that a checklist improves lasting retention. | Abstract | Use optional idea checks to compare the original response with the answer. |
| [Froese & Roelle, 2022](https://doi.org/10.1007/s11409-022-09293-z) | Standards based on expert examples helped learners evaluate examples they created. Standards based on separate idea units did not. | Abstract. Requirements must suit the task. | Use Answer or Reference for complete examples. Planned generation activities can reuse these standards. |
| [Fiechter & Benjamin, 2019](https://doi.org/10.3758/s13423-019-01617-6) | Six vocabulary experiments tested learning 12 to 36 hours later. Adaptive cues and cues that gradually diminished involved tradeoffs between effectiveness and time. | Full. This is not direct evidence for hints about complex meaning. | Consider time costs, keep help optional, and record whether the original attempt was assisted. |
| [van den Broek et al., 2019](https://pubmed.ncbi.nlm.nih.gov/30998028/) | In three school vocabulary experiments, elaborate hints took time without improving later recall without cues compared with answer feedback. | Abstract | Retain direct access to the full answer without requiring hints. |
| [Vaughn et al., 2022](https://doi.org/10.1002/acp.3929) | Two small studies on naming skeletal structures included 41 and 32 participants. Strong consonant hints impaired final performance despite being preferred. At least one test followed a filler task lasting two minutes. | Publisher abstract and inspected methods description. This is not evidence of lasting retention. | Keep contrary hint evidence visible for optional help and planned image labeling. |
| [Double et al., 2025](https://doi.org/10.1007/s11409-025-09413-5) | A study registered in advance included 710 people learning categories. Confidence judgments impaired applying rules to new tasks. | Full. This was not a test of later flashcard retention. | Keep planned uncertainty reporting optional. Do not require confidence judgments. |
| [Kandemir et al., 2026](https://doi.org/10.1007/s10648-026-10117-8) | Across 51 studies and 160 effects, immediate feedback compared with delayed feedback had an effect of g=.03, with a confidence interval from negative .08 to .13. Definitions varied, and few studies used long delays. | Publisher abstract | Do not prescribe one feedback delay for every task in answer feedback or optional help. |
| [Smith & Karpicke, 2014](https://pubmed.ncbi.nlm.nih.gov/24059563/) | In four experiments with 372 people, all question formats improved learning after one week. Advantages for short answers or combined formats depended on successful recall. | Abstract | Offer choice and recall alternatives in planned comparison practice. |
| [McDermott et al., 2014](https://www.apa.org/pubs/journals/features/xap-0000004.pdf) | Science and history classroom quizzes with feedback benefited unit exams and exams at the end of the semester. Questions with choices and questions requiring short answers were both useful. | Primary abstract and manuscript excerpts | Keep evidence from later classroom exams visible for planned choice practice. |
| [Butler, 2018](https://doi.org/10.1016/j.jarmac.2018.07.002) | This review connects assessment and learning research. Clear formats, appropriate challenge, and questions that test the intended ability matter. It is not a new controlled trial of effectiveness. | Publisher and author institution abstract | Write clear choices without tricks in planned comparison practice. |
| [Alpizar et al., 2020](https://doi.org/10.1007/s11423-020-09748-7) | Across 29 experiments and 2,726 participants, cues that direct attention improved instructional outcomes on average, with d=.38. Study characteristics affected the result. | Abstract. This is not specifically evidence of lasting flashcard retention. | Use emphasis to identify relevant content in rich text and answer parts. Image labeling remains planned. |
| [Schroeder & Cenkci, 2018](https://doi.org/10.1007/s10648-018-9435-9) | Across 58 comparisons and 2,426 participants, integrating related text and diagrams benefited learning, with g=.63. | Abstract and notes. These were short instructional studies, not general evidence for a compact interface. | Put explanations near the content they describe in rich text and answer parts. Image labeling remains planned. |

## Additional directions for later work

| Source | Short findings and limits | Access | Twill decision |
| --- | --- | --- | --- |
| [Rawson et al., 2020](https://doi.org/10.1007/s10648-020-09528-y) | Across three experiments with 431 college students, relearning procedures over successive sessions had a small advantage on new problems after one week, with d=.28. | Abstract. This differs from large effects on verbal memory. | Keep the limits of Problem and mastery practice visible when designing guided problems and learning records. |
| [Cummings et al.](https://profiles.wustl.edu/en/publications/do-not-forget-the-keyword-method-learning-educational-content-wit/) | Two experiments with arbitrary associations between names and contributions supported combining keyword memory aids and recall practice. | Abstract. Published online in 2022 and in the 2023 volume. This is not general evidence of applying concepts to new tasks. | Offer optional memory aid notes in Hint or answer feedback. |

## Maintaining this register

Add sources used to make a decision, including contrary findings. Summarize only
what was inspected. Do not list every search result or every paper cited by a
research review. A paper is not proof of a benefit merely because it mentions
the same technique. Twill’s editors, text comparison, grading rules, queue
algorithms, scheduling parameters, and interface remain product decisions unless
they are evaluated directly. No study here validates Twill as a whole.
