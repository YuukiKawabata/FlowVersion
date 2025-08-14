use crate::error::{FlowError, Result};
use crate::core::{repository::Repository, intention::Intention};
use std::path::Path;
use tracing::{info, debug, warn};

pub struct InitHandler;

impl InitHandler {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn handle(&self, name: Option<String>, ai_mode: String) -> Result<()> {
        let current_dir = std::env::current_dir()
            .map_err(|e| FlowError::IoError(e))?;
        
        debug!("Initializing repository in: {:?}", current_dir);
        
        // Check if already initialized
        if Repository::is_initialized(&current_dir)? {
            return Err(FlowError::RepoAlreadyExists);
        }
        
        // Initialize repository
        let repo_name = name.unwrap_or_else(|| {
            current_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("flowversion-repo")
                .to_string()
        });
        
        Repository::init(&current_dir, &repo_name, &ai_mode).await?;
        
        info!("Initialized FlowVersion repository: {}", repo_name);
        println!("Initialized empty FlowVersion repository in {}", current_dir.display());
        println!("Repository name: {}", repo_name);
        println!("AI mode: {}", ai_mode);
        
        Ok(())
    }
}

pub struct AddHandler;

impl AddHandler {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn handle(&self, files: Vec<String>, intention: Option<String>) -> Result<()> {
        let current_dir = std::env::current_dir()
            .map_err(|e| FlowError::IoError(e))?;
        
        let repo = Repository::open(&current_dir).await?;
        
        debug!("Adding files: {:?}", files);
        
        let mut added_files = Vec::new();
        for file_path in files {
            let path = Path::new(&file_path);
            
            if !path.exists() {
                warn!("File does not exist: {}", file_path);
                return Err(FlowError::FileNotFound(file_path));
            }
            
            repo.add_file(&file_path).await?;
            added_files.push(file_path);
        }
        
        info!("Added {} files to staging area", added_files.len());
        for file in &added_files {
            println!("added: {}", file);
        }
        
        if let Some(intent_text) = intention {
            info!("Associated intention: {}", intent_text);
            println!("Associated intention: {}", intent_text);
        }
        
        Ok(())
    }
}

pub struct CommitHandler;

impl CommitHandler {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn handle(
        &self, 
        goal: String, 
        context: Option<String>, 
        impact: Option<String>, 
        confidence: Option<f32>
    ) -> Result<()> {
        let current_dir = std::env::current_dir()
            .map_err(|e| FlowError::IoError(e))?;
        
        let repo = Repository::open(&current_dir).await?;
        
        debug!("Creating commit with intention: {}", goal);
        
        // Create intention object
        let intention = Intention::new(goal, context, impact, confidence.unwrap_or(0.8));
        
        // Execute commit
        let commit_id = repo.commit_with_intention(intention).await?;
        
        info!("Created commit: {}", commit_id);
        let commit_str = commit_id.to_string();
        let short_id = if commit_str.len() >= 8 { &commit_str[..8] } else { &commit_str };
        println!("[{}] {}", short_id, intention.goal);
        
        if let Some(ctx) = &intention.context {
            println!("Context: {}", ctx);
        }
        
        if let Some(imp) = &intention.impact {
            println!("Impact: {}", imp);
        }
        
        println!("Confidence: {:.1}%", intention.confidence * 100.0);
        
        Ok(())
    }
}

pub struct LogHandler;

impl LogHandler {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn handle(&self, oneline: bool, graph: bool, intentions: bool) -> Result<()> {
        let current_dir = std::env::current_dir()
            .map_err(|e| FlowError::IoError(e))?;
        
        let repo = Repository::open(&current_dir).await?;
        
        debug!("Retrieving commit log");
        
        let commits = repo.get_log().await?;
        
        if commits.is_empty() {
            println!("No commits found");
            return Ok(());
        }
        
        info!("Retrieved {} commits", commits.len());
        
        for commit in commits.iter().rev() {
            if oneline {
                let commit_str = commit.id.to_string();
                let short_id = if commit_str.len() >= 8 { &commit_str[..8] } else { &commit_str };
                print!("{} ", short_id);
                print!("{}", commit.intention.goal);
                if intentions && commit.intention.context.is_some() {
                    print!(" ({})", commit.intention.context.as_ref().unwrap());
                }
                println!();
            } else {
                println!("commit {}", commit.id);
                println!("Date: {}", commit.timestamp.format("%Y-%m-%d %H:%M:%S"));
                println!();
                println!("    Goal: {}", commit.intention.goal);
                
                if let Some(context) = &commit.intention.context {
                    println!("    Context: {}", context);
                }
                
                if let Some(impact) = &commit.intention.impact {
                    println!("    Impact: {}", impact);
                }
                
                println!("    Confidence: {:.1}%", commit.intention.confidence * 100.0);
                
                if intentions {
                    println!("    Tags: {:?}", commit.intention.tags);
                }
                
                println!();
            }
        }
        
        Ok(())
    }
}