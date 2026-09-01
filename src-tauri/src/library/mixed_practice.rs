use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::library::StudyCard;

const PRIORITY_GROUP_SIZE: usize = 12;

pub fn mix_due_cards(
    cards: Vec<StudyCard>,
    concept_tags: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<StudyCard> {
    let mut remaining = VecDeque::from(cards);
    let mut mixed = Vec::with_capacity(remaining.len());

    while !remaining.is_empty() {
        let group_size = remaining.len().min(PRIORITY_GROUP_SIZE);
        let mut group = (0..group_size)
            .filter_map(|_| remaining.pop_front())
            .collect::<Vec<_>>();

        while !group.is_empty() {
            let next_index = next_card_index(&group, mixed.last(), concept_tags);

            mixed.push(group.remove(next_index));
        }
    }

    mixed
}

fn next_card_index(
    cards: &[StudyCard],
    previous: Option<&StudyCard>,
    concept_tags: &BTreeMap<String, BTreeSet<String>>,
) -> usize {
    let Some(previous) = previous else {
        return 0;
    };

    cards
        .iter()
        .enumerate()
        .max_by_key(|(index, candidate)| {
            let changes_concept = candidate.concept_id != previous.concept_id;
            let shared_tag_count = if changes_concept {
                shared_tag_count(
                    concept_tags.get(&previous.concept_id),
                    concept_tags.get(&candidate.concept_id),
                )
            } else {
                0
            };

            (
                changes_concept,
                shared_tag_count,
                candidate.retrieval_kind != previous.retrieval_kind,
                candidate.scheduling_state != previous.scheduling_state,
                Reverse(*index),
            )
        })
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn shared_tag_count(first: Option<&BTreeSet<String>>, second: Option<&BTreeSet<String>>) -> usize {
    match (first, second) {
        (Some(first), Some(second)) => first.intersection(second).count(),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::{RetrievalFormKind, SchedulingState, StudyCard};

    fn card(
        id: &str,
        concept_id: &str,
        retrieval_kind: RetrievalFormKind,
        scheduling_state: SchedulingState,
    ) -> StudyCard {
        StudyCard {
            id: id.to_owned(),
            concept_id: concept_id.to_owned(),
            concept_last_change_id: format!("change-{concept_id}"),
            concept_title: concept_id.to_owned(),
            content: Default::default(),
            retrieval_kind,
            explain: None,
            problem: None,
            cloze: None,
            image_occlusion: None,
            type_answer: None,
            template: None,
            scheduling_state,
            due_at: 0,
            pretest_eligible: false,
        }
    }

    #[test]
    fn mixed_groups_separate_concepts_and_prefer_authored_contrast() {
        let cards = vec![
            card(
                "a-recall",
                "a",
                RetrievalFormKind::Recall,
                SchedulingState::New,
            ),
            card(
                "a-explain",
                "a",
                RetrievalFormKind::Explain,
                SchedulingState::New,
            ),
            card(
                "unrelated",
                "b",
                RetrievalFormKind::Recall,
                SchedulingState::New,
            ),
            card(
                "contrast",
                "c",
                RetrievalFormKind::Problem,
                SchedulingState::Review,
            ),
        ];
        let concept_tags = BTreeMap::from([
            ("a".to_owned(), BTreeSet::from(["shared".to_owned()])),
            ("c".to_owned(), BTreeSet::from(["shared".to_owned()])),
        ]);

        let ids = mix_due_cards(cards, &concept_tags)
            .into_iter()
            .map(|card| card.id)
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["a-recall", "contrast", "a-explain", "unrelated"]);
    }

    #[test]
    fn mixed_groups_keep_stable_due_priority_when_no_signal_applies() {
        let cards = vec![
            card(
                "first",
                "a",
                RetrievalFormKind::Recall,
                SchedulingState::Review,
            ),
            card(
                "second",
                "b",
                RetrievalFormKind::Recall,
                SchedulingState::Review,
            ),
            card(
                "third",
                "c",
                RetrievalFormKind::Recall,
                SchedulingState::Review,
            ),
        ];

        let ids = mix_due_cards(cards, &BTreeMap::new())
            .into_iter()
            .map(|card| card.id)
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["first", "second", "third"]);
    }

    #[test]
    fn mixing_does_not_move_cards_between_due_priority_groups() {
        let mut cards = (0..PRIORITY_GROUP_SIZE)
            .map(|index| {
                card(
                    &format!("priority-{index}"),
                    &format!("priority-{index}"),
                    RetrievalFormKind::Recall,
                    SchedulingState::Review,
                )
            })
            .collect::<Vec<_>>();
        cards.push(card(
            "later-contrast",
            "later",
            RetrievalFormKind::Problem,
            SchedulingState::New,
        ));
        let concept_tags = BTreeMap::from([
            (
                "priority-0".to_owned(),
                BTreeSet::from(["shared".to_owned()]),
            ),
            ("later".to_owned(), BTreeSet::from(["shared".to_owned()])),
        ]);

        let ids = mix_due_cards(cards, &concept_tags)
            .into_iter()
            .map(|card| card.id)
            .collect::<Vec<_>>();

        assert_eq!(ids.last().map(String::as_str), Some("later-contrast"));
    }
}
