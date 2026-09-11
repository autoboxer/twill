use std::collections::BTreeSet;

use serde_json::json;

use super::ID_BATCH_SIZE;
use crate::data::LocalDataStore;
use crate::library::{
    ConceptLibrary, CreateTemplateInput, RecordReviewInput, ReviewRating, TemplateLibrary,
};

#[test]
fn queue_shares_documents_and_templates_without_losing_form_configuration() {
    let directory = tempfile::tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let library = ConceptLibrary::new(&store);
    let template = TemplateLibrary::new(&store)
        .create_template(CreateTemplateInput {
            name: "Shared template".into(),
            content: Default::default(),
        })
        .unwrap();
    let prompt = json!({ "type": "doc", "content": [{ "type": "paragraph", "content": [
        { "type": "text", "text": "Shared content. ".repeat(500) }
    ] }] });
    let first = library
        .create_concept(
            serde_json::from_value(json!({
                "title": "First", "templateIds": [template.id],
                "content": { "schemaVersion": 1, "prompt": prompt, "answer": prompt },
                "typeAnswer": { "acceptedAnswers": ["Exact answer"] },
                "explain": { "focus": "why", "keyPoints": ["Key point"] },
                "problem": { "checkpoints": ["Checkpoint"] }
            }))
            .unwrap(),
        )
        .unwrap();
    let second = library
        .create_concept(
            serde_json::from_value(json!({
                "title": "Second", "templateIds": [template.id]
            }))
            .unwrap(),
        )
        .unwrap();
    let queue = library.study_queue().unwrap();

    assert_eq!(queue.cards.len(), first.cards.len() + second.cards.len());
    assert_eq!(queue.concepts.len(), 2);
    assert_eq!(queue.templates.len(), 1);
    assert_eq!(queue.templates[0].id, template.id);
    assert_eq!(
        queue
            .concepts
            .iter()
            .find(|concept| concept.id == first.id)
            .unwrap()
            .content,
        first.content
    );

    for card in &first.cards {
        let loaded = queue
            .cards
            .iter()
            .find(|candidate| candidate.id == card.id)
            .unwrap();
        assert_eq!(loaded.concept_id, first.id);
        assert_eq!(
            loaded.template_id,
            card.template.as_ref().map(|template| template.id.clone())
        );
        assert_eq!(loaded.type_answer, card.type_answer);
        assert_eq!(loaded.explain, card.explain);
        assert_eq!(loaded.problem, card.problem);
    }

    let wire = serde_json::to_value(&queue).unwrap();
    assert!(wire["cards"]
        .as_array()
        .unwrap()
        .iter()
        .all(|card| card.get("content").is_none() && card.get("template").is_none()));
    let shared_bytes = serde_json::to_vec(&queue.concepts).unwrap().len();
    let old_document_bytes = serde_json::to_vec(&first.content).unwrap().len() * first.cards.len();
    assert!(shared_bytes * 3 < old_document_bytes);

    for card in &first.cards {
        library
            .record_review(RecordReviewInput {
                card_id: card.id.clone(),
                rating: ReviewRating::Good,
            })
            .unwrap();
    }
    let remaining = library.study_queue().unwrap();
    assert_eq!(remaining.concepts.len(), 1);
    assert_eq!(remaining.concepts[0].id, second.id);
    assert_eq!(remaining.templates.len(), 1);

    library.set_concept_archived(&second.id, true).unwrap();
    let empty = library.study_queue().unwrap();
    assert!(empty.cards.is_empty());
    assert!(empty.concepts.is_empty());
    assert!(empty.templates.is_empty());
    assert!(empty.next_due_at.is_some());
    assert_eq!(empty.total_cards, first.cards.len() as i64);
}

#[test]
fn batched_content_reads_keep_complete_membership_and_mixed_order() {
    let directory = tempfile::tempdir().unwrap();
    let store = LocalDataStore::open(directory.path()).unwrap();
    let library = ConceptLibrary::new(&store);
    let tag = library.create_tag("Shared tag".into()).unwrap();

    for index in 0..=ID_BATCH_SIZE {
        library
            .create_concept(
                serde_json::from_value(json!({
                    "title": format!("Concept {index}"), "tagIds": [tag.id],
                }))
                .unwrap(),
            )
            .unwrap();
    }

    let ordered = library.study_queue().unwrap();
    assert_eq!(ordered.cards.len(), ID_BATCH_SIZE + 1);
    assert_eq!(ordered.concepts.len(), ID_BATCH_SIZE + 1);
    assert!(ordered
        .cards
        .windows(2)
        .all(|pair| { (pair[0].due_at, &pair[0].id) <= (pair[1].due_at, &pair[1].id) }));
    library.set_mixed_practice_enabled(true).unwrap();
    let mixed = library.study_queue().unwrap();
    let card_ids = |queue: &crate::library::StudyQueue| {
        queue
            .cards
            .iter()
            .map(|card| &card.id)
            .cloned()
            .collect::<BTreeSet<_>>()
    };

    assert_eq!(card_ids(&mixed), card_ids(&ordered));
    assert_eq!(mixed.concepts, ordered.concepts);
    assert_eq!(mixed, library.study_queue().unwrap());
    assert!(mixed.mixed_practice_enabled);
}
