use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct CTCLabelDecode {
    pub character_dict: Vec<char>,
    pub blank_index: usize,
}

impl CTCLabelDecode {
    pub fn new(character_list: Option<&[String]>, use_space_char: bool) -> Self {
        let mut chars: Vec<char> = Vec::new();
        chars.push('\0');

        if let Some(list) = character_list {
            for s in list {
                if let Some(c) = s.chars().next() {
                    chars.push(c);
                }
            }
        } else {
            chars.extend("0123456789abcdefghijklmnopqrstuvwxyz".chars());
        }

        if use_space_char && !chars.contains(&' ') {
            chars.push(' ');
        }

        Self {
            character_dict: chars,
            blank_index: 0,
        }
    }

    pub fn decode_sequence(&self, sequence_idx: &[usize], sequence_prob: &[f32]) -> (String, f32) {
        let mut text = String::with_capacity(sequence_idx.len());
        let mut filtered_prob = Vec::with_capacity(sequence_idx.len());
        let mut prev_idx = self.blank_index;

        for (i, &idx) in sequence_idx.iter().enumerate() {
            if idx != self.blank_index && idx != prev_idx {
                if let Some(&ch) = self.character_dict.get(idx) {
                    text.push(ch);
                    if i < sequence_prob.len() {
                        filtered_prob.push(sequence_prob[i]);
                    }
                }
            }
            prev_idx = idx;
        }

        let score = if filtered_prob.is_empty() {
            0.0
        } else {
            filtered_prob.iter().sum::<f32>() / filtered_prob.len() as f32
        };

        (text, score)
    }

    pub fn apply<S>(&self, preds: &ndarray::ArrayBase<S, ndarray::Ix3>) -> (Vec<String>, Vec<f32>)
    where
        S: ndarray::Data<Elem = f32> + Sync,
    {
        if preds.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let batch_size = preds.shape()[0];

        (0..batch_size)
            .into_par_iter()
            .map(|batch_idx| {
                let pred = preds.index_axis(ndarray::Axis(0), batch_idx);
                let time_steps = pred.shape()[0];

                let mut sequence_idx = Vec::with_capacity(time_steps);
                let mut sequence_prob = Vec::with_capacity(time_steps);

                for row in pred.outer_iter() {
                    let mut max_idx = 0;
                    let mut max_val = f32::NEG_INFINITY;
                    for (vocab_idx, &val) in row.iter().enumerate() {
                        if val > max_val {
                            max_val = val;
                            max_idx = vocab_idx;
                        }
                    }
                    sequence_idx.push(max_idx);
                    sequence_prob.push(max_val);
                }

                self.decode_sequence(&sequence_idx, &sequence_prob)
            })
            .unzip()
    }
}
