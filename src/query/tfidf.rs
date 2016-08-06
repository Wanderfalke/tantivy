use super::MultiTermAccumulator;
use super::Scorer;
use super::MultiTermScorer;
use super::Explanation;

#[derive(Clone)]
pub struct TfIdfScorer {
    coords: Vec<f32>,
    idf: Vec<f32>,
    score: f32,
    num_fields: usize,
    term_names: Option<Vec<String>>, //< only here for explain
}

impl MultiTermAccumulator for TfIdfScorer {
    
    #[inline(always)]
    fn update(&mut self, term_ord: usize, term_freq: u32, fieldnorm: u32) {
        assert!(term_freq != 0u32);
        self.score += self.term_score(term_ord, term_freq, fieldnorm);
        self.num_fields += 1;
    }

    #[inline(always)]
    fn clear(&mut self,) {
        self.score = 0f32;
        self.num_fields = 0;
    }
}

impl TfIdfScorer {
    pub fn new(coords: Vec<f32>, idf: Vec<f32>) -> TfIdfScorer {
        TfIdfScorer {
            coords: coords,
            idf: idf,
            score: 0f32,
            num_fields: 0,
            term_names: None,
        }
    }
    
    #[inline(always)]
    fn coord(&self,) -> f32 {
        self.coords[self.num_fields]
    }
    
    pub fn set_term_names(&mut self, term_names: Vec<String>) {
        self.term_names = Some(term_names);
    }

    fn term_name(&self, ord: usize) -> String {
        match &self.term_names {
            &Some(ref term_names_vec) => term_names_vec[ord].clone(),
            &None => format!("Field({})", ord)
        }
    }
    
    #[inline(always)]
    fn term_score(&self, term_ord: usize, term_freq: u32, field_norm: u32) -> f32 {
        (term_freq as f32 / field_norm as f32).sqrt() * self.idf[term_ord]
    }
}

impl Scorer for TfIdfScorer {
    #[inline(always)]
    fn score(&self, ) -> f32 {
        self.score * self.coord()
    }
}

impl MultiTermScorer for TfIdfScorer {
    fn explain(&self, vals: &Vec<(usize, u32, u32)>) -> Explanation {
        let score = self.score();
        let mut explanation = Explanation::with_val(score);
        let formula_components: Vec<String> = vals.iter()
            .map(|&(ord, _, _)| ord)
            .map(|ord| format!("<score for ({}>", self.term_name(ord)))
            .collect();
        let formula = format!("<coord> * ({})", formula_components.join(" + "));
        explanation.set_formula(&formula);
        for &(ord, term_freq, field_norm) in vals.iter() {
            let term_score = self.term_score(ord, term_freq, field_norm);
            let term_explanation = explanation.add_child(&self.term_name(ord), term_score);
            term_explanation.set_formula(" sqrt(<term_freq> / <field_norm>) * <idf>");    
        }
        explanation
    }
}



#[cfg(test)]
mod tests {
    
    use super::*;
    use query::Scorer;
    use query::MultiTermAccumulator;
            
    fn abs_diff(left: f32, right: f32) -> f32 {
        (right - left).abs()
    }  

    #[test]
    pub fn test_multiterm_scorer() {
        let mut tfidf_scorer = TfIdfScorer::new(vec!(0f32, 1f32, 2f32), vec!(1f32, 4f32));
        {
            tfidf_scorer.update(0, 1, 1);
            assert!(abs_diff(tfidf_scorer.score(), 1f32) < 0.001f32);
            tfidf_scorer.clear();
            
        }
        {
            tfidf_scorer.update(1, 1, 1);
            assert_eq!(tfidf_scorer.score(), 4f32);    
            tfidf_scorer.clear();
        }
        {
            tfidf_scorer.update(0, 2, 1);
            assert!(abs_diff(tfidf_scorer.score(), 1.4142135) < 0.001f32);          
            tfidf_scorer.clear();
        }
        {
            tfidf_scorer.update(0, 1, 1);
            tfidf_scorer.update(1, 1, 1);
            assert_eq!(tfidf_scorer.score(), 10f32);    
            tfidf_scorer.clear();
        }


    }

}