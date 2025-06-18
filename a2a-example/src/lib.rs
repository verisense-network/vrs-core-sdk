// src/agent_state.rs
mod model;
use chrono::Utc;
use dashmap::DashMap;
use model::{
    AgentCapabilities, AgentCard, AgentProvider, AgentSkill, JSONRPCErrorData, Message,
    MessageSendParams, Part, Task, TaskIdParams, TaskQueryParams, TaskState, TaskStatus,
    TextPartPayload,
};
use std::sync::Arc;
use vrs_core_sdk::{export, get, post, timer::now};

#[post]
fn get_task() -> Result<Task, String> {
    Err("Not implemented".to_string())
}
// Global state variables
static AGENT_CARD: std::sync::OnceLock<AgentCard> = std::sync::OnceLock::new();
static TASKS: std::sync::OnceLock<Arc<DashMap<String, Task>>> = std::sync::OnceLock::new();

fn get_agent_card() -> &'static AgentCard {
    AGENT_CARD.get_or_init(|| AgentCard {
        name: "Rust A2A Agent".to_string(),
        description: "A sample A2A Agent written in Rust".to_string(),
        url: "http://localhost:3000".to_string(),
        icon_url: None,
        provider: Some(AgentProvider {
            organization: "My Org".to_string(),
            url: "http://myorg.com".to_string(),
        }),
        version: "0.1.0".to_string(),
        documentation_url: None,
        capabilities: AgentCapabilities {
            streaming: Some(false),
            push_notifications: Some(false),
            state_transition_history: Some(true),
            extensions: None,
        },
        security_schemes: None,
        security: None,
        default_input_modes: vec!["text/plain".to_string()],
        default_output_modes: vec!["text/plain".to_string()],
        skills: vec![AgentSkill {
            id: "echo_skill".to_string(),
            name: "Echo Skill".to_string(),
            description: "Echoes back the input message.".to_string(),
            tags: vec!["echo".to_string(), "utility".to_string()],
            examples: Some(vec!["Say 'hello'".to_string()]),
            input_modes: Some(vec!["text/plain".to_string()]),
            output_modes: Some(vec!["text/plain".to_string()]),
        }],
        supports_authenticated_extended_card: Some(false),
    })
}

fn get_tasks() -> &'static Arc<DashMap<String, Task>> {
    TASKS.get_or_init(|| Arc::new(DashMap::new()))
}

// --- RPC Interface Functions ---

/// 7.1 agent.getCard - Returns the agent card
#[get(hidden)]
pub fn get_card() -> Result<AgentCard, JSONRPCErrorData> {
    Ok(get_agent_card().clone())
}

/// 7.2 agent.sendMessage - Sends a message to the agent
#[post]
pub fn send_message(params: MessageSendParams) -> Result<Task, JSONRPCErrorData> {
    let tasks = get_tasks();

    // Create or get existing task
    let task = if let Some(task_id) = &params.message.task_id {
        // Update existing task
        if let Some(mut task_entry) = tasks.get_mut(task_id) {
            let task = task_entry.value_mut();

            // Add message to history
            if let Some(ref mut history) = task.history {
                history.push(params.message.clone());
            } else {
                task.history = Some(vec![params.message.clone()]);
            }

            // Update task status to Working
            task.status = TaskStatus {
                state: TaskState::Working,
                message: Some(Message {
                    role: "agent".to_string(),
                    parts: vec![Part::Text(TextPartPayload {
                        text: "Processing message...".to_string(),
                        metadata: None,
                    })],
                    message_id: "".to_string(),
                    task_id: Some(task_id.clone()),
                    kind: "message".to_string(),
                    metadata: None,
                    extensions: None,
                    reference_task_ids: None,
                    context_id: None,
                }),
                timestamp: Some(now()),
            };

            task.clone()
        } else {
            return Err(JSONRPCErrorData::task_not_found(task_id));
        }
    } else {
        // Create new task
        create_task(params.message)
    };

    Ok(task)
}

/// 7.3 agent.getTask - Retrieves a task by ID
#[get]
pub fn get_task(params: TaskQueryParams) -> Result<Task, JSONRPCErrorData> {
    if let Some(task) = get_task_by_id(&params.id_params.id) {
        Ok(task)
    } else {
        Err(JSONRPCErrorData::task_not_found(&params.id_params.id))
    }
}

/// 7.4 agent.cancelTask - Cancels a task
#[post]
pub fn cancel_task(params: TaskIdParams) -> Result<Task, JSONRPCErrorData> {
    let tasks = get_tasks();

    if let Some(mut task_entry) = tasks.get_mut(&params.id) {
        let task = task_entry.value_mut();

        // Check if task can be canceled
        match task.status.state {
            TaskState::Completed | TaskState::Canceled | TaskState::Failed => {
                return Err(JSONRPCErrorData::task_not_cancelable(&params.id));
            }
            _ => {}
        }

        // Update task status to Canceled
        task.status = TaskStatus {
            state: TaskState::Canceled,
            message: Some(Message {
                role: "agent".to_string(),
                parts: vec![Part::Text(TextPartPayload {
                    text: "Task canceled".to_string(),
                    metadata: None,
                })],
                message_id: "".to_string(),
                task_id: Some(params.id.clone()),
                kind: "message".to_string(),
                metadata: None,
                extensions: None,
                reference_task_ids: None,
                context_id: None,
            }),
            timestamp: Some(now()),
        };

        Ok(task.clone())
    } else {
        Err(JSONRPCErrorData::task_not_found(&params.id))
    }
}

// --- Helper Functions ---

/// Creates a new task with the given initial message
pub fn create_task(initial_message: Message) -> Task {
    let tasks = get_tasks();
    let task_id = "".to_string();
    let context_id = initial_message
        .context_id
        .clone()
        .unwrap_or_else(|| "".to_string());

    let task = Task {
        id: task_id.clone(),
        context_id,
        status: TaskStatus {
            state: TaskState::Submitted,
            message: Some(Message {
                role: "agent".to_string(),
                parts: vec![Part::Text(TextPartPayload {
                    text: "Task submitted".to_string(),
                    metadata: None,
                })],
                message_id: "".to_string(),
                task_id: Some(task_id.clone()),
                kind: "message".to_string(),
                metadata: None,
                extensions: None,
                reference_task_ids: None,
                context_id: None,
            }),
            timestamp: Some(now()),
        },
        history: Some(vec![initial_message]),
        artifacts: None,
        metadata: None,
        kind: "task".to_string(),
    };

    tasks.insert(task_id, task.clone());
    task
}

/// Retrieves a task by ID
pub fn get_task_by_id(task_id: &str) -> Option<Task> {
    let tasks = get_tasks();
    tasks.get(task_id).map(|task_ref| task_ref.value().clone())
}

/// Updates task status
#[post]
pub fn update_task_status(
    task_id: String,
    new_state: TaskState,
    agent_message: Option<Message>,
) -> Option<Task> {
    let tasks = get_tasks();

    if let Some(mut task_entry) = tasks.get_mut(&task_id) {
        let task = task_entry.value_mut();
        task.status = TaskStatus {
            state: new_state,
            message: agent_message.or_else(|| {
                Some(Message {
                    role: "agent".to_string(),
                    parts: vec![],
                    message_id: "".to_string(),
                    task_id: Some(task_id.to_string()),
                    kind: "message".to_string(),
                    metadata: None,
                    extensions: None,
                    reference_task_ids: None,
                    context_id: None,
                })
            }),
            timestamp: Some(now()),
        };
        Some(task.clone())
    } else {
        None
    }
}
