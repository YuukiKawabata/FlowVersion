use crate::error::{FlowError, Result};
use crate::core::repository::Repository;
use crate::cli::handlers::{InitHandler, AddHandler, CommitHandler, LogHandler};
use crate::Commands;
use tracing::{info, debug};

pub async fn handle_command(command: Option<Commands>) -> Result<()> {
    match command {
        Some(Commands::Init { name, ai_mode }) => {
            debug!("Handling init command: name={:?}, ai_mode={}", name, ai_mode);
            InitHandler::new().handle(name, ai_mode).await
        }
        
        Some(Commands::Add { files, intention }) => {
            debug!("Handling add command: files={:?}, intention={:?}", files, intention);
            AddHandler::new().handle(files, intention).await
        }
        
        Some(Commands::Commit { 
            intention, 
            context, 
            impact, 
            confidence, 
            ai_suggest 
        }) => {
            debug!("Handling commit command: intention={}", intention);
            
            if ai_suggest {
                return Err(FlowError::NotImplemented("AI suggestion feature".to_string()));
            }
            
            if let Some(conf) = confidence {
                if conf < 0.0 || conf > 1.0 {
                    return Err(FlowError::InvalidConfidenceScore(conf));
                }
            }
            
            CommitHandler::new().handle(intention, context, impact, confidence).await
        }
        
        Some(Commands::Log { oneline, graph, intentions }) => {
            debug!("Handling log command: oneline={}, graph={}, intentions={}", 
                   oneline, graph, intentions);
            LogHandler::new().handle(oneline, graph, intentions).await
        }
        
        Some(Commands::Show { commit_id }) => {
            debug!("Handling show command: commit_id={}", commit_id);
            Err(FlowError::NotImplemented("show command".to_string()))
        }
        
        Some(Commands::Diff { commit1, commit2 }) => {
            debug!("Handling diff command: commit1={:?}, commit2={:?}", commit1, commit2);
            Err(FlowError::NotImplemented("diff command".to_string()))
        }
        
        None => {
            info!("No command provided, showing help");
            println!("FlowVersion - Next-generation version control system");
            println!("Use 'flow --help' for available commands");
            Ok(())
        }
    }
}