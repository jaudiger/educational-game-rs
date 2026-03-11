use std::collections::HashSet;

use rand::Rng;
use rand::seq::{IndexedRandom, SliceRandom};

use super::types::{
    ComparisonAnswer, ComparisonDifficulty, ComparisonSide, ComparisonSideWithConversion,
    ExplanationVisual, FractionComparisonDefinition, FractionComparisonTemplate,
    FractionIdentificationDefinition, FractionIdentificationTemplate,
    FractionVisualizationDefinition, FractionVisualizationTemplate, LocalizedText, McqDefinition,
    McqResolver, McqTemplate, NumericInputDefinition, NumericInputTemplate, ParameterRange,
    QuestionVisual,
};

impl McqTemplate {
    fn param_values(&self, name: &str, default: &[i32]) -> Vec<i32> {
        self.parameters
            .iter()
            .find(|p: &&ParameterRange| p.name == name)
            .map_or_else(|| default.to_vec(), |p| p.values.clone())
    }

    /// Dispatch to the resolver strategy matching `self.resolver`, producing a
    /// concrete `McqDefinition` from randomized parameters.
    pub fn resolve(&self, rng: &mut impl Rng) -> McqDefinition {
        match self.resolver {
            McqResolver::FractionValue => self.resolve_fraction_value(rng),
            McqResolver::FractionAddition => self.resolve_fraction_addition(rng),
            McqResolver::FractionName => self.resolve_fraction_name(rng),
            McqResolver::Multiplication | McqResolver::MultiplyByPowerOf10 => {
                self.resolve_multiplication(rng)
            }
        }
    }

    #[allow(clippy::literal_string_with_formatting_args)]
    fn resolve_fraction_value(&self, rng: &mut impl Rng) -> McqDefinition {
        let a_values = self.param_values("a", &[2]);
        let b_values = self.param_values("b", &[2]);

        // Keep only pairs where a is exactly divisible by b.
        let valid_pairs: Vec<(i32, i32)> = a_values
            .iter()
            .flat_map(|&a| b_values.iter().map(move |&b| (a, b)))
            .filter(|&(a, b)| b != 0 && a % b == 0)
            .collect();

        let &(a, b) = valid_pairs.choose(rng).unwrap_or(&(2, 1));
        let result = a / b;

        let prompt = LocalizedText::new(
            substitute_template(&self.prompt_template.fr, &[("a", a), ("b", b)]),
            substitute_template(&self.prompt_template.en, &[("a", a), ("b", b)]),
        );

        // Build 4 unique choices including the correct answer.
        let mut seen: HashSet<i32> = HashSet::from([result]);
        for &candidate in &[a, b, result * 2, result + 1] {
            if candidate > 0 {
                seen.insert(candidate);
            }
            if seen.len() >= 4 {
                break;
            }
        }
        let mut fill = 1;
        while seen.len() < 4 {
            seen.insert(fill);
            fill += 1;
        }

        let mut choices: Vec<String> = seen.into_iter().map(|v| v.to_string()).collect();
        choices.shuffle(rng);
        let correct_str = result.to_string();
        let correct_index = choices.iter().position(|c| *c == correct_str).unwrap_or(0);

        let explanation = LocalizedText::new(
            substitute_template(&self.explanation_template.fr, &[("a", a), ("b", b)])
                .replace("{result}", &result.to_string()),
            substitute_template(&self.explanation_template.en, &[("a", a), ("b", b)])
                .replace("{result}", &result.to_string()),
        );

        McqDefinition {
            prompt,
            choices,
            correct_index,
            explanation,
            explanation_visual: Some(ExplanationVisual::WholeFractions {
                count: result.unsigned_abs(),
                denominator: b.unsigned_abs(),
            }),
            question_visual: None,
        }
    }

    /// Resolve a same-denominator fraction addition MCQ.
    ///
    /// Expected parameters: `a` (numerator of first fraction),
    /// `b` (shared denominator), `c` (numerator of second fraction).
    /// Correct answer: `(a+c)/b`.
    #[allow(clippy::literal_string_with_formatting_args)]
    fn resolve_fraction_addition(&self, rng: &mut impl Rng) -> McqDefinition {
        let a_values = self.param_values("a", &[1]);
        let b_values = self.param_values("b", &[4]);
        let c_values = self.param_values("c", &[1]);

        // Valid triplets: a > 0, c > 0, b > 0, a < b, c < b, and a+c <= b
        // (no improper fractions in the result).
        let mut valid_triplets: Vec<(i32, i32, i32)> = Vec::new();
        for &a in &a_values {
            for &b in &b_values {
                for &c in &c_values {
                    if a > 0 && b > 0 && c > 0 && a < b && c < b && a + c <= b {
                        valid_triplets.push((a, b, c));
                    }
                }
            }
        }

        let &(a, b, c) = valid_triplets.choose(rng).unwrap_or(&(1, 4, 1));
        let sum = a + c;

        let prompt = LocalizedText::new(
            substitute_template(&self.prompt_template.fr, &[("a", a), ("b", b), ("c", c)]),
            substitute_template(&self.prompt_template.en, &[("a", a), ("b", b), ("c", c)]),
        );

        let (choices, correct_index) = generate_fraction_addition_choices(a, b, c, sum, rng);

        let explanation = LocalizedText::new(
            substitute_template(
                &self.explanation_template.fr,
                &[("a", a), ("b", b), ("c", c), ("sum", sum)],
            )
            .replace("{result}", &format!("{sum}/{b}")),
            substitute_template(
                &self.explanation_template.en,
                &[("a", a), ("b", b), ("c", c), ("sum", sum)],
            )
            .replace("{result}", &format!("{sum}/{b}")),
        );

        McqDefinition {
            prompt,
            choices,
            correct_index,
            explanation,
            explanation_visual: Some(ExplanationVisual::FractionAddition {
                a: a.unsigned_abs(),
                b: b.unsigned_abs(),
                c: c.unsigned_abs(),
            }),
            question_visual: Some(QuestionVisual::FractionAddition {
                a: a.unsigned_abs(),
                b: b.unsigned_abs(),
                c: c.unsigned_abs(),
            }),
        }
    }

    /// Resolve a "name the fraction" MCQ.
    ///
    /// Expected parameter: `d` (denominator of a unit fraction 1/d).
    /// The resolver maps `d` to a localized name (e.g. 2 = "la moitie" /
    /// "one half") and substitutes `{name}` in both prompt and explanation
    /// templates. Correct answer: `1/d`.
    #[allow(clippy::literal_string_with_formatting_args)]
    fn resolve_fraction_name(&self, rng: &mut impl Rng) -> McqDefinition {
        let d_values = self.param_values("d", &[2, 3, 4]);

        let &d = d_values.choose(rng).unwrap_or(&2);

        let (name_fr, name_en) = fraction_name(d);

        let prompt = LocalizedText::new(
            self.prompt_template.fr.replace("{name}", name_fr),
            self.prompt_template.en.replace("{name}", name_en),
        );

        let correct = format!("1/{d}");
        let mut choices: Vec<String> = vec![correct.clone()];

        // Inverted fraction.
        let d1 = format!("{d}/1");
        if !choices.contains(&d1) {
            choices.push(d1);
        }
        // Other unit fractions as distractors.
        for &other_d in &[2, 3, 4, 5, 6, 8] {
            if choices.len() >= 4 {
                break;
            }
            if other_d != d {
                let dist = format!("1/{other_d}");
                if !choices.contains(&dist) {
                    choices.push(dist);
                }
            }
        }
        choices.truncate(4);

        choices.shuffle(rng);
        let correct_index = choices.iter().position(|ch| *ch == correct).unwrap_or(0);

        let d_str = d.to_string();
        let explanation = LocalizedText::new(
            self.explanation_template
                .fr
                .replace("{name}", name_fr)
                .replace("{fraction}", &correct)
                .replace("{d}", &d_str),
            self.explanation_template
                .en
                .replace("{name}", name_en)
                .replace("{fraction}", &correct)
                .replace("{d}", &d_str),
        );

        McqDefinition {
            prompt,
            choices,
            correct_index,
            explanation,
            #[allow(clippy::cast_sign_loss)]
            explanation_visual: Some(ExplanationVisual::FractionBar {
                numerator: 1,
                denominator: d as u32,
            }),
            question_visual: None,
        }
    }

    /// Resolve a multiplication MCQ (`a * b`).
    ///
    /// Expected parameters: `a` and `b`.
    /// Correct answer: `a * b` (string). Distractors are neighbouring products.
    #[allow(clippy::literal_string_with_formatting_args)]
    fn resolve_multiplication(&self, rng: &mut impl Rng) -> McqDefinition {
        let a_values = self.param_values("a", &[2]);
        let b_values = self.param_values("b", &[3]);

        let &a = a_values.choose(rng).unwrap_or(&2);
        let &b = b_values.choose(rng).unwrap_or(&3);
        let result = a * b;

        let prompt = LocalizedText::new(
            substitute_template(&self.prompt_template.fr, &[("a", a), ("b", b)]),
            substitute_template(&self.prompt_template.en, &[("a", a), ("b", b)]),
        );

        #[allow(clippy::cast_sign_loss)]
        let (choices, correct_index) = generate_multiplication_distractors(a as u32, b as u32, rng);

        let explanation = LocalizedText::new(
            substitute_template(&self.explanation_template.fr, &[("a", a), ("b", b)])
                .replace("{result}", &result.to_string()),
            substitute_template(&self.explanation_template.en, &[("a", a), ("b", b)])
                .replace("{result}", &result.to_string()),
        );

        #[allow(clippy::cast_sign_loss)]
        let visual = if matches!(self.resolver, McqResolver::MultiplyByPowerOf10) {
            ExplanationVisual::PlaceValueTable {
                number: a as u32,
                multiplier: b as u32,
            }
        } else {
            ExplanationVisual::MultiplicationGrid {
                rows: a as u32,
                cols: b as u32,
            }
        };

        #[allow(clippy::cast_sign_loss)]
        let question_visual = if self.with_grid {
            Some(QuestionVisual::MultiplicationGrid {
                rows: a as u32,
                cols: b as u32,
            })
        } else {
            None
        };

        McqDefinition {
            prompt,
            choices,
            correct_index,
            explanation,
            explanation_visual: Some(visual),
            question_visual,
        }
    }
}

/// Map a denominator to its common French / English name.
const fn fraction_name(denominator: i32) -> (&'static str, &'static str) {
    match denominator {
        2 => ("la moitié", "one half"),
        3 => ("le tiers", "one third"),
        4 => ("le quart", "one quarter"),
        5 => ("le cinquième", "one fifth"),
        6 => ("le sixième", "one sixth"),
        8 => ("le huitième", "one eighth"),
        _ => ("la fraction", "the fraction"),
    }
}

impl FractionVisualizationTemplate {
    /// Pick a random numerator/denominator pair from the configured ranges and
    /// produce a `FractionVisualizationDefinition`.
    pub fn resolve(&self, rng: &mut impl Rng) -> FractionVisualizationDefinition {
        let denominator = *self.denominator_range.choose(rng).unwrap_or(&4);

        let valid_nums: Vec<u32> = self
            .numerator_range
            .iter()
            .copied()
            .filter(|&n| n > 0 && n <= denominator)
            .collect();
        let numerator = *valid_nums.choose(rng).unwrap_or(&1);

        let num_str = numerator.to_string();
        let den_str = denominator.to_string();

        let prompt = LocalizedText::new(
            self.prompt_template
                .fr
                .replace("{n}", &num_str)
                .replace("{d}", &den_str),
            self.prompt_template
                .en
                .replace("{n}", &num_str)
                .replace("{d}", &den_str),
        );

        let explanation = LocalizedText::new(
            self.explanation_template
                .fr
                .replace("{n}", &num_str)
                .replace("{d}", &den_str),
            self.explanation_template
                .en
                .replace("{n}", &num_str)
                .replace("{d}", &den_str),
        );

        FractionVisualizationDefinition {
            prompt,
            numerator,
            denominator,
            explanation,
        }
    }
}

impl FractionComparisonTemplate {
    /// Generate two fractions according to the difficulty tier and determine
    /// which side is larger.
    #[allow(clippy::literal_string_with_formatting_args)]
    pub fn resolve(&self, rng: &mut impl Rng) -> FractionComparisonDefinition {
        let (fraction_a, fraction_b) = match self.difficulty {
            ComparisonDifficulty::SameDenominator => {
                let d = *self.denominator_range.choose(rng).unwrap_or(&8);
                let mut valid_nums: Vec<u32> = self
                    .numerator_range
                    .iter()
                    .copied()
                    .filter(|&n| n > 0 && n < d)
                    .collect();
                valid_nums.shuffle(rng);
                let na = valid_nums.first().copied().unwrap_or(1);
                let nb = valid_nums.get(1).copied().unwrap_or(2);
                ((na, d), (nb, d))
            }
            ComparisonDifficulty::MultipleDenominator => {
                // Pick two distinct denominators where one divides the other.
                let mut denom_pairs: Vec<(u32, u32)> = Vec::new();
                for &d1 in &self.denominator_range {
                    for &d2 in &self.denominator_range {
                        if d1 != d2 && (d1 % d2 == 0 || d2 % d1 == 0) {
                            denom_pairs.push((d1, d2));
                        }
                    }
                }
                denom_pairs.shuffle(rng);
                let &(da, db) = denom_pairs.first().unwrap_or(&(4, 8));

                let mut nums_for_a: Vec<u32> = self
                    .numerator_range
                    .iter()
                    .copied()
                    .filter(|&n| n > 0 && n < da)
                    .collect();
                let mut nums_for_b: Vec<u32> = self
                    .numerator_range
                    .iter()
                    .copied()
                    .filter(|&n| n > 0 && n < db)
                    .collect();
                nums_for_a.shuffle(rng);
                nums_for_b.shuffle(rng);
                let na = nums_for_a.first().copied().unwrap_or(1);
                let nb = nums_for_b.first().copied().unwrap_or(1);
                ((na, da), (nb, db))
            }
            ComparisonDifficulty::SameNumerator => {
                let n = *self.numerator_range.choose(rng).unwrap_or(&3);
                let mut valid_dens: Vec<u32> = self
                    .denominator_range
                    .iter()
                    .copied()
                    .filter(|&d| d > n)
                    .collect();
                valid_dens.shuffle(rng);
                let da = valid_dens.first().copied().unwrap_or(n + 1);
                let db = valid_dens.get(1).copied().unwrap_or(n + 2);
                ((n, da), (n, db))
            }
        };

        // Determine which side is larger.
        let val_a = f64::from(fraction_a.0) / f64::from(fraction_a.1);
        let val_b = f64::from(fraction_b.0) / f64::from(fraction_b.1);
        let answer = if (val_a - val_b).abs() < f64::EPSILON {
            ComparisonAnswer::Equal
        } else if val_a > val_b {
            ComparisonAnswer::A
        } else {
            ComparisonAnswer::B
        };

        let num_a = fraction_a.0.to_string();
        let den_a = fraction_a.1.to_string();
        let num_b = fraction_b.0.to_string();
        let den_b = fraction_b.1.to_string();

        let substitute_explanation = |template: &str| {
            template
                .replace("{na}", &num_a)
                .replace("{da}", &den_a)
                .replace("{nb}", &num_b)
                .replace("{db}", &den_b)
                .replace("{char_a}", &self.character_a)
                .replace("{char_b}", &self.character_b)
        };

        let explanation = LocalizedText::new(
            substitute_explanation(&self.explanation_template.fr),
            substitute_explanation(&self.explanation_template.en),
        );

        let explanation_visual = Some(self.build_visual(fraction_a, fraction_b));

        FractionComparisonDefinition {
            prompt: self.prompt.clone(),
            character_a: self.character_a.clone(),
            fraction_a,
            character_b: self.character_b.clone(),
            fraction_b,
            answer,
            difficulty: self.difficulty,
            explanation,
            explanation_visual,
        }
    }

    fn build_visual(&self, fraction_a: (u32, u32), fraction_b: (u32, u32)) -> ExplanationVisual {
        match self.difficulty {
            ComparisonDifficulty::SameDenominator | ComparisonDifficulty::SameNumerator => {
                ExplanationVisual::FractionComparison {
                    a: ComparisonSide {
                        character: self.character_a.clone(),
                        fraction: fraction_a,
                    },
                    b: ComparisonSide {
                        character: self.character_b.clone(),
                        fraction: fraction_b,
                    },
                }
            }
            ComparisonDifficulty::MultipleDenominator => {
                let common_d = fraction_a.1.max(fraction_b.1);
                ExplanationVisual::FractionComparisonWithConversion {
                    a: ComparisonSideWithConversion {
                        character: self.character_a.clone(),
                        fraction: fraction_a,
                        converted: (fraction_a.0 * (common_d / fraction_a.1), common_d),
                    },
                    b: ComparisonSideWithConversion {
                        character: self.character_b.clone(),
                        fraction: fraction_b,
                        converted: (fraction_b.0 * (common_d / fraction_b.1), common_d),
                    },
                }
            }
        }
    }
}

impl FractionIdentificationTemplate {
    /// Pick a random fraction from the configured ranges and build the
    /// identification MCQ with distractors.
    pub fn resolve(&self, rng: &mut impl Rng) -> FractionIdentificationDefinition {
        let denominator = *self.denominator_range.choose(rng).unwrap_or(&4);

        let valid_nums: Vec<u32> = self
            .numerator_range
            .iter()
            .copied()
            .filter(|&n| n > 0 && n <= denominator)
            .collect();
        let numerator = *valid_nums.choose(rng).unwrap_or(&1);

        generate_fraction_identification(numerator, denominator, rng)
    }
}

impl NumericInputTemplate {
    /// Pick random factors from the configured ranges and build a numeric-input
    /// multiplication question.
    #[allow(clippy::literal_string_with_formatting_args)]
    pub fn resolve(&self, rng: &mut impl Rng) -> NumericInputDefinition {
        let a = *self.factor_a_range.choose(rng).unwrap_or(&2);
        let b = *self.factor_b_range.choose(rng).unwrap_or(&3);
        let correct_answer = a * b;

        let a_str = a.to_string();
        let b_str = b.to_string();
        let result_str = correct_answer.to_string();

        let prompt = LocalizedText::new(
            self.prompt_template
                .fr
                .replace("{a}", &a_str)
                .replace("{b}", &b_str),
            self.prompt_template
                .en
                .replace("{a}", &a_str)
                .replace("{b}", &b_str),
        );
        let explanation = LocalizedText::new(
            self.explanation_template
                .fr
                .replace("{a}", &a_str)
                .replace("{b}", &b_str)
                .replace("{result}", &result_str),
            self.explanation_template
                .en
                .replace("{a}", &a_str)
                .replace("{b}", &b_str)
                .replace("{result}", &result_str),
        );

        let question_visual = if self.with_grid {
            Some(QuestionVisual::MultiplicationGrid { rows: a, cols: b })
        } else {
            None
        };

        NumericInputDefinition {
            prompt,
            correct_answer,
            explanation,
            explanation_visual: Some(if self.place_value_explanation {
                ExplanationVisual::PlaceValueTable {
                    number: a,
                    multiplier: b,
                }
            } else {
                ExplanationVisual::MultiplicationGrid { rows: a, cols: b }
            }),
            question_visual,
        }
    }
}

/// Build 4 shuffled MCQ choices for a same-denominator fraction addition
/// `(a+c)/b`, including the correct answer and distractors.
/// Returns `(choices, correct_index)`.
fn generate_fraction_addition_choices(
    a: i32,
    b: i32,
    c: i32,
    sum: i32,
    rng: &mut impl Rng,
) -> (Vec<String>, usize) {
    let correct = format!("{sum}/{b}");
    let mut seen: HashSet<String> = HashSet::from([correct.clone()]);

    // Distractor: adding denominators too (common student mistake).
    seen.insert(format!("{sum}/{}", b * 2));
    // Distractor: first operand only.
    seen.insert(format!("{a}/{b}"));
    // Distractor: second operand only.
    seen.insert(format!("{c}/{b}"));
    // Distractor: off-by-one numerator.
    if seen.len() < 4 {
        seen.insert(format!("{}/{b}", sum + 1));
    }
    // Fill remaining slots.
    let mut fill = 1;
    while seen.len() < 4 {
        seen.insert(format!("{fill}/{b}"));
        fill += 1;
    }

    let mut choices: Vec<String> = seen.into_iter().collect();
    choices.shuffle(rng);
    let correct_index = choices.iter().position(|ch| *ch == correct).unwrap_or(0);
    (choices, correct_index)
}

/// Generate 4 MCQ choices for a multiplication `a * b`, including the correct
/// answer and 3 plausible distractors. Returns `(choices, correct_index)`.
fn generate_multiplication_distractors(a: u32, b: u32, rng: &mut impl Rng) -> (Vec<String>, usize) {
    let correct = a * b;
    let mut seen: HashSet<u32> = HashSet::from([correct]);

    // Plausible distractors: neighbouring products.
    let candidates = [
        a.checked_sub(1).map(|v| v * b),
        Some((a + 1) * b),
        a.checked_mul(b.checked_sub(1).unwrap_or(b)),
        Some(a * (b + 1)),
        correct.checked_sub(1),
        Some(correct + 1),
        Some(correct + a),
        correct.checked_sub(a),
    ];

    for candidate in candidates.into_iter().flatten() {
        if candidate > 0 {
            seen.insert(candidate);
        }
        if seen.len() >= 4 {
            break;
        }
    }

    // Fill remaining with offset values.
    let mut fill = correct + 2;
    while seen.len() < 4 {
        seen.insert(fill);
        fill += 1;
    }

    let mut choices: Vec<String> = seen.into_iter().map(|v| v.to_string()).collect();
    choices.shuffle(rng);
    let correct_str = correct.to_string();
    let correct_index = choices.iter().position(|c| *c == correct_str).unwrap_or(0);

    (choices, correct_index)
}

/// Build a `FractionIdentificationDefinition` with plausible distractors
/// derived from the given numerator/denominator.
fn generate_fraction_identification(
    numerator: u32,
    denominator: u32,
    rng: &mut impl Rng,
) -> FractionIdentificationDefinition {
    let correct = format!("{numerator}/{denominator}");
    let mut seen: HashSet<String> = HashSet::from([correct.clone()]);

    let candidates = [
        (denominator != numerator).then(|| format!("{denominator}/{numerator}")),
        (numerator > 1).then(|| format!("{}/{denominator}", numerator - 1)),
        (numerator < denominator).then(|| format!("{}/{denominator}", numerator + 1)),
        Some(format!("{numerator}/{}", denominator + 1)),
    ];
    for candidate in candidates.into_iter().flatten() {
        seen.insert(candidate);
        if seen.len() >= 4 {
            break;
        }
    }
    // Fill remaining slots.
    let mut d = denominator + 2;
    while seen.len() < 4 {
        seen.insert(format!("{numerator}/{d}"));
        d += 1;
    }

    let mut choices: Vec<String> = seen.into_iter().collect();
    choices.shuffle(rng);

    let correct_index = choices.iter().position(|c| *c == correct).unwrap_or(0);

    FractionIdentificationDefinition {
        numerator,
        denominator,
        choices,
        correct_index,
        explanation: LocalizedText::new(
            format!(
                "Il y a {denominator} parts et {numerator} sont coloriées : c'est {numerator}/{denominator}."
            ),
            format!(
                "There are {denominator} parts and {numerator} are colored: that's {numerator}/{denominator}."
            ),
        ),
        explanation_visual: Some(ExplanationVisual::FractionBar {
            numerator,
            denominator,
        }),
    }
}

fn substitute_template(template: &str, params: &[(&str, i32)]) -> String {
    let mut result = template.to_owned();
    for &(name, value) in params {
        result = result.replace(&format!("{{{name}}}"), &value.to_string());
    }
    result
}
