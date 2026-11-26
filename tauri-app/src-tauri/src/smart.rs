use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
enum CriterionType {
    Benefit,
    Cost,
}

#[derive(Debug, Clone, Copy)]
struct Criterion {
    code: &'static str,
    weight: f64,
    kind: CriterionType,
}

const CRITERIA: [Criterion; 4] = [
    Criterion {
        code: "profit",
        weight: 0.30,
        kind: CriterionType::Benefit,
    },
    Criterion {
        code: "units_sold",
        weight: 0.40,
        kind: CriterionType::Benefit,
    },
    Criterion {
        code: "stock",
        weight: 0.15,
        kind: CriterionType::Cost,
    },
    Criterion {
        code: "purchase_price",
        weight: 0.15,
        kind: CriterionType::Cost,
    },
];

#[derive(Debug, Deserialize)]
pub struct SmartInput {
    pub variant_id: i64,
    pub variant_name: String,
    pub sku: String,
    pub profit: f64,
    pub units_sold: f64,
    pub stock: f64,
    pub purchase_price: f64,
}

#[derive(Debug, Serialize)]
pub struct SmartCriterionResult {
    pub code: String,
    pub weight: f64,
    pub raw_value: f64,
    pub utility_value: f64,
    pub weighted_contribution: f64,
}

#[derive(Debug, Serialize)]
pub struct SmartResult {
    pub variant_id: i64,
    pub variant_name: String,
    pub sku: String,
    pub score: f64,
    pub ranking_position: usize,
    pub criteria: Vec<SmartCriterionResult>,
}

fn utility(value: f64, minimum: f64, maximum: f64, kind: CriterionType) -> f64 {
    if (maximum - minimum).abs() < f64::EPSILON {
        return 1.0;
    }
    let result = match kind {
        CriterionType::Benefit => (value - minimum) / (maximum - minimum),
        CriterionType::Cost => (maximum - value) / (maximum - minimum),
    };
    result.clamp(0.0, 1.0)
}

pub fn calculate(mut inputs: Vec<SmartInput>) -> Vec<SmartResult> {
    if inputs.is_empty() {
        return Vec::new();
    }
    let values: Vec<[f64; 4]> = inputs
        .iter()
        .map(|item| {
            [
                item.profit,
                item.units_sold,
                item.stock,
                item.purchase_price,
            ]
        })
        .collect();
    let minimums = (0..4)
        .map(|index| {
            values
                .iter()
                .map(|row| row[index])
                .fold(f64::INFINITY, f64::min)
        })
        .collect::<Vec<_>>();
    let maximums = (0..4)
        .map(|index| {
            values
                .iter()
                .map(|row| row[index])
                .fold(f64::NEG_INFINITY, f64::max)
        })
        .collect::<Vec<_>>();
    let mut results = inputs
        .drain(..)
        .enumerate()
        .map(|(index, item)| {
            let raw = values[index];
            let criteria = CRITERIA
                .iter()
                .enumerate()
                .map(|(criterion_index, criterion)| {
                    let normalized = utility(
                        raw[criterion_index],
                        minimums[criterion_index],
                        maximums[criterion_index],
                        criterion.kind,
                    );
                    SmartCriterionResult {
                        code: criterion.code.to_string(),
                        weight: criterion.weight,
                        raw_value: raw[criterion_index],
                        utility_value: normalized,
                        weighted_contribution: normalized * criterion.weight,
                    }
                })
                .collect::<Vec<_>>();
            let score = criteria
                .iter()
                .map(|criterion| criterion.weighted_contribution)
                .sum();
            SmartResult {
                variant_id: item.variant_id,
                variant_name: item.variant_name,
                sku: item.sku,
                score,
                ranking_position: 0,
                criteria,
            }
        })
        .collect::<Vec<_>>();
    results.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.variant_id.cmp(&right.variant_id))
    });
    for (index, result) in results.iter_mut().enumerate() {
        result.ranking_position = index + 1;
    }
    results
}

#[cfg(test)]
mod tests {
    use super::{SmartInput, calculate};

    fn input(id: i64, profit: f64, sold: f64, stock: f64, price: f64) -> SmartInput {
        SmartInput {
            variant_id: id,
            variant_name: format!("Variant {id}"),
            sku: format!("SKU-{id}"),
            profit,
            units_sold: sold,
            stock,
            purchase_price: price,
        }
    }

    #[test]
    fn applies_benefit_and_cost_directions() {
        let results = calculate(vec![
            input(1, 100.0, 10.0, 50.0, 20.0),
            input(2, 200.0, 20.0, 10.0, 10.0),
        ]);
        assert_eq!(results[0].variant_id, 2);
        assert_eq!(results[0].ranking_position, 1);
        assert!((results[0].score - 1.0).abs() < f64::EPSILON);
        assert!((results[1].score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn equal_criteria_receive_neutral_utility() {
        let results = calculate(vec![
            input(1, 10.0, 5.0, 3.0, 2.0),
            input(2, 10.0, 5.0, 3.0, 2.0),
        ]);
        assert!((results[0].score - 1.0).abs() < f64::EPSILON);
        assert_eq!(results[0].score, results[1].score);
    }
}
