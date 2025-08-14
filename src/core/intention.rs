use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::error::{FlowError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intention {
    pub id: Uuid,
    pub goal: String,
    pub context: Option<String>,
    pub impact: Option<String>,
    pub confidence: f32,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub tags: Vec<String>,
    pub related_intentions: Vec<Uuid>,
}

impl Intention {
    pub fn new(
        goal: String,
        context: Option<String>,
        impact: Option<String>,
        confidence: f32,
    ) -> Self {
        let tags = Self::generate_tags(&goal, context.as_deref(), impact.as_deref());
        
        Self {
            id: Uuid::new_v4(),
            goal,
            context,
            impact,
            confidence: confidence.clamp(0.0, 1.0),
            author: Self::get_current_user(),
            timestamp: Utc::now(),
            tags,
            related_intentions: Vec::new(),
        }
    }
    
    pub fn validate(&self) -> Result<()> {
        if self.goal.trim().is_empty() {
            return Err(FlowError::InvalidIntention("Goal cannot be empty".to_string()));
        }
        
        if self.goal.len() > 200 {
            return Err(FlowError::InvalidIntention("Goal too long (max 200 chars)".to_string()));
        }
        
        if self.confidence < 0.0 || self.confidence > 1.0 {
            return Err(FlowError::InvalidConfidenceScore(self.confidence));
        }
        
        Ok(())
    }
    
    fn generate_tags(goal: &str, context: Option<&str>, impact: Option<&str>) -> Vec<String> {
        let mut tags = Vec::new();
        let text = format!("{} {} {}", 
            goal, 
            context.unwrap_or(""), 
            impact.unwrap_or("")
        ).to_lowercase();
        
        // Common development tags
        if text.contains("bug") || text.contains("fix") || text.contains("error") {
            tags.push("bugfix".to_string());
        }
        
        if text.contains("feature") || text.contains("add") || text.contains("implement") {
            tags.push("feature".to_string());
        }
        
        if text.contains("refactor") || text.contains("cleanup") || text.contains("optimize") {
            tags.push("refactor".to_string());
        }
        
        if text.contains("test") {
            tags.push("test".to_string());
        }
        
        if text.contains("doc") || text.contains("comment") {
            tags.push("documentation".to_string());
        }
        
        if text.contains("security") || text.contains("auth") || text.contains("login") {
            tags.push("security".to_string());
        }
        
        if text.contains("performance") || text.contains("speed") || text.contains("fast") {
            tags.push("performance".to_string());
        }
        
        if text.contains("ui") || text.contains("interface") || text.contains("design") {
            tags.push("ui".to_string());
        }
        
        if text.contains("api") || text.contains("endpoint") || text.contains("service") {
            tags.push("api".to_string());
        }
        
        if text.contains("database") || text.contains("db") || text.contains("sql") {
            tags.push("database".to_string());
        }
        
        if tags.is_empty() {
            tags.push("general".to_string());
        }
        
        tags
    }
    
    fn get_current_user() -> String {
        std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string())
    }
    
    pub fn similarity_score(&self, other: &Intention) -> f32 {
        let mut score = 0.0;
        let mut factors = 0;
        
        // Goal similarity (using simple word matching)
        let self_words: std::collections::HashSet<&str> = self.goal
            .to_lowercase()
            .split_whitespace()
            .collect();
        let other_words: std::collections::HashSet<&str> = other.goal
            .to_lowercase()
            .split_whitespace()
            .collect();
        
        let intersection = self_words.intersection(&other_words).count();
        let union = self_words.union(&other_words).count();
        
        if union > 0 {
            score += intersection as f32 / union as f32;
            factors += 1;
        }
        
        // Tag similarity
        let self_tags: std::collections::HashSet<&str> = self.tags.iter().map(|s| s.as_str()).collect();
        let other_tags: std::collections::HashSet<&str> = other.tags.iter().map(|s| s.as_str()).collect();
        
        let tag_intersection = self_tags.intersection(&other_tags).count();
        let tag_union = self_tags.union(&other_tags).count();
        
        if tag_union > 0 {
            score += tag_intersection as f32 / tag_union as f32;
            factors += 1;
        }
        
        // Author similarity
        if self.author == other.author {
            score += 1.0;
        }
        factors += 1;
        
        // Time proximity (within 24 hours gets bonus)
        let time_diff = (self.timestamp - other.timestamp).abs();
        if time_diff.num_hours() < 24 {
            score += 0.5;
        }
        factors += 1;
        
        if factors > 0 {
            score / factors as f32
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_intention_creation() {
        let intention = Intention::new(
            "Add authentication".to_string(),
            Some("Security requirement".to_string()),
            Some("Login system".to_string()),
            0.9
        );
        
        assert_eq!(intention.goal, "Add authentication");
        assert_eq!(intention.confidence, 0.9);
        assert!(intention.tags.contains(&"security".to_string()));
        assert!(!intention.id.is_nil());
    }
    
    #[test]
    fn test_intention_validation() {
        let valid_intention = Intention::new(
            "Valid goal".to_string(),
            None,
            None,
            0.5
        );
        assert!(valid_intention.validate().is_ok());
        
        let mut invalid_intention = valid_intention.clone();
        invalid_intention.goal = "".to_string();
        assert!(invalid_intention.validate().is_err());
        
        let mut invalid_confidence = valid_intention.clone();
        invalid_confidence.confidence = 1.5;
        assert!(invalid_confidence.validate().is_err());
    }
    
    #[test]
    fn test_tag_generation() {
        let intention = Intention::new(
            "Fix bug in authentication".to_string(),
            Some("Security issue".to_string()),
            None,
            0.8
        );
        
        assert!(intention.tags.contains(&"bugfix".to_string()));
        assert!(intention.tags.contains(&"security".to_string()));
    }
    
    #[test]
    fn test_similarity_score() {
        let intention1 = Intention::new(
            "Add user authentication".to_string(),
            None,
            None,
            0.8
        );
        
        let intention2 = Intention::new(
            "Add user login system".to_string(),
            None,
            None,
            0.9
        );
        
        let score = intention1.similarity_score(&intention2);
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }
}