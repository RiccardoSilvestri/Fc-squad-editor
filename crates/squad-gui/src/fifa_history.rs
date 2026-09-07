use std::collections::HashMap;
use std::path::Path;

use squad_core::FieldValue;

pub struct HistoricalCard {
    pub fifa_version: i32,
    pub fields: HashMap<String, String>,
}

pub struct HistoricalDatabase {
    cards: HashMap<i64, Vec<HistoricalCard>>,
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                current.push(c);
            }
        } else if c == '"' {
            in_quotes = true;
        } else if c == ',' {
            out.push(std::mem::take(&mut current));
        } else {
            current.push(c);
        }
    }
    out.push(current);
    out
}

impl HistoricalDatabase {
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut lines = content.lines();
        let header = lines.next().unwrap_or_default();
        let columns = parse_csv_line(header);
        let player_id_idx = columns.iter().position(|c| c == "player_id").unwrap_or(0);
        let fifa_version_idx = columns.iter().position(|c| c == "fifa_version");
        let fifa_update_idx = columns.iter().position(|c| c == "fifa_update");

        let mut cards: HashMap<i64, Vec<HistoricalCard>> = HashMap::new();
        let mut latest_update: HashMap<(i64, i32), i32> = HashMap::new();

        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            let values = parse_csv_line(line);
            let Some(player_id_str) = values.get(player_id_idx) else {
                continue;
            };
            let Ok(player_id) = player_id_str.parse::<i64>() else {
                continue;
            };
            let fifa_version = fifa_version_idx
                .and_then(|i| values.get(i))
                .and_then(|v| v.parse::<i32>().ok())
                .unwrap_or(0);
            let fifa_update = fifa_update_idx
                .and_then(|i| values.get(i))
                .and_then(|v| v.parse::<i32>().ok())
                .unwrap_or(0);

            let key = (player_id, fifa_version);
            let is_newer = fifa_update >= *latest_update.get(&key).unwrap_or(&-1);
            if !is_newer {
                continue;
            }
            latest_update.insert(key, fifa_update);

            let mut fields = HashMap::with_capacity(columns.len());
            for (i, col) in columns.iter().enumerate() {
                if let Some(v) = values.get(i) {
                    fields.insert(col.clone(), v.clone());
                }
            }
            let card = HistoricalCard {
                fifa_version,
                fields,
            };

            let entry = cards.entry(player_id).or_default();
            if let Some(pos) = entry.iter().position(|c| c.fifa_version == fifa_version) {
                entry[pos] = card;
            } else {
                entry.push(card);
            }
        }

        for list in cards.values_mut() {
            list.sort_by_key(|card| std::cmp::Reverse(card.fifa_version));
        }

        Ok(HistoricalDatabase { cards })
    }

    pub fn versions_for(&self, player_id: i64) -> Vec<i32> {
        self.cards
            .get(&player_id)
            .map(|list| list.iter().map(|c| c.fifa_version).collect())
            .unwrap_or_default()
    }

    pub fn card_for(&self, player_id: i64, fifa_version: i32) -> Option<&HistoricalCard> {
        self.cards
            .get(&player_id)?
            .iter()
            .find(|c| c.fifa_version == fifa_version)
    }

    pub fn player_count(&self) -> usize {
        self.cards.len()
    }
}

const DIRECT_FIELD_MAP: &[(&str, &str)] = &[
    ("overall", "overallrating"),
    ("potential", "potential"),
    ("height_cm", "height"),
    ("weight_kg", "weight"),
    ("weak_foot", "weakfootabilitytypecode"),
    ("skill_moves", "skillmoves"),
    ("attacking_crossing", "crossing"),
    ("attacking_finishing", "finishing"),
    ("attacking_heading_accuracy", "headingaccuracy"),
    ("attacking_short_passing", "shortpassing"),
    ("attacking_volleys", "volleys"),
    ("skill_dribbling", "dribbling"),
    ("skill_curve", "curve"),
    ("skill_fk_accuracy", "freekickaccuracy"),
    ("skill_long_passing", "longpassing"),
    ("skill_ball_control", "ballcontrol"),
    ("movement_acceleration", "acceleration"),
    ("movement_sprint_speed", "sprintspeed"),
    ("movement_agility", "agility"),
    ("movement_reactions", "reactions"),
    ("movement_balance", "balance"),
    ("power_shot_power", "shotpower"),
    ("power_jumping", "jumping"),
    ("power_stamina", "stamina"),
    ("power_strength", "strength"),
    ("power_long_shots", "longshots"),
    ("mentality_aggression", "aggression"),
    ("mentality_interceptions", "interceptions"),
    ("mentality_positioning", "positioning"),
    ("mentality_vision", "vision"),
    ("mentality_penalties", "penalties"),
    ("mentality_composure", "composure"),
    ("defending_marking_awareness", "defensiveawareness"),
    ("defending_standing_tackle", "standingtackle"),
    ("defending_sliding_tackle", "slidingtackle"),
    ("goalkeeping_diving", "gkdiving"),
    ("goalkeeping_handling", "gkhandling"),
    ("goalkeeping_kicking", "gkkicking"),
    ("goalkeeping_positioning", "gkpositioning"),
    ("goalkeeping_reflexes", "gkreflexes"),
];

pub struct ApplyReport {
    pub skipped: usize,
}

pub fn apply_card_to_field_values(
    card: &HistoricalCard,
) -> (Vec<(String, FieldValue)>, ApplyReport) {
    let mut updates = Vec::new();
    let mut skipped = 0;

    for (csv_col, ea_field) in DIRECT_FIELD_MAP {
        match card
            .fields
            .get(*csv_col)
            .and_then(|v| v.trim().parse::<i64>().ok())
        {
            Some(v) => updates.push((ea_field.to_string(), FieldValue::Int(v))),
            None => skipped += 1,
        }
    }

    if let Some(foot) = card.fields.get("preferred_foot") {
        let value = match foot.trim() {
            "Left" => Some(0i64),
            "Right" => Some(1i64),
            _ => None,
        };
        match value {
            Some(v) => updates.push(("preferredfoot".to_string(), FieldValue::Int(v))),
            None => skipped += 1,
        }
    }

    if let Some(rep) = card
        .fields
        .get("international_reputation")
        .and_then(|v| v.trim().parse::<i64>().ok())
    {
        updates.push((
            "internationalrep".to_string(),
            FieldValue::Int((rep - 1).max(0)),
        ));
    } else {
        skipped += 1;
    }

    (updates, ApplyReport { skipped })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_quoted_fields_with_commas() {
        let line = r#"1,"CF, ST",normal"#;
        let fields = parse_csv_line(line);
        assert_eq!(fields, vec!["1", "CF, ST", "normal"]);
    }

    #[test]
    fn loads_and_finds_card_by_version() {
        let csv = "player_id,fifa_version,fifa_update,overall,preferred_foot\n158023,23,9,91,Left\n158023,22,5,93,Left\n";
        let dir = std::env::temp_dir();
        let path = dir.join("test_fifa_history.csv");
        std::fs::write(&path, csv).unwrap();

        let db = HistoricalDatabase::load(&path).unwrap();
        let mut versions = db.versions_for(158023);
        versions.sort();
        assert_eq!(versions, vec![22, 23]);

        let card = db.card_for(158023, 22).unwrap();
        assert_eq!(card.fields.get("overall").unwrap(), "93");

        let (updates, report) = apply_card_to_field_values(card);
        assert!(updates.contains(&("overallrating".to_string(), FieldValue::Int(93))));
        assert!(updates.contains(&("preferredfoot".to_string(), FieldValue::Int(0))));
        assert!(!updates.is_empty());
        assert!(report.skipped > 0);

        std::fs::remove_file(&path).ok();
    }
}
